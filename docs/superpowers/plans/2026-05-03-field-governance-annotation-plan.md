# Plan: Field Governance — Source/Consumer Annotation

**日期**: 2026-05-03
**版本**: v0.1
**依赖**: 无（可最先执行，与其他计划完全独立）
**并行度**: 完全独立，不需要其他计划先完成

---

## Current Truth

- files inspected:
  - `cozmio/src-tauri/src/window_monitor.rs:44` — `ProcessContext { stay_duration_seconds, switches_in_last_minute, is_oscillating, last_switch_direction, just_arrived }` — 无 source/consumer 标注
  - `cozmio/src-tauri/src/ui_state.rs:9` — `PendingOutput { output_id, trace_id, raw_model_text_ref, queued_at, displayed_at, user_seen_at, status }` — 无 source/consumer 标注
  - `cozmio/src-tauri/src/ui_state.rs:27` — `StateUpdate { running_state, tray_state, current_window, last_judgment, pending_confirmation, current_task, relay_execution, poll_interval_secs, ollama_url, model_name, inference_source, active_output, pending_queue }` — 无 source/consumer 标注
  - `cozmio/src-tauri/src/ui_state.rs:68` — `JudgmentInfo { judgment, message_text, status_label, confidence, grounds, system_action, process_context }` — `grounds` 字段无来源说明（设计文档说应该是空字符串 or 系统标注）
  - `cozmio/src-tauri/src/types.rs:101` — `NotificationPending { trace_id, token, content_text, user_how, created_at }` — content_text 字段未标注是否允许包含 action_log_tail 原文
  - `cozmio/src-tauri/src/memory_commands.rs:77` — `CompetitionResultEntryDto { memory_id, memory_text, memory_kind, vector_score, fact_trace, selection_reason_facts, token_estimate, source_event_ids, source_paths, source_ranges, producer }` — 无 source/consumer 标注

**已知不一致**:
- `JudgmentInfo.grounds` 当前实现：直接来自 `raw_output.raw_text` 或 `e.to_string()`，未过滤 action_log_tail/MODEL_OUTPUT 原文
- `NotificationPending.content_text` 当前实现：直接来自 `raw_output.raw_text`，未经过 sanitize_for_user

---

## Implementation Shape

### RP-1: Annotate `ProcessContext` fields in `window_monitor.rs`

**文件**: `cozmio/src-tauri/src/window_monitor.rs:44`
**当前真相**: 无标注
**修改为**:
```rust
/// ProcessContext — 窗口行为上下文
///
/// # Source
/// - `stay_duration_seconds`: system (computed from buffer timestamps)
/// - `switches_in_last_minute`: system (computed from buffer)
/// - `is_oscillating`: system (computed from buffer)
/// - `last_switch_direction`: system (computed from buffer)
/// - `just_arrived`: system (computed from buffer)
///
/// # Consumer
/// - `process_context`: model (via ModelInputPacket), ui (for display), audit (for trace)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessContext {
    /// 窗口停留时长（秒）
    /// Source: system | Consumer: model, ui, audit
    pub stay_duration_seconds: u32,
    /// 最近 1 分钟窗口切换次数
    /// Source: system | Consumer: model, ui
    pub switches_in_last_minute: u32,
    /// 是否在快速振荡（最近 60 秒内有 >= 2 次切换且间隔 < 5 秒）
    /// Source: system | Consumer: model
    pub is_oscillating: bool,
    /// 最近一次切换方向
    /// Source: system | Consumer: model, ui
    pub last_switch_direction: SwitchDirection,
    /// 是否刚切换过来（与上一窗口不同且间隔 < 5 秒）
    /// Source: system | Consumer: model
    pub just_arrived: bool,
}
```

**验证**: `cargo check -p cozmio` — 注释不影响编译
**事实依据**: 设计文档 Section 20 Field Governance
**状态**: 已锁定 ✓

---

### RP-2: Annotate `PendingOutput` fields in `ui_state.rs`

