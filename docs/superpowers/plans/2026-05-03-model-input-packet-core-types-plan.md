# Plan: ModelInputPacket Core Types

**日期**: 2026-05-03
**版本**: v0.1
**依赖**: 无（最先执行）
**并行度**: 完全独立，可最先执行

---

## Current Truth

- files inspected:
  - `cozmio/src-tauri/src/model_client.rs:45` — `build_prompt_with_context()` 直接接收 `process_context` 字符串 + `popup_context` 字符串，混在一起
  - `cozmio/src-tauri/src/prompt_context.rs:19` — `build_popup_context()` 直接拼接 `action_log_tail` + `process_context` + `competition_entries` 原始文本，无 Admission 控制
  - `cozmio/src-tauri/src/types.rs:101` — `NotificationPending { trace_id, token, content_text, user_how, created_at }` 缺少 `trigger_route/freshness/lineage_ref`
  - `cozmio/src-tauri/src/main_loop.rs:220` — `model_client.call_raw_with_context(&snapshot, &process_context, Some(&popup_context))` 直接传 snapshot+process_context，无 ModelInputPacket 包裹
  - `cozmio/src-tauri/src/window_monitor.rs:11` — `BufferedEntry` 只有 `window_title/process_name/timestamp`，无 `source` 标注

- existing entry points:
  - `ModelClient::call_raw_with_context()` at `model_client.rs:72` — 签名：`(snapshot: &WindowSnapshot, process_context: &ProcessContext, popup_context: Option<&str>) -> Result<ModelRawOutput, String>`
  - `build_popup_context()` at `prompt_context.rs:13` — 签名：`(logger, window_title, process_name, process_context, reminder_context) -> String`
  - `NotificationPending::new()` at `types.rs:109` — 签名：`(trace_id, content_text, user_how) -> Self`

---

## Implementation Shape

### RP-1: Create `Freshness` enum in `types.rs`

**文件**: `cozmio/src-tauri/src/types.rs`
**当前真相**: 无 Freshness 类型
**修改为**:
```rust
/// Freshness 一等概念 — 时间即语义，不是 timestamp 字段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Freshness {
    /// < 5 秒，当前状态
    Current,
    /// 5-30 秒，刚刚发生
    Recent,
    /// 30-120 秒，已陈旧
    Stale,
    /// > 120 秒，历史记录
    Historical,
}

impl Freshness {
    pub fn from_timestamp(created_at: i64) -> Self {
        let age_secs = chrono::Utc::now().timestamp() - created_at;
        if age_secs < 5 {
            Freshness::Current
        } else if age_secs < 30 {
            Freshness::Recent
        } else if age_secs < 120 {
            Freshness::Stale
        } else {
            Freshness::Historical
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            Freshness::Current => "刚刚",
            Freshness::Recent => "刚才",
            Freshness::Stale => "较早",
            Freshness::Historical => "历史",
        }
    }

    pub fn is_fresh(&self) -> bool {
        matches!(self, Freshness::Current | Freshness::Recent)
    }
}
```

**验证**: `cargo test -p cozmio -- Freshness --nocapture` 测试 Freshness::from_timestamp 边界值
**事实依据**: 设计文档 Section 3.2
**状态**: 已锁定 ✓

---

### RP-2: Create `TriggerRoute` enum in `types.rs`

**文件**: `cozmio/src-tauri/src/types.rs`
**当前真相**: 无 TriggerRoute 类型
**修改为**:
```rust
/// 弹窗触发路线 — 每条弹窗声明触发路线，用于归因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerRoute {
    CurrentScreen {
        window_title: String,
        process_name: String,
        freshness: Freshness,
    },
    HistoryMemory {
        source_trace_id: String,
        similarity_score: f32,
        source_event: String,
        freshness: Freshness,
    },
    ExecutionStatus {
        session_id: String,
        status: String,
        change_type: String,
        freshness: Freshness,
    },
    DialogueContinuation {
        goal_id: String,
        source_dialogue: String,
        matched_window: String,
        freshness: Freshness,
    },
    Debug {
        reason: String,
        input_sample_id: Option<String>,
        lineage_ref: String,
    },
}

impl TriggerRoute {
    pub fn badge_label(&self) -> &'static str {
        match self {
            TriggerRoute::CurrentScreen { .. } => "当前页面",
            TriggerRoute::HistoryMemory { .. } => "历史记忆",
            TriggerRoute::ExecutionStatus { .. } => "执行状态",
            TriggerRoute::DialogueContinuation { .. } => "对话延续",
            TriggerRoute::Debug { .. } => "调试",
        }
    }

    pub fn is_debug_only(&self) -> bool {
        matches!(self, TriggerRoute::Debug { .. })
    }
}
```

