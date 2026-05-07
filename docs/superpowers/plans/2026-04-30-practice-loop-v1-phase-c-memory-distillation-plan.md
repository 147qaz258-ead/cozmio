# Practice Loop V1 — Phase C: Memory Distillation 实施方案

> **智能执行体须知**：本方案以运行效果、验证资产和飞轮写回为中心。默认不含代码示例。

## 1. Flywheel Context

- active task: `PRACTICE-LOOP-V1`
- current phase: Phase C — Memory Distillation
- latest verification: Phase B ✅ (build pass, 90 tests pass pre-disk-full)
- blocker (if any): None
- next expected step: Phase D — Vector Memory Competition

## 2. Goal

从 Event Ledger 中的真实 execution trace 生成记忆候选项（MemoryCandidate），带完整 provenance（source_event_ids、producer、importance_signal）；支持手动「提炼所选 trace/日期」操作；将候选项存储并使其可供后续 Vector Memory Competition 使用。

## 3. Product Type

- type: `deterministic_software` + `model_output_validated`
- core risk: 记忆候选项 provenance 链路断裂、distillation 调用模型的质量控制
- verification style: cargo build + unit tests + ledger trace evidence

## 4. Global Roadmap

| Phase | 目标 | 依赖 | 验收意图 |
|-------|------|------|---------|
| H1: Phase A | Ledger Foundation (JSONL + SQLite + Content Store) | — | ✅ PASS |
| H2: Phase B | Execution Return Stabilization | Phase A | ✅ PASS |
| H3: Phase C | Memory Distillation | Phase A+B | relay 结果/错误始终返回，带完整 trace_id；结果可查 |
| H4: Phase D | Vector Memory Competition | Phase C | 向量搜索选记忆；context pack 预览可用 |
| H5: Phase E | Practice Dashboard | Phase A+B | Loop 时间线、记忆收件箱、效果信号可见于 UI |
| H6: Phase F | Evaluation Loop | Phase C | 样本集评估；PASS/PARTIAL/FAIL 判定 |

## 5. Scope

### In（本次包含）

- 定义 `MemoryCandidate` struct（含 memory_id、created_at、producer、source_event_ids、source_paths、memory_text、memory_kind、embedding、importance_signal、last_selected_at、selection_count、supersedes、expires_at）
- 新增 `MemoryCandidatesStore`（SQLite 表 + CRUD）
- 新增 `distill_trace(trace_id)` IPC 命令：从 LedgerEvent 中提取 relay_dispatched → execution_result_received 链路，调用 configured command（relay-agent 或 script）生成 memory_text
- 新增 `distill_date(date: String)` IPC 命令：批量处理指定日期所有 execution trace
- 新增 `get_memory_candidates(limit)` IPC 命令：返回候选项列表（供 Phase D 使用）
- 新增 `approve_candidate(memory_id)` IPC 命令：将候选项标记为 approved（Phase D 即可用）
- LedgerManager 集成：在 `record_event` 时，如果 event_type 是 `execution_result_received` 且 session 成功，自动生成 pending candidate（不自动调用模型，仅创建 record with empty memory_text，等待手动 distill）
- `importance_signal` 初始值：factual（user_confirmed = high, relay_result = medium）或 model-produced

### Out（本次不包含）

- 自动调度 distillation（属于 Phase C 后期或 Phase D）
- Vector memory competition 逻辑（Phase D）
- Memory 删除/过期逻辑（Phase D）
- Dashboard UI（Phase E）
- 执行 agent 结果直接写入 memory（由 Phase C distill command 统一处理）

## 6. Current Truth

- files inspected: `cozmio_memory/src/lib.rs`, `cozmio_memory/src/decision_memory.rs`, `cozmio_memory/src/competition.rs`, `src-tauri/src/ledger.rs`, `src-tauri/src/memory_commands.rs`, `src-tauri/src/relay_bridge.rs`
- 现有 memory 类型：
  - `Decision` (memory_type, content, evidence, related_thread_id, evidence_source) — 已有但 provenance 不足
  - `MemoryCore` + `MemoryCompetition` — 提供 `ReminderContext` 构建
  - `ContextSlice` — 已有 embedding 支持
- 现有 LedgerEvent 通过 Phase A/B 完整记录：relay_dispatched、execution_progress_received、execution_result_received、execution_error_received
- 现有 IPC：`get_execution_result(trace_id)` — 可查询单个 trace 的结果
- 现有 `cozmio_memory` 数据库：`%LOCALAPPDATA%/cozmio/memory/memory.db`
- Phase A 建立的 LedgerWriter 路径：`%LOCALAPPDATA%/cozmio/event-log/`
- cozmio_memory 已有 schema migrations 框架

## 7. Implementation Steps（Phase C）

### Step 1: 定义 MemoryCandidate Schema 并建表

在 `cozmio_memory/src/` 新增 `memory_candidate.rs`：

- `MemoryCandidate` struct：memory_id (UUID), created_at (i64), producer (String), source_event_ids (Vec<String>), source_paths (Vec<String>), memory_text (String), memory_kind (String enum), embedding (Option<Vec<f32>>), importance_signal (f32), last_selected_at (Option<i64>), selection_count (i32), supersedes (Option<String>), expires_at (Option<i64>), status (pending/approved/rejected)
- `MemoryCandidateStore` struct：封装 SQLite CRUD
- 在 `schema.rs` 中添加 `run_memory_candidate_migration()` 创建表 `memory_candidates`
- 表字段索引：memory_id (PK), created_at, status, memory_kind, importance_signal, last_selected_at

