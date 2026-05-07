# 桌面端模型输出统一实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标**：将桌面端（src-tauri）替换为已验证的模型输出格式（CONTINUE/ABSTAIN），并实现阻塞式弹窗

**架构思路**：使用 cozmio_model 中已验证的 prompt 和解析逻辑，输出简化为 `InterventionMode`（Continue/Abstain），CONTINUE → 弹确认框，ABSTAIN → 不弹窗

---

## 现状问题

| 文件 | 问题 |
|------|------|
| `src-tauri/src/model_client.rs:87-122` | prompt 是未验证的另一套（"You are an AI assistant..."） |
| `src-tauri/src/model_client.rs:155-218` | 解析 5-field 输出（judgment/next_step/level/confidence/grounds），从未验证 |
| `src-tauri/src/executor.rs` | 依赖 InitiativeLevel（Suggexecute），但模型从未输出过这种格式 |
| `src-tauri/src/main_loop.rs:191-213` | 弹窗是 async spawn，回调不等待用户结果 |

---

## 任务1：替换 model_client.rs 的 prompt 和解析逻辑

**涉及文件**：
- 修改：`cozmio/cozmio/src-tauri/src/model_client.rs:87-218`

- [ ] **步骤1：替换 build_prompt 为验证过的系统提示词**

将 `model_client.rs:87-122` 的 `build_prompt` 方法替换为：

```rust
fn build_prompt(&self, snapshot: &WindowSnapshot) -> String {
    let window = &snapshot.window_info;
    format!(
        r#"你是一个窗口判断器。

你的任务不是描述页面，也不是推测用户完整意图。
你的任务是：根据窗口截图和机械元信息，判断当前证据是否足以支持 agent 继续介入。

只允许两种最终结果：

1. CONTINUE
表示：当前可见证据已经足以支持 agent 继续往前一轮判断。

2. ABSTAIN
表示：当前证据不足，agent 不应继续延伸。

要求：
- 只能依据截图和提供的机械元信息作判断。
- 不要补充截图中看不见、元信息中没有的事实。
- 不要输出页面描述作为最终结果。
- 不要输出世界标签，例如"idle状态""高风险操作""用户正在做X"。
- 不要推荐具体动作，不要替用户做决定。
- 不要展示思考过程，只输出最终结果。

输出格式必须严格为：

MODE: CONTINUE 或 MODE: ABSTAIN
REASON: 一句简短理由，理由只能引用可见证据。

窗口标题: {}
进程名: {}"#,
        window.title,
        window.process_name,
    )
}
```

- [ ] **步骤2：替换 parse_response 解析 CONTINUE/ABSTAIN**

将 `model_client.rs:154-218` 的 `parse_response` 替换为：

```rust
/// Parse the model response into a ModelOutput struct
fn parse_response(&self, response: &str) -> Result<ModelOutput, String> {
    let trimmed = response.trim();

    // Parse MODE line
    let mode_re = Regex::new(r"(?i)MODE:\s*(CONTINUE|ABSTAIN)").unwrap();
    let reason_re = Regex::new(r"(?i)REASON:\s*(.+)").unwrap();

    let mode_cap = mode_re.captures(trimmed).ok_or_else(|| {
        "Missing MODE line - model output format error".to_string()
    })?;
    let mode_str = mode_cap.get(1).unwrap().as_str().to_uppercase();
    let mode = if mode_str == "CONTINUE" {
        InitiativeLevel::Continue
    } else {
        InitiativeLevel::Abstain
    };

    let reason = reason_re.captures(trimmed)
        .map(|c| c.get(1).unwrap().as_str().trim().to_string())
        .ok_or_else(|| "Missing REASON line".to_string())?;

    Ok(ModelOutput {
        mode,
        reason,
    })
}
```

- [ ] **步骤3：更新 ModelOutput 结构体**

将 `model_client.rs:38-46` 替换为：

```rust
/// Model output modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterventionMode {
    Continue,
    Abstain,
}

impl std::fmt::Display for InterventionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterventionMode::Continue => write!(f, "CONTINUE"),
            InterventionMode::Abstain => write!(f, "ABSTAIN"),
        }
    }
}

/// Output from the model
#[derive(Debug, Clone)]
pub struct ModelOutput {
    pub mode: InterventionMode,
    pub reason: String,
}
```

- [ ] **步骤4：添加 Regex import**

在 `model_client.rs:1` 顶部添加：
```rust
use regex::Regex;
```

并在 Cargo.toml 中添加依赖：
```toml
regex = "1"
```

- [ ] **步骤5：验证编译**

执行命令：`cd cozmio && cargo build --package cozmio 2>&1`

预期结果：编译通过

---

## 任务2：更新 executor.rs 适配新输出格式

**涉及文件**：
- 修改：`cozmio/cozmio/src-tauri/src/executor.rs:1-155`

