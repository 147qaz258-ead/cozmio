# Agent Self-Maintained Memory Flywheel — Stage 1 实施方案

**版本：** 1.0
**日期：** 2026-05-07
**状态：** 已转为自动小闭环执行中
**设计文档：** `docs/superpowers/specs/2026-05-07-agent-self-maintained-memory-flywheel-design.md`

---

## Flywheel Bootstrap

- `claude-progress.txt`: 存在，但当前主任务仍记录为旧的 `AUTO-UPDATE` 完成态；本计划不沿用该任务状态。
- `feature_list.json`: 根目录不存在；真实文件在 `cozmio/feature_list.json`，其中未记录本次 memory flywheel 任务。
- `verification/last_result.json`: 根目录未找到，仓库内也未找到；本计划将在首次验证后补写验证结果。
- 当前设计文档: 已存在，路径见上。
- 当前仓库状态: 存在多个未跟踪文件；本计划只新增本计划文件，不处理其他未跟踪项。

---

## 1. Goal

实现 Stage 1 的最小自动记忆飞轮闭环：

```text
已有事实日志
  -> 生成可读 consolidation packet
  -> 自动保存由本地模型自然输出、用户反馈或执行结果构成的 memory operations
  -> 代码校验 provenance 并存储 active memory
  -> popup context 召回这些 agent-written memories
  -> 下一轮模型输入发生可验证变化
```

Stage 1 的目标不是自动学习一切，而是证明 Cozmio 可以用一个稳定的小自动闭环，从真实经历生成可追溯记忆，并让这些记忆进入后续观察上下文。

---

## 2. Non-Goals

- 不启用大规模后台 consolidation。
- 不上传原始屏幕日志到云端。
- 不重写完整 memory architecture。
- 不删除现有 `human_context.md`、`action_log.jsonl`、event ledger、向量检索、decision memory、skill memory。
- 不把 vector score 当作用户意图、任务阶段或 popup 许可。
- 不把 consolidation prompt 设计成语义字段填表器。
- 不接入完整 memory review UI；Stage 1 只提供自动写入、自动召回和 IPC/命令层可检查结果。
- 不把 `src-tauri/src/distill_commands.rs` 当作已接通主链路；它当前没有在 `main.rs` 注册。

---

## 3. Product Type

- type: `agent_system` + `model_output_validated` + `traditional_software`
- core risk:
  - 代码再次承担语义判断，破坏 semantic boundary。
  - 自动 consolidation 只写入数据，但不会影响后续 popup context。
  - memory operation 无 provenance，无法审计。
  - 现有 distillation/candidate 代码和当前 memory crate 状态不一致，导致误接线。
  - 测试只验证存储成功，没有验证上下文真的改变。

---

## 4. Product Experience Contract

Primary user object: **memory note produced from a lived desktop work session**。

User visible flow:

1. Cozmio 完成一次观察、弹窗、用户反馈或执行结果记录。
2. 系统生成一份只含事实和来源的 consolidation packet。
3. 系统把本地模型自然输出、用户反馈或执行端结果作为 agent-authored memory material。
4. 系统校验来源并保存 memory note。
5. 下一次 popup prompt 中出现这条被召回的 memory note。

Required states:

- empty: 没有 active agent memory；popup context 只使用当前观察和旧 action log tail。
- pending: 已生成 consolidation packet，尚未应用 memory operation。
- running: 正在应用 memory operation 或生成 recall context。
- completed: memory operation 已保存，且可被 recall 查询到。
- failed: packet 生成、operation 校验、存储或 recall 任一步失败；错误必须可见。

Experience acceptance:

- 自动观察或调用 Stage 1 命令后，能看到一份包含 source refs 的 factual packet。
- 应用 memory operation 后，能列出 active memory note。
- 下一次构建 popup context 时，能看到 `recalled_memory` 自然语言片段。
- memory note 的来源能追到 action log 或 ledger event。

---

## 5. Agent Boundary

### Model Owns

- 读取 factual packet 后决定是否记忆。
- 写自然语言 memory body。
- 选择 memory layer: episode、reflection、procedure、hot_context_proposal。
- 在材料不足时输出 abstain。

### Code Owns

- 读取 action log、memory events、ledger events。
- 构建事实 packet，不解释语义。
- 校验 memory operation 的 source refs 存在。
- 存储 memory operation、memory note、consolidation run。
- 按 token budget 召回少量 active memory notes。
- 将召回片段拼入 popup context。

### User Owns

