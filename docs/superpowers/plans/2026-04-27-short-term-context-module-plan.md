# 短时过程上下文模块 实施方案

> **智能执行体须知**：本方案以运行效果、验证资产和飞轮写回为中心。

## 1. Flywheel Context

- **active task**: SHORT-TERM-CONTEXT-MODULE
- **current phase**: 设计方案完成，待实施
- **latest verification**: n/a（新任务）
- **blocker (if any)**: 无
- **next expected step**: 实现 ProcessBuffer + ProcessContext + 系统噪音清理

---

## 2. Goal

在 window_monitor 里新增短时 buffer（最多 20 条，约 60 秒），每次 poll 时计算行为事实（停留时长、切换次数、震荡检测）；在 main_loop 里增加独立的系统噪音清理步骤（仅限明确系统自噪声）；保持"模型判断"和"系统观察事实"的分离。

---

## 3. Product Type

- **type**: `deterministic_software`（Rust 结构、buffer、规则过滤）
- **core risk**: buffer 计算逻辑正确性、噪音规则命中准确性
- **verification style**: 编译通过 + 单元测试 + action_log 样本回放对比

---

## 4. Global Roadmap

| Phase | 目标 | 依赖 | 验收意图 |
|-------|------|------|---------|
| **H1（本次）** | ProcessBuffer + ProcessContext + 噪音清理 + cargo build 通过 | — | 编译通过，buffer 计算逻辑正确 |
| **H2** | 用 action_log 样本回放验证噪音过滤命中 + 行为事实正确计算 | H1 | 噪音被正确过滤，行为事实正确表达 |
| **H3** | 真实窗口验证弹窗时机是否有上下文依据 | H2 | 用户体感验证 |

---

## 5. Scope

### In（本次包含）

- `window_monitor.rs`：`ProcessBuffer` 结构（最大 20 条 VecDeque）、`BufferedEntry`、`ProcessContext` 结构（5 个字段）、`compute_context()` 方法
- `window_monitor.rs`：`WindowMonitor` 持有 `ProcessBuffer`；`capture()` 后写入 buffer
- `window_monitor.rs`：`has_changed()` 保留（用于窗口变化检测），buffer 逻辑独立
- `main_loop.rs`：系统噪音清理（仅限明确系统自噪声），独立于上下文模块
- `model_client.rs`：`model_client.call(snapshot, process_context)` — 增加 ProcessContext 参数作为调用上下文，prompt 本次不改
- `ui_state.rs`：`PendingConfirmationInfo` / `LastJudgmentInfo` 附加 ProcessContext（与 judgment 结果分开存储）
- 编译通过 + 现有单元测试通过

### Out（本次不包含）

- memory crate（不接 context_slices）
- relay_bridge
- 视频流/录制
- prompt 模板改动（ProcessContext 不写入 prompt 文案）
- UI 层展示 process_context（H2 验证阶段再做）
- 停留/震荡/刚切换作为 Rust 层裁决规则
- ProcessContext 作为 ModelOutput 的一部分（ModelOutput 只包含模型输出）

### 架构理解（关键边界）

ProcessContext 是**上下文模块**的产出，作为**调用上下文**进入判断链路，不是在 prompt 里硬编码，也不是停在 ModelOutput 上的附加字段：

```
main_loop:
  capture() → compute_context() → model_client.call(snapshot, ProcessContext) → ModelOutput
       ↑                                                          ↓
  push_snapshot()                                         ExecutionResult + ProcessContext
                                                          ↓
                                                     UI state (分开存储)
```

- `model_client.call(snapshot, ProcessContext)`：ProcessContext 作为第二个参数传入，prompt 不改
- ModelOutput（模型输出）和 ProcessContext（系统观察事实）并行流动，不合并
- 执行结果 + ProcessContext 一起推送到 UI state，分开存储

---

## 6. 设计原则（澄清）

### 原则 1：模型判断 vs 系统观察事实分离

- **ModelOutput**：模型输出（mode + reason + user_how），由 `parse_response` 解析得到
- **ProcessContext**：系统观察事实（stay_duration_seconds、switches_in_last_minute、is_oscillating、last_switch_direction、just_arrived），由 Rust 层计算

两者在 Rust 层并行流动，不合并。ProcessContext 附加到 UI state，不进 ModelOutput。

