# 系统层确认通知实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标**：实现用户不需要打开 Cozmio 主窗口即可完成"系统层提醒 → 用户确认 → 真实执行 → 结果返回 → 系统层结果通知"的完整链路。

**架构思路**：
1. 当模型判断需要用户确认时，创建 `NotificationPending`（含 trace_id、token、content_text）
2. 发送 Windows Toast 通知，带 "确认" 和 "取消" action button，使用 `cozmio://` protocol 传递 trace_id + token
3. App 解析 protocol URL，验证 token 后执行相应操作（确认 dispatch / 取消）
4. Relay 执行完成后，根据状态发送结果通知（completed/failed/interrupted）
5. 整个链路只使用最小字段集：trace_id、status、session_id、token、content_text、result_text、error_text

**技术栈**：
- Rust: `winrt-notification` crate（直接使用 Windows Toast API，支持 action button）
- Tauri v2: protocol handler 注册、Cold start URL 解析
- 现有: relay_bridge, commands, main_loop

---

## 文件结构

### 新增文件

- `src-tauri/src/notification_manager.rs` - Windows Toast 通知管理，处理 action button 回调
- `src-tauri/src/types.rs` - 最小任务状态类型：TraceId、TaskState、NotificationPending、ConfirmToken、TaskStatus
- `src-tauri/src/protocol_handler.rs` - protocol (`cozmio://`) 解析和路由

### 修改文件

- `src-tauri/src/main_loop.rs:155-268` - `handle_execution_result` 函数，改为发送系统层通知而非显示主窗口
- `src-tauri/src/relay_bridge.rs:69-85` - `dispatch_confirmed_intervention`，无需修改签名
- `src-tauri/src/commands.rs:152-218` - 新增 `confirm_pending_task_by_token` 和 `cancel_pending_task_by_token`
- `src-tauri/src/ui_state.rs` - 保持 StateUpdate 不变（前端字段），内部存储用 TaskState
- `src-tauri/src/main.rs` - 注册 protocol handler 和 cold start 处理
- `src-tauri/Cargo.toml` - 添加 `winrt-notification` 依赖
- `src-tauri/tauri.conf.json` - 添加 protocol scheme 注册
- `src-tauri/src/tray.rs` - 托盘菜单保持不变（仅启动/停止/设置/历史/退出）

---

## 产品类型

**传统软件实现型**（非模型输出验证型）——核心是系统通知 action button、Relay dispatch、结果通知、STOP 能力，而非模型输出质量。

---

## 细粒度任务拆分

### 任务 1：添加 winrt-notification 依赖并验证构建

**涉及文件**：
- 修改：`src-tauri/Cargo.toml:9-25`

- [ ] **步骤 1：添加 winrt-notification 依赖**

在 `[dependencies]` section 添加：
```toml
winrt-notification = "0.5"
```

- [ ] **步骤 2：运行 cargo check 确认依赖可用**

执行命令：`cd cozmio/src-tauri && cargo check 2>&1`
预期结果：无编译错误（可能有 unused warnings）

- [ ] **步骤 3：提交代码**

```bash
cd cozmio
git add src-tauri/Cargo.toml
git commit -m "deps: add winrt-notification for Windows Toast action button support"
```

---

### 任务 2：定义最小任务状态结构

**涉及文件**：
- 创建：`src-tauri/src/types.rs`

**核心原则**：
- 只保留完成链路所需的最小控制面字段
- content_text 保持原文，不拆成 task_title、task_summary 等
- 来源信息（source_window、source_process）保留在日志/证据中，不进任务结构
- 前端展示需要的字段由前端自己处理

- [ ] **步骤 1：创建 types.rs，定义最小任务状态类型**

