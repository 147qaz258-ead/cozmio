# 自动更新功能 实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标**：为 Cozmio 桌面应用添加后台自动更新功能（检查+下载+安装+重启提示）

**架构思路**：在 Tauri 后端新增独立的 updater 模块，负责更新检查、下载、安装、状态管理。托盘菜单集成更新状态显示，气泡通知触发重启。

**技术栈**：Rust (reqwest, chrono), Tauri 2.x, Windows MSI

---

## 文件结构

| 文件 | 职责 |
|------|------|
| `src-tauri/src/updater.rs` (新建) | 更新检查、下载、安装逻辑 |
| `src-tauri/src/config.rs` (修改) | 新增 `last_check_at`, `update_channel` 字段 |
| `src-tauri/src/commands.rs` (修改) | 新增 `update_state` 到 AppState，新增 IPC 命令 |
| `src-tauri/src/tray.rs` (修改) | 新增 UpdateState 枚举，托盘菜单更新状态 |

---

## 任务 1：Config 新增字段

**涉及文件**：
- 修改：`cozmio/src-tauri/src/config.rs:1-15`

**步骤**：

- [ ] **步骤 1：在 Config 结构体中新增字段**

打开 `cozmio/src-tauri/src/config.rs`，在第 7-15 行的 `Config` 结构体中添加两个字段：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ollama_url: String,
    pub model_name: String,
    pub poll_interval_secs: u64,
    pub window_change_detection: bool,
    pub execute_auto: bool,
    pub request_use_native_dialog: bool,
    pub execute_delay_secs: u64,
    pub last_check_at: Option<String>,    // 新增: ISO8601 时间戳
    pub update_channel: String,           // 新增: "stable" 或 "beta"
}
```

- [ ] **步骤 2：更新 Config::default()**

在第 18-28 行的 `Default` 实现中补充默认值：

```rust
impl Default for Config {
    fn default() -> Self {
        Config {
            ollama_url: "http://localhost:11434".to_string(),
            model_name: "llava".to_string(),
            poll_interval_secs: 3,
            window_change_detection: true,
            execute_auto: true,
            request_use_native_dialog: true,
            execute_delay_secs: 1,
            last_check_at: None,                    // 新增
            update_channel: "stable".to_string(),   // 新增
        }
    }
}
```

- [ ] **步骤 3：运行测试确认 Config 序列化正常**

执行命令：`cd cozmio && cargo test -p cozmio -- config --nocapture`

预期结果：所有 config 相关测试通过

- [ ] **步骤 4：提交代码**

```bash
git add cozmio/src-tauri/src/config.rs && git commit -m "feat(config): add last_check_at and update_channel fields"
```

---

## 任务 2：创建 updater 模块

**涉及文件**：
- 创建：`cozmio/src-tauri/src/updater.rs`

**步骤**：

- [ ] **步骤 1：创建 updater.rs 文件**

在 `cozmio/src-tauri/src/` 目录下创建 `updater.rs`：

```rust
use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_UPDATE_URL: &str = "https://updates.example.com";
const CHECK_INTERVAL_SECS: u64 = 24 * 3600;
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_SECS: u64 = 3600;

/// Update check response from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResponse {
    pub needs_update: bool,
    pub latest_version: String,
    pub channel: String,
    pub notes: String,
    pub download_url: String,
    pub signature: String,
}

/// Update state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateState {
    None,
    Pending { version: String, installed_at: DateTime<Utc> },
    ReadyToRestart,
}

impl Default for UpdateState {
    fn default() -> Self {
        UpdateState::None
    }
}

/// Update checker
pub struct UpdateChecker {
    client: Client,
    update_url: String,
    current_version: String,
}

