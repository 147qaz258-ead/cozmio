# Plan: Context Admission Pipeline

**日期**: 2026-05-03
**版本**: v0.1
**依赖**: `2026-05-03-model-input-packet-core-types-plan.md`（类型定义完成后）
**并行度**: 与主循环改造计划、UI 计划互不冲突，可独立执行

---

## Current Truth

- files inspected:
  - `cozmio/src-tauri/src/memory_commands.rs:295` — `build_activity_context_sync()` 返回 `ReminderContextDto`，包含 `competition_entries: Vec<CompetitionResultEntryDto>` + `competition_trace: Option<CompetitionTraceDto>`
  - `cozmio/src-tauri/src/prompt_context.rs:19` — `build_popup_context()` 直接拼接 `action_log_tail` + 6 条 `format_competition_entry()` 原始文本，无 token budget 概念
  - `cozmio/src-tauri/src/main_loop.rs:165-185` — `build_activity_context_sync()` 调用时传 `token_budget=Some(600)`，但返回的 `ReminderContextDto.competition_entries` 全部直接进入 `popup_context`
  - `cozmio/src-tauri/src/model_client.rs:77` — `build_prompt_with_context()` 拼接 `process_context_block` + `popup_context_block`，但 `popup_context` 已经是完整字符串，无 Evidence Card 概念

- existing entry points:
  - `build_activity_context_sync()` at `memory_commands.rs:262` — 签名：`(window_title, content_text, current_thread_id, token_budget) -> Result<ReminderContextDto, String>`
  - `build_popup_context()` at `prompt_context.rs:13` — 签名：`(logger, window_title, process_name, process_context, reminder_context) -> String`
  - `ModelClient::build_prompt_with_context()` at `model_client.rs:77` — 签名：`(snapshot, process_context, popup_context) -> String`

**已知不一致**:
- `ReminderContextDto.competition_entries` 当前实现：全部进入 `popup_context`（无 budget 限制）
- 设计 spec 要求：全部进入 Candidate Pool，再经 Context Admission 选出最多 3 张 Evidence Card

---

## Implementation Shape

### RP-1: Create `ContextAdmission` struct and `admit()` method

**文件**: `cozmio/src-tauri/src/context_admission.rs`（新文件）
**当前真相**: 无 ContextAdmission 类型，无独立 Admission 逻辑
**修改为**:
```rust
//! Context Admission — 候选池 → Evidence Card → Model Input Packet 管道
//!
//! 设计原则：
//! - 全部进入 Candidate Pool（不预先过滤）
//! - 软排序（非硬阈值截断）
//! - 固定 token budget（最多 3 张 Card）
//! - Evidence Card 内容只能来自已有摘要/标签/窗口标题，禁止系统规则生成语义解释

use crate::types::{EvidenceCard, Freshness};

/// Token budget for evidence cards entering model input
pub const MAX_EVIDENCE_CARDS: usize = 3;

/// Context Admission 管道
pub struct ContextAdmission;

impl ContextAdmission {
    /// Build Evidence Cards from ReminderContextDto competition entries
    ///
    /// 全部进入 Candidate Pool → 软排序 → 取前 MAX_EVIDENCE_CARDS
    pub fn admit(competition_entries: &[crate::memory_commands::CompetitionResultEntryDto]) -> Vec<EvidenceCard> {
        // Step 1: 全部进入候选池（不预过滤）
        let mut candidates: Vec<CandidateEntry> = competition_entries
            .iter()
            .map(|e| CandidateEntry {
                entry: e,
                // 软排序分 = 向量分（若有）* 0.7 + recency 折扣 * 0.3
                soft_score: Self::compute_soft_score(e),
            })
            .collect();

        // Step 2: 按 soft_score 降序排列
        candidates.sort_by(|a, b| b.soft_score.partial_cmp(&a.soft_score).unwrap());

        // Step 3: 取前 MAX_EVIDENCE_CARDS
        candidates
            .into_iter()
            .take(MAX_EVIDENCE_CARDS)
            .map(|c| Self::entry_to_card(c.entry))
            .collect()
    }

    fn compute_soft_score(entry: &crate::memory_commands::CompetitionResultEntryDto) -> f32 {
        let vector_component = entry.vector_score.unwrap_or(0.5) * 0.7;
        // recency 折扣：越老分越低（简化版，不依赖 observed_at 精确时间戳）
        let recency_component = 0.3; // TODO: 后续可按 created_at 计算
        vector_component + recency_component
    }

    fn entry_to_card(entry: &crate::memory_commands::CompetitionResultEntryDto) -> EvidenceCard {
        EvidenceCard {
            source: entry.producer.clone(), // "distill-command" / "memory-core" / etc.
            ref_id: entry.memory_id.clone(),
            // age_label 来自 token_estimate 映射（不是精确时间）
            age_label: Self::token_to_age_label(entry.token_estimate),
            // short_summary：直接使用 memory_text 截断（不得系统生成）
            short_summary: truncate(&entry.memory_text, 80),
            // why_maybe_relevant：来自 selection_reason_facts（禁止系统规则生成）
            why_maybe_relevant: entry
                .selection_reason_facts
                .first()
                .map(|s| truncate(s, 60))
                .unwrap_or_default(),
            similarity_score: entry.vector_score,
        }
    }

    fn token_to_age_label(token_estimate: usize) -> String {
        if token_estimate < 20 {
            "刚刚".to_string()
        } else if token_estimate < 50 {
            "刚才".to_string()
        } else {
            "较早".to_string()
        }
    }
}

struct CandidateEntry<'a> {
    entry: &'a crate::memory_commands::CompetitionResultEntryDto,
    soft_score: f32,
}

fn truncate(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect::<String>() + if s.len() > max_chars { "..." } else { "" }
}
```

