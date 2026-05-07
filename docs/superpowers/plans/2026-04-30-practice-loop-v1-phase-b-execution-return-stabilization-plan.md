# Practice Loop V1 — Phase B: Execution Return Stabilization 实施方案

> **智能执行体须知**：本方案以运行效果、验证资产和飞轮写回为中心。默认不含代码示例。

## 1. Flywheel Context

- active task: `PRACTICE-LOOP-V1`
- current phase: Phase B — Execution Return Stabilization
- latest verification: Phase A ✅ (8d3aaee, 17 tests pass)
- blocker (if any): None
- next expected step: Phase C — Memory Distillation

## 2. Goal

确保 relay 执行结果和错误始终以完整 provenance（trace_id + session_id）返回 Cozmio；增加执行结果查看视图；对大输出附加 ContentRef。

## 3. Product Type

- type: `deterministic_software`
- core risk: 结果通知丢失、trace_id 链路断裂、大输出内存膨胀
- verification style: cargo build + unit tests + integration path verification

## 4. Global Roadmap

| Phase | 目标 | 依赖 | 验收意图 |
|-------|------|------|---------|
| H1: Phase A | Ledger Foundation (JSONL + SQLite + Content Store) | — | ✅ PASS |
| H2: Phase B | Execution Return Stabilization | Phase A | relay 结果/错误始终返回，带完整 trace_id；结果可查 |
| H3: Phase C | Memory Distillation | Phase A+B | 记忆候选项从真实 trace 生成，带 provenance |
| H4: Phase D | Vector Memory Competition | Phase C | 向量搜索选记忆；context pack 预览可用 |
| H5: Phase E | Practice Dashboard | Phase A+B | Loop 时间线、记忆收件箱、效果信号可见于 UI |
| H6: Phase F | Evaluation Loop | Phase C | 样本集评估；PASS/PARTIAL/FAIL 判定 |

## 5. Scope

### In（本次包含）

- 将 `send_result_notification` 接入 `EXECUTION_RESULT_RECEIVED` 路径（relay_bridge.rs）
- 新增 `get_execution_result` IPC 命令，通过 trace_id 或 session_id 查询执行结果
- 对大于阈值（4KB）的 `result_text` 使用 `ContentStoreWriter` 存储，仅在 LedgerEvent 中保留 content_ref
- 验证 relay 事件链路完整性：relay_dispatched → execution_progress_received → execution_result_received/error_received
- `EXECUTION_PROGRESS_RECEIVED` 事件中 `session_id` 字段补全（当前缺失）

### Out（本次不包含）

- Memory distillation（Phase C）
- Vector memory competition（Phase D）
- Practice Dashboard UI（Phase E）
- Relay 结果的自动重试逻辑
- `get_content` IPC 命令（Phase A 已明确不含）

## 6. Current Truth

- files inspected: `relay_bridge.rs`, `notification_manager.rs`, `commands.rs`, `ledger.rs`
- 现有事件记录：
  - `relay_bridge.rs:441` — `EXECUTION_RESULT_RECEIVED` ledger event 已记录（trace_id ✅, session_id ✅）
  - `relay_bridge.rs:480` — `EXECUTION_ERROR_RECEIVED` ledger event 已记录（trace_id ✅, session_id ✅）
  - `relay_bridge.rs:514` — `EXECUTION_ERROR_RECEIVED`（fetch 失败）ledger event 已记录
  - `notification_manager.rs:65` — `send_result_notification` 函数已存在但**未被调用**
- relay_bridge.rs 中 `EXECUTION_PROGRESS_RECEIVED` 事件在 line ~330，`session_id` 字段传值为 `snapshot.trace_id`（bug：应用了错误的字段）
- `result_text` 直接内联在 LedgerEvent.raw_text 中，无大小限制
- 现有 IPC：`get_timeline`, `get_trace_detail`, `get_trace_events`, `rebuild_ledger_projection`（Phase A）

## 7. Implementation Steps（Phase B）

### Step 1: 修复 progress 事件 session_id 错误 + 补充 notification 调用

在 `relay_bridge.rs` 中：

- 找到 `EXECUTION_PROGRESS_RECEIVED` 事件创建位置（约 line ~330）
- 将 `session_id: Some(snapshot.trace_id.clone())` 修正为 `session_id: Some(session_id.clone())`（snapshot.trace_id 是 request.trace_id，不是 session_id）
- 在 `EXECUTION_RESULT_RECEIVED` 路径中，ledger event 记录成功后，调用 `notification_manager::send_result_notification()`
- 参数映射：`trace_id` ← event.trace_id，`content_text` ← request.original_suggestion，`status` ← if result.success { "completed" } else { "failed" }，`result_text` ← snapshot.result_output，`error_text` ← snapshot.error_message

