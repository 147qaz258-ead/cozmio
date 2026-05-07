# Semantic Boundary Fix Design

**版本：** 1.0
**日期：** 2026-05-07
**状态：** 待审阅

---

## 1. Violation 溯源

| Violation | 代码位置 | 设计来源 | 违规内容 |
|-----------|---------|---------|---------|
| V1 | `model_client.rs:parse_response()` | 未被调用 | `parse_response()` 解析自然语言为 Continue/Abstain 枚举 |
| V2 | `model_client.rs:140-145` + `prompt_context.rs:137-142` | `context-harness-h1-design.md:29-36` | prompt 注入 `is_oscillating`、`just_arrived`、`last_switch_direction` 语义标签 |
| V3 | `main_loop.rs:265,277-288` | `context-harness-h1-design.md:29` | `should_intervene` 从非空文本推断语义标签并注入日志 |
| V4 | `main_loop.rs:315-335` | `stage1-plan.md:190` | `auto_remember_model_output` 在 feedback 前写入记忆 |
| V5 | `executor.rs` 整文件 | 未被调用 | 整文件死代码 |

---

## 2. V1 + V5：删除死代码

### 2.1 `model_client.rs` — 删除 `parse_response/call/InterventionMode/ModelOutput`

**理由**：从未被调用（grep 确认）。`call_raw_with_context()` + `ModelRawOutput` 已在热路径上。

删除内容：
- `InterventionMode` 枚举（line 8-22）
- `ModelOutput` 结构体（line 24-31）
- `call()` 函数（line 114-123）
- `parse_response()` 函数（line 380-416）
- `Regex` 导入（line 1）
- `model_client.rs` 中所有对 `InterventionMode` 的引用

### 2.2 `executor.rs` — 删除整文件

**理由**：从未被调用（grep 确认）。删除后从 `main.rs` 的 mod 声明和 `generate_handler!` 中同步移除。

---

## 3. V2：ProcessContext 语义标签从 prompt 移除

### 3.1 设计意图

`context-harness-h1-design.md:52-63` 的允许列表：

```text
process_context: stay_duration_seconds=..., switches_last_minute=...
```

`context_harness_h1_design.md:29-36` 的禁止列表包含：`stuck/not stuck`、`task stage`、`user intent`。

`is_oscillating`、`just_arrived`、`last_switch_direction` 属于代码推断的语义标签，违反禁止列表。

### 3.2 ProcessContext 改动

**保留**：字段定义、计算逻辑、测试 — 因为：
1. `window_monitor.rs` 外可能还有其他 caller 依赖这些字段
2. 计算逻辑本身没有错，错的是注入 prompt

**文件**：`window_monitor.rs:52-58`

ProcessContext 结构体不变，计算逻辑不变，测试不变。只从 prompt 输出中移除。

### 3.3 `model_client.rs` — 修改 prompt 格式

**文件**：`model_client.rs:136-148`

修改前（违规）：
```rust
format!(
    "process_context: stay_duration_seconds={}, switches_last_minute={}, is_oscillating={}, just_arrived={}, last_switch_direction={:?}",
    context.stay_duration_seconds,
    context.switches_in_last_minute,
    context.is_oscillating,
    context.just_arrived,
    context.last_switch_direction
)
```

修改后（合规）：
```rust
format!(
    "process_context: stay_duration_seconds={}, switches_last_minute={}",
    context.stay_duration_seconds,
    context.switches_in_last_minute
)
```

### 3.4 `prompt_context.rs` — 修改 prompt 格式

**文件**：`prompt_context.rs:137-142`

同样的改动：只保留 `stay_duration_seconds` 和 `switches_last_minute`，移除 `is_oscillating`、`just_arrived`、`last_switch_direction`。

---

## 4. V3：`should_intervene` 语义映射修正

### 4.1 设计意图

代码只传递 raw_text 原样，不解释 raw_text 意味着什么。语义判断权属于 Agent，不是代码。

### 4.2 `main_loop.rs:265,277-288`

