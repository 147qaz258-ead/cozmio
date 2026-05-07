# Practice Loop V1 — Phase A: Ledger Foundation 实施方案

> **智能执行体须知**：本方案以运行效果、验证资产和飞轮写回为中心。默认不含代码示例。

## 1. Flywheel Context

- active task: `PRACTICE-LOOP-V1` (new)
- current phase: Phase A — Ledger Foundation
- latest verification: N/A (new feature)
- blocker (if any): None
- next expected step: Phase B — Execution Return Stabilization

## 2. Goal

Build the Event Ledger layered storage foundation: canonical JSONL append-only event stream, SQLite query projection, and content-ref-ready large-content addressing. Phase A routes existing model output, pending confirmation, user actions, relay progress, and relay results through this ledger without breaking existing UI/history compatibility.

Phase A must stay focused on the ledger backbone. It must not pull Phase C/D responsibilities forward: no memory distillation, no vector search, no execution-agent content resolver, and no dashboard rebuild.

## 3. Product Type

- type: `deterministic_software`
- core risk: JSONL append correctness, SQLite ingest correctness, backward compatibility with existing `action_log.jsonl` importers
- verification style: `cargo build` + unit tests + integration path verification

## 4. Global Roadmap

| Phase | 目标 | 依赖 | 验收意图 |
|-------|------|------|---------|
| H1: Phase A | Ledger Foundation (JSONL + SQLite + Content Store) | — | JSONL appends correctly; SQLite projection queryable; legacy action_log.jsonl readers still work |
| H2: Phase B | Execution Return Stabilization | Phase A | Relay results and errors always return to Cozmio with trace_id linkage |
| H3: Phase C | Memory Distillation | Phase A+B | Memory candidates generated from real traces with provenance |
| H4: Phase D | Vector Memory Competition | Phase C | Vector search selects memories; context pack preview works |
| H5: Phase E | Practice Dashboard | Phase A+B | Loop timeline, memory inbox, effect signals visible in UI |

## 5. Scope

### In（本次包含）

- Canonical JSONL event stream at `%LOCALAPPDATA%/cozmio/event-log/YYYY-MM-DD.jsonl`
- SQLite projection database at `%LOCALAPPDATA%/cozmio/cozmio-ledger.sqlite`
- Content Store directory convention at `%LOCALAPPDATA%/cozmio/content-store/{year}/{month}/{content_ref}`
- `LedgerEvent` schema with required fields: `event_id`, `trace_id`, `session_id`, `timestamp`, `event_type`, `source`, `window_title`, `process_name`, `raw_text`, `content_ref`, `parent_event_id`, `metadata`
- `ContentRef` minimum structure: `content_ref`, `content_type`, `storage_backend`, `path_or_key`, `content_hash`, `created_at`, `producer`, optional `byte_range`, optional `line_range`
- New `ledger` module: JSONL writer, SQLite ingest/projection, content-ref struct and write-path helpers
- Migrate `log_task_action` and `store_error_judgment` to write `LedgerEvent` (with legacy `ActionRecord` conversion for backwards compat)
- `get_timeline`, `get_trace_detail` query APIs via SQLite projection
- Event types: `observation_captured`, `model_called`, `model_output_received`, `popup_displayed`, `pending_confirmation_created`, `user_confirmed`, `user_cancelled`, `user_dismissed`, `relay_dispatched`, `execution_progress_received`, `execution_result_received`, `execution_error_received`
- Backward compatibility: existing `action_log.jsonl` at `%LOCALAPPDATA%/cozmio/action_log.jsonl` continues to be written through the existing `ActionLogger` compatibility path
- UI/history compatibility: old `ActionRecord` JSONL format still readable

### Out（本次不包含）

- Memory distillation (Phase C)
- Vector memory competition (Phase D)
- Practice Loop Dashboard UI (Phase E)
- Evaluation loop (Phase F)
- Scheduled/manual distillation decision
- Execution-agent content resolver. Phase A records structured refs; resolver behavior is Phase B/C.
- `get_content` IPC command. Do not expose content reading before the resolver contract is designed.

## 6. Current Truth

- files inspected: `logging.rs`, `executor.rs`, `main_loop.rs`, `commands.rs`
- existing entry points:
  - `commands::log_task_action(app, ActionRecord)` — writes legacy `ActionRecord` to `action_log.jsonl`
  - `main_loop.rs:store_error_judgment(...)` — creates `ActionRecord` for capture/model errors
  - `relay_bridge.rs:356` — calls `log_task_action` for relay events
- existing runtime path:
  - `%LOCALAPPDATA%/cozmio/action_log.jsonl` — single JSONL file, append-only
  - `ActionLogger` in `logging.rs` with `log()`, `get_recent()`, `get_recent_tail()`
  - `FactualActionRecord` + `FactualEventType` + `SystemRoute` types already exist in `logging.rs`
  - `log_factual()` converts `FactualActionRecord` to legacy `ActionRecord` for backwards compat