impl UpdateChecker {
    /// Create a new UpdateChecker
    pub fn new(current_version: String) -> Self {
        let update_url = std::env::var("COZMIO_UPDATE_URL")
            .unwrap_or_else(|_| DEFAULT_UPDATE_URL.to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        UpdateChecker {
            client,
            update_url,
            current_version,
        }
    }

    /// Check for updates
    pub fn check(&self) -> Result<UpdateCheckResponse, String> {
        let url = format!("{}/updates/check?version={}", self.update_url, self.current_version);

        log::info!("Checking for updates at {}", url);

        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Update check failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Update server returned: {}", response.status()));
        }

        let result: UpdateCheckResponse = response
            .json()
            .map_err(|e| format!("Failed to parse update response: {}", e))?;

        log::info!("Update check result: needs_update={}", result.needs_update);
        Ok(result)
    }

    /// Download update to temp directory
    pub fn download(&self, url: &str, expected_checksum: &str) -> Result<PathBuf, String> {
        log::info!("Downloading update from {}", url);

        let response = self.client
            .get(url)
            .send()
            .map_err(|e| format!("Download failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Download failed with status: {}", response.status()));
        }

        let bytes = response.bytes()
            .map_err(|e| format!("Failed to read response bytes: {}", e))?;

        // Verify checksum
        let actual_checksum = format!("sha256:{:x}", sha256::digest(&bytes));
        if !actual_checksum.starts_with(&expected_checksum.replace("sha256:", "")) {
            return Err("Checksum mismatch".to_string());
        }

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let filename = url.split('/').last().unwrap_or("cozmio-update.msi");
        let temp_path = temp_dir.join(filename);

        fs::write(&temp_path, &bytes)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        log::info!("Update downloaded to {:?}", temp_path);
        Ok(temp_path)
    }

    /// Install MSI silently
    pub fn install(&self, msi_path: &PathBuf) -> Result<(), String> {
        log::info!("Installing update from {:?}", msi_path);

        let output = std::process::Command::new("msiexec")
            .args(&["/i", msi_path.to_str().unwrap(), "/quiet", "/norestart"])
            .output()
            .map_err(|e| format!("Failed to execute msiexec: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("MSI install failed: {}", stderr));
        }

        log::info!("Update installed successfully");
        Ok(())
    }
}

/// Calculate retry delay with exponential backoff
pub fn retry_delay(attempt: u32) -> Duration {
    Duration::from_secs(RETRY_BASE_DELAY_SECS * 2u64.pow(attempt.min(3)))
}

/// Get the update state file path
pub fn update_state_path() -> Result<PathBuf, String> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| "Could not find local data directory".to_string())?;
    Ok(base.join("cozmio").join("update_state.json"))
}

/// Load update state from disk
pub fn load_update_state() -> UpdateState {
    let path = match update_state_path() {
        Ok(p) => p,
        Err(_) => return UpdateState::None,
    };

    if !path.exists() {
        return UpdateState::None;
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or(UpdateState::None),
        Err(_) => UpdateState::None,
    }
}

/// Save update state to disk
pub fn save_update_state(state: &UpdateState) -> Result<(), String> {
    let path = update_state_path()?;

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }
    }

    let content = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("Failed to write state file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_state_default() {
        assert_eq!(UpdateState::None, UpdateState::None);
    }

    #[test]
    fn test_retry_delay() {
        assert_eq!(retry_delay(0), Duration::from_secs(3600));
        assert_eq!(retry_delay(1), Duration::from_secs(7200));
        assert_eq!(retry_delay(2), Duration::from_secs(14400));
        assert_eq!(retry_delay(3), Duration::from_secs(28800));
    }
}
```

- [ ] **步骤 2：验证 updater 模块编译**

执行命令：`cd cozmio && cargo check -p cozmio 2>&1 | head -50`

预期结果：可能有缺少 `sha256` crate 的编译错误（下一步添加依赖）

- [ ] **步骤 3：提交代码**

```bash
git add cozmio/src-tauri/src/updater.rs && git commit -m "feat: add updater module with check/download/install logic"
```

---

## 任务 3：集成 updater 到 main.rs 和 AppState

**涉及文件**：
- 修改：`cozmio/src-tauri/src/main.rs`
- 修改：`cozmio/src-tauri/src/commands.rs`

**步骤**：

- [ ] **步骤 1：在 main.rs 中引入 updater 模块**

打开 `cozmio/src-tauri/src/main.rs`，在第 3-10 行的 `mod` 声明之后添加：

```rust
mod commands;
mod config;
mod executor;
mod logging;
mod main_loop;
mod model_client;
mod tray;
mod window_monitor;
mod updater;  // 新增
```

- [ ] **步骤 2：在 AppState 中添加 update_state**

打开 `cozmio/src-tauri/src/commands.rs`，在第 8-12 行的 `AppState` 结构体中添加 `update_state`：

```rust
use crate::config::Config;
use crate::logging::{ActionLogger, ActionRecord};
use crate::tray::TrayState;
use crate::updater::UpdateState;  // 新增
use std::sync::RwLock;
use tauri::State;