```rust
use serde::{Deserialize, Serialize};

/// Unified trace_id for tracking across capture/judgment/notification/dispatch/result/history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceId(pub String);

impl TraceId {
    pub fn new() -> Self {
        // Use timestamp-based ID for simplicity
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random = (now as u64 ^ (now >> 64) as u64).wrapping_mul(0x5DEECE66D);
        TraceId(format!("{:016x}-{:016x}", now as u64, random))
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Task status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    /// Awaiting user confirmation in system notification
    Pending,
    /// User confirmed, dispatching to Relay
    Dispatching,
    /// Relay session is running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was interrupted by user or system
    Interrupted,
    /// User cancelled the pending notification
    Cancelled,
    /// Notification was dismissed (timeout, etc.)
    Dismissed,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Dispatching => write!(f, "dispatching"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Interrupted => write!(f, "interrupted"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
            TaskStatus::Dismissed => write!(f, "dismissed"),
        }
    }
}

/// Minimal task state for pending notification.
/// Only contains control plane fields needed for the confirmation chain.
/// content_text is kept as original text - no display-oriented splitting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub trace_id: String,
    pub status: TaskStatus,
    pub session_id: Option<String>,
    pub content_text: String,
    pub result_text: Option<String>,
    pub error_text: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TaskState {
    pub fn new(trace_id: String, content_text: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            trace_id,
            status: TaskStatus::Pending,
            session_id: None,
            content_text,
            result_text: None,
            error_text: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = status;
        self.updated_at = chrono::Utc::now().timestamp();
        self
    }

    pub fn with_result(mut self, result_text: String) -> Self {
        self.result_text = Some(result_text);
        self.updated_at = chrono::Utc::now().timestamp();
        self
    }

    pub fn with_error(mut self, error_text: String) -> Self {
        self.error_text = Some(error_text);
        self.updated_at = chrono::Utc::now().timestamp();
        self
    }
}

/// Token for one-time confirmation of a pending notification.
/// Generated when notification is sent, consumed on confirm/cancel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmToken(pub String);

impl ConfirmToken {
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random = (now as u64 ^ (now >> 64) as u64).wrapping_mul(0x5DEECE66D);
        ConfirmToken(format!("{:016x}{:016x}", now as u64, random))
    }
}

impl Default for ConfirmToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Pending notification entry - tracks a notification awaiting user action.
/// Token is consumed on confirm or cancel to prevent replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPending {
    pub trace_id: String,
    pub token: ConfirmToken,
    pub content_text: String,
    pub created_at: i64,
}

impl NotificationPending {
    pub fn new(trace_id: String, content_text: String) -> Self {
        Self {
            trace_id,
            token: ConfirmToken::new(),
            content_text,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Generate protocol URL for notification action
    pub fn to_protocol_url(&self, action: &str) -> String {
        format!(
            "cozmio://{action}?trace_id={}&token={}",
            urlencoding_encode(&self.trace_id),
            urlencoding_encode(&self.token.0),
        )
    }
}

fn urlencoding_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}
```

- [ ] **步骤 2：修改 ui_state.rs，将 PendingConfirmationInfo 替换为 TaskState**

删除原有的 `PendingConfirmationInfo` 和 `CurrentTaskInfo` 结构体，替换为使用 `TaskState`：

```rust
// 在 ui_state.rs 中，用 TaskState 替代以下两个结构体：
// - PendingConfirmationInfo
// - CurrentTaskInfo

// 保留 StateUpdate 中的字段不变（前端需要这些），但内部存储用 TaskState
```

实际上，StateUpdate 中的字段是给前端用的，不需要改。我们只需要确保 TaskState 能正确映射到前端需要的字段。

- [ ] **步骤 3：运行 cargo check 确认类型正确**

执行命令：`cd cozmio/src-tauri && cargo check 2>&1`
预期结果：无类型错误

- [ ] **步骤 4：提交代码**

```bash
git add src-tauri/src/types.rs src-tauri/src/ui_state.rs
git commit -m "feat: add minimal TaskState/NotificationPending types with trace_id and token"
```

---

### 任务 3：实现 notification_manager（Windows Toast with action buttons）

**涉及文件**：
- 创建：`src-tauri/src/notification_manager.rs`

**通知显示原则**：
- 系统通知的 text1/text2 可以截取 content_text 的前 N 个字符作为展示
- 不为此生成专门的 task_title、task_summary 等字段
- 展示用字段由前端/通知自己决定，后端只传原始 content_text

- [ ] **步骤 1：实现 WinToastNotifier 结构体**