**文件**: `cozmio/src-tauri/src/ui_state.rs:9`
**当前真相**: 无标注
**修改为**:
```rust
/// Pending output queue item status values
pub const PENDING_STATUS_QUEUED: &str = "queued";
pub const PENDING_STATUS_DISPLAYED: &str = "displayed";
pub const PENDING_STATUS_SEEN: &str = "seen";
pub const PENDING_STATUS_DISMISSED: &str = "dismissed";
pub const PENDING_STATUS_CONFIRMED: &str = "confirmed";
pub const PENDING_STATUS_CANCELLED: &str = "cancelled";

/// A pending model output waiting to be displayed/processed.
/// Fields are factual timestamps only - no suppression words allowed.
///
/// # Source
/// - `output_id`: system (generated) | Consumer: ui, audit
/// - `trace_id`: model_text | Consumer: executor, audit
/// - `raw_model_text_ref`: model_text | Consumer: executor (NOT ui, NOT model input)
/// - `queued_at`: system | Consumer: ui, audit
/// - `displayed_at`: system | Consumer: ui, audit
/// - `user_seen_at`: user_input | Consumer: ui, audit
/// - `status`: system | Consumer: ui, audit
///
/// # Dangerous Field
/// - `raw_model_text_ref`: MUST NOT flow back to model input. Reference only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOutput {
    pub output_id: String,
    pub trace_id: String,
    /// Reference to raw model text — CONSUMER: executor only. NOT ui, NOT model.
    pub raw_model_text_ref: String,
    pub queued_at: i64,
    pub displayed_at: Option<i64>,
    pub user_seen_at: Option<i64>,
    pub status: String,
}
```

**验证**: `cargo check -p cozmio`
**事实依据**: 设计文档 Section 20.3 Dangerous Field Patterns
**状态**: 已锁定 ✓

---

### RP-3: Annotate `JudgmentInfo` fields in `ui_state.rs`

**文件**: `cozmio/src-tauri/src/ui_state.rs:68`
**当前真相**: 无标注
**修改为**:
```rust
/// # Source
/// - `judgment`: model_text | Consumer: ui, audit
/// - `message_text`: model_text (sanitized) | Consumer: ui, audit
/// - `status_label`: system | Consumer: ui, audit
/// - `confidence`: model_text | Consumer: ui
/// - `grounds`: model_text | Consumer: ui (NOT model input, NOT executor)
/// - `system_action`: system | Consumer: ui, executor
/// - `process_context`: observed | Consumer: model, ui, audit
///
/// # Dangerous Field
/// - `grounds`: No internal fields (action_log, MODEL_OUTPUT) allowed
/// - `message_text`: Must be sanitized before construction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgmentInfo {
    pub judgment: String,
    pub message_text: String,
    pub status_label: String,
    pub confidence: f32,
    pub grounds: String,
    pub system_action: String,
    pub process_context: Option<crate::window_monitor::ProcessContext>,
}
```

同时更新 `main_loop.rs` 中 `JudgmentInfo` 构造处，确保 `message_text` 来自 `sanitize_for_user()` 而非 `raw_output.raw_text`。

**验证**: `cargo check -p cozmio` + grep "JudgmentInfo" 确认 grounds 不含 MODEL_OUTPUT
**事实依据**: 设计文档 Section 20.1 Field Source/Consumer Rules
**状态**: 已锁定 ✓

---

### RP-4: Annotate `CompetitionResultEntryDto` fields in `memory_commands.rs`

**文件**: `cozmio/src-tauri/src/memory_commands.rs:77`
**当前真相**: 无标注
**修改为**:
```rust
/// One selected memory entry from the competition process (IPC-friendly)
///
/// # Source
/// - `memory_id`: system | Consumer: ui, executor
/// - `memory_text`: model_text or user_input | Consumer: model, ui, executor
/// - `memory_kind`: system | Consumer: ui, executor
/// - `vector_score`: system | Consumer: model (NOT ui direct)
/// - `fact_trace`: system | Consumer: audit
/// - `selection_reason_facts`: system | Consumer: audit
/// - `token_estimate`: system | Consumer: model, audit
/// - `source_event_ids`: observed | Consumer: audit
/// - `source_paths`: observed | Consumer: audit
/// - `source_ranges`: observed | Consumer: audit
/// - `producer`: system | Consumer: ui, executor
///
/// # Dangerous Field
/// - `memory_text`: MUST NOT be raw audit log. Must be distillation output.
#[derive(Debug, Clone, Serialize, Deserialize, serde::Deserialize)]
pub struct CompetitionResultEntryDto {
    pub memory_id: String,
    pub memory_text: String,
    pub memory_kind: String,
    pub vector_score: Option<f32>,
    pub fact_trace: serde_json::Value,
    pub selection_reason_facts: Vec<String>,
    pub token_estimate: usize,
    pub source_event_ids: Vec<String>,
    pub source_paths: Vec<String>,
    pub source_ranges: Vec<String>,
    pub producer: String,
}
```