**验证**: `cargo test -p cozmio -- TriggerRoute --nocapture` 测试 variant序列化 + badge_label
**事实依据**: 设计文档 Section 3.1
**状态**: 已锁定 ✓

---

### RP-3: Create `EvidenceCard` struct in `types.rs`

**文件**: `cozmio/src-tauri/src/types.rs`
**当前真相**: 无 EvidenceCard 类型
**修改为**:
```rust
/// Evidence Card — 历史进入模型输入的唯一合法格式
/// 不能包含 raw logs、model errors、full traces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCard {
    /// 来源: "memory" | "execution" | "window_history" | "dialogue"
    pub source: String,
    /// 引用 ID (ledger event id / memory id / trace_id)
    pub ref_id: String,
    /// 年龄标签（来自证据的 observed_at，不是系统当前时间）
    pub age_label: String,
    /// 短摘要（来自已有摘要/标签/窗口标题，禁止系统规则生成语义解释）
    pub short_summary: String,
    /// 为什么可能相关（只能来自已有摘要/标签/窗口标题；无来源时留空）
    pub why_maybe_relevant: String,
    /// 向量相似分（若有）
    pub similarity_score: Option<f32>,
}
```

**验证**: `cargo test -p cozmio -- EvidenceCard --nocapture`
**事实依据**: 设计文档 Section 3.3 EvidenceObject + Section 18 Context Admission
**状态**: 已锁定 ✓

---

### RP-4: Create `ModelInputPacket` struct in `types.rs`

**文件**: `cozmio/src-tauri/src/types.rs`
**当前真相**: 无 ModelInputPacket 类型；model_client 直接接收 raw snapshot/process_context
**修改为**:
```rust
/// Model Input Packet — model_client.call() 的唯一合法输入
/// 禁止直接把 snapshot/process_context/action_log_tail 传给模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInputPacket {
    /// 当前窗口观察（始终存在）
    pub current_observation: CurrentObservation,
    /// 最近窗口摘要（最近 2 分钟，可选）
    pub recent_windows: Option<String>,
    /// Context Admission 后的 Evidence Cards（不超过 3 张）
    pub evidence_cards: Vec<EvidenceCard>,
    /// 指令契约（始终存在）
    pub instruction_contract: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentObservation {
    pub window_title: String,
    pub process_name: String,
    /// OCR 摘要（短，不超过 100 字）
    pub screen_summary: String,
    /// 截图 base64（可选，模型支持多图时可以传）
    pub screenshot_ref: Option<String>,
}

/// 固定指令契约文本
pub const INSTRUCTION_CONTRACT: &str = r#"你是 Cozmio 的桌面观察助手。

你看到的是用户当前屏幕的一小段现场。
你的输出会被原样交给桌面端展示。
Cozmio 只提供事实材料和工具材料，不提供结论。

请只把下面的系统材料当作事实输入，不要把它们当成用户意图、任务阶段或项目结论。
是否出现、说什么、说多少、是否接入工作流，都由你基于截图和事实材料自行判断。
不要为了迎合上下文而编造屏幕上或材料中没有出现的内容。

禁止输出内部字段名（如 process_context、action_log、MODEL_OUTPUT 等）。
不确定就说"不确定"，不要强行给建议。"#;
```

**验证**: `cargo check -p cozmio` 通过即可
**事实依据**: 设计文档 Section 3 Model Input Packet 铁律
**状态**: 已锁定 ✓

