# 长期驻留桌面应用实施方案（修订版 v2）

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标**：将 cozmio 打造成长期驻留运行的桌面智能体，实现应用驻留、运行状态、窗口显示三者解耦

**架构思路**：
- 单一权威运行状态（AtomicBool）驱动一切
- 托盘状态只展示权威状态，不自行管理"运行状态"
- 运行状态变更通过事件机制同步到所有观察者
- 应用日志正式写入文件，支持运行时查看

**技术栈**：Rust + Tauri v2 + tauri-plugin-single-instance + tauri-plugin-shell

---

## 核心概念澄清

| 旧名称（问题） | 新名称（正确） | 原因 |
|----------------|----------------|------|
| start_monitoring / stop_monitoring | start_running / stop_running | 产品是"运行器"，不是"监控工具" |
| get_monitoring_state | get_running_state | 保持语义一致 |
| TrayState::Running/Stopped | 只用 is_running() | 避免双重状态源 |
| is_running() + tray_state 双重检查 | 只检查 is_running() | 单一真相源 |

**状态设计原则**：
- 唯一权威状态：`static is_running: AtomicBool`
- TrayState 只保留内部 Processing 状态（main_loop 工作中）
- 所有 UI（托盘、主窗口）只读 is_running()
- 事件在调用处发送，不在状态设置函数内发送（避免 AppHandle 访问问题）

**单实例行为确认**：
- 再次启动 → 聚焦已有窗口 + 显示主界面
- 这意味着用户启动即希望看到内容界面

---

## 任务1：添加 Tauri 插件依赖

**涉及文件**：
- 修改：`cozmio/src-tauri/Cargo.toml`

- [ ] **步骤1：添加 single-instance 和 shell 插件**

在 `[dependencies]` 下添加：
```toml
tauri-plugin-single-instance = "2"
tauri-plugin-shell = "2"
```

注意：不加 autostart，后续单独任务处理

- [ ] **步骤2：验证编译**

执行命令：`cd cozmio && cargo build --package cozmio 2>&1 | tail -10`

预期结果：编译通过

- [ ] **步骤3：提交代码**

```bash
cd cozmio && git add src-tauri/Cargo.toml
git commit -m "feat: add single-instance and shell plugins"
```

---

## 任务2：定义单一权威运行状态

**涉及文件**：
- 创建：`cozmio/src-tauri/src/app_running.rs`

- [ ] **步骤1：创建 app_running.rs**

创建 `src-tauri/src/app_running.rs`：
```rust
use std::sync::atomic::{AtomicBool, Ordering};

/// 单一权威运行状态
/// - true = Running（应用在分析窗口）
/// - false = Stopped（应用驻留但不分析）
static RUNNING_STATE: AtomicBool = AtomicBool::new(false);

/// 获取当前运行状态
pub fn is_running() -> bool {
    RUNNING_STATE.load(Ordering::SeqCst)
}

/// 设置运行状态
/// 注意：不在此处发送事件，调用方负责发送 running-state-changed 事件
pub fn set_running(running: bool) {
    RUNNING_STATE.store(running, Ordering::SeqCst);
}
```

简洁明确：`set_running()` 只负责存状态，事件由调用方在调用处发送。

- [ ] **步骤2：验证编译**

执行命令：`cd cozmio && cargo build --package cozmio 2>&1 | tail -15`

预期结果：编译通过

- [ ] **步骤3：提交代码**

```bash
cd cozmio && git add src-tauri/src/app_running.rs
git commit -m "feat: add single authoritative running state"
```

---

## 任务3：初始化 AppHandle（主进程级别）

**涉及文件**：
- 修改：`cozmio/src-tauri/src/main.rs`

- [ ] **步骤1：在 main.rs 顶部声明全局 AppHandle**

在 `src-tauri/src/main.rs` 的开头添加：
```rust
use std::sync::{Arc, Mutex};

static APP_HANDLE: OnceCell<tauri::AppHandle> = OnceCell::new();

pub fn get_app_handle() -> Option<&tauri::AppHandle> {
    APP_HANDLE.get()
}
```

需要 `use std::sync::OnceCell;`。

- [ ] **步骤2：在 setup 闭包里初始化**

