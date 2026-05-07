# Model Output Decoupling Design

## Status

Draft — 待用户审阅

---

## 背景

当前系统存在一个根本性的架构问题：

**模型输出被强制结构化以服务前后端流转需求，而不是服务于模型的判断质量。**

表现：
- Prompt 写死 `MODE: / REASON: / USER_HOW:` 格式，限制模型自然输出
- `parse_response()` 硬编码解析固定字段
- `model_client` 返回 `ModelOutput { mode, reason, user_how }` 固定结构
- Relay bridge 做语义包装，构造 `task_text`
- 执行端收到的是干巴巴的状态描述，而不是有血有肉的任务指令

核心原则：**模型输出是高维语义产物，系统是低维结构。系统必须适配模型的自然输出，而不是反过来约束模型。**

---

## 目标

1. **解耦模型输出与系统数据结构** — model_client 不再负责语义解析，只保留原始输出和元信息
2. **解耦调度层与执行层** — Relay bridge 只透传，不包装语义
3. **执行端自主规划** — 执行端（模型/agent）基于原始输出自行判断如何执行
4. **保留可追溯性** — 元信息（trace、timestamp、model_name、source_window）用于日志和 trace

---

## 新架构

### 数据流

```
旧架构：
Model raw → parse_response() → ModelOutput {mode,reason,user_how} → executor.route() [匹配枚举] → Toast(task_text=reason) → Relay(task_text=reason+包装)

新架构：
Model raw → model_client.call() → (raw_output, ModelMeta)
  → 日志: raw_output 自然语言存储
  → Toast: raw_output 自然语言显示
  → Relay bridge: 透传 raw_output + 元信息，不改写
  → 执行端: 接收原始输出 + 上下文，自主规划执行
```

### 边界定义

#### model_client（调用层）

职责：
- 调用 Ollama API
- 保留原始输出（完整自然语言，未经解析）
- 附加元信息：trace_id、timestamp、model_name、source_window、screenshot_base64

返回结构：
```rust
pub struct ModelRawOutput {
    pub raw_text: String,           // 模型原始输出，自然语言
    pub trace_id: String,           // 本次调用唯一 ID
    pub model_name: String,         // 实际调用的模型名
    pub source_window: String,      // 来源窗口标题
    pub captured_at: i64,           // 截图时间戳
    pub call_started_at: i64,      // API 调用开始时间戳
    pub call_duration_ms: u64,     // API 调用耗时
}
```

不做什么：
- 不解析 CONTINUE/ABSTAIN
- 不提取意图
- 不构造 task_text
- 不做语义判断

#### Relay bridge（传递层）

职责：
- 接收 `ModelRawOutput` 和 `source_window`
- 将 `raw_text` 和元信息透传到执行端
- 记录传递日志（trace_id、timestamp）
- 管理权限和确认流程

不做什么：
- 不提取意图
- 不改写 `raw_text`
- 不构造新的 task_text
- 不做语义包装

#### 执行端（模型/Agent）

职责：
- 接收：`raw_text` + 来源上下文（source_window、user_confirm_event）
- 自行判断：是否可以执行、如何规划、是否需要澄清、是否失败
- 自行返回执行结果

不依赖：
- 不依赖上游的结构化 judgment 字段
- 不依赖 CONTINUE/ABSTAIN 二值判断

#### 日志层（action_log.jsonl）

存储内容：
- `raw_text`: 模型原始输出（自然语言，不包装）
- `trace_id`: 贯穿链路的唯一 ID
- `model_name`: 本次调用模型名
- `captured_at`: 截图时间戳
- `call_started_at`: API 调用开始时间戳
- `call_duration_ms`: API 调用耗时
- `source_window`: 来源窗口

Toast 显示：
- 内容 = `raw_text`（自然语言）
- 如果执行端决定不介入，`raw_text` 为空或仅有观察描述，不显示 Toast

---

## 新 Prompt 设计

### 角色定义

```
你是 Cozmio 的桌面观察助手。

你看到的是用户当前屏幕的一小段现场。
你的目标不是复述窗口标题，也不是证明你看到了什么。
你的目标是判断：此刻你是否真的能帮用户推进一点事情。

请先像一个旁边的助手一样理解现场：
用户大概在做什么？
现在有没有明显的问题、卡点、错误、等待、反复尝试或需要整理的内容？
如果你出现，会不会打扰用户？
如果你能帮，应该用一句用户听得懂的话说明你能帮什么。

当你只是看到普通页面、普通阅读、普通切换、或者你只能说出"用户正在使用某某软件"时，保持沉默。
当你确实看到了一个可以帮忙推进的机会时，用自然语言说明：
你为什么现在出现，以及你准备帮用户做什么。
```

### 设计说明

- **不锁格式**：模型自然输出，可长可短，可描述可沉默
- **不要求 CONTINUE/ABSTAIN**：由执行端判断是否介入
- **自然语言优先**：输出的内容本身就可以直接用于 Toast 或 Relay
- **自我判断**：模型自己判断是否要沉默，而不是被迫输出 ABSTAIN

---

## 向后兼容

由于 model_client 返回结构从 `ModelOutput { mode, reason, user_how }` 改为 `ModelRawOutput { raw_text, ... }`，调用方需要适配：

1. **executor** — 不再匹配 `InterventionMode` 枚举，改为接收 `ModelRawOutput`，执行端自主判断
2. **main_loop** — Toast 显示内容从 `reason` 改为 `raw_text`
3. **NotificationPending** — `task_text` 字段存 `raw_text`
4. **RelayDispatchRequest** — `original_suggestion` 存 `raw_text`，不包装

---

## 实施步骤

1. 新增 `ModelRawOutput` 结构体
2. 修改 `model_client.call()` 返回 `ModelRawOutput`
3. 更新 `main_loop.rs` 使用新返回结构
4. 更新 `NotificationPending` 构造
5. 更新 `RelayDispatchRequest` 透传 `raw_text`
6. 更新 executor 路由逻辑（执行端自主规划）
7. 旧 `ModelOutput`、`parse_response`、`InterventionMode` 保留或移除（视兼容性而定）
8. 更新 `action_log.jsonl` 日志字段

---

## 验证

- Build 通过
- 用当前窗口截图跑一次新 prompt，看 `raw_text` 输出
- 对比新旧 prompt 输出质量
- Toast 显示内容是否自然
- Relay 透传是否完整