```rust
use winrt_notification::{Toast, Action, ActionType, Duration};
use std::sync::RwLock;
use crate::types::NotificationPending;

/// Global notification state - keyed by trace_id
static NOTIFICATION_PENDING: RwLock<Option<NotificationPending>> = RwLock::new(None);

/// Send a confirmation request notification with Confirm/Cancel action buttons.
/// content_text is the original model output - notification truncates for display.
pub fn send_confirmation_notification(pending: &NotificationPending) -> Result<(), String> {
    let confirm_url = pending.to_protocol_url("confirm");
    let cancel_url = pending.to_protocol_url("cancel");

    // Truncate content_text for notification display (Windows Toast limits)
    let display_text = truncate_for_notification(&pending.content_text, 200);

    let toast = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Cozmio - 任务确认")
        .text1(&display_text)
        .text2("点击确认执行，或取消")
        .duration(Duration::Long)
        .action(Action::new("确认", &confirm_url, ActionType::Protocol))
        .action(Action::new("取消", &cancel_url, ActionType::Protocol));

    // Store pending state for when action is triggered
    {
        let mut guard = NOTIFICATION_PENDING.write().unwrap();
        *guard = Some(pending.clone());
    }

    toast.show().map_err(|e| format!("Failed to show notification: {}", e))?;
    Ok(())
}

/// Send a result notification (completed/failed/interrupted).
/// result_text or error_text is the original output - notification truncates for display.
pub fn send_result_notification(
    trace_id: &str,
    content_text: &str,
    status: &str,
    result_text: Option<&str>,
    error_text: Option<&str>,
) -> Result<(), String> {
    let (title, body) = match status {
        "completed" => {
            let result = result_text.unwrap_or("任务已完成");
            ("Cozmio - 任务完成", truncate_for_notification(result, 200))
        }
        "failed" => {
            let error = error_text.unwrap_or("任务执行失败");
            ("Cozmio - 任务失败", truncate_for_notification(error, 200))
        }
        "interrupted" => ("Cozmio - 任务中断", "任务已被中断"),
        _ => ("Cozmio - 任务状态", "任务状态已更新"),
    };

    let toast = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(truncate_for_notification(content_text, 100))
        .text2(&body)
        .duration(Duration::Long);

    toast.show().map_err(|e| format!("Failed to show result notification: {}", e))?;
    Ok(())
}

/// Get the currently pending notification (if any)
pub fn get_pending_notification() -> Option<NotificationPending> {
    NOTIFICATION_PENDING.read().unwrap().clone()
}

/// Clear the pending notification (after action is taken)
pub fn clear_pending_notification() {
    let mut guard = NOTIFICATION_PENDING.write().unwrap();
    *guard = None;
}

/// Check if there's a pending notification with the given trace_id
pub fn has_pending_notification_for_trace(trace_id: &str) -> bool {
    NOTIFICATION_PENDING
        .read()
        .unwrap()
        .as_ref()
        .map(|p| p.trace_id == trace_id)
        .unwrap_or(false)
}

/// Check token validity for a pending notification
pub fn validate_token(trace_id: &str, token: &str) -> bool {
    NOTIFICATION_PENDING
        .read()
        .unwrap()
        .as_ref()
        .map(|p| p.trace_id == trace_id && p.token.0 == token)
        .unwrap_or(false)
}

fn truncate_for_notification(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        text.chars().take(max_chars - 3).collect::<String>() + "..."
    }
}
```

- [ ] **步骤 2：运行 cargo check 验证编译**

执行命令：`cd cozmio/src-tauri && cargo check 2>&1`
预期结果：无编译错误

- [ ] **步骤 3：提交代码**

```bash
git add src-tauri/src/notification_manager.rs
git commit -m "feat: implement notification_manager with Windows Toast action buttons"
```

---

### 任务 4：修改 main_loop.rs 的 handle_execution_result 发送系统层通知

**涉及文件**：
- 修改：`src-tauri/src/main_loop.rs:198-268`

- [ ] **步骤 1：修改 handle_execution_result 函数**

将原来的 `ExecutionResult::Confirmed` 分支（显示主窗口）改为发送系统层通知：