- 决定是否启用/禁用自动 memory flywheel。
- 提供或批准 agent 生成的 memory operation。
- 后续可 reject memory note。

### Executor Owns

- Stage 1 中 executor 不主动生成额外总结。
- 若执行端已有结果或错误文本，系统可把该自然语言结果作为 memory material，但不解释其语义。

### UI Owns

- Stage 1 不新增完整 UI。
- 已有前端可继续显示当前 popup/context 结果；新增 IPC 返回值用于调试。

---

## 6. Current Truth

- files inspected:
  - `docs/superpowers/specs/2026-05-07-agent-self-maintained-memory-flywheel-design.md:1` — 最新设计文档，定义 agent-owned memory flywheel。
  - `cozmio/src-tauri/src/human_memory.rs:21` — `load_human_context()` 读取 hot memory 文件。
  - `cozmio/src-tauri/src/human_memory.rs:28` — `write_human_context()` 直接覆盖写入 `human_context.md`。
  - `cozmio/src-tauri/src/human_memory.rs:90` — `build_write_request()` 用 observation signature 防重复。
  - `cozmio/src-tauri/src/human_memory.rs:126` — `build_memory_update_prompt()` 要求模型输出更新后的记忆文件全文。
  - `cozmio/src-tauri/src/main_loop.rs:189` — hot path 调用 `build_popup_context()`。
  - `cozmio/src-tauri/src/main_loop.rs:206` — popup context 被传入 `ModelClient::call_raw_with_context()`。
  - `cozmio/src-tauri/src/main_loop.rs:243` — model raw output 写入 action log。
  - `cozmio/src-tauri/src/prompt_context.rs:10` — `build_popup_context()` 从 action log tail 构建 local context。
  - `cozmio/src-tauri/src/model_client.rs:134` — prompt 拼接窗口、process context、local context。
  - `cozmio/src-tauri/src/logging.rs:128` — `ActionLogger::new()` 默认写 `%LOCALAPPDATA%/cozmio/action_log.jsonl`。
  - `cozmio/src-tauri/src/logging.rs:153` — `log_factual()` 将 factual action 转成 legacy ActionRecord。
  - `cozmio/src-tauri/src/ledger.rs:234` — `ledger_events` SQLite projection schema。
  - `cozmio/src-tauri/src/ledger.rs:304` — `query_trace()` 可按 trace 取事件。
  - `cozmio/src-tauri/src/ledger.rs:340` — `query_by_date_range()` 可按 timestamp range 取事件。
  - `cozmio/cozmio_memory/src/schema.rs:9` — 现有 `memory_events` 表。
  - `cozmio/cozmio_memory/src/schema.rs:25` — 现有 `context_slices` 表。
  - `cozmio/cozmio_memory/src/schema.rs:55` — 现有 `decision_memory` 表。
  - `cozmio/cozmio_memory/src/schema.rs:68` — 现有 `skill_memory` 表。
  - `cozmio/cozmio_memory/src/memory_events.rs:6` — `MemoryEvent` 事实记录类型。
  - `cozmio/cozmio_memory/src/competition.rs:35` — `MemoryCompetition::build_reminder_context()` 当前从 slices/events/decisions/skills 组装 context。
  - `cozmio/src-tauri/src/distill_commands.rs:443` — 存在 `create_memory_candidate()`，但当前主入口未注册该模块。
  - `cozmio/src-tauri/src/main.rs:1` — 当前 mod 列表不包含 `distill_commands`。
  - `cozmio/src-tauri/src/main.rs:139` — memory IPC 只注册 `memory_commands` 中的命令。
  - `cozmio/cozmio_memory/src/lib.rs:1` — 当前 crate exports 不包含 `MemoryCandidateStore` 或 `DistillationJobStore`。
  - `cozmio/cozmio_memory/Cargo.toml:1` — `cozmio_memory` 是独立 crate，默认 feature 不启用 vec/fastembed。

- existing entry points:
  - `build_popup_context(logger, window_title, process_name, process_context)` at `prompt_context.rs:10`.
  - `ModelClient::call_raw_with_context(snapshot, process_context, popup_context)` at `model_client.rs:93`.
  - `import_existing_logs()` at `memory_commands.rs:178`.
  - `search_memory()` at `memory_commands.rs:287`.
  - `build_activity_context()` at `memory_commands.rs:330`.
  - `get_memory_stats()` at `memory_commands.rs:120`.

- existing runtime path:
  - `main_loop::start_main_loop`
  - captures current window
  - builds `popup_context`
  - calls local model
  - logs raw output
  - creates pending confirmation if output is non-empty