---

### RP-5: Enhance `NotificationPending` with new fields

**文件**: `cozmio/src-tauri/src/types.rs`（同一文件，继续添加）
**当前真相**:
```rust
pub struct NotificationPending {
    pub trace_id: String,
    pub token: ConfirmToken,
    pub content_text: String,  // 用户可见文本（已解释）
    pub user_how: Option<String>,
    pub created_at: i64,
}
```
**修改为**:
```rust
pub struct NotificationPending {
    pub trace_id: String,
    pub token: ConfirmToken,
    /// 用户可见文本（已解释，非 raw audit trace）
    pub user_facing_text: String,
    pub user_how: Option<String>,
    /// 触发路线（用于归因）
    pub trigger_route: TriggerRoute,
    /// 新鲜度
    pub freshness: Freshness,
    /// Lineage 引用（用于问题追溯）
    pub lineage_ref: String,
    pub created_at: i64,
}

impl NotificationPending {
    pub fn new(
        trace_id: String,
        user_facing_text: String,
        user_how: Option<String>,
        trigger_route: TriggerRoute,
        freshness: Freshness,
        lineage_ref: String,
    ) -> Self {
        Self {
            trace_id,
            token: ConfirmToken::new(),
            user_facing_text,
            user_how,
            trigger_route,
            freshness,
            lineage_ref,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}
```

**验证**: `cargo check -p cozmio` — NotificationPending 构造处需要更新（main_loop.rs + notification_manager.rs）
**事实依据**: 设计文档 Section 3.4 NotificationPending 增强
**状态**: 已锁定 ✓

---

### RP-6: Add `LineageRecord` stub for audit traceability

**文件**: `cozmio/src-tauri/src/types.rs`
**当前真相**: 无 LineageRecord 类型
**修改为**:
```rust
/// LineageRecord — 输入输出谱系，用于可追溯性
/// 系统Recorded fields，不是模型生成
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageRecord {
    pub trace_id: String,
    /// 触发路线
    pub trigger_route: TriggerRoute,
    /// 输入证据集（使用引用 ID）
    pub inputs: EvidenceSet,
    /// 模型输入 prompt SHA256
    pub model_input_prompt_sha256: String,
    /// 模型原始输出引用
    pub model_raw_output_ref: String,
    /// 解释后用户可见文本引用
    pub interpretation_ref: Option<String>,
    /// 给执行端的任务说明
    pub agent_brief: String,
    /// 审计日志引用
    pub audit_trace_ref: String,
    /// 新鲜度
    pub freshness: Freshness,
    /// 创建时间
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidenceSet {
    pub window_snapshot_ref: Option<String>,
    pub process_context_ref: Option<String>,
    pub memory_context_ref: Option<String>,
    pub execution_state_ref: Option<String>,
}
```

**验证**: `cargo check -p cozmio`
**事实依据**: 设计文档 Section 3.3 LineageRecord
**状态**: 已锁定 ✓

---

## Key Path Tracing

```
类型定义顺序（无循环依赖）:
  Freshness (无依赖)
  → TriggerRoute (引用 Freshness)
  → EvidenceCard (无依赖)
  → CurrentObservation (无依赖)
  → ModelInputPacket (引用 EvidenceCard)
  → NotificationPending (引用 TriggerRoute + Freshness)
  → LineageRecord (引用 TriggerRoute + EvidenceSet)
```

所有类型在 `types.rs` 中定义，无循环依赖，可并行编写。

## Risk → Verification Mapping

| Risk | 验证命令 | 预期结果 |
|------|---------|---------|
| Freshness 边界计算错误 | `cargo test -p cozmio -- Freshness` | Current(<5s)/Recent(5-30s)/Stale(30-120s)/Historical(>120s) 正确 |
| TriggerRoute badge_label 不匹配 | `cargo test -p cozmio -- TriggerRoute` | 5 个 variant 标签正确 |
| NotificationPending 构造处未更新 | `cargo check -p cozmio 2>&1 | grep "does not have field"` | 0 errors |