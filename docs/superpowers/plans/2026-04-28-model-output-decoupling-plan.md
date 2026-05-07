# Model Output Decoupling 落地计划

## 1. Flywheel Context

- active task: Model Output Decoupling（从 `docs/superpowers/specs/2026-04-28-model-output-decoupling-design.md`）
- current phase: H1 — model_client 重构 + 新 prompt 接入
- latest verification: 无
- blocker: 无
- next expected step: 执行 H1，真实截图跑新 prompt

---

## 2. Goal

将 model_client 从"解析结构化字段"重构为"保留原始输出 + 元信息"，Relay bridge 只透传不包装，执行端自主规划。真实接入新 prompt，用当前窗口截图验证输出质量。

---

## 3. Product Type

- type: model_output_validated + deterministic_software
- core risk: 模型输出质量（自然语言输出是否可用）、Relay 透传是否完整
- verification style: 真实截图调用 + action_log.jsonl 证据 + Toast 截图

---

## 4. Global Roadmap

| Phase | 目标 | 依赖 | 验收意图 |
|-------|------|------|---------|
| H1 | model_client 返回 ModelRawOutput，新 prompt 接入，main_loop 适配 | — | Build 通过，真实截图调用验证输出 |
| H2 | Relay bridge 解耦，只透传 raw_text | H1 | action_log 里 raw_text 完整透传 |
| H3 | 执行端自主规划验证 | H2 | 执行端收到原始输出自己规划 |

---

## 5. Scope

### In（本次包含）

- 新 `ModelRawOutput` 结构体
- `model_client.call()` 返回新结构
- 新 prompt 替换旧 prompt
- `main_loop.rs` 适配新返回结构
- `NotificationPending` 存储 `raw_text`
- `action_log.jsonl` 字段更新

### Out（本次不包含）

- Relay bridge 改动（H2）
- 执行端自主规划（H3）
- 旧 `ModelOutput`、`parse_response`、`InterventionMode` 移除（后续清理）

---

## 6. Current Truth

- files inspected:
  - `src-tauri/src/model_client.rs` — `call()`, `parse_response()`, `ModelOutput`
  - `src-tauri/src/executor.rs` — `route()`, `handle_continue()`, `handle_abstain()`
  - `src-tauri/src/main_loop.rs` — `handle_execution_result()`
  - `src-tauri/src/relay_bridge.rs` — `dispatch_confirmed_intervention()`, `RelayDispatchRequest`
  - `src-tauri/src/ui_state.rs` — `PendingConfirmationInfo`
- existing entry points:
  - `model_client.call()` 被 `main_loop.rs` 调用
  - `executor.route()` 被 `main_loop.rs` 调用
- existing runtime path:
  - `monitor.capture()` → `model_client.call()` → `executor.route()` → `handle_execution_result()` → `send_confirmation_notification()`
- existing verification: 无（尚未运行过新 prompt）

---

## 7. Implementation Shape（H1 可执行）

### Step 1 — 新增 ModelRawOutput 结构体

在 `model_client.rs` 添加：

```rust
pub struct ModelRawOutput {
    pub raw_text: String,       // 模型原始输出
    pub trace_id: String,       // 本次调用唯一 ID
    pub model_name: String,     // 实际调用模型名
    pub source_window: String,  // 来源窗口标题
    pub captured_at: i64,      // 截图时间戳
    pub call_started_at: i64,  // API 调用开始时间戳
    pub call_duration_ms: u64, // API 调用耗时
}
```

### Step 2 — 修改 model_client.call()

`call()` 返回 `ModelRawOutput`：
- 调用前记录 `call_started_at`
- 调用后记录 `call_duration_ms`
- `raw_text` = `response.response`（不解析，直接返回）
- `trace_id` = `TraceId::new().0`
- `model_name` = 实际使用模型名（fallback 后也要记录）
- 截图时间戳从 `snapshot.timestamp` 取

### Step 3 — 替换 build_prompt()

用新 prompt 替换 `build_prompt()` 的内容（新 prompt 见规格文档）。

### Step 4 — main_loop.rs 适配

`main_loop.rs` 中调用 `model_client.call()` 的地方适配新返回结构：
- `model_output.mode` → 删除（不再需要）
- `model_output.reason` → `model_output.raw_text`
- `model_output.user_how` → 删除
- Toast content = `raw_text`
- 传给 `handle_execution_result()` 的参数调整