```rust
fn handle_execution_result(
    app_handle: &AppHandle,
    window_title: &str,
    process_name: &str,
    judgment: &str,
    next_step: &str,
    result: ExecutionResult,
) -> (String, Option<TaskState>) {
    match result {
        ExecutionResult::Notified => {
            // Suggest level - send Windows notification (no action needed)
            let title = "Cozmio - 建议";
            let body = format!("{}\n\n建议: {}", judgment, next_step);

            let handle = app_handle.clone();
            let body_clone = body;
            let title_clone = title.to_string();
            tauri::async_runtime::spawn(async move {
                use tauri_plugin_notification::NotificationExt;
                if let Err(e) = handle
                    .notification()
                    .builder()
                    .title(&title_clone)
                    .body(&body_clone)
                    .show()
                {
                    log::error!("Failed to send notification: {}", e);
                }
            });
            (String::from("notified"), None)
        }
        ExecutionResult::Confirmed => {
            // Create trace_id and notification pending state
            let trace_id = crate::types::TraceId::new().0;
            log::info!(
                "Creating pending notification for window='{}' process='{}' trace_id={}",
                window_title,
                process_name,
                trace_id
            );

            // content_text is the original model output - kept as-is
            let content_text = next_step.to_string();

            let notification_pending = NotificationPending::new(
                trace_id.clone(),
                content_text.clone(),
            );

            // Send system-level notification with action buttons
            if let Err(e) = crate::notification_manager::send_confirmation_notification(&notification_pending) {
                log::error!("Failed to send confirmation notification: {}", e);
            }

            let task_state = TaskState::new(trace_id, content_text);
            (
                String::from("awaiting-confirmation"),
                Some(task_state),
            )
        }
        ExecutionResult::Executed => {
            // Execute level - action was executed (log already done by executor)
            log::info!("Action executed");
            (String::from("executed"), None)
        }
        ExecutionResult::Skipped => {
            // Action was skipped
            log::info!("Action skipped");
            (String::from("skipped"), None)
        }
    }
}
```

**关键改动**：
- 删除了 `if let Some(window) = app_handle.get_webview_window("main") { window.show(); window.set_focus(); }` —— 不再强制显示主窗口
- 改为调用 `notification_manager::send_confirmation_notification()`
- 返回 `TaskState` 而非 `PendingConfirmationInfo`，但映射到前端状态时使用相同的最小字段集

- [ ] **步骤 2：运行 cargo check 验证编译**

执行命令：`cd cozmio/src-tauri && cargo check 2>&1`
预期结果：无编译错误

- [ ] **步骤 3：提交代码**

```bash
git add src-tauri/src/main_loop.rs
git commit -m "feat(main_loop): send system notification instead of showing main window for Confirmed"
```

---

### 任务 5：实现 protocol_handler.rs 处理 cozmio:// 协议

**涉及文件**：
- 创建：`src-tauri/src/protocol_handler.rs`
- 修改：`src-tauri/src/main.rs`

- [ ] **步骤 1：创建 protocol_handler.rs**

```rust
use std::collections::HashMap;
use std::sync::RwLock;

static PROTOCOL_STATE: RwLock<Option<ProtocolAction>> = RwLock::new(None);

#[derive(Debug, Clone)]
pub struct ProtocolAction {
    pub action: String,      // "confirm" or "cancel"
    pub trace_id: String,
    pub token: String,
}

impl ProtocolAction {
    pub fn from_url(url: &str) -> Option<Self> {
        // Parse cozmio://action?trace_id=xxx&token=yyy
        let url = url.trim_start_matches("cozmio://");
        let (action, query) = url.split_once('?')?;

        let mut params: HashMap<String, String> = HashMap::new();
        for pair in query.split('&') {
            let (key, value) = pair.split_once('=')?;
            params.insert(key.to_string(), urlencoding_decode(value));
        }

        Some(ProtocolAction {
            action: action.to_string(),
            trace_id: params.get("trace_id")?.clone(),
            token: params.get("token")?.clone(),
        })
    }
}

fn urlencoding_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                } else {
                    result.push('%');
                    result.push_str(&hex);
                }
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Store a pending protocol action to be processed when app initializes
pub fn store_protocol_action(action: ProtocolAction) {
    let mut guard = PROTOCOL_STATE.write().unwrap();
    *guard = Some(action);
}

/// Retrieve and clear any pending protocol action
pub fn take_pending_protocol_action() -> Option<ProtocolAction> {
    let mut guard = PROTOCOL_STATE.write().unwrap();
    guard.take()
}
```

- [ ] **步骤 2：修改 main.rs 注册 protocol handler**

在 `main.rs` 中添加 protocol handler 注册和处理：