File: `relay_bridge.rs`

### Step 2: 实现 get_execution_result IPC 命令

在 `commands.rs` 中新增 `get_execution_result` 命令：

- 签名：`get_execution_result(trace_id: String) -> Result<serde_json::Value, String>`
- 通过 `LedgerManager::get_trace(trace_id)` 查询所有 `execution_result_received` 和 `execution_error_received` 事件
- 如果 `content_ref` 非空，通过 `ContentStoreWriter` 读取实际内容并替换回 `raw_text`
- 返回结构化结果：trace_id、session_id、status、result_text/error、content_ref、timestamp

File: `commands.rs`

### Step 3: 大输出 ContentRef 写入路径

修改 `relay_bridge.rs` 中 `EXECUTION_RESULT_RECEIVED` 事件创建逻辑：

- 在记录 ledger event 之前，检查 `snapshot.result_output` 长度
- 如果 > 4096 bytes：调用 `ledger_manager.record_event_with_content()`，将内容写入 content store，返回的 LedgerEvent 中 `raw_text` 置为 `None`（或截断摘要），`content_ref` 填充
- 如果 ≤ 4096 bytes：保持现有逻辑（raw_text 内联）

同时修改 Step 2 中 `get_execution_result` 的 content_ref 解析逻辑：当 `raw_text` 为空但 `content_ref` 非空时，从 content store 加载。

File: `relay_bridge.rs`, `commands.rs`

### Step 4: 验证 progress 事件链路

检查 `EXECUTION_PROGRESS_RECEIVED` 事件中 `trace_id` 来源是否正确（应为 `request.trace_id`，不是 `snapshot.trace_id`）。如果错误，修复。

File: `relay_bridge.rs`

### Step 5: 单元测试

在 `ledger.rs` 或 `relay_bridge.rs` 的 `#[cfg(test)]` 模块中添加：

- `test_execution_result_notification_wired` — 验证 result + error 事件触发通知（mock notification_manager）
- `test_large_result_stored_as_content_ref` — 验证 > 4KB 结果通过 content store 存储
- `test_get_execution_result_loads_content_ref` — 验证 IPC 命令能还原 content_ref 内容
- `test_progress_event_session_id_correct` — 验证 progress 事件中 session_id 正确

File: `relay_bridge.rs` (inline test module)

## 8. Verification Asset

- verification type: `deterministic_software`
- command: `cargo build -p cozmio && cargo test -p cozmio -- relay`
- expected evidence:
  - `cargo build` passes without errors
  - Unit tests pass
  - `send_result_notification` 在 result/error 路径中被调用（通过代码审查确认调用点存在）
  - `get_execution_result` IPC 命令注册成功
  - `session_id` 在 progress/result/error 所有事件中正确填充
  - 大输出结果存储到 content-store 而非内联
- failure condition: build fails, tests fail, or relay result path missing notification
- writeback targets:
  - `verification/last_result.json`
  - `feature_list.json`
  - `claude-progress.txt`

## 9. Phase Gate

Phase B 只有满足以下条件才能标记为完成：

- [ ] `cargo build -p cozmio` 通过，无编译错误
- [ ] `cargo test -p cozmio -- relay` 所有 relay 相关测试通过
- [ ] `send_result_notification` 在 `EXECUTION_RESULT_RECEIVED` 和 `EXECUTION_ERROR_RECEIVED` 路径中被调用
- [ ] `get_execution_result` IPC 命令存在且能查询结果
- [ ] 所有 relay 事件（progress/result/error）中 `session_id` 和 `trace_id` 均正确填充
- [ ] > 4KB 结果通过 ContentStoreWriter 存储，不内联在 raw_text 中
- [ ] Phase B 不实现 `get_content` IPC（属于 Phase C/D）
- [ ] `verification/last_result.json` 已更新
- [ ] `feature_list.json` 相关条目已更新 `PRACTICE-LOOP-V1-PHASE-B` 状态
- [ ] `claude-progress.txt` 已有下一轮交接内容

## 10. Next Execution Step

- next phase: Phase C — Memory Distillation
- goal: 从真实 execution trace 生成记忆候选项，带 provenance；记忆通过 Practice Loop 评估循环验证
- entry skill: `superpowers:subagent-driven-development`
- stop condition: Phase C 验证资产通过；记忆候选项出现在 ledger 中