**验证**: `cargo test -p cozmio -- context_admission --nocapture`
**事实依据**: `memory_commands.rs:295` 返回 `ReminderContextDto`；设计文档 Section 18 Context Admission
**状态**: 已锁定 ✓

---

### RP-2: Refactor `build_popup_context()` to use ContextAdmission

**文件**: `cozmio/src-tauri/src/prompt_context.rs`
**当前真相**:
```rust
// prompt_context.rs:55-60
if !ctx.competition_entries.is_empty() {
    lines.push(String::from("runtime_selected_memory_entries:"));
    for entry in ctx.competition_entries.iter().take(6) {
        lines.push(format_competition_entry(entry));  // 直接拼接原始 entry
    }
}
```

**修改为**:
```rust
// prompt_context.rs — 新增 evidence card 格式化
use crate::context_admission::ContextAdmission;

// 在 build_popup_context() 末尾，替换 competition_entries 直接拼接逻辑：
if !ctx.competition_entries.is_empty() {
    // 经过 Context Admission 管道
    let evidence_cards = ContextAdmission::admit(&ctx.competition_entries);
    if !evidence_cards.is_empty() {
        lines.push(String::from("evidence_cards:"));
        for card in evidence_cards {
            lines.push(format_evidence_card(&card));
        }
    }
}
```

新增 `format_evidence_card()` 辅助函数：
```rust
fn format_evidence_card(card: &EvidenceCard) -> String {
    format!(
        "- [{}] {} ({}) — {}",
        card.source,
        card.short_summary,
        card.age_label,
        card.why_maybe_relevant
    )
}
```

**验证**: `cargo test -p cozmio -- build_popup_context --nocapture` — 测试 Evidence Card 格式化 + budget 限制
**事实依据**: `prompt_context.rs:19-80` 整体逻辑
**状态**: 已锁定 ✓

---

### RP-3: Refactor `build_model_input_packet()` in `model_client.rs`

**文件**: `cozmio/src-tauri/src/model_client.rs`
**当前真相**:
```rust
// model_client.rs:77 — build_prompt_with_context()
fn build_prompt_with_context(
    &self,
    snapshot: &WindowSnapshot,
    process_context: Option<&crate::window_monitor::ProcessContext>,
    popup_context: Option<&str>,  // 传入完整字符串
) -> String {
    // 直接拼接 process_context_block + popup_context_block
    // 禁止 raw action_log_tail 进入
}
```