- known inconsistencies:
  - `distill_commands.rs` contains useful distillation ideas, but it is not registered in `main.rs`.
  - `distill_commands.rs` imports `MemoryCandidate`, `MemoryCandidateStore`, `DistillationJob`, and `DistillationJobStore`, but inspected `cozmio_memory/src` does not currently define or export those symbols.
  - `cozmio_memory` compiles by itself with `cargo check -p cozmio_memory`.
  - `cargo check -p cozmio` from the workspace timed out at 120 seconds during this planning pass; app-level compile status is not confirmed in this plan.

---

## 7. Key Path Tracing

Stage 1 write path:

```text
automatic hot-path hook / IPC
  -> collect recent factual records from action_log and memory_events
  -> optionally collect ledger events by trace_id or timestamp range
  -> build ConsolidationPacket
  -> local model raw output / user feedback / executor result becomes MemoryOperation input
  -> apply_memory_operation validates source refs
  -> insert into agent_memories and memory_operations
```

Stage 1 read path:

```text
main_loop::start_main_loop
  -> prompt_context::build_popup_context
  -> recall_active_agent_memories(current window + recent action text, token budget)
  -> append natural-language recalled_memory block
  -> ModelClient::call_raw_with_context
```

Validation path:

```text
test creates temp action log and temp memory db
  -> prepare_consolidation_packet
  -> apply valid memory operation
  -> build popup context
  -> assert recalled memory text appears
```

---

## 8. Implementation Order

```text
Slice 1: Memory storage foundation
    ↓
Slice 2: Factual automatic consolidation packet
    ↓
Slice 3: Apply automatic agent-written memory operation
    ↓
Slice 4: Recall active memories into popup context
    ↓
Slice 5: Automatic debug IPC and verification assets
```

---

## Slice 1: Memory Storage Foundation

**用户可见结果**：开发者可以保存一条 agent-written memory note，并且它带生命周期、layer、provenance 和 source refs。

### 涉及文件

- `cozmio/cozmio_memory/src/schema.rs`
- `cozmio/cozmio_memory/src/lib.rs`
- 新增 `cozmio/cozmio_memory/src/agent_memory.rs`

### 数据来源

- memory body 来自 agent-written operation。
- source refs 来自 action log、memory event、ledger event。
- lifecycle state 由代码存储为 technical state，不表达用户语义。

### RP-1: Add agent memory tables

**文件**: `cozmio/cozmio_memory/src/schema.rs:3`

**当前真相**:

`run_migrations()` 只创建 `memory_events`、`context_slices`、`task_threads`、`decision_memory`、`skill_memory` 和 `memory_events_fts`。

**修改为**:

在 `run_migrations()` 中新增两个表：

```sql
CREATE TABLE IF NOT EXISTS agent_memories (
    memory_id TEXT PRIMARY KEY,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    layer TEXT NOT NULL,
    lifecycle TEXT NOT NULL,
    body TEXT NOT NULL,
    source_refs TEXT NOT NULL,
    supersedes TEXT,
    last_used_at INTEGER,
    use_count INTEGER DEFAULT 0,
    producer TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS memory_operations (
    operation_id TEXT PRIMARY KEY,
    created_at INTEGER NOT NULL,
    operation_type TEXT NOT NULL,
    target_memory_id TEXT,
    resulting_memory_id TEXT,
    body TEXT,
    source_refs TEXT NOT NULL,
    status TEXT NOT NULL,
    error_text TEXT,
    producer TEXT NOT NULL
);
```

Allowed `layer` values:

- `episode`
- `reflection`
- `procedure`
- `hot_context_proposal`

Allowed `lifecycle` values:

- `draft`
- `active`
- `superseded`
- `rejected`
- `expired`
- `archived`

Allowed `operation_type` values:

- `remember_episode`
- `remember_reflection`
- `remember_skill`
- `update_hot_context`
- `remove_or_supersede`
- `abstain`

These value sets are storage enums represented as strings. They do not encode user intent.

**验证**:

- `cargo test -p cozmio_memory -- agent_memory_schema`
- Assert tables exist after `Database::new()` + `run_migrations()`.
- Assert insert/select works for one active episode memory.

**事实依据**:

- `schema.rs:3` central migration function.
- `db.rs:8` `Database::new()` opens SQLite connection without automatically forcing semantic behavior.

**状态**: 已锁定 ✓

### RP-2: Add AgentMemoryStore

**文件**: 新增 `cozmio/cozmio_memory/src/agent_memory.rs`