在 `mod cozmio` 的 `setup` 闭包开头添加：
```rust
APP_HANDLE.set(app.handle().clone()).ok();
```

- [ ] **步骤3：验证编译**

执行命令：`cd cozmio && cargo build --package cozmio 2>&1 | tail -15`

预期结果：编译通过

- [ ] **步骤4：提交代码**

```bash
cd cozmio && git add src-tauri/src/main.rs
git commit -m "feat: initialize app handle at main process level"
```

---

## 任务4：添加运行控制命令（start/stop/get）

**涉及文件**：
- 修改：`cozmio/src-tauri/src/commands.rs`

- [ ] **步骤1：更新 commands.rs 导入和命令**

将 `src-tauri/src/commands.rs` 添加导入：
```rust
use crate::app_running::{is_running, set_running};
```

添加新命令：
```rust
#[tauri::command]
fn start_running(app: tauri::AppHandle) -> Result<String, String> {
    set_running(true);
    // 在调用处发送事件
    let _ = app.emit("running-state-changed", "Running");
    Ok("Running".to_string())
}

#[tauri::command]
fn stop_running(app: tauri::AppHandle) -> Result<String, String> {
    set_running(false);
    // 在调用处发送事件
    let _ = app.emit("running-state-changed", "Stopped");
    Ok("Stopped".to_string())
}

#[tauri::command]
fn get_running_state() -> String {
    if is_running() { "Running" } else { "Stopped" }.to_string()
}
```

在 `generate_handler!` 中替换旧的 start_monitoring/stop_monitoring 命令。

- [ ] **步骤2：验证编译**

执行命令：`cd cozmio && cargo build --package cozmio 2>&1 | tail -15`

预期结果：编译通过

- [ ] **步骤3：提交代码**

```bash
cd cozmio && git add src-tauri/src/commands.rs
git commit -m "feat: rename commands to start/stop/get_running"
```

---

## 任务5：配置主窗口默认隐藏 + 单实例 + 窗口关闭事件

**涉及文件**：
- 修改：`cozmio/src-tauri/tauri.conf.json`
- 修改：`cozmio/src-tauri/src/main.rs`

- [ ] **步骤1：修改 tauri.conf.json 主窗口配置**

将 `src-tauri/tauri.conf.json:18-27` 替换为：
```json
"windows": [
  {
    "title": "Cozmio - 主动智能体",
    "width": 900,
    "height": 650,
    "resizable": true,
    "visible": false,
    "center": true,
    "minimizable": true,
    "closable": true
  }
]
```

- [ ] **步骤2：修改 main.rs 添加单实例和窗口事件处理**

将 `mod cozmio` 块替换为：
```rust
mod cozmio {
    use super::*;
    use commands::{get_config, save_config, get_history, clear_history, get_tray_state, set_tray_state};

    pub fn run() {
        tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_notification::init())
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                // 单实例：聚焦已有窗口并显示
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }))
            .plugin(tauri_plugin_shell::init())
            .manage(AppState::new())
            .invoke_handler(tauri::generate_handler![
                get_config,
                save_config,
                get_history,
                clear_history,
                get_tray_state,
                set_tray_state
            ])
            .setup(|app| {
                log::info!("Cozmio starting...");
                // 初始化运行状态句柄
                crate::APP_HANDLE.set(app.handle().clone()).ok();

                // Setup system tray
                if let Err(e) = TrayManager::setup_tray(app.handle()) {
                    log::error!("Failed to setup tray: {}", e);
                }

                // Load config and logger
                let config = Config::load().unwrap_or_default();
                let logger = ActionLogger::new();

                log::info!(
                    "Cozmio ready (poll_interval={}s, model={})",
                    config.poll_interval_secs,
                    config.model_name
                );

                // Spawn main loop in background thread
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    main_loop::start_main_loop(app_handle, config, logger);
                });

                Ok(())
            })
            .on_window_event(|window, event| {
                // 拦截窗口关闭事件，改为隐藏而不是退出
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let _ = window.hide();
                    api.prevent_close();
                }
            })
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
```

- [ ] **步骤3：验证编译**

执行命令：`cd cozmio && cargo build --package cozmio 2>&1 | tail -15`

预期结果：编译通过

- [ ] **步骤4：提交代码**