- [ ] **步骤1：更新 import 和类型**

将 `executor.rs:3` 替换为：
```rust
use crate::model_client::{InterventionMode, ModelOutput};
```

删除 `InitiativeLevel` 相关代码（executor.rs:9-35）。

- [ ] **步骤2：替换 route 方法**

将 `executor.rs:46-84` 的 `route` 方法替换为：

```rust
pub fn route(&self, output: &ModelOutput, window_title: &str) -> Result<ExecutionResult, String> {
    let record = ActionRecord {
        timestamp: chrono::Utc::now().timestamp(),
        window_title: window_title.to_string(),
        judgment: output.mode.to_string(),
        next_step: output.reason.clone(),
        level: match output.mode {
            InterventionMode::Continue => "continue".to_string(),
            InterventionMode::Abstain => "abstain".to_string(),
        },
        confidence: 1.0,
        grounds: output.reason.clone(),
        system_action: "".to_string(),
        user_feedback: None,
    };

    let result = match output.mode {
        InterventionMode::Continue => {
            // CONTINUE: 需要弹确认框
            self.handle_continue(output, &record)
        }
        InterventionMode::Abstain => {
            // ABSTAIN: 不弹窗，不干预
            self.handle_abstain(output, &record)
        }
    };

    if let Ok(ref exec_result) = result {
        let mut updated_record = record;
        updated_record.system_action = exec_result.to_string();
        if let Err(e) = self.logger.log(updated_record) {
            eprintln!("Failed to log action: {}", e);
        }
    }

    result
}
```

- [ ] **步骤3：替换 handle_* 方法**

将 `executor.rs:87-136` 的三个 handle_* 方法替换为：

```rust
fn handle_continue(&self, output: &ModelOutput, _record: &ActionRecord) -> Result<ExecutionResult, String> {
    log::info!(
        "CONTINUE: {} (reason: {})",
        output.mode,
        output.reason,
    );
    Ok(ExecutionResult::Confirmed)
}

fn handle_abstain(&self, output: &ModelOutput, _record: &ActionRecord) -> Result<ExecutionResult, String> {
    log::info!(
        "ABSTAIN: {} (reason: {})",
        output.mode,
        output.reason,
    );
    Ok(ExecutionResult::Skipped)
}
```

- [ ] **步骤4：删除不再需要的方法**

删除 `executor.rs:139-151` 的 `confirm()` 和 `skip()` 方法（可选，如果没人调用）。

- [ ] **步骤5：验证编译**

执行命令：`cd cozmio && cargo build --package cozmio 2>&1`

预期结果：编译通过

---

## 任务3：实现阻塞式弹窗

**涉及文件**：
- 修改：`cozmio/cozmio/src-tauri/src/main_loop.rs:160-223`

- [ ] **步骤1：检查 tauri-plugin-dialog 是否有阻塞 API**

执行命令：`grep -r "blocking" cozmio/src-tauri/node_modules/tauri-plugin-dialog 2>/dev/null | head -20`

如果支持 blocking_show_message_dialog，直接使用。

- [ ] **步骤2：实现阻塞弹窗**

将 `main_loop.rs:191-213` 的 Confirmed 分支替换为：

```rust
ExecutionResult::Confirmed => {
    use tauri_plugin_dialog::DialogExt;
    use tauri_plugin_dialog::MessageDialogKind;
    let title = "Cozmio - 确认请求";
    let msg = format!("{}\n\n理由: {}", judgment, next_step);

    let confirmed = app_handle
        .dialog()
        .blocking_show_message_dialog(
            MessageDialogKind::Info,
            Some(title),
            Some(&msg),
        )
        .unwrap_or(false);

    if confirmed {
        log::info!("User confirmed");
        // 用户确认 - 执行
    } else {
        log::info!("User declined");
        // 用户拒绝 - 跳过
    }
}
```

- [ ] **步骤3：验证编译**

执行命令：`cd cozmio && cargo build --package cozmio 2>&1`

预期结果：编译通过

---

## 任务4：验证模型输出 → 弹窗链路

**涉及文件**：
- 修改：`cozmio/cozmio/src-tauri/src/model_client.rs`
- 测试：`cozmio/cozmio/src-tauri/src/tests/model_output_test.rs`

**核心问题**：模型输出 CONTINUE/ABSTAIN → 弹窗是否真的出现？结果是否写回日志？

- [ ] **步骤1：添加单元测试验证 parse_response 正确解析 CONTINUE/ABSTAIN**