### 原则 2：系统噪音清理仅限明确系统自噪声

| 噪音类型 | 检测条件 | 处理动作 |
|----------|----------|----------|
| 自环 | 进程名为 cozmio.exe | 跳过本次调用，直接 ABSTAIN |
| 调试窗口 | cmd.exe/powershell.exe 且窗口标题含 cozmio 相关路径 | 跳过本次调用，直接 ABSTAIN |

**这些是系统自身异常状态，不是用户行为，不应进入模型判断链路。**

### 原则 3：compute_context 计算顺序

**不正确的做法：**
1. 先将当前快照写入 buffer
2. 再用当前 buffer（含当前条目）计算停留时长
3. 结果：stay_duration_seconds 永远接近 0

**正确的做法：**
1. 先用当前快照 + **旧 buffer**（不含当前条目）计算 ProcessContext
2. 再将当前快照写入 buffer

**计算 stay_duration_seconds 时：**
- 在 buffer 中查找当前窗口的**上一条**记录（排除当前条目）
- stay = 当前时间戳 - 上一条记录的时间戳
- 如果 buffer 中没有当前窗口的历史记录，stay_duration_seconds = 0

---

## 7. Current Truth

- **files inspected**: `window_monitor.rs`（ProcessBuffer 尚不存在）、`main_loop.rs`（capture → model → executor 链路清晰）、`model_client.rs`（ModelOutput 已有 mode/reason/user_how）
- **existing entry points**: `WindowMonitor::capture()` 每 poll 调用一次；`has_changed()` 用于窗口变化检测
- **existing runtime path**: `main_loop.rs:59` capture → `main_loop.rs:101` model_client.call → `main_loop.rs:133` executor.route → `main_loop.rs:155` handle_execution_result
- **existing verification**: cargo build 通过；action_log 样本中已有真实自触和调试窗口记录可用于回放验证

---

## 8. Implementation Steps

**Step 1. 在 `window_monitor.rs` 中新增 ProcessBuffer 和 ProcessContext**

- `ProcessBuffer` 结构：持有一个 `VecDeque<BufferedEntry>`，容量 20；`push()` 写入新条目；超出容量时移除最老的
- `BufferedEntry`：`window_title`、`process_name`、`timestamp`
- `ProcessContext`：`stay_duration_seconds`、`switches_in_last_minute`、`is_oscillating`、`last_switch_direction`、`just_arrived`；`last_switch_direction` 为 `SwitchDirection` 枚举（Arrived/Left/None）
- buffer 由 `WindowMonitor` 单例持有（`WindowMonitor::new()` 时初始化）

**Step 2. 在 `window_monitor.rs` 中新增 `compute_context()` 方法**

- 输入：当前快照的 `window_title`、`process_name`、`timestamp`
- **计算顺序**：先用当前快照 + 旧 buffer（排除当前条目）计算，再写入 buffer
- `stay_duration_seconds`：在 buffer 中查找当前窗口的上一条记录（排除当前条目），stay = 当前时间戳 - 上一条记录的时间戳；无历史记录时为 0
- `switches_in_last_minute`：扫描 buffer（排除当前条目），统计 60 秒内不同窗口标题的条目数
- `is_oscillating`：扫描 buffer（排除当前条目），检测最近 60 秒内是否有 >= 2 次切换且每次间隔 < 5 秒
- `last_switch_direction`：比较当前窗口与 buffer 中最近一条不同窗口的条目，判断是"切来(Arrived)"还是"切走(Left)"
- `just_arrived`：当前窗口与 buffer 中最近一条不是同一个窗口，且距上次切换 < 5 秒
- 返回：`ProcessContext`

**Step 3. 在 `window_monitor.rs` 中新增 `push_snapshot()` 方法（写 buffer）**

- 在 `compute_context()` 完成之后调用
- 将当前快照的元数据（标题、进程名、时间戳）写入 buffer
- 超出容量时移除最老的条目

**Step 4. 在 `main_loop.rs` 中新增系统噪音清理（独立步骤）**