- existing verification: `cargo build` + `cargo test` on affected crates

## 7. Implementation Steps（Phase A）

### Step 1: Define LedgerEvent Schema and ContentRef Structure

Create `ledger.rs` in `src-tauri/src/` with:

- `LedgerEvent` struct: `event_id` (UUID), `trace_id`, `session_id`, `timestamp`, `event_type` (string enum matching spec), `source`, `window_title`, `process_name`, `raw_text`, `content_ref` (Option), `parent_event_id` (Option), `metadata` (Map)
- `ContentRef` struct: `content_ref`, `content_type`, `storage_backend`, `path_or_key`, `content_hash`, `created_at`, `producer`, optional `byte_range`, optional `line_range`
- `ContentRef` is only a structured address in Phase A. Do not implement an execution-agent resolver in this phase.

File: `src-tauri/src/ledger.rs`

### Step 2: Implement LedgerWriter (Canonical JSONL Append)

In `ledger.rs`, add `LedgerWriter`:

- `new(base_dir: PathBuf) -> Self` — creates `event-log/` subdirectory
- `append(event: LedgerEvent) -> Result<()>` — serialize to JSON line, append to `event-log/YYYY-MM-DD.jsonl`
- `get_event_path(date: &str) -> PathBuf` — returns path for date-based file
- Uses `serde_json::to_string(&event)?` then `writeln!(file, "{}", json)`
- File mode: `create + append`
- JSONL is canonical append-only. Do not deduplicate, rewrite, or compact JSONL in the writer.
- `event_id` UUID is used for identity and projection idempotency, not for mutating the canonical event stream.

File: `src-tauri/src/ledger.rs`

### Step 3: Implement LedgerProjection (SQLite Index)

In `ledger.rs`, add `LedgerProjection`:

- `new(db_path: PathBuf) -> Self` — opens or creates SQLite database
- `ingest(event: &LedgerEvent) -> Result<()>` — insert event fields into indexed SQLite table
- `ensure_schema()` — create table with indexes on `event_id`, `trace_id`, `session_id`, `event_type`, `timestamp`, `parent_event_id`
- `query_timeline(limit: usize, offset: usize) -> Result<Vec<LedgerEvent>>`
- `query_trace(trace_id: &str) -> Result<Vec<LedgerEvent>>`
- `query_by_session(session_id: &str) -> Result<Vec<LedgerEvent>>`
- `query_by_event_type(event_type: &str, limit: usize) -> Result<Vec<LedgerEvent>>`
- `rebuild_from_jsonl(jsonl_path: &Path) -> Result<usize>` — full reingest from JSONL for recovery
- Uses `rusqlite` with `Connection`
- SQLite projection may use `event_id` upsert/ignore semantics so projection rebuild is idempotent. This does not change JSONL append-only semantics.

File: `src-tauri/src/ledger.rs`

### Step 4: Implement ContentRef Write Helpers, Not Resolver

In `ledger.rs`, add content-ref support:

- `ContentRef` struct as defined in Step 1
- `ContentStoreWriter::store(content: &[u8], content_type: &str, producer: &str) -> Result<ContentRef>` may be implemented only as a write helper
- `store_path(content_ref: &str) -> PathBuf` — returns `{base_dir}/content-store/{year}/{month}/{content_ref}`
- Storage layout: `content-store/{year}/{month}/{content_ref}` with `.bin` extension for binary, `.txt` for text
- Do not implement `resolve(...)` in Phase A.
- Do not expose `get_content` IPC in Phase A.
- Do not make execution agents parse `path_or_key`; agent-readable resolver design belongs to Phase B/C.

File: `src-tauri/src/ledger.rs`

### Step 5: Define LedgerManager (Facade)

In `ledger.rs`, add `LedgerManager`:

- `new(base_dir: PathBuf) -> Self` — initializes JSONL writer + SQLite projection + optional content store writer
- `record_event(event: LedgerEvent) -> Result<()>` — appends to JSONL, ingests to SQLite
- `record_event_with_content(event: LedgerEvent, content: &[u8]) -> Result<(LedgerEvent, ContentRef)>` — optional write-helper path only; stores content first, then records event with `content_ref`
- `get_timeline(limit: usize, offset: usize) -> Result<Vec<LedgerEvent>>` — via projection
- `get_trace(trace_id: &str) -> Result<Vec<LedgerEvent>>` — via projection
- `rebuild_projection() -> Result<usize>` — reingest all JSONL files into SQLite

File: `src-tauri/src/ledger.rs`

### Step 6: Add LedgerManager to AppState

Modify `commands.rs` and `main.rs`:

- Add `LedgerManager` to `AppState` struct in `commands.rs`
- Initialize in `main.rs` using `%LOCALAPPDATA%/cozmio/` as base
- Wire into existing runtime flow

File: `commands.rs`, `main.rs`

### Step 7: Migrate log_task_action to LedgerEvent