**修改为**:
```rust
/// Build ModelInputPacket from snapshot + reminder_context
pub fn build_model_input_packet(
    snapshot: &WindowSnapshot,
    reminder_context: Option<&crate::memory_commands::ReminderContextDto>,
) -> ModelInputPacket {
    // 1. Current Observation（始终存在）
    let current_observation = CurrentObservation {
        window_title: snapshot.window_info.title.clone(),
        process_name: snapshot.window_info.process_name.clone(),
        screen_summary: "截图已提供".to_string(), // OCR 摘要后续可扩展
        screenshot_ref: Some(snapshot.screenshot_base64.clone()),
    };

    // 2. Recent Windows（可选，最短保留）
    let recent_windows = build_recent_windows_summary(snapshot);

    // 3. Evidence Cards（经过 Context Admission）
    let evidence_cards = reminder_context
        .map(|ctx| ContextAdmission::admit(&ctx.competition_entries))
        .unwrap_or_default();

    // 4. Instruction Contract（始终存在）
    let instruction_contract = INSTRUCTION_CONTRACT.to_string();

    ModelInputPacket {
        current_observation,
        recent_windows,
        evidence_cards,
        instruction_contract,
    }
}
```

替换 `build_prompt_with_context()` 的调用方，改为 `build_model_input_packet()` 返回 `ModelInputPacket`，再序列化prompt。

新增 `ModelInputPacket::to_prompt_string()`:
```rust
impl ModelInputPacket {
    pub fn to_prompt_string(&self) -> String {
        let mut parts = vec![
            format!("窗口标题: {}", self.current_observation.window_title),
            format!("进程名: {}", self.current_observation.process_name),
            format!("屏幕摘要: {}", self.current_observation.screen_summary),
        ];

        if let Some(ref recent) = self.recent_windows {
            parts.push(format!("最近窗口: {}", recent));
        }

        if !self.evidence_cards.is_empty() {
            parts.push(String::from("Evidence Cards:"));
            for card in &self.evidence_cards {
                parts.push(format_evidence_card(card));
            }
        }

        parts.push(String::from("\n[系统指令]\n"));
        parts.push(self.instruction_contract.clone());

        parts.join("\n")
    }
}
```

**验证**: `cargo test -p cozmio -- build_model_input_packet --nocapture`
**事实依据**: `model_client.rs:77` + 设计文档 Section 3 Model Input Packet 铁律
**状态**: 已锁定 ✓

---

### RP-4: Update `main_loop.rs` to use `build_model_input_packet()`

**文件**: `cozmio/src-tauri/src/main_loop.rs:220`
**当前真相**:
```rust
// main_loop.rs:220
let popup_context = build_popup_context(
    &logger, &snapshot.window_info.title, &snapshot.window_info.process_name,
    &process_context, reminder_context.as_ref(),
);
let call_result = model_client.call_raw_with_context(
    &snapshot, &process_context, Some(&popup_context)  // 绕过 ModelInputPacket
);
```

**修改为**:
```rust
// main_loop.rs:220
// 必须经过 Context Admission 构建 ModelInputPacket
let input_packet = model_client.build_model_input_packet(
    &snapshot,
    reminder_context.as_ref(),
);
let call_result = model_client.call_with_packet(&input_packet);
```

新增 `call_with_packet()` 方法：
```rust
pub fn call_with_packet(&self, packet: &ModelInputPacket) -> Result<ModelRawOutput, String> {
    let prompt = packet.to_prompt_string();
    // ... send request using prompt + screenshot from packet.current_observation.screenshot_ref
}
```

**验证**: `cargo check -p cozmio` — 确认无" does not have field" 错误
**事实依据**: `main_loop.rs:220` 调用点
**状态**: 已锁定 ✓

---

## Key Path Tracing

```
main_loop.rs:220
  → model_client.build_model_input_packet(&snapshot, reminder_context.as_ref())
      → ContextAdmission::admit(&ctx.competition_entries)  ← 核心逻辑在 RP-1
      → ModelInputPacket::to_prompt_string()
  → model_client.call_with_packet(&input_packet)
      → send_request_timed(prompt, screenshot_base64)
```

**缺失链路**:
- `model_client.rs` 当前没有 `call_with_packet()` 方法 → RP-4 新增

## Risk → Verification Mapping

| Risk | 验证命令 | 预期结果 |
|------|---------|---------|
| ContextAdmission 软排序错误导致关键 entry 被排除 | `cargo test -p cozmio -- context_admission --nocapture` | 软排序后 top 3 正确 |
| prompt 不含 action_log_tail 原文 | `cargo test -p cozmio -- build_model_input_packet --nocapture` | prompt 中无 "action_log_tail" 字段 |
| main_loop 仍绕过 ModelInputPacket | `cargo check -p cozmio 2>&1 | grep "call_raw_with_context"` | 0 results（已替换为 call_with_packet） |