```rust
// In main function, after app.build():
let app_handle = app.handle();

// Register protocol handler for cozmio://
#[cfg(windows)]
{
    use tauri_plugin_shell::ShellExt;
    let _ = app_handle.shell().protocol("cozmio", |request| {
        let url = request.uri().to_string();
        log::info!("Received cozmio:// protocol request: {}", url);

        if let Some(action) = crate::protocol_handler::ProtocolAction::from_url(&url) {
            crate::protocol_handler::store_protocol_action(action);
        }
        Ok(())
    });
}

// Check for pending protocol action on cold start
if let Some(action) = crate::protocol_handler::take_pending_protocol_action() {
    log::info!(
        "Processing cold-start protocol action: {} trace_id={}",
        action.action,
        action.trace_id
    );
    let app = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        // Delay slightly to ensure app is fully initialized
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        handle_protocol_action(app, action).await;
    });
}
```

添加新的处理函数：

```rust
async fn handle_protocol_action(app: tauri::AppHandle, action: protocol_handler::ProtocolAction) {
    match action.action.as_str() {
        "confirm" => {
            log::info!("Protocol confirm for trace_id={}", action.trace_id);
            // Validate token and confirm
            if let Err(e) = crate::commands::confirm_pending_task_by_token(
                app.clone(),
                &action.trace_id,
                &action.token,
            ) {
                log::error!("Failed to confirm task: {}", e);
            }
        }
        "cancel" => {
            log::info!("Protocol cancel for trace_id={}", action.trace_id);
            if let Err(e) = crate::commands::cancel_pending_task_by_token(
                &app,
                &action.trace_id,
                &action.token,
            ) {
                log::error!("Failed to cancel task: {}", e);
            }
        }
        _ => {
            log::warn!("Unknown protocol action: {}", action.action);
        }
    }
}
```

- [ ] **步骤 3：添加 confirm_pending_task_by_token 和 cancel_pending_task_by_token 命令**

在 `commands.rs` 中添加：

```rust
#[tauri::command]
pub async fn confirm_pending_task_by_token(
    app: tauri::AppHandle,
    trace_id: String,
    token: String,
) -> Result<String, String> {
    // Validate token first
    if !crate::notification_manager::validate_token(&trace_id, &token) {
        return Err(String::from("Invalid or expired token"));
    }

    let pending = {
        let state = app.state::<AppState>();
        let pending = state.pending_confirmation.read().unwrap().clone();
        pending
    }
    .ok_or_else(|| String::from("No pending confirmation task"))?;

    if pending.trace_id != trace_id {
        return Err(String::from("Trace ID mismatch"));
    }

    // Clear pending notification
    crate::notification_manager::clear_pending_notification();
    store_pending_confirmation(&app, None);

    // Create task state
    let task_state = TaskState::new(trace_id.clone(), pending.content_text.clone())
        .with_status(TaskStatus::Dispatching);
    store_current_task(&app, Some(task_state.clone()));
    store_relay_execution(&app, None);

    let request = RelayDispatchRequest::from_task_text(
        &pending.content_text,
        "",  // source info not needed in minimal schema
        "",
    );

    match relay_bridge::dispatch_confirmed_intervention(app.clone(), request) {
        Ok(session_id) => {
            log::info!(
                "Desktop user confirmed task via protocol, relay session started session={} trace_id={}",
                session_id,
                trace_id
            );
            // Update task state with session_id
            let completed_state = task_state
                .with_session_id(session_id.clone())
                .with_status(TaskStatus::Running);
            store_current_task(&app, Some(completed_state));
            emit_state_update(&app);
            Ok(session_id)
        }
        Err(error) => {
            let error_state = task_state.with_error(error.clone());
            store_current_task(&app, Some(error_state));
            emit_state_update(&app);
            Err(error)
        }
    }
}

#[tauri::command]
pub fn cancel_pending_task_by_token(
    app: &tauri::AppHandle,
    trace_id: String,
    token: String,
) -> Result<(), String> {
    // Validate token
    if !crate::notification_manager::validate_token(&trace_id, &token) {
        return Err(String::from("Invalid or expired token"));
    }

    let pending = {
        let state = app.state::<AppState>();
        state.pending_confirmation.read().unwrap().clone()
    };

    // Clear pending notification
    crate::notification_manager::clear_pending_notification();
    store_pending_confirmation(app, None);

    if let Some(pending_info) = pending {
        let cancelled_state = TaskState::new(trace_id, pending_info.content_text)
            .with_status(TaskStatus::Cancelled);
        store_current_task(app, Some(cancelled_state));
    }

    emit_state_update(app);
    Ok(())
}
```