修改前（违规）：
```rust
let should_intervene = !raw_output.raw_text.trim().is_empty();

let original_judgment = if should_intervene {
    "CONTINUE"
} else {
    "ABSTAIN"
}
let execution_result_str = if should_intervene {
    "awaiting-confirmation"
} else {
    "silence"
}
```

修改后（合规）：
```rust
// raw_text 原样传递，不做语义推断
let raw_text = &raw_output.raw_text;
let is_empty = raw_text.trim().is_empty();

// judgment 和 execution_result_str 不从 should_intervene 衍生
// 日志记录原样数据，下游自行解释
```

日志中 `original_judgment` 和 `execution_result_str` 改为：
- `original_judgment`: 空字符串（代码不持有语义判断）
- `execution_result_str`: 空字符串（等待 executor 回填）
- `system_route`: 设为 `Unknown`（不预设路由）

---

## 5. V4：`auto_remember_model_output` 触发时机修正

### 5.1 设计意图

`stage1-plan.md:190`：`local model raw output / user feedback / executor result` 作为触发条件。但实际语义应该是：model 输出是素材，feedback 和 executor result 才是触发写入的条件。

### 5.2 `main_loop.rs:315-335`

修改前（违规）：`should_intervene=true` 时立即写入

```rust
if should_intervene {
    auto_remember_model_output(...);
}
```

修改后（合规）：在 user feedback 或 executor result 返回后再触发写入

移除 `should_intervene` 下的 `auto_remember_model_output` 调用。改为在以下位置触发：
1. 用户确认后（`experience_recorder::record_confirmation()`）
2. 用户拒绝后（`experience_recorder::record_confirmation_dismissed()`）
3. popup result 返回后（`experience_recorder::record_popup_result()`）

### 5.3 同步修改 `stage1-plan.md:190`

当前：`local model raw output / user feedback / executor result` 并列

修改为：`user feedback` 或 `executor result` 返回时触发写入，`model raw output` 作为写入素材传入

---

## 6. 实施顺序

```
Step 1: 删死代码（无依赖风险）
  - 删除 executor.rs 整文件
  - 从 main.rs 移除 mod executor 和 generate_handler 注册
  - model_client.rs: 删除 parse_response, call, InterventionMode, ModelOutput 及所有引用

Step 2: 修 prompt 格式（两处）
  - model_client.rs: prompt 只保留 stay_duration, switches_count
  - prompt_context.rs: 同上
  - ProcessContext 结构体和计算逻辑保留不动

Step 3: 修 main_loop.rs 语义映射
  - 移除 should_intervene 映射的 judgment 和 execution_result_str
  - system_route 设为 Unknown
  - 日志记录原样数据

Step 4: 修 auto_remember 触发
  - main_loop.rs: 移除 should_intervene 下的写入调用
  - 在 feedback/executor result 返回处触发
  - 同步修改 stage1-plan.md 写入条件描述
```

---

## 7. 验证

```bash
cd cozmio && cargo build

# 无死代码残留
grep -r "InterventionMode\|parse_response\|executor" cozmio/src-tauri/src/

# prompt 不含语义标签
grep "is_oscillating\|just_arrived\|last_switch_direction" cozmio/src-tauri/src/model_client.rs cozmio/src-tauri/src/prompt_context.rs

# main_loop 不做语义映射
grep "should_intervene.*CONTINUE\|should_intervene.*awaiting" cozmio/src-tauri/src/main_loop.rs

# auto_remember 不在 should_intervene 下调用
grep -A3 "should_intervene" cozmio/src-tauri/src/main_loop.rs | grep auto_remember
```

---

## 8. 不做什么

- ProcessContext 计算逻辑不删除（其他 caller 可能依赖）
- `call_raw_with_context()` 和 `ModelRawOutput` 不改动（已在正确路径）
- `experience_recorder.rs` 不改动（已是防御层）
- `memory_consolidation.rs` 的 `auto_remember_model_output` 函数本身不删除，只修改调用时机