Replace `commands::log_task_action(app, ActionRecord)` with internal call to `LedgerManager::record_event`:

- Map `ActionRecord` fields to `LedgerEvent` (preserve all factual fields)
- `event_type`: `"model_output_received"` for model output events
- `raw_text`: from `content_text` or `message_text`
- `trace_id`, `window_title`, `process_name` directly mapped
- Continue writing legacy `ActionRecord` through the existing `ActionLogger` compatibility path for `cozmio_memory` importer compatibility
- `log_task_action` becomes a thin wrapper that calls `LedgerManager` and then writes legacy

File: `commands.rs`, `main_loop.rs`

### Step 8: Migrate store_error_judgment to LedgerEvent

Modify `store_error_judgment` in `main_loop.rs`:

- Create `LedgerEvent` with `event_type: "model_error"` or `"system_error"`
- `trace_id: None` for capture errors, use raw output trace_id for model errors
- Record via `LedgerManager`
- Still write legacy `ActionRecord` for compatibility

File: `main_loop.rs`

### Step 9: Migrate Relay Bridge Logging

Modify `relay_bridge.rs:356` call to `log_task_action`:

- Add event types: `relay_dispatched`, `execution_progress_received`, `execution_result_received`, `execution_error_received`
- Extract `session_id` from relay events and populate `session_id` field
- `trace_id` from dispatch context

File: `relay_bridge.rs`

### Step 10: Add Tauri IPC Commands for Ledger Queries

In `commands.rs`, add new commands:

- `get_timeline` — `tauri::command` that calls `LedgerProjection::query_timeline`
- `get_trace_detail` — `tauri::command` that calls `LedgerProjection::query_trace`
- `get_trace_events` — `tauri::command` for trace detail view
- `rebuild_ledger_projection` — `tauri::command` for recovery/reindex
- Keep existing `get_history` working (reads legacy `action_log.jsonl`)
- Do not add `get_content` yet.

File: `commands.rs`

### Step 11: Unit Tests

Add tests to `ledger.rs`:

- `test_append_and_read_event` — append event, read back
- `test_jsonl_append_only` — verify append-only semantics
- `test_ledger_projection_query_trace` — ingest events, query by trace_id
- `test_content_ref_store_writes_blob_and_ref` — store blob, verify ref fields and file existence; do not test resolver
- `test_ledger_projection_rebuild` — create events, rebuild from JSONL, verify counts
- `test_legacy_action_record_backwards_compat` — verify `log_factual` still produces valid legacy format

File: `src-tauri/src/ledger.rs` (inline `#[cfg(test)]` module)

## 8. Verification Asset

- verification type: `deterministic_software`
- command: `cargo build -p cozmio && cargo test -p cozmio -- ledger`
- expected evidence:
  - `cargo build` passes without errors
  - Unit tests in `ledger.rs` pass
  - `event-log/YYYY-MM-DD.jsonl` file created and appends events
  - `cozmio-ledger.sqlite` created with indexed tables
  - SQLite projection can be rebuilt from JSONL
  - Legacy `action_log.jsonl` still written and readable by `cozmio_memory` importer
  - `get_timeline` and `get_trace_detail` IPC commands return data
- evidence location: `target/debug/cozmio.exe` (build artifact), runtime files at `%LOCALAPPDATA%/cozmio/`
- failure condition: build fails, tests fail, or legacy action_log.jsonl no longer readable
- writeback targets:
  - `verification/last_result.json`
  - `feature_list.json`
  - `claude-progress.txt`

## 9. Phase Gate

Phase A 只有满足以下条件才能标记为完成：

- [ ] `cargo build -p cozmio` 通过，无编译错误
- [ ] `cargo test -p cozmio -- ledger` 所有 ledger 模块测试通过
- [ ] JSONL 文件在 `%LOCALAPPDATA%/cozmio/event-log/` 下正确追加（运行时验证）
- [ ] SQLite projection 在 `%LOCALAPPDATA%/cozmio/cozmio-ledger.sqlite` 可查询（运行时验证）
- [ ] 现有 `action_log.jsonl` 仍然可读（`cozmio_memory` 导入器兼容性）
- [ ] `get_timeline` 和 `get_trace_detail` IPC 命令返回 ledger data after ledger events exist
- [ ] Phase A does not expose execution-agent content resolver or `get_content`
- [ ] `verification/last_result.json` 已更新（含时间戳、验证类型、结果摘要）
- [ ] `feature_list.json` 相关条目已添加 `PRACTICE-LOOP-V1-PHASE-A` 状态
- [ ] `claude-progress.txt` 已有下一轮交接内容

## 10. Next Execution Step

- next phase: Phase B — Execution Return Stabilization
- goal: Ensure relay results and errors always return to Cozmio with proper trace_id/session_id linkage; add execution result view
- entry skill: `superpowers:subagent-driven-development`
- stop condition: Phase B verification assets pass; relay result events appear in ledger with full provenance