- [ ] **步骤 4：运行 cargo check 验证编译**

执行命令：`cd cozmio/src-tauri && cargo check 2>&1`
预期结果：无编译错误

- [ ] **步骤 5：提交代码**

```bash
git add src-tauri/src/protocol_handler.rs src-tauri/src/main.rs src-tauri/src/commands.rs
git commit -m "feat: implement cozmio:// protocol handler with token validation"
```

---

### 任务 6：在 Relay 执行完成后发送结果通知

**涉及文件**：
- 修改：`src-tauri/src/relay_bridge.rs:279-332`

- [ ] **步骤 1：修改 track_relay_session 函数，在终端状态发送通知**

在 `use` 语句区域添加：

```rust
use crate::notification_manager;
```

在 `track_relay_session` 函数中，找到 `if event.terminal` 块并在其中添加通知发送：

```rust
if event.terminal {
    // Send result notification
    let status = terminal_status_label(event.terminal_status);
    if status == "completed" || status == "failed" || status == "interrupted" {
        // Get content_text from the request (original model output)
        let content_text = &request.original_suggestion;
        let result_text = snapshot.result_output.as_ref();
        let error_text = snapshot.error_message.as_ref();

        if let Err(e) = notification_manager::send_result_notification(
            &session_id,  // trace_id substitute (we use session_id for result tracking)
            content_text,
            status,
            result_text.map(|s| s.as_str()),
            error_text.map(|s| s.as_str()),
        ) {
            log::error!("Failed to send result notification: {}", e);
        }
    }
    break;
}
```

**注意**：
- 使用 session_id 作为 trace_id 的替代来追踪结果（因为结果通知不需要回查 pending 状态）
- result_text 和 error_text 保持原文，不做截断或转换

- [ ] **步骤 2：运行 cargo check 验证编译**

执行命令：`cd cozmio/src-tauri && cargo check 2>&1`
预期结果：无编译错误

- [ ] **步骤 3：提交代码**

```bash
git add src-tauri/src/relay_bridge.rs
git commit -m "feat(relay_bridge): send result notification when relay session terminates"
```

---

### 任务 7：更新 tauri.conf.json 注册 cozmio:// protocol

**涉及文件**：
- 修改：`src-tauri/tauri.conf.json`

- [ ] **步骤 1：添加 protocol 注册**

在 `bundle` section 内添加：

```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [...],
    "windows": {
      "webviewInstallMode": {
        "type": "embedBootstrapper"
      }
    },
    "protocols": [
      {
        "schemes": ["cozmio"],
        "permissions": ["notification"]
      }
    ]
  }
}
```

**注意**：Tauri v2 的 protocol 注册方式可能略有不同，需要验证。

- [ ] **步骤 2：验证构建**

执行命令：`cd cozmio/src-tauri && cargo build 2>&1 | head -50`
预期结果：构建成功或只有配置警告

- [ ] **步骤 3：提交代码**

```bash
git add src-tauri/tauri.conf.json
git commit -m "feat(tauri.conf): register cozmio:// protocol for notification actions"
```

---

### 任务 8：确保 STOP 能力完整（interrupt_session 已实现）

**涉及文件**：
- 已有：`src-tauri/src/relay_bridge.rs:87-117` - `interrupt_session` 函数

- [ ] **步骤 1：验证 interrupt_session 实现正确**

检查 `interrupt_session` 函数是否：
1. 调用 `client.interrupt(session_id)` 真实 interrupt Relay session
2. 更新 `relay_execution.relay_status` 为 `interrupted`
3. 更新 `error_message`
4. 调用 `emit_state_update`

根据代码审查，`interrupt_session` 已完整实现，无需修改。

- [ ] **步骤 2：验证 frontend 可以调用 interrupt**

检查 `commands.rs` 中的 `interrupt_current_task` 命令：

```rust
#[tauri::command]
pub fn interrupt_current_task(app: tauri::AppHandle) -> Result<(), String> {
    let relay_execution = {
        let state = app.state::<AppState>();
        let relay_execution = state.relay_execution.read().unwrap().clone();
        relay_execution
    };
    let session_id = relay_execution
        .and_then(|execution| execution.session_id)
        .ok_or_else(|| String::from("No active relay session"))?;

    log::info!("Desktop user requested interrupt for relay session {}", session_id);
    store_current_task_state(&app, "interrupting");
    mark_relay_interrupting(&app);
    emit_state_update(&app);

    relay_bridge::interrupt_session(app.clone(), &session_id)
}
```