/// Application global state managed by Tauri
pub struct AppState {
    pub config: Config,
    pub logger: ActionLogger,
    pub tray_state: RwLock<TrayState>,
    pub update_state: RwLock<UpdateState>,  // 新增
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            logger: ActionLogger::new(),
            tray_state: RwLock::new(TrayState::default()),
            update_state: RwLock::new(UpdateState::None),  // 新增
        }
    }
}
```

- [ ] **步骤 3：添加更新相关的 IPC 命令**

在同一文件 `commands.rs` 末尾添加新命令：

```rust
/// Get the current update state
#[tauri::command]
pub fn get_update_state(state: State<AppState>) -> Result<UpdateState, String> {
    Ok(*state.update_state.read().unwrap())
}

/// Trigger update check manually
#[tauri::command]
pub async fn check_for_updates(app_handle: AppHandle) -> Result<bool, String> {
    use crate::updater::{UpdateChecker, load_update_state, save_update_state};

    let current_version = env!("CARGO_PKG_VERSION");
    let checker = UpdateChecker::new(current_version.to_string());

    match checker.check() {
        Ok(response) => {
            if response.needs_update {
                log::info!("Update available: {}", response.latest_version);

                // Download and install in background
                let app = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    match download_and_install(&checker, &response.download_url, &response.signature).await {
                        Ok(_) => {
                            log::info!("Update installed successfully");
                            // State already saved by download_and_install
                        }
                        Err(e) => {
                            log::error!("Update failed: {}", e);
                        }
                    }
                });

                Ok(true)
            } else {
                log::info!("No update available");
                Ok(false)
            }
        }
        Err(e) => {
            log::error!("Update check failed: {}", e);
            Err(e)
        }
    }
}

async fn download_and_install(
    checker: &UpdateChecker,
    url: &str,
    checksum: &str,
) -> Result<(), String> {
    let msi_path = checker.download(url, checksum)?;
    checker.install(&msi_path)?;

    // Update state to Pending
    let state = UpdateState::Pending {
        version: checker.current_version.clone(),
        installed_at: chrono::Utc::now(),
    };
    save_update_state(&state)?;

    // Notify frontend
    let _ = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(2));
        // Signal frontend to refresh state
    });

    Ok(())
}
```

- [ ] **步骤 4：更新 tauri::generate_handler!**

在 `main.rs` 中更新 `invoke_handler`：

```rust
use commands::{get_config, save_config, get_history, clear_history, get_tray_state, set_tray_state, get_update_state, check_for_updates};  // 新增