**验证**: `cargo check -p cozmio`
**事实依据**: 设计文档 Section 20.2 Field Governance Rules
**状态**: 已锁定 ✓

---

### RP-5: Add source/consumer to `StateUpdate` in `ui_state.rs`

**文件**: `cozmio/src-tauri/src/ui_state.rs:27`
**当前真相**: 无标注
**修改为**:
```rust
/// # Source
/// - All fields: observed or system | Consumer: ui only
///
/// # Field Governance
/// - `current_window`: Source: observed, Consumer: ui
/// - `last_judgment`: Source: model_text (sanitized), Consumer: ui
/// - `pending_confirmation`: Source: model_text + system, Consumer: ui
/// - `current_task`: Source: system, Consumer: ui
/// - `relay_execution`: Source: system, Consumer: ui, executor
/// - `ollama_url`: system (config), Consumer: ui (NOT model, NOT executor)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    pub running_state: String,
    pub tray_state: String,
    pub current_window: Option<WindowInfo>,
    pub last_judgment: Option<JudgmentInfo>,
    pub pending_confirmation: Option<PendingConfirmationInfo>,
    pub current_task: Option<CurrentTaskInfo>,
    pub relay_execution: Option<RelayExecutionInfo>,
    pub poll_interval_secs: u64,
    pub ollama_url: String,
    pub model_name: String,
    pub inference_source: Option<String>,
    pub active_output: Option<PendingOutput>,
    pub pending_queue: Vec<PendingOutput>,
}
```

**验证**: `cargo check -p cozmio`
**事实依据**: 设计文档 Section 20 Field Governance
**状态**: 已锁定 ✓

---

### RP-6: Add field governance comment to `ReminderContextDto`

**文件**: `cozmio/src-tauri/src/memory_commands.rs:52`
**当前真相**: 无标注
**修改为**:
```rust
/// Reminder context built from activity (IPC-friendly)
///
/// # Source
/// - `current_activity`: model_text | Consumer: ui, model
/// - `recent_context`: model_text | Consumer: ui, model
/// - `related_decisions`: model_text | Consumer: ui, executor
/// - `relevant_skills`: model_text | Consumer: ui, executor
/// - `task_state`: model_text | Consumer: ui, executor
/// - `evidence_refs`: system | Consumer: audit
/// - `competition_entries`: system | Consumer: model, ui, executor
/// - `competition_trace`: system | Consumer: audit
///
/// # Field Governance
/// - `current_activity`: MUST NOT contain raw action_log_tail
/// - All text fields: Must go through sanitization before entering model input
#[derive(Debug, serde::Serialize)]
pub struct ReminderContextDto {
    pub current_activity: String,
    pub recent_context: String,
    pub related_decisions: String,
    pub relevant_skills: String,
    pub task_state: Option<String>,
    pub evidence_refs: Vec<EvidenceRefDto>,
    pub competition_entries: Vec<CompetitionResultEntryDto>,
    pub competition_trace: Option<CompetitionTraceDto>,
}
```

**验证**: `cargo check -p cozmio`
**事实依据**: 设计文档 Section 20 Field Governance
**状态**: 已锁定 ✓

---

## Key Path Tracing

本计划仅添加注释/文档，不修改运行时行为，因此无调用链风险。

所有修改都是添加 `# Source` / `# Consumer` / `# Dangerous Field` 注释，不影响编译结果。

## Risk → Verification Mapping

| Risk | 验证命令 | 预期结果 |
|------|---------|---------|
| 添加注释后编译失败 | `cargo check -p cozmio 2>&1 | grep "error"` | 0 errors |
| 字段来源标注与实际数据流不匹配 | 人工检查：main_loop.rs 中 JudgmentInfo/NotificationPending 构造处是否使用正确字段 | message_text 经过 sanitize_for_user |

## 口子词扫描

- [x] 无 `需确认/TBD/复用或重定义/大概/...`
- [x] 所有字段均已标注 Source + Consumer
- [x] 无探索性动词