### Step 5 — handle_execution_result 调整

由于没有 `mode` 了，判断逻辑需要调整：
- 如果 `raw_text` 为空 → 不显示 Toast（abstain）
- 如果 `raw_text` 非空 → 显示 Toast（continue）
- 具体判断逻辑先简单处理：raw_text 是否包含有效干预内容（后续可优化）

### Step 6 — NotificationPending 和 ActionRecord 适配

- `NotificationPending.task_text` = `raw_text`
- `ActionRecord.content_text` = `raw_text`
- `ActionRecord` 新增 `model_name` 字段

### Step 7 — Build + 真实截图验证

- `cargo build`
- 启动 cozmio，用当前窗口截图
- 检查 action_log.jsonl 里的 raw_text 是否为自然语言
- 截图 Toast 验证显示内容

---

## 8. Verification Asset

### H1 真实效果验收标准

H1 通过不等于 Build 通过。Build 通过只是入口条件。

**action_log.jsonl 字段要求**

每条记录必须包含：
- `model_name`: 本次调用模型名（如 `qwen3-vl:4b`）
- `raw_text`: 模型原始输出（自然语言，不解析）
- `source_window`: 来源窗口标题
- `captured_at`: 截图时间戳（Unix）
- `call_started_at`: API 调用开始时间戳
- `call_duration_ms`: API 调用耗时

**样本采集要求**

用真实当前窗口跑 5-10 条，覆盖以下场景：
1. 普通网页/阅读页面
2. ChatGPT 对话页面
3. 代码或报错页面
4. Cozmio 自己窗口
5. 无明确任务的普通切换

**每条样本标注**

| 字段 | 含义 |
|------|------|
| `模型是否沉默` | raw_text 为空或只有观察无行动建议 |
| `raw_text 质量` | 有帮助（直接推进任务）/ 页面描述 / 空洞描述 |
| `Toast 滞后感` | 用户看到时是否已经离开那个窗口 |
| `用户是否能看懂` | 一眼是否能明白"它为什么出现" |

**判断标准**

H1 通过条件（同时满足）：
1. Build 通过
2. 5-10 条样本全部入库，`action_log.jsonl` 含全部字段
3. 沉默率 ≥ 30%（即 ≥30% 的样本 raw_text 为空或只有观察描述）
4. 非沉默样本中，`有帮助` 占比 ≥ 40%

H1 不通过条件（满足任一即停）：
- `raw_text` 仍然大量是页面描述（占比 >60%）
- 即使有 raw_text，也不是在"帮忙推进"，而是在"复述页面"
- 结论：如果 raw_text 质量不行，先回 prompt / 模型 / 输入质量，不要往 Relay 走

### 验证记录格式

```
## H1 样本记录

| # | 场景 | 模型沉默？ | raw_text 质量 | 滞后感 | 能看懂？ | 结论 |
|---|------|----------|--------------|-------|---------|------|
| 1 | ChatGPT 对话 | 是/否 | 有帮助/页面描述/空洞 | 有/无 | 是/否 | 通过/不通过 |
...
```

### writeback

- `verification/last_result.json` — 含 H1 样本记录表
- `feature_list.json` — 新增 MODEL-OUTPUT-DECOUPLE 条目
- `claude-progress.txt` — 更新

---

## 9. Phase Gate

H1 完成后才进入 H2：

### 入口条件（先检查这些再启动）
- [ ] `cargo build` 通过

### 通过条件
- [ ] 5-10 条样本入库，`action_log.jsonl` 含全部 6 个字段
- [ ] 沉默率 ≥ 30%
- [ ] 非沉默样本中 `有帮助` 占比 ≥ 40%
- [ ] H1 样本记录表已写入 `verification/last_result.json`
- [ ] `feature_list.json` 新增 MODEL-OUTPUT-DECOUPLE 条目
- [ ] `claude-progress.txt` 已更新下一轮交接

### 不通过条件（H1 停下来，不要往 H2 走）
- [ ] raw_text 仍大量是页面描述（>60%）
- [ ] 非沉默样本中 `有帮助` 占比 < 40%
- [ ] 结论：回 prompt / 模型 / 输入质量，H1 重做

---

## 10. Next Execution Step

- next phase: H2
- goal: Relay bridge 只透传 raw_text，不包装语义
- entry skill: `superpowers:subagent-driven-development`
- stop condition: H2 验证通过