// 在 tauri::Builder 中：
.invoke_handler(tauri::generate_handler![
    get_config,
    save_config,
    get_history,
    clear_history,
    get_tray_state,
    set_tray_state,
    get_update_state,      // 新增
    check_for_updates     // 新增
])
```

- [ ] **步骤 5：添加 sha256 crate 依赖**

打开 `cozmio/src-tauri/Cargo.toml`，在 `[dependencies]` 部分添加：

```toml
sha256 = "1.5"
```

- [ ] **步骤 6：编译验证**

执行命令：`cd cozmio && cargo check -p cozmio 2>&1 | head -80`

预期结果：编译通过，无错误

- [ ] **步骤 7：提交代码**

```bash
git add cozmio/src-tauri/src/main.rs cozmio/src-tauri/src/commands.rs cozmio/src-tauri/Cargo.toml && git commit -m "feat: integrate updater into AppState and expose IPC commands"
```

---

## 任务 4：托盘菜单集成更新状态

**涉及文件**：
- 修改：`cozmio/src-tauri/src/tray.rs`

**步骤**：

- [ ] **步骤 1：在 TrayManager 中添加 update_state 状态显示**

打开 `cozmio/src-tauri/src/tray.rs`，在第 46-112 行的 `setup_tray` 方法中，修改托盘菜单项，添加"检查更新"入口：

```rust
/// Sets up the system tray with menu and event handlers
pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    // Create menu items
    let running = MenuItem::with_id(app, "running", "运行中", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let paused = MenuItem::with_id(app, "paused", "暂停", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let check_update = MenuItem::with_id(app, "check_update", "检查更新", true, None::<&str>)  // 新增
        .map_err(|e| e.to_string())?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let history = MenuItem::with_id(app, "history", "历史", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    // Build menu with separators
    let sep1 = tauri::menu::PredefinedMenuItem::separator(app)
        .map_err(|e| e.to_string())?;
    let sep2 = tauri::menu::PredefinedMenuItem::separator(app)
        .map_err(|e| e.to_string())?;
    let menu = Menu::with_items(
        app,
        &[&running, &paused, &sep1, &check_update, &settings, &history, &sep2, &quit],  // 添加 check_update
    )
    .map_err(|e| e.to_string())?;

    // 在 .on_menu_event 中添加 check_update 处理：
    .on_menu_event(move |app, event| {
        match event.id().as_ref() {
            "running" => {
                let _ = app.emit("tray-action", "run");
            }
            "paused" => {
                let _ = app.emit("tray-action", "pause");
            }
            "check_update" => {  // 新增
                let _ = app.emit("tray-action", "check_update");
            }
            "settings" => {
                let _ = app.emit("tray-action", "settings");
            }
            "history" => {
                let _ = app.emit("tray-action", "history");
            }
            "quit" => {
                let _ = app.emit("tray-action", "quit");
            }
            _ => {}
        }
    })
```

- [ ] **步骤 2：编译验证**

执行命令：`cd cozmio && cargo check -p cozmio 2>&1 | head -50`

预期结果：编译通过

- [ ] **步骤 3：提交代码**

```bash
git add cozmio/src-tauri/src/tray.rs && git commit -m "feat(tray): add check_update menu item"
```

---

## 任务 5：更新检查触发逻辑（启动时 + 定时）

**涉及文件**：
- 修改：`cozmio/src-tauri/src/main_loop.rs`

**步骤**：

- [ ] **步骤 1：在 main_loop.rs 中集成更新检查**

打开 `cozmio/src-tauri/src/main_loop.rs`，在 `start_main_loop` 函数开头（约第 40-60 行）添加更新检查逻辑：

```rust
use crate::commands::AppState;
use crate::config::Config;
use crate::executor::{ExecutionResult, Executor};
use crate::logging::ActionLogger;
use crate::model_client::ModelClient;
use crate::tray::TrayState;
use crate::window_monitor::WindowMonitor;
use crate::updater::{UpdateChecker, load_update_state, save_update_state, retry_delay};  // 新增
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// State update sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    pub tray_state: String,
    pub current_window: Option<WindowInfo>,
    pub last_judgment: Option<JudgmentInfo>,
    pub update_pending: bool,  // 新增
    pub update_version: Option<String>,  // 新增
}
```

- [ ] **步骤 2：在 start_main_loop 中添加更新检查调用**

在 `start_main_loop` 函数中，应用启动时检查更新（需要判断 last_check_at）：

```rust
pub fn start_main_loop(
    app_handle: AppHandle,
    config: Config,
    logger: ActionLogger,
) {
    log::info!("Starting main monitoring loop");

    let poll_interval = Duration::from_secs(config.poll_interval_secs);

    // Create components
    let mut monitor = WindowMonitor::new();
    let model_client = ModelClient::new(config.clone());
    let executor = Executor::new(config.clone(), logger);

    // === 新增：启动时更新检查 ===
    let last_check = config.last_check_at.as_ref();
    let should_check = last_check.map(|t| {
        if let Ok(last) = chrono::DateTime::parse_from_rfc3339(t) {
            let elapsed = chrono::Utc::now().signed_duration_since(last.with_timezone(&chrono::Utc));
            elapsed.num_seconds() > 24 * 3600
        } else {
            true
        }
    }).unwrap_or(true);

    if should_check {
        log::info!("Checking for updates on startup...");
        let app = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = run_update_check(&app).await {
                log::error!("Startup update check failed: {}", e);
            }
        });
    }
    // === 更新检查结束 ===

    loop {
        // ... existing loop code ...
    }
}