在 `model_client.rs` 底部添加测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response_continue() {
        let config = create_test_config();
        let client = ModelClient::new(config);

        let response = r#"MODE: CONTINUE
REASON: 截图中明确展示协作框架定义讨论的具体建议，包含明确的决策路径和可操作选项。"#;

        let output = client.parse_response(response).unwrap();
        assert_eq!(output.mode, InterventionMode::Continue);
        assert!(output.reason.contains("协作框架"));
    }

    #[test]
    fn test_parse_response_abstain() {
        let config = create_test_config();
        let client = ModelClient::new(config);

        let response = r#"MODE: ABSTAIN
REASON: 当前证据不足，无法支持继续介入。"#;

        let output = client.parse_response(response).unwrap();
        assert_eq!(output.mode, InterventionMode::Abstain);
    }

    #[test]
    fn test_parse_response_missing_mode() {
        let config = create_test_config();
        let client = ModelClient::new(config);

        let response = r#"REASON: some reason"#;
        let result = client.parse_response(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing MODE"));
    }

    #[test]
    fn test_parse_response_missing_reason() {
        let config = create_test_config();
        let client = ModelClient::new(config);

        let response = r#"MODE: CONTINUE"#;
        let result = client.parse_response(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing REASON"));
    }
}
```

- [ ] **步骤2：添加 executor 路由测试验证 CONTINUE → Confirmed，ABSTAIN → Skipped**

在 `executor.rs` 底部添加测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_executor() -> Executor {
        let config = create_test_config();
        let logger = create_test_logger();
        Executor::new(config, logger)
    }

    #[test]
    fn test_route_continue_returns_confirmed() {
        let executor = create_test_executor();
        let output = ModelOutput {
            mode: InterventionMode::Continue,
            reason: "测试理由".to_string(),
        };
        let result = executor.route(&output, "Test Window").unwrap();
        assert_eq!(result, ExecutionResult::Confirmed);
    }

    #[test]
    fn test_route_abstain_returns_skipped() {
        let executor = create_test_executor();
        let output = ModelOutput {
            mode: InterventionMode::Abstain,
            reason: "证据不足".to_string(),
        };
        let result = executor.route(&output, "Test Window").unwrap();
        assert_eq!(result, ExecutionResult::Skipped);
    }

    #[test]
    fn test_route_continue_logs_action() {
        let executor = create_test_executor();
        let output = ModelOutput {
            mode: InterventionMode::Continue,
            reason: "测试理由".to_string(),
        };
        executor.route(&output, "Test Window").unwrap();

        let recent = executor.logger.get_recent(1).unwrap();
        assert_eq!(recent[0].judgment, "CONTINUE");
        assert_eq!(recent[0].system_action, "confirmed");
    }

    #[test]
    fn test_route_abstain_does_not_log_confirmed_action() {
        let executor = create_test_executor();
        let output = ModelOutput {
            mode: InterventionMode::Abstain,
            reason: "证据不足".to_string(),
        };
        executor.route(&output, "Test Window").unwrap();

        let recent = executor.logger.get_recent(1).unwrap();
        assert_eq!(recent[0].judgment, "ABSTAIN");
        assert_eq!(recent[0].system_action, "skipped");
    }
}
```

- [ ] **步骤3：运行测试验证**

执行命令：`cd cozmio && cargo test --package cozmio model_client::tests executor::tests 2>&1`

预期结果：所有测试通过

---

## 任务5：端到端验证弹窗行为

**涉及文件**：
- 测试：手动验证流程

- [ ] **步骤1：启动 Tauri 应用**

执行命令：`cd cozmio && cargo tauri dev 2>&1`

预期结果：应用启动，托盘图标显示

- [ ] **步骤2：验证 CONTINUE 时弹窗出现**

切换到一个模型可能输出 CONTINUE 的窗口（如文档编辑界面），观察：
1. 是否弹出 Windows 确认框
2. 弹窗是否阻塞主循环（窗口不可交互直到点击）
3. 点击"是"后日志是否记录 `user_feedback: "confirmed"`
4. 点击"否"后日志是否记录 `user_feedback: "skipped"`

- [ ] **步骤3：验证 ABSTAIN 时不弹窗**

切换到空白或信号不足的窗口，观察：
1. 是否没有弹出确认框
2. 日志是否记录 `system_action: "skipped"`

---

## 验收标准

| 检查项 | 标准 |
|--------|------|
| 编译通过 | `cargo build --package cozmio` 无错误 |
| prompt 替换 | 使用 "你是窗口判断器" 格式 |
| 输出解析测试 | `cargo test model_client::tests` 全部通过 |
| executor 路由测试 | `cargo test executor::tests` 全部通过 |
| CONTINUE 路由 | `route(CONTINUE)` 返回 `ExecutionResult::Confirmed` |
| ABSTAIN 路由 | `route(ABSTAIN)` 返回 `ExecutionResult::Skipped` |
| 日志写入 | `action_log.jsonl` 包含正确的 `user_feedback` |
| 弹窗阻塞 | 确认框弹出并等待用户点击 |
| user_feedback | 日志正确记录 confirmed/declined |