File: `cozmio_memory/src/memory_candidate.rs`, `cozmio_memory/src/lib.rs`, `cozmio_memory/src/schema.rs`

### Step 2: 实现 distill_trace IPC 命令

在 `src-tauri/src/` 新增 `distill_commands.rs`：

- `distill_trace(app, trace_id: String) -> Result<MemoryCandidate, String>`
- 通过 `LedgerManager::get_trace(trace_id)` 获取 relay_dispatched + execution_result_received 事件
- 提取：window_title, process_name, raw_text (execution result), session_id
- 调用 configured distillation command/script（从 config 读取 `distill_command`）
- 将 command 输出作为 memory_text 存入 MemoryCandidate
- 设置 source_event_ids = [所有相关 event_id]，producer = "distill-command"，memory_kind = "execution_memory"
- status = "pending"，importance_signal = 基于 execution success + user confirmation 信号

File: `src-tauri/src/distill_commands.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/main.rs`

### Step 3: 实现 distill_date IPC 命令

在 `distill_commands.rs` 新增：

- `distill_date(app, date: String) -> Result<Vec<MemoryCandidate>, String>`
- date 格式：YYYY-MM-DD
- 通过 `LedgerManager::get_timeline()` 查询指定日期范围的事件
- 过滤所有 execution_result_received 事件，按 trace_id 去重
- 对每个 trace_id 调用 distill_trace（串行）
- 返回所有生成的 MemoryCandidate 列表

File: `src-tauri/src/distill_commands.rs`

### Step 4: 实现 get_memory_candidates + approve_candidate IPC 命令

在 `distill_commands.rs` 或 `memory_commands.rs` 新增：

- `get_memory_candidates(limit: Option<usize>, status: Option<String>) -> Result<Vec<serde_json::Value>, String>` — 从 MemoryCandidateStore 查询候选项
- `approve_candidate(app, memory_id: String) -> Result<(), String>` — 将 status 从 "pending" 改为 "approved"；同时将 approved memory 写入 ContextSlice store（Phase D 即可用）

File: `src-tauri/src/distill_commands.rs`, `src-tauri/src/commands.rs`

### Step 5: LedgerManager auto-candidate 生成（可选，略过如果太复杂）

如果 `record_event` 收到 `execution_result_received` 且 `result.success == true`：

- 创建 pending MemoryCandidate（memory_text = empty）
- source_event_ids = [event_id], producer = "ledger-auto"
- importance_signal = 0.5（中等）
- status = "pending"

此步骤为可选，如果实现成本太高可跳过手动 distill。

### Step 6: 单元测试

在 `memory_candidate.rs` 的 `#[cfg(test)]` 模块添加：

- `test_memory_candidate_store_insert_and_get` — 插入候选，获取验证 fields
- `test_memory_candidate_store_update_status` — 插入后改 status 为 approved
- `test_distill_trace_extracts_events` — mock ledger events，验证提取逻辑
- `test_importance_signal_computation` — 验证 success + confirmation → importance_signal 计算

File: `cozmio_memory/src/memory_candidate.rs`, `src-tauri/src/distill_commands.rs`

## 8. Verification Asset

- verification type: `deterministic_software`
- command: `cargo build -p cozmio && cargo build -p cozmio_memory && cargo test -p cozmio_memory -- memory_candidate`
- expected evidence:
  - `cargo build` passes without errors
  - `memory_candidates` table created in memory.db
  - `distill_trace` 命令返回 MemoryCandidate（mock config 时返回错误，非失败）
  - `distill_date` 批量处理多个 trace
  - `get_memory_candidates` 返回候选项
  - `approve_candidate` 将候选项标记为 approved
- failure condition: build fails, table not created, IPC commands not registered
- writeback targets:
  - `verification/last_result.json`
  - `feature_list.json`
  - `claude-progress.txt`

## 9. Phase Gate

Phase C 只有满足以下条件才能标记为完成：

- [ ] `cargo build -p cozmio && cargo build -p cozmio_memory` 通过，无编译错误
- [ ] `cargo test -p cozmio_memory -- memory_candidate` 所有测试通过
- [ ] `distill_trace` IPC 命令存在且能处理 trace_id
- [ ] `distill_date` IPC 命令存在且能批量处理日期
- [ ] `get_memory_candidates` IPC 命令返回候选项
- [ ] `approve_candidate` IPC 命令能将候选项标记为 approved
- [ ] MemoryCandidate 包含完整 provenance 字段（source_event_ids, producer, memory_kind）
- [ ] Phase C 不实现 vector search 或 memory competition（属于 Phase D）
- [ ] `verification/last_result.json` 已更新
- [ ] `feature_list.json` 相关条目已更新 `PRACTICE-LOOP-V1-PHASE-C` 状态
- [ ] `claude-progress.txt` 已有下一轮交接内容

## 10. Next Execution Step

- next phase: Phase D — Vector Memory Competition
- goal: 向量搜索从 approved memories 中选择候选项；context pack 预览可用
- entry skill: `superpowers:subagent-driven-development`
- stop condition: Phase D 验证资产通过；selected memories 出现在 context pack 中