- 在 `capture()` 之后、模型调用之前，插入独立的噪音清理判断
- 自环检测：`if process_name == "cozmio.exe" { 直接跳到本次 poll 末尾，继续； }`
- 调试窗口检测：`if (process_name == "cmd.exe" || process_name == "powershell.exe") && window_title.contains(cozmio_path_chars) { 直接跳到本次 poll 末尾，继续； }`
- 注意：MODEL_ERROR 不受此过滤限制

**Step 5. 修改 `model_client.rs` 的 call 方法签名**

- `ModelClient::call(&self, snapshot: &WindowSnapshot)` → `ModelClient::call(&self, snapshot: &WindowSnapshot, process_context: &ProcessContext)`
- prompt 本次不修改，ProcessContext 作为结构化调用上下文传入，不进 prompt 文本
- `parse_response` 不变，ModelOutput 保持原样（不含 ProcessContext）

**Step 6. 在 `main_loop.rs` 中接入完整调用链路**

- 在噪音清理之后，调用 `window_monitor.compute_context()`，获取 `ProcessContext`
- 调用 `model_client.call(snapshot, &process_context)` — ProcessContext 作为第二个参数传入
- 在本次 poll 所有处理完成后，调用 `window_monitor.push_snapshot()` 写入 buffer
- 执行结果（ModelOutput + ProcessContext）并行流动：ModelOutput → executor.route() → ExecutionResult；ProcessContext 附加到 UI state

**Step 7. ProcessContext 附加到 UI state**

- 在 `handle_execution_result` 中，将 `ProcessContext` 附加到 `PendingConfirmationInfo`（当前已含 task_text/source_window/source_process/created_at）
- `ui_state.rs` 中 `PendingConfirmationInfo` 新增 `process_context: Option<ProcessContext>` 字段
- `LastJudgmentInfo` 同样新增 `process_context` 字段
- ModelOutput 和 ProcessContext 分开存储，不合并

**Step 8. 编译验证**

- `cargo build` 通过
- `cargo test` 现有测试通过

---

## 9. Verification Asset

- **verification type**: `deterministic_software` + `execution_trace`
- **command / script**: `cargo build 2>&1`
- **expected evidence**: 编译成功，无 error
- **evidence location**: 编译输出
- **failure condition**: 编译报 error；测试失败
- **writeback targets**:
  - `verification/last_result.json`
  - `feature_list.json`
  - `claude-progress.txt`

**补充验证（噪音过滤回放 + 行为事实计算）**：
- 用 action_log 中含 cozmio.exe 自触和 cmd.exe 调试窗口的记录，验证噪音清理是否命中
- 用一段窗口序列（可来自 action_log 或人工构造），验证 ProcessContext 计算结果正确

---

## 10. Phase Gate

本 Phase（H1）只有满足以下条件才能标记为完成：

- [ ] `cargo build` 通过
- [ ] `cargo test` 现有测试通过
- [ ] ProcessBuffer 容量正确（最大 20 条，超出时移除最老条目）
- [ ] ProcessContext 5 个字段计算逻辑正确（特别是：stay_duration 基于旧 buffer 不含当前条目）
- [ ] 系统噪音清理（自环 + 调试窗口）正确拦截 cozmio.exe 自触和调试窗口
- [ ] `model_client.call(snapshot, &process_context)` 签名正确（ProcessContext 作为调用上下文传入，prompt 不改）
- [ ] ModelOutput 结构不变（不含 ProcessContext — 模型输出和系统观察事实分离）
- [ ] ProcessContext 正确附加到 UI state（PendingConfirmationInfo + LastJudgmentInfo）
- [ ] 用窗口序列证明：compute_context 能正确表达"最近一小段过程事实"（停留多久、切换几次、是否震荡、是否刚切换）
- [ ] 证明行为事实（停留/震荡/刚切换）没有用于 Rust 层前置裁决（系统噪音清理规则只包含 cozmio.exe 自环和调试窗口，不包含行为条件）
- [ ] `verification/last_result.json` 已更新（含时间戳、验证类型、结果摘要）
- [ ] `feature_list.json` 相关条目已添加

---

## 11. Next Execution Step

- **next phase**: H1 - 实施
- **goal**: ProcessBuffer + ProcessContext + 噪音清理 + cargo build 通过
- **entry skill**: `next`（直接执行实现）
- **stop condition**: cargo build 通过 + 单元测试通过 + Phase Gate 全部条件满足