```bash
cd cozmio && git add src-tauri/tauri.conf.json src-tauri/src/main.rs
git commit -m "feat: configure hidden window, single instance, close-to-hide"
```

---

## 任务6：重构 TrayState，移除 Running/Stopped

**涉及文件**：
- 修改：`cozmio/src-tauri/src/tray.rs`
- 修改：`cozmio/src-tauri/src/commands.rs`

- [ ] **步骤1：修改 tray.rs**

将 `TrayState` 改为只包含内部状态：
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrayState {
    Idle,       // 应用空闲
    Processing, // 正在处理窗口
}
```

移除 "Running" 和 "Paused" 变体。

- [ ] **步骤2：更新 commands.rs**

移除对 TrayState::Running/Paused 的引用，改为只操作 AppState。

- [ ] **步骤3：更新 main_loop.rs**

将 `main_loop.rs` 中所有对 `TrayState::Running` 的检查改为检查 `is_running()`。

- [ ] **步骤4：验证编译**

执行命令：`cd cozmio && cargo build --package cozmio 2>&1 | tail -15`

预期结果：编译通过

- [ ] **步骤5：提交代码**

```bash
cd cozmio && git add src-tauri/src/tray.rs src-tauri/src/commands.rs src-tauri/src/main_loop.rs
git commit -m "refactor: remove Running/Paused from TrayState, use is_running()"
```

---

## 任务7：更新托盘菜单（动态状态展示）

**涉及文件**：
- 修改：`cozmio/src-tauri/src/tray.rs`

- [ ] **步骤1：更新 tray.rs 菜单结构和事件处理**

```rust
pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    let show = MenuItem::with_id(app, "show", "打开主界面", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let separator1 = tauri::menu::PredefinedMenuItem::separator(app)
        .map_err(|e| e.to_string())?;
    let status = MenuItem::with_id(app, "status", "● Stopped", false, None::<&str>)
        .map_err(|e| e.to_string())?;
    let separator2 = tauri::menu::PredefinedMenuItem::separator(app)
        .map_err(|e| e.to_string())?;
    let start = MenuItem::with_id(app, "start", "启动", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let stop = MenuItem::with_id(app, "stop", "停止", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let separator3 = tauri::menu::PredefinedMenuItem::separator(app)
        .map_err(|e| e.to_string())?;
    let open_log = MenuItem::with_id(app, "open_log", "打开日志目录", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let separator4 = tauri::menu::PredefinedMenuItem::separator(app)
        .map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(
        app,
        &[&show, &separator1, &status, &separator2, &start, &stop, &separator3, &open_log, &separator4, &quit],
    )
    .map_err(|e| e.to_string())?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "start" => {
                    crate::app_running::set_running(true);
                    let _ = app.emit("running-state-changed", "Running");
                }
                "stop" => {
                    crate::app_running::set_running(false);
                    let _ = app.emit("running-state-changed", "Stopped");
                }
                "open_log" => {
                    let log_dir = dirs::data_local_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("cozmio");
                    let _ = std::process::Command::new("explorer")
                        .arg(&log_dir)
                        .spawn();
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    // 监听运行状态变化，动态更新菜单状态项
    let app_handle = app.clone();
    app.listen("running-state-changed", move |event| {
        let state_str = event.payload().unwrap_or("Stopped");
        let status_text = format!("● {}", state_str);
        if let Some(tray) = app_handle.tray_by_id("main") {
            if let Some(menu) = tray.menu() {
                if let Some(status_item) = menu.get("status") {
                    let _ = status_item.set_title(&status_text);
                }
            }
        }
    });

    Ok(())
}
```

- [ ] **步骤2：验证编译**

执行命令：`cd cozmio && cargo build --package cozmio 2>&1 | tail -15`

预期结果：编译通过

- [ ] **步骤3：提交代码**

```bash
cd cozmio && git add src-tauri/src/tray.rs
git commit -m "feat: update tray menu with dynamic state display"
```

---

## 任务8：修改 main_loop 使用 is_running()

**涉及文件**：
- 修改：`cozmio/src-tauri/src/main_loop.rs`

- [ ] **步骤1：修改 main_loop.rs 开头检查**

将 `main_loop.rs:54-67` 替换为：
```rust
loop {
    // Step 0: Check running state - if stopped, skip analysis
    if !crate::app_running::is_running() {
        std::thread::sleep(poll_interval);
        continue;
    }

    // Step 1: Set processing state
    {
        let state = app_handle.state::<AppState>();
        let mut guard = state.tray_state.write().unwrap();
        *guard = TrayState::Processing;
    }

    // ... rest of the loop
}
```

- [ ] **步骤2：验证编译**

执行命令：`cd cozmio && cargo build --package cozmio 2>&1 | tail -15`

预期结果：编译通过

- [ ] **步骤3：提交代码**

```bash
cd cozmio && git add src-tauri/src/main_loop.rs
git commit -m "feat: main_loop uses is_running() for running state"
```

---

## 任务9：添加窗口控制命令

**涉及文件**：
- 修改：`cozmio/src-tauri/src/commands.rs`

- [ ] **步骤1：添加窗口控制命令**

```rust
#[tauri::command]
fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn hide_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

在 `generate_handler!` 中添加 `show_main_window` 和 `hide_main_window`。

- [ ] **步骤2：验证编译**

执行命令：`cd cozmio && cargo build --package cozmio 2>&1 | tail -15`

预期结果：编译通过

- [ ] **步骤3：提交代码**

```bash
cd cozmio && git add src-tauri/src/commands.rs
git commit -m "feat: add show/hide_main_window commands"
```

---

## 任务10：应用日志文件写入

**涉及文件**：
- 修改：`cozmio/src-tauri/src/main.rs`

- [ ] **步骤1：在 main.rs 添加文件日志写入器**

在 `main.rs` 顶部添加：
```rust
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

fn get_log_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cozmio")
        .join("cozmio.log")
}

fn setup_file_logging() {
    // 创建日志目录
    if let Some(log_dir) = get_log_path().parent() {
        let _ = std::fs::create_dir_all(log_dir);
    }

    // 设置 env_logger，但自定义写入目标到文件
    // 使用 Mutex 包装文件句柄，确保线程安全
    static LOG_FILE: OnceCell<Mutex<File>> = OnceCell::new();

    // 打开文件用于写入（截断模式，每次启动清空）
    // 注意：这里用截断模式，每天一个日志文件的方案留待后续
    if let Ok(file) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&get_log_path())
    {
        LOG_FILE.set(Mutex::new(file)).ok();
    }

    // 初始化 env_logger，将所有日志写入文件
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format(move |buf, record| {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let msg = format!(
                "{} [{}] {} - {}:{} | {}\n",
                timestamp,
                record.level(),
                record.target(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            );

            // 同时写入文件（如果文件可用）
            if let Some(file) = LOG_FILE.get() {
                if let Ok(mut f) = file.lock() {
                    let _ = f.write_all(msg.as_bytes());
                    let _ = f.flush();
                }
            }

            // 同时输出到 stderr（开发时可见）
            writeln!(buf, "{}", msg.trim())
        })
        .init();

    log::info!("File logging initialized at {:?}", get_log_path());
}
```

需要添加 `OnceCell`（Rust 标准库）：`use std::sync::OnceCell;`

注意：此实现使用截断模式（每次启动清空文件），确保日志从干净状态开始。后续可改为追加模式。

- [ ] **步骤2：在 main() 函数开头调用**

在 `src-tauri/src/main.rs:14` 的 `fn main()` 开头添加：
```rust
fn main() {
    setup_file_logging(); // 初始化文件日志
    cozmio::run()
}
```

- [ ] **步骤3：添加 chrono 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 下添加：
```toml
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **步骤4：验证编译**

执行命令：`cd cozmio && cargo build --package cozmio 2>&1 | tail -15`

预期结果：编译通过

- [ ] **步骤5：提交代码**

```bash
cd cozmio && git add src-tauri/src/main.rs src-tauri/Cargo.toml
git commit -m "feat: add application log file writing with sink to cozmio.log"
```

---

## 任务11：端到端验证

**涉及文件**：
- 修改：`cozmio/src-tauri/src/main_loop.rs`

确保 `main_loop.rs` 在每次循环迭代时写入行为记录（调用 `logger.log_action()`）。

- [ ] **步骤1：构建并运行**

执行命令：
```bash
cd cozmio && cargo build --package cozmio 2>&1 | tail -5
./target/debug/cozmio.exe &
sleep 2
```

- [ ] **步骤2：验证主窗口隐藏**

应用启动后主窗口不显示，托盘图标存在。

- [ ] **步骤3：验证托盘菜单**

右键托盘：
- "打开主界面" → 显示主窗口
- "● Stopped" → 状态显示正确
- "启动" → 点击后变为 "● Running"
- "停止" → 点击后变为 "● Stopped"
- "打开日志目录" → explorer 打开目录
- "退出" → 应用关闭

- [ ] **步骤4：验证 X 按钮隐藏**

打开主界面 → 点击 X 按钮 → 窗口隐藏，应用仍在托盘运行

- [ ] **步骤5：验证单实例**

再次运行 `cozmio.exe` → 旧实例主窗口显示并聚焦。

- [ ] **步骤6：验证日志文件存在**

检查 `%LOCALAPPDATA%\cozmio\cozmio.log` 存在。

- [ ] **步骤7：验证停止语义（Stopped 时不推进）**

执行命令：
```bash
# 启动应用
./target/debug/cozmio.exe &
sleep 2
# 记录当前行为记录数量
before_count=$(wc -l "%LOCALAPPDATA%\cozmio\action_log.jsonl" 2>/dev/null || echo "0")
# 通过托盘点击"启动"，等待一个轮询周期（默认5秒）
# ... 点击"停止"
sleep 6
# 再次检查行为记录数量
after_count=$(wc -l "%LOCALAPPDATA%\cozmio\action_log.jsonl" 2>/dev/null || echo "0")
# 两次数量应该相同（停止后无新增）
```

预期结果：停止后行为记录不再增长。

- [ ] **步骤8：验证启动语义（Running 时恢复推进）**

执行命令：
```bash
# 继续上一步环境，应用处于 Stopped 状态
# 点击"启动"
sleep 6
# 检查行为记录是否有新增
after_start_count=$(wc -l "%LOCALAPPDATA%\cozmio\action_log.jsonl" 2>/dev/null || echo "0")
```

预期结果：启动后行为记录恢复增长。

- [ ] **步骤9：验证单实例无重复驻留**

执行命令：
```bash
# 确认有一个 cozmio 进程在运行
tasklist | grep -i cozmio
# 再次运行 cozmio.exe
./target/debug/cozmio.exe &
sleep 2
# 检查进程数量
tasklist | grep -i cozmio | wc -l
```

预期结果：进程数量仍为 1（旧实例被聚焦，没有产生新实例）。

- [ ] **步骤10：验证退出后无残留**

执行命令：
```bash
# 通过托盘点击"退出"
taskkill //F //IM cozmio.exe 2>/dev/null
sleep 1
# 确认无 cozmio 驻留进程
tasklist | grep -i cozmio
# 检查 cozmio.log 最后一行是否为退出相关标记（可选）
```

预期结果：托盘消失，无进程残留。

---

## 验收标准

| 检查项 | 标准 |
|--------|------|
| 编译通过 | `cargo build --package cozmio` 无错误 |
| 主窗口默认隐藏 | 启动后主窗口不可见 |
| 托盘常驻 | 托盘图标存在，菜单可点击 |
| 状态动态更新 | 托盘状态项随运行状态变化而更新 |
| 打开主界面 | 托盘菜单点击后主窗口显示 |
| 启动/停止控制 | 托盘可启动/停止运行 |
| X 按钮隐藏 | 点击 X 按钮窗口隐藏，应用继续运行 |
| 退出正常 | 托盘"退出"完全关闭应用，无残留 |
| 单实例 | 重复启动不产生第二个驻留实例，只聚焦已有实例 |
| 日志目录 | 托盘菜单可打开日志目录 |
| 应用日志 | `%LOCALAPPDATA%\cozmio\cozmio.log` 存在并记录 |
| **停止语义成立** | `Stopped` 时主循环不推进，不产生新的运行记录 |
| **启动语义成立** | 从 `Stopped` 切回 `Running` 后，运行记录恢复增长 |
| **单实例无重复驻留** | 重复启动时无第二个 cozmio 进程或托盘图标 |
| **退出后无残留** | 退出后托盘消失，进程结束，不再产生新的运行记录 |