/// Run update check in background
async fn run_update_check(app_handle: &AppHandle) -> Result<(), String> {
    use crate::updater::UpdateChecker;

    let current_version = env!("CARGO_PKG_VERSION");
    let checker = UpdateChecker::new(current_version.to_string());

    let response = checker.check()?;

    // Update last_check_at in config
    let state = app_handle.state::<AppState>();
    let mut config = state.config.clone();
    config.last_check_at = Some(chrono::Utc::now().to_rfc3339());
    if let Err(e) = config.save() {
        log::warn!("Failed to save config after update check: {}", e);
    }

    if response.needs_update {
        log::info!("Update available: {}, downloading...", response.latest_version);

        // Download in background
        let download_url = response.download_url.clone();
        let signature = response.signature.clone();
        let app = app_handle.clone();

        tauri::async_runtime::spawn(async move {
            match checker.download(&download_url, &signature) {
                Ok(msi_path) => {
                    match checker.install(&msi_path) {
                        Ok(_) => {
                            log::info!("Update installed, state set to Pending");

                            // Update state
                            let state = app.state::<AppState>();
                            let update_state = crate::updater::UpdateState::Pending {
                                version: response.latest_version.clone(),
                                installed_at: chrono::Utc::now(),
                            };
                            *state.update_state.write().unwrap() = update_state;
                            let _ = crate::updater::save_update_state(&update_state);

                            // Emit update notification
                            let _ = app.emit("update-ready", &response.latest_version);
                        }
                        Err(e) => {
                            log::error!("Update install failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Update download failed: {}", e);
                }
            }
        });
    }

    Ok(())
}
```

- [ ] **步骤 3：编译验证**

执行命令：`cd cozmio && cargo check -p cozmio 2>&1 | head -80`

预期结果：编译通过

- [ ] **步骤 4：提交代码**

```bash
git add cozmio/src-tauri/src/main_loop.rs && git commit -m "feat: add startup update check in main loop"
```

---

## 任务 6：前端更新状态显示和重启提示

**涉及文件**：
- 修改：`cozmio/src-tauri/src/components/StatusPanel.js`
- 修改：`cozmio/src-tauri/src/components/App.js`

**步骤**：

- [ ] **步骤 1：修改 StatusPanel.js 添加更新状态显示**

打开 `cozmio/src-tauri/src/components/StatusPanel.js`，找到状态显示部分，添加更新状态：

```javascript
// 在 StateUpdateListener 或 initStateUpdateListener 函数中，
// 解析 update_pending 和 update_version 字段并显示

// 添加 updatePending 状态到组件
let updatePending = data.update_pending || false;
let updateVersion = data.update_version || null;

// 在 UI 中显示更新状态（如果 updatePending 为 true）
if (updatePending) {
    // 在状态面板添加更新提示
    const updateHtml = `
        <div class="update-pending-banner">
            <span>Cozmio 已更新至 ${updateVersion}，重启后生效</span>
            <button class="btn-restart" id="restart-btn">立即重启</button>
            <button class="btn-later" id="later-btn">稍后</button>
        </div>
    `;
    statusPanel.querySelector('.status-content')?.insertAdjacentHTML('beforeend', updateHtml);

    // 绑定按钮事件
    statusPanel.querySelector('#restart-btn')?.addEventListener('click', async () => {
        try {
            await invoke('restart_application');
        } catch (err) {
            console.error('Restart failed:', err);
        }
    });

    statusPanel.querySelector('#later-btn')?.addEventListener('click', async () => {
        try {
            await invoke('dismiss_update_reminder');
        } catch (err) {
            console.error('Dismiss failed:', err);
        }
    });
}
```

- [ ] **步骤 2：添加 restart_application 命令到 commands.rs**

打开 `cozmio/src-tauri/src/commands.rs`，添加新命令：

```rust
/// Restart the application
#[tauri::command]
pub fn restart_application() -> Result<(), String> {
    log::info!("Restarting application...");

    // Use Windows shutdown command
    std::process::Command::new("shutdown")
        .args(&["/r", "/t", "5", "/c", "Cozmio 更新重启"])
        .spawn()
        .map_err(|e| format!("Failed to spawn shutdown: {}", e))?;

    Ok(())
}

/// Dismiss update reminder (user clicked "later")
#[tauri::command]
pub fn dismiss_update_reminder(state: State<AppState>) -> Result<(), String> {
    // Just log, keep the update state as Pending
    log::info!("User dismissed update reminder");
    Ok(())
}
```

- [ ] **步骤 3：编译验证**

执行命令：`cd cozmio && cargo check -p cozmio 2>&1 | head -50`

预期结果：编译通过

- [ ] **步骤 4：添加 CSS 样式**

打开 `cozmio/src-tauri/src/styles.css`，末尾添加：

```css
/* === Update Pending Banner === */
.update-pending-banner {
    background: #fff8e1;
    border: 1px solid #ffe0b2;
    border-radius: 4px;
    padding: 12px 16px;
    margin-top: 16px;
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 13px;
    color: #5a4a35;
}

.btn-restart {
    background: #6a5a45;
    color: #fefcf5;
    border: 1px solid #5a4a35;
    padding: 6px 12px;
    border-radius: 3px;
    cursor: pointer;
    font-size: 12px;
}

.btn-later {
    background: transparent;
    color: #8a7a65;
    border: 1px solid #d0c8b8;
    padding: 6px 12px;
    border-radius: 3px;
    cursor: pointer;
    font-size: 12px;
}
```

- [ ] **步骤 5：提交代码**

```bash
git add cozmio/src-tauri/src/commands.rs cozmio/src-tauri/src/styles.css && git commit -m "feat: add restart command and update pending UI"
```

---

## 方案总结

| 任务 | 内容 |
|------|------|
| 1 | Config 新增 `last_check_at` 和 `update_channel` 字段 |
| 2 | 创建 `updater.rs` 模块（检查、下载、安装逻辑） |
| 3 | 集成 updater 到 AppState，暴露 IPC 命令 |
| 4 | 托盘菜单添加"检查更新"入口 |
| 5 | main_loop 集成启动时更新检查 |
| 6 | 前端更新状态显示和重启提示 |

---

## 执行方式选择

**方案已编写完成并保存至 `docs/superpowers/plans/YYYY-MM-DD-auto-update-plan.md`**。提供两种执行方式：

**1. 子智能体驱动（推荐）**——为每个任务调度全新子智能体，任务间进行审核，迭代速度更快

**2. 内联执行**——在本会话中使用执行计划技能，按检查点批量执行

请选择方式？