**当前真相**:

No inspected file provides storage for agent-owned natural-language memories with lifecycle and source refs.

**修改为**:

Create:

```rust
pub struct AgentMemory {
    pub memory_id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub layer: String,
    pub lifecycle: String,
    pub body: String,
    pub source_refs: Vec<String>,
    pub supersedes: Option<String>,
    pub last_used_at: Option<i64>,
    pub use_count: i64,
    pub producer: String,
}

pub struct MemoryOperation {
    pub operation_id: String,
    pub created_at: i64,
    pub operation_type: String,
    pub target_memory_id: Option<String>,
    pub resulting_memory_id: Option<String>,
    pub body: Option<String>,
    pub source_refs: Vec<String>,
    pub status: String,
    pub error_text: Option<String>,
    pub producer: String,
}
```

Implement:

- `AgentMemoryStore::insert_memory(&AgentMemory) -> Result<(), MemoryError>`
- `AgentMemoryStore::insert_operation(&MemoryOperation) -> Result<(), MemoryError>`
- `AgentMemoryStore::list_active(limit: usize) -> Result<Vec<AgentMemory>, MemoryError>`
- `AgentMemoryStore::get(memory_id: &str) -> Result<Option<AgentMemory>, MemoryError>`
- `AgentMemoryStore::reject(memory_id: &str) -> Result<(), MemoryError>`
- `AgentMemoryStore::mark_used(memory_id: &str, timestamp: i64) -> Result<(), MemoryError>`

Serialization rule:

- `source_refs` stored as JSON array string.
- `body` stored exactly as model/user provided, after trimming empty outer whitespace.

**验证**:

- `cargo test -p cozmio_memory -- agent_memory_store`
- Assert `source_refs` round-trip exactly.
- Assert `reject()` changes lifecycle to `rejected`.
- Assert `mark_used()` increments `use_count` and sets `last_used_at`.

**事实依据**:

- `decision_memory.rs` and `skill_memory.rs` are simple store patterns.
- `memory_events.rs:20` shows store wrapper style.

**状态**: 已锁定 ✓

### RP-3: Re-export agent memory types

**文件**: `cozmio/cozmio_memory/src/lib.rs:1`

**当前真相**:

`lib.rs` re-exports existing memory stores, but no agent memory module exists.

**修改为**:

- Add `pub mod agent_memory;`
- Re-export `AgentMemory`, `AgentMemoryStore`, and `MemoryOperation`.

**验证**:

- `cargo check -p cozmio_memory`
- Test module imports `cozmio_memory::{AgentMemory, AgentMemoryStore, MemoryOperation}`.

**事实依据**:

- `lib.rs:1` module declarations.
- `lib.rs:30` existing re-export pattern.

**状态**: 已锁定 ✓

### 不做什么

- 不迁移 existing `decision_memory` 或 `skill_memory` 数据。
- 不接 `distill_commands.rs`。
- 不添加 vector index 到 `agent_memories`。

---

## Slice 2: Factual Consolidation Packet

**用户可见结果**：开发者可以生成一份 readable factual packet，交给 agent 写记忆；packet 不包含代码编造的语义结论。

### 涉及文件

- 新增 `cozmio/src-tauri/src/memory_consolidation.rs`
- `cozmio/src-tauri/src/main.rs`
- `cozmio/src-tauri/src/commands.rs` only if `AppState` access requires helper reuse

### 数据来源

- `%LOCALAPPDATA%/cozmio/action_log.jsonl` via `ActionLogger`.
- `%LOCALAPPDATA%/cozmio/memory/cozmio.db` via `MemoryEventsStore`.
- Ledger events via `LedgerManager` when trace or timestamp range is provided.

### RP-4: Add consolidation packet types

**文件**: 新增 `cozmio/src-tauri/src/memory_consolidation.rs`

**当前真相**:

`distill_commands.rs` has `DistillationMaterial`, but it is not active runtime code because `main.rs` does not register `distill_commands`.

**修改为**:

Create runtime-owned packet types:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationSource {
    pub source_ref: String,
    pub timestamp: i64,
    pub source_kind: String,
    pub window_title: Option<String>,
    pub process_name: Option<String>,
    pub factual_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationPacket {
    pub packet_id: String,
    pub created_at: i64,
    pub source_range_label: String,
    pub sources: Vec<ConsolidationSource>,
    pub existing_hot_context: String,
    pub instructions: String,
}
```

`factual_text` may include raw model text, user feedback, executor result, and error text. It must not include code-owned interpretation such as "user intent" or "workflow stage".

**验证**:

- `cargo test -p cozmio -- memory_consolidation_packet`
- Assert packet contains source refs.
- Assert packet includes existing hot context.
- Assert packet builder does not add forbidden phrases listed in the test: `user is stuck`, `workflow stage`, `student discount intent`, `project iteration`.

**事实依据**:

- `logging.rs:6` legacy `ActionRecord` fields.
- `ledger.rs:26` `LedgerEvent` preserves factual fields and raw text.
- `human_memory.rs:21` loads existing hot context.

**状态**: 已锁定 ✓

### RP-5: Build packet from action log tail

**文件**: `cozmio/src-tauri/src/memory_consolidation.rs`

**当前真相**:

`prompt_context.rs:10` can read recent action log tail for popup, but no function builds a reviewable packet for memory consolidation.

**修改为**:

Implement:

```rust
pub fn prepare_consolidation_packet_from_recent_actions(
    logger: &ActionLogger,
    limit: usize,
    max_tail_bytes: u64,
    existing_hot_context: String,
) -> Result<ConsolidationPacket, String>
```

Rules:

- Use `ActionLogger::get_recent_tail()`.
- Default caller limit should be 20 records.
- Default caller tail budget should be 128 KiB.
- Each action source ref format: `action_log:<timestamp>:<trace_id-or-no-trace>`.
- Include `window_title`, `content_text`, `result_text`, `error_text`, `user_feedback`, `model_name`, `call_duration_ms` when present.
- Clip each source factual text to 800 chars.
- Preserve raw model output as raw text, not parsed fields.

**验证**:

- Test with temp action log containing confirmed, dismissed, and error records.
- Assert packet sources are newest-first or timestamp-descending consistently.
- Assert no source factual text exceeds 800 chars.

**事实依据**:

- `logging.rs:241` `get_recent_tail()` exists for bounded tail reads.
- `prompt_context.rs:6` uses 64 KiB tail for popup; consolidation can use larger 128 KiB outside the tight popup budget.

**状态**: 已锁定 ✓

### RP-6: Add automatic prepare IPC

**文件**:

- `cozmio/src-tauri/src/memory_consolidation.rs`
- `cozmio/src-tauri/src/main.rs`

**当前真相**:

`main.rs:139` registers memory IPC from `memory_commands`, but there is no automatic consolidation IPC.

**修改为**:

Add Tauri command:

```rust
#[tauri::command]
pub fn prepare_memory_consolidation(
    app: tauri::AppHandle,
    limit: Option<usize>,
) -> Result<ConsolidationPacket, String>
```

Behavior:

- Resolve `AppState` logger if available through existing state; otherwise construct `ActionLogger::new()`.
- Load hot context using `human_memory::load_human_context()`.
- Call `prepare_consolidation_packet_from_recent_actions`.
- Return packet JSON to frontend/devtools.

Registration:

- Add `mod memory_consolidation;` in `main.rs`.
- Register `prepare_memory_consolidation` in `generate_handler!`.

**验证**:

- `cargo check -p cozmio`
- Unit test packet builder.
- Manual IPC registration can be verified by compile-time generate_handler registration.

**事实依据**:

- `main.rs:139` handler list.
- `memory_commands.rs` shows command style for memory IPC.

**状态**: 已锁定 ✓

### 不做什么

- 不 call local model automatically.
- 不 write memory in this slice.
- 不 read screenshots.

---

## Slice 3: Apply Agent-Written Memory Operation

**用户可见结果**：agent 写出的自然语言 memory operation 可以被代码校验并保存；无效来源会被拒绝。

### 涉及文件

- `cozmio/src-tauri/src/memory_consolidation.rs`
- `cozmio/cozmio_memory/src/agent_memory.rs`
- `cozmio/src-tauri/src/main.rs`

### 数据来源

- operation body 来自 agent/user。
- source refs 必须来自 packet 中的 refs。

### RP-7: Define memory operation input DTO

**文件**: `cozmio/src-tauri/src/memory_consolidation.rs`

**当前真相**:

No active command accepts agent-written memory operations.

**修改为**:

Add IPC DTO:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryOperationInput {
    pub operation_type: String,
    pub layer: Option<String>,
    pub body: Option<String>,
    pub source_refs: Vec<String>,
    pub target_memory_id: Option<String>,
    pub producer: Option<String>,
}
```

Validation:

- `operation_type` must be one of the six allowed values from the design.
- `layer` required for remember operations.
- `body` required for remember operations and hot context proposals.
- `source_refs` must be non-empty except for `abstain`; abstain still records packet id when available.
- `producer` defaults to `local-popup-agent`.

This DTO is a tool protocol. It does not force the model to reason in fields; it only lets code store a chosen operation.

**验证**:

- `cargo test -p cozmio -- memory_operation_validation`
- Invalid operation type rejected.
- Empty body for `remember_episode` rejected.
- `abstain` with reason body accepted when no memory is created.

**事实依据**:

- Design doc `Memory Operations` section defines operation set.
- Agent boundary permits structured tool parameters while preserving natural-language memory body.

**状态**: 已锁定 ✓

### RP-8: Validate source refs

**文件**: `cozmio/src-tauri/src/memory_consolidation.rs`

**当前真相**:

`distill_commands.rs` validates source event ids for backend responses, but Stage 1 does not use that inactive module.

**修改为**:

Implement:

```rust
fn validate_source_refs_against_packet(
    input_refs: &[String],
    packet: &ConsolidationPacket,
) -> Result<(), String>
```

Rules:

- Every input ref must exactly match one `ConsolidationSource.source_ref` in the packet.
- Duplicate refs are allowed but normalized to one copy before storage.
- Unknown ref returns error beginning with `Unknown source_ref:`.

**验证**:

- Test valid refs pass.
- Test unknown ref fails.
- Test duplicate refs are stored once.

**事实依据**:

- `distill_commands.rs:2179` tests provenance preservation for content refs; Stage 1 mirrors the same intent without depending on inactive code.

**状态**: 已锁定 ✓

### RP-9: Apply operation into AgentMemoryStore

**文件**: `cozmio/src-tauri/src/memory_consolidation.rs`

**当前真相**:

No command stores agent-owned memory notes with lifecycle.

**修改为**:

Add:

```rust
pub fn apply_memory_operation_impl(
    db: &Database,
    packet: &ConsolidationPacket,
    input: MemoryOperationInput,
) -> Result<AgentMemory, String>
```

Behavior:

- For `remember_episode`, `remember_reflection`, and `remember_skill`:
  - create `AgentMemory` with lifecycle `active`;
  - map `remember_skill` layer to `procedure`;
  - write corresponding `MemoryOperation` with status `accepted`.
- For `update_hot_context`:
  - create memory with layer `hot_context_proposal`, lifecycle `draft`;
  - do not write `human_context.md` in Stage 1.
- For `remove_or_supersede`:
  - require `target_memory_id`;
  - mark target lifecycle as `superseded`;
  - create replacement only if `body` is non-empty.
- For `abstain`:
  - write `MemoryOperation` with status `accepted`;
  - return a no-memory JSON result through IPC instead of creating `AgentMemory`.

IPC wrapper:

```rust
#[tauri::command]
pub fn apply_memory_operation(
    packet: ConsolidationPacket,
    input: MemoryOperationInput,
) -> Result<serde_json::Value, String>
```

**验证**:

- Apply episode creates active memory.
- Apply reflection creates active memory.
- Apply hot context proposal creates draft memory and does not change `human_context.md`.
- Apply abstain creates operation but no active memory.
- Unknown source ref fails before insert.

**事实依据**:

- `Database::memory_dir()` in `db.rs:16` defines memory DB location.
- Existing memory command pattern opens `cozmio.db` in `memory_commands.rs:112`.

**状态**: 已锁定 ✓

### 不做什么

- 不让代码总结 memory body。
- 不自动编辑 `human_context.md`。
- 不实现 full memory conflict resolution beyond supersede target.

---

## Slice 4: Recall Active Memories Into Popup Context

**用户可见结果**：已保存的 active memory note 会作为自然语言 prior experience 出现在下一轮 popup prompt 中。

### 涉及文件

- `cozmio/src-tauri/src/prompt_context.rs`
- `cozmio/cozmio_memory/src/agent_memory.rs`
- `cozmio/src-tauri/src/main_loop.rs` only if function signature needs memory db path injection

### 数据来源

- Current window title and process name.
- Recent action log tail.
- Active agent memories.

### RP-10: Add simple recall admission for active agent memories

**文件**: `cozmio/cozmio_memory/src/agent_memory.rs`

**当前真相**:

No active agent memory recall exists.

**修改为**:

Implement:

```rust
pub fn recall_active_by_text(
    &self,
    query_text: &str,
    limit: usize,
) -> Result<Vec<AgentMemory>, MemoryError>
```

Stage 1 ranking:

- Use simple case-insensitive substring score over `body`.
- Add small recency tiebreaker from `updated_at`.
- Return only lifecycle `active`.
- Default caller limit is 3.

This is deliberately not semantic ranking. It is a deterministic admission helper to prove the end-to-end path. Vector admission can replace it later.

**验证**:

- Active memory matching query is returned.
- Rejected memory is not returned.
- Result limit respected.

**事实依据**:

- `competition.rs:35` already builds context from stores.
- Design says vector belongs to recall admission, not judgment; Stage 1 can use simpler recall first.

**状态**: 已锁定 ✓

### RP-11: Append recalled memory block to popup context

**文件**: `cozmio/src-tauri/src/prompt_context.rs:10`

**当前真相**:

`build_popup_context()` includes process context, current window, and action log tail. It does not include agent-written memory.

**修改为**:

Add helper:

```rust
fn format_recalled_memories(memories: &[AgentMemory], max_chars: usize) -> Option<String>
```

Add recall inside `build_popup_context()`:

- Open memory DB from `Database::memory_dir().join("cozmio.db")`.
- If DB open or migration fails, omit recalled memory and log warning.
- Build query from current window title, process name, and last included action texts.
- Call `AgentMemoryStore::recall_active_by_text(&query, 3)`.
- Append block:

```text
recalled_memory:
- <natural language memory body> [source: <memory_id>]
```

Budget:

- Entire recalled memory block clipped to 800 chars.
- Existing `MAX_CONTEXT_CHARS` remains the final cap.

**验证**:

- Existing `prompt_context` tests still pass.
- New test with temp memory DB verifies `recalled_memory:` appears.
- New test with missing DB verifies context still builds.

**事实依据**:

- `prompt_context.rs:4` constants define context budget.
- `prompt_context.rs:85` currently compacts action records.

**状态**: 已锁定 ✓

### 不做什么

- 不 change popup decision contract.
- 不 add cooldown.
- 不 let code decide popup permission from recalled memory.

---

## Slice 5: Manual Debug IPC And Verification Assets

**用户可见结果**：开发者可以准备 packet、应用 operation、列出 memories，并验证 popup context 已改变。

### 涉及文件

- `cozmio/src-tauri/src/memory_consolidation.rs`
- `cozmio/src-tauri/src/main.rs`
- `cozmio/feature_list.json`
- `cozmio/verification/last_result.json`

### 数据来源

- Stored agent memories.
- Consolidation packet.
- Unit test fixtures.

### RP-12: Add list active memories IPC

**文件**: `cozmio/src-tauri/src/memory_consolidation.rs`

**当前真相**:

`memory_commands.rs` exposes old decision/skill/memory events, but not new agent memories.

**修改为**:

Add:

```rust
#[tauri::command]
pub fn list_agent_memories(limit: Option<usize>) -> Result<Vec<serde_json::Value>, String>
```

Behavior:

- Opens memory DB.
- Lists active memories by updated time descending.
- Default limit 50.
- Returns memory id, layer, lifecycle, body, source refs, producer, use count, last used time.

**验证**:

- Unit test store list ordering.
- `cargo check -p cozmio` confirms command registration.

**事实依据**:

- `memory_commands.rs:120` command pattern for memory stats.
- `distill_commands.rs:904` old candidate listing pattern is useful but inactive.

**状态**: 已锁定 ✓

### RP-13: Register commands

**文件**: `cozmio/src-tauri/src/main.rs:1`

**当前真相**:

`main.rs` declares `mod memory_commands;` and imports memory commands into `generate_handler!`.

**修改为**:

- Add `mod memory_consolidation;`
- Import:
  - `prepare_memory_consolidation`
  - `apply_memory_operation`
  - `list_agent_memories`
- Register all three in `tauri::generate_handler!`.

**验证**:

- `cargo check -p cozmio`
- Compile fails if any command signature is not serializable.

**事实依据**:

- `main.rs:139` handler registration list.

**状态**: 已锁定 ✓

### RP-14: Add verification tests and result record

**文件**:

- Rust tests in touched modules.
- `cozmio/feature_list.json`
- `cozmio/verification/last_result.json`

**当前真相**:

`cozmio/feature_list.json` exists and tracks feature verification status. No `verification/last_result.json` exists.

**修改为**:

Add or update feature entry:

```json
{
  "id": "MEMORY-FLYWHEEL-STAGE1",
  "category": "agent-memory",
  "title": "Agent-owned automatic memory consolidation",
  "type": "agent_system",
  "description": "Manual factual packet -> agent memory operation -> active memory recall into popup context",
  "verification_script": "cargo test -p cozmio_memory -- agent_memory && cargo test -p cozmio -- memory_consolidation",
  "status": "pending",
  "passes": false,
  "last_verification": null,
  "notes": "Stage 1 automatic small flywheel implementation pending"
}
```

After implementation verification, create `cozmio/verification/last_result.json` with:

```json
{
  "feature_id": "MEMORY-FLYWHEEL-STAGE1",
  "status": "pass",
  "commands": [
    "cargo test -p cozmio_memory -- agent_memory",
    "cargo test -p cozmio -- memory_consolidation",
    "cargo check -p cozmio"
  ],
  "summary": "Manual consolidation path writes active agent memory and recalls it into popup context."
}
```

During planning only, do not mark the feature pass.

**验证**:

- JSON parses.
- Feature entry exists exactly once.
- Verification result is created only after tests pass.

**事实依据**:

- `cozmio/feature_list.json` existing schema.
- Writing-plans bootstrap requires verification result after first verification.

**状态**: 已锁定 ✓

### 不做什么

- 不 create commit unless user explicitly asks.
- 不 mark feature pass during plan writing.

---

## 9. Risk → Verification Mapping

| Risk | Verification command | Expected result |
|------|----------------------|-----------------|
| Code owns semantic memory meaning | `cargo test -p cozmio -- memory_consolidation_packet` | Packet contains factual fields and does not inject forbidden semantic phrases |
| Memory without provenance | `cargo test -p cozmio -- memory_operation_validation` | Unknown source refs rejected before insert |
| Memory saved but not recalled | `cargo test -p cozmio -- prompt_context` | `recalled_memory:` appears when active memory matches context |
| Rejected memories still used | `cargo test -p cozmio_memory -- agent_memory_store` | lifecycle `rejected` is excluded from recall |
| Missing DB breaks popup context | `cargo test -p cozmio -- prompt_context_missing_memory_db` | context builds without recalled memory and no panic |
| Existing memory crate regresses | `cargo check -p cozmio_memory` | crate compiles |
| App command registration broken | `cargo check -p cozmio` | app compiles with registered commands |
| Plan accidentally depends on inactive distill code | `cargo test -p cozmio -- memory_consolidation_packet` | tests use `memory_consolidation`, not `distill_commands` |

---

## 10. Verification Commands

Run from workspace root:

```bash
cd D:/C_Projects/Agent/cozmio/cozmio
cargo test -p cozmio_memory -- agent_memory
cargo test -p cozmio -- memory_consolidation
cargo test -p cozmio -- prompt_context
cargo check -p cozmio_memory
cargo check -p cozmio
```

Known planning-time verification:

- `cargo check -p cozmio_memory` passed on 2026-05-07 from `D:/C_Projects/Agent/cozmio/cozmio`.
- `cargo check -p cozmio` timed out at 120 seconds during planning; implementation verification must rerun it with a longer timeout.

---

## 11. Plan Quality Checks

### A. Execution Judgment

- Every new file is named.
- Every runtime entry point is named.
- Every command added to Tauri is named.
- Distill orphan code is explicitly excluded from Stage 1.

### B. Truth Check

- Existing entry points were inspected.
- Existing memory schema was inspected.
- Existing action log path was inspected.
- `cozmio_memory` compile status was checked.

### C. Key Path Check

- Write path is packet -> operation -> store.
- Read path is store -> recall -> popup context.
- Validation path checks both storage and prompt context effect.

### D. Semantic Boundary Check

- Code records and retrieves facts.
- Model writes memory body.
- Code validates provenance and lifecycle only.
- Vector search is not used as semantic authority.

### E. Scope Check

- Stage 1 proves the flywheel automatically on small safe events.
- Automation, full UI, cloud routing, vector refactor, and hot context rewriting are later stages.

### F. Freeze Check

- All RP items are marked `已锁定 ✓`.
- No implementation step requires the executor to decide whether to use `distill_commands`.
- No implementation step requires the executor to invent schema names.

---

## 12. Next Step After Approval

Execute this plan as Stage 1 automatic small flywheel. Because the write sets are fairly clear, it can be implemented in one session. Parallel worker split is possible only if workers have disjoint ownership:

- Worker A: `cozmio_memory/src/agent_memory.rs`, `schema.rs`, `lib.rs`.
- Worker B: `src-tauri/src/memory_consolidation.rs`, `main.rs`.
- Main/integrator: `prompt_context.rs`, verification, feature list, and final build.

Do not start implementation until this plan is approved.