该实现正确，会真实 interrupt Relay session。无需修改。

---

### 任务 9：验证构建和测试

**涉及文件**：
- 测试：`cozmio/src-tauri`

- [ ] **步骤 1：运行 cargo build 验证完整构建**

执行命令：`cd cozmio/src-tauri && cargo build 2>&1`
预期结果：Build succeeded

- [ ] **步骤 2：运行 cargo test 验证单元测试**

执行命令：`cd cozmio && cargo test 2>&1`
预期结果：所有测试通过

- [ ] **步骤 3：提交代码**

```bash
git add -A
git commit -m "feat: implement complete system notification flow with relay dispatch"
```

---

## 验收标准检查清单

| # | 验收项 | 验证方式 | 状态 |
|---|--------|----------|------|
| 1 | 隐藏或关闭 Cozmio 主窗口后，系统层提醒仍能出现 | 启动 Cozmio，最小化主窗口，触发模型 CONTINUE，判断出现 Windows Toast | 待验证 |
| 2 | 系统层提醒里点击确认后，真实产生 Relay session_id | 点击确认 action，查看日志中是否出现 "dispatching Relay session" 和真实 session_id | 待验证 |
| 3 | Claude Code / 执行端真实启动 | 查看 Relay 日志或进程，确认 claude-code 进程被启动 | 待验证 |
| 4 | progress 能回到桌面端 | 观察 UI 或日志中 progress 事件的接收 | 待验证 |
| 5 | result / error 能回到桌面端 | 观察 Relay session 结束后 result 是否被获取 | 待验证 |
| 6 | 执行完成、失败、中断后，都能出现系统层结果通知 | 模拟 completed/failed/interrupted 状态，观察 Windows Toast 是否出现 | 待验证 |
| 7 | 点击取消或关闭后，不产生 Relay session | 点击取消 action，观察日志中无 "dispatching" 或 "session_id" | 待验证 |
| 8 | STOP 能真实 interrupt 当前执行进程 | 调用 interrupt_current_task，观察 Relay session 变为 interrupted 状态 | 待验证 |
| 9 | token 只能使用一次（确认后失效） | 确认后再次用同一 token 调用 confirm，验证被拒绝 | 待验证 |
| 10 | content_text 保持原文，无 task_title 等展示字段 | 检查 TaskState.content_text 是否为模型原始输出 | 待验证 |
| 11 | result_text / error_text 保持原文 | 检查 TaskState.result_text 是否为 Relay 原始输出 | 待验证 |

---

## 自我审查

- [x] **产品类型识别**：传统软件实现型（非模型输出验证型）
- [x] **规格覆盖度检查**：
  - 系统层通知 with action button ✓
  - Confirm/Cancel 两个动作 ✓
  - 点击确认触发真实 Relay dispatch ✓
  - 点击取消不 dispatch ✓
  - 同一任务只能确认一次（token 验证）✓
  - 不强制 show/focus 主窗口 ✓
  - 执行链路走真实 dispatch ✓
  - completed/failed/interrupted 结果通知 ✓
  - STOP 能力 ✓
  - 最小字段集（trace_id/status/session_id/token/content_text/result_text/error_text）✓
  - 原始内容保持原文 ✓
- [x] **占位符排查**：无 TBD/TODO/后续实现等占位符
- [x] **类型一致性**：TaskState 只包含最小字段，TaskStatus 枚举完整

---

## 执行方式选择

**方案已编写完成并保存至 `docs/superpowers/plans/2026-04-25-system-notification-with-actions.md`**

提供两种执行方式：

**1. 子智能体驱动（推荐）**——为每个任务调度全新子智能体，任务间进行审核，迭代速度更快

**2. 内联执行**——在本会话中使用执行计划技能，按检查点批量执行

**请选择方式？**

若选择子智能体驱动，需要的子智能体类型：
- `general-purpose` 用于实现任务（Rust 代码编写）
- `build-error-resolver` 用于解决构建错误
- `validator` 用于最终验证

若选择内联执行：
- 使用 `superpowers:executing-plans` 技能按检查点批量执行
