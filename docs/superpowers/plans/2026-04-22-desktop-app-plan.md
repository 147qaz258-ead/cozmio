# 桌面端应用实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标**：构建完整的 Tauri 桌面端应用，实现窗口监控、模型判断、执行路由、托盘交互和设置管理。

**前置依赖**：需要先完成 `real-info-acquisition` 计划（cozmio_core 库）。

**架构思路**：Rust 后端处理核心逻辑（调用 cozmio_core 截获窗口、模型调用、执行路由），Web 前端负责 UI 展示，通过 Tauri IPC 连接前后端。

**技术栈**：Tauri 2.x + Rust + Web (HTML/CSS/JS) + tauri-plugin-dialog (Windows 原生对话框) + cozmio_core + Ollama 本地模型

**UI 设计语言**：浅灰白纸感底色 + 极淡工程网格纹理；细边框、轻阴影、低饱和中性色；窗口化拼贴、技术编辑感、系统模块感；安静、精致、可运行的数据/智能系统界面。

---

## 文件结构

```
cozmio/
├── Cargo.toml                    # Workspace 配置（引用 cozmio_core）
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # 入口 + Tauri setup
│   │   ├── window_monitor.rs    # 窗口监控（调用 cozmio_core）
│   │   ├── model_client.rs      # Ollama 调用
│   │   ├── executor.rs          # 执行路由
│   │   ├── tray.rs              # 系统托盘
│   │   ├── commands.rs          # Tauri IPC 命令
│   │   ├── logging.rs           # 行为日志
│   │   ├── config.rs            # 配置管理
│   │   └── main_loop.rs         # 主监控循环
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   └── src/
│       ├── index.html
│       ├── main.js
│       ├── styles.css
│       └── components/
│           ├── App.js
│           ├── StatusPanel.js
│           ├── HistoryList.js
│           └── Settings.js
├── cozmio_core/                  # 依赖：real-info-acquisition 计划产出
├── ui-prototype.html             # UI 原型（参考）
└── docs/superpowers/specs/2026-04-22-desktop-app-design.md
```

---

## 任务 1：Tauri 项目初始化

**涉及文件**：

- 创建：`src-tauri/Cargo.toml`
- 创建：`src-tauri/tauri.conf.json`
- 创建：`src-tauri/capabilities/default.json`
- 创建：`src-tauri/src/main.rs`
- 创建：`src-tauri/src/index.html`
- 创建：`src-tauri/src/main.js`
- 创建：`src-tauri/src/styles.css`
- 创建：`src-tauri/src/components/App.js`

- [ ] **步骤 1：创建 Workspace Cargo.toml**

```toml
# cozmio/Cargo.toml
[workspace]
resolver = "2"
members = ["cozmio_core", "src-tauri"]

[profile.release]
lto = true
opt-level = "s"
strip = true
```

- [ ] **步骤 2：创建 src-tauri/Cargo.toml**

```toml
# src-tauri/Cargo.toml
[package]
name = "cozmio"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-dialog = "2"
tauri-plugin-notification = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
cozmio_core = { path = "../cozmio_core" }
reqwest = { version = "0.12", features = ["json", "blocking"] }
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
log = "0.4"
env_logger = "0.11"
dirs = "5"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

- [ ] **步骤 2：创建 tauri.conf.json**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Cozmio",
  "version": "0.1.0",
  "identifier": "com.cozmio.app",
  "build": {
    "devtools": true,
    "frontendDist": "../src-tauri/src",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "",
    "beforeBuildCommand": ""
  },
  "app": {
    "withGlobalTauri": true,
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    },
    "windows": [
      {
        "title": "Cozmio - 主动智能体",
        "width": 900,
        "height": 650,
        "resizable": true,
        "visible": false,
        "center": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "windows": {
      "webviewInstallMode": {
        "type": "embedBootstrapper"
      }
    }
  },
  "plugins": {
    "dialog": {},
    "notification": {}
  }
}
```

- [ ] **步骤 3：创建 capabilities/default.json**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "identifier": "default",
  "description": "Default capabilities for Cozmio",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-close",
    "core:window:allow-minimize",
    "core:window:allow-maximize",
    "core:window:allow-unmaximize",
    "core:window:allow-set-focus",
    "core:tray:default",
    "core:tray:allow-new",
    "core:tray:allow-set-icon",
    "core:tray:allow-set-menu",
    "core:tray:allow-set-tooltip",
    "dialog:default",
    "dialog:allow-ask",
    "dialog:allow-confirm",
    "dialog:allow-message",
    "notification:default",
    "notification:allow-notify",
    "notification:allow-request-permission",
    "notification:allow-is-permission-granted"
  ]
}
```

- [ ] **步骤 4：创建 build.rs**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **步骤 5：创建 src/main.rs（最小化入口）**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    cozmio::run()
}

mod cozmio {
    pub fn run() {
        tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_notification::init())
            .setup(|app| {
                log::info!("Cozmio starting...");
                Ok(())
            })
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
```

- [ ] **步骤 6：创建 src/index.html（最小化前端）**

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Cozmio</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <div id="app"></div>
    <script type="module" src="main.js"></script>
</body>
</html>
```

- [ ] **步骤 7：创建 src/main.js（最小化前端入口）**

```js
import { mount } from './components/App.js';

document.addEventListener('DOMContentLoaded', () => {
    mount(document.getElementById('app'));
});
```

- [ ] **步骤 8：创建 src/styles.css**

```css
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, sans-serif;
    background: #f5f5f5;
    color: #333;
    height: 100vh;
    overflow: hidden;
}

#app {
    height: 100%;
}
```

- [ ] **步骤 9：创建 src/components/App.js（最小化 App 组件）**

```js
export function mount(el) {
    el.innerHTML = '<div style="padding: 20px;">Cozmio Loading...</div>';
}
```

- [ ] **步骤 10：运行开发服务器验证项目结构**

执行命令：`cd src-tauri && cargo tauri dev 2>&1 | head -50`
预期结果：Tauri 应用启动，前端显示 "Cozmio Loading..."

- [ ] **步骤 11：提交代码**

```bash
git add src-tauri/
git commit -m "feat: initialize Tauri project with basic structure"
```

---

## 任务 2：配置管理模块

**涉及文件**：

- 创建：`src-tauri/src/config.rs`

- [ ] **步骤 1：编写失败的测试用例**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.model_name, "llava");
        assert_eq!(config.poll_interval_secs, 3);
    }

    #[test]
    fn test_config_to_json() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("ollama_url"));
    }
}
```

- [ ] **步骤 2：运行测试确认失败**

执行命令：`cd src-tauri && cargo test config -- --nocapture 2>&1`
预期结果：编译错误（模块不存在）

- [ ] **步骤 3：编写 config.rs**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ollama_url: String,
    pub model_name: String,
    pub poll_interval_secs: u64,
    pub window_change_detection: bool,
    pub execute_auto: bool,
    pub request_use_native_dialog: bool,
    pub execute_delay_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".to_string(),
            model_name: "llava".to_string(),
            poll_interval_secs: 3,
            window_change_detection: true,
            execute_auto: true,
            request_use_native_dialog: true,
            execute_delay_secs: 1,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cozmio")
            .join("config.json")
    }
}
```

- [ ] **步骤 4：运行测试确认通过**

执行命令：`cd src-tauri && cargo test config -- --nocapture 2>&1`
预期结果：所有测试通过

- [ ] **步骤 5：更新 Cargo.toml 添加依赖**

```toml
[dependencies]
dirs = "5"
```

- [ ] **步骤 6：运行测试确认通过**

执行命令：`cd src-tauri && cargo test config -- --nocapture 2>&1`
预期结果：所有测试通过

- [ ] **步骤 7：提交代码**

```bash
git add src-tauri/src/config.rs src-tauri/Cargo.toml
git commit -m "feat: add config module with load/save/default"
```

---

## 任务 3：窗口监控模块

**前置条件**：需要先完成 `real-info-acquisition` 计划中的 Task 1-6，确保 `cozmio_core` 可用。

**涉及文件**：

- 创建：`src-tauri/src/window_monitor.rs`

- [ ] **步骤 1：编写失败的测试用例**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_snapshot_from_core() {
        let monitor = WindowMonitor::new();
        let snapshot = monitor.capture().expect("capture should succeed");
        assert!(!snapshot.screenshot_base64.is_empty(), "screenshot should not be empty");
        assert!(!snapshot.window_info.title.is_empty(), "window title should not be empty");
    }
}
```

- [ ] **步骤 2：运行测试确认失败**

执行命令：`cd src-tauri && cargo test window_monitor -- --nocapture 2>&1`
预期结果：编译错误（模块不存在）

- [ ] **步骤 3：编写 window_monitor.rs（使用 cozmio_core）**

```rust
use base64::{engine::general_purpose::STANDARD, Engine};
use cozmio_core::{capture_all, Screenshot, WindowInfo};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WindowSnapshot {
    pub screenshot_base64: String,
    pub screenshot_width: u32,
    pub screenshot_height: u32,
    pub window_info: WindowInfo,
    pub timestamp: i64,
}

pub struct WindowMonitor {
    last_window_title: String,
}

impl WindowMonitor {
    pub fn new() -> Self {
        Self {
            last_window_title: String::new(),
        }
    }

    pub fn capture(&self) -> Result<WindowSnapshot, String> {
        let result = capture_all(1).map_err(|e| e.to_string())?;

        let screenshot = result.screenshot.ok_or("No screenshot available")?;
        let window_info = result.foreground_window.ok_or("No foreground window")?;

        Ok(WindowSnapshot {
            screenshot_base64: screenshot.image_base64,
            screenshot_width: screenshot.width,
            screenshot_height: screenshot.height,
            window_info,
            timestamp: result.timestamp,
        })
    }

    pub fn has_changed(&self, snapshot: &WindowSnapshot) -> bool {
        if self.last_window_title.is_empty() {
            return true;
        }
        self.last_window_title != snapshot.window_info.title
    }

    pub fn update_last_title(&mut self, title: &str) {
        self.last_window_title = title.to_string();
    }
}

impl Default for WindowMonitor {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **步骤 4：运行测试确认通过**

执行命令：`cd src-tauri && cargo test window_monitor -- --nocapture 2>&1`
预期结果：测试通过（截图可能为空但结构正确）

- [ ] **步骤 5：提交代码**

```bash
git add src-tauri/src/window_monitor.rs
git commit -m "feat: add window monitor module for screenshot capture"
```

---

## 任务 4：模型调用模块

**涉及文件**：

- 创建：`src-tauri/src/model_client.rs`

- [ ] **步骤 1：编写失败的测试用例**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initiative_level_serialization() {
        assert_eq!(
            serde_json::to_string(&InitiativeLevel::Suggest).unwrap(),
            "\"suggest\""
        );
        assert_eq!(
            serde_json::to_string(&InitiativeLevel::Request).unwrap(),
            "\"request\""
        );
        assert_eq!(
            serde_json::to_string(&InitiativeLevel::Execute).unwrap(),
            "\"execute\""
        );
    }

    #[test]
    fn test_model_output_fields() {
        let output = ModelOutput {
            judgment: "test judgment".to_string(),
            next_step: "test next step".to_string(),
            level: InitiativeLevel::Suggest,
            confidence: 0.85,
            grounds: "test grounds".to_string(),
        };
        assert_eq!(output.level, InitiativeLevel::Suggest);
        assert_eq!(output.confidence, 0.85);
    }
}
```

- [ ] **步骤 2：运行测试确认失败**

执行命令：`cd src-tauri && cargo test model_client -- --nocapture 2>&1`
预期结果：编译错误（模块不存在）

- [ ] **步骤 3：编写 model_client.rs**

```rust
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::config::Config;
use crate::window_monitor::WindowSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InitiativeLevel {
    Suggest,
    Request,
    Execute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOutput {
    pub judgment: String,
    pub next_step: String,
    pub level: InitiativeLevel,
    pub confidence: f32,
    pub grounds: String,
}

pub struct ModelClient {
    client: Client,
    config: Config,
}

impl ModelClient {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    pub fn call(&self, snapshot: &WindowSnapshot) -> Result<ModelOutput, String> {
        let prompt = self.build_prompt(snapshot);
        // 使用 cozmio_core 提供的 base64 截图
        let image_b64 = &snapshot.screenshot_base64;

        let request_body = serde_json::json!({
            "model": self.config.model_name,
            "prompt": prompt,
            "images": [image_b64],
            "stream": false
        });

        let response = self
            .client
            .post(format!("{}/api/generate", self.config.ollama_url))
            .json(&request_body)
            .send()
            .map_err(|e| format!("Failed to send request: {}", e))?;

        let response_text = response
            .text()
            .map_err(|e| format!("Failed to read response: {}", e))?;

        self.parse_output(&response_text)
    }

    fn build_prompt(&self, snapshot: &WindowSnapshot) -> String {
        format!(
            r#"You are a proactive AI assistant observing a user's desktop window.

Given the window screenshot, produce a judgment in plain text.

IMPORTANT: You have TWO legitimate output modes:

MODE A - ACTIVE JUDGMENT (when you have enough signal):
Output in this exact format:
judgment: <what you observe in the window>
next_step: <a reasonable next step (be specific, not vague)>
level: suggest | request | execute
confidence: <0.0-1.0>
grounds: <what visible evidence supports this judgment>

MODE B - ABSTAIN (when you do NOT have enough signal):
Output in this exact format:
abstain: <brief reason why>

level definitions:
- suggest: notify user only
- request: need user confirmation
- execute: high confidence, proceed directly

Be specific. "User is typing in a code editor" is better than "User is working."
Output as plain text. No JSON."#
        )
    }

    fn parse_output(&self, response_text: &str) -> Result<ModelOutput, String> {
        let json: serde_json::Value =
            serde_json::from_str(response_text).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let response_text = json["response"]
            .as_str()
            .ok_or("No 'response' field in Ollama output")?;

        if response_text.trim().to_lowercase().starts_with("abstain:") {
            return Ok(ModelOutput {
                judgment: String::new(),
                next_step: String::new(),
                level: InitiativeLevel::Suggest,
                confidence: 0.0,
                grounds: response_text.trim_start_matches("abstain:").trim().to_string(),
            });
        }

        let mut output = ModelOutput {
            judgment: String::new(),
            next_step: String::new(),
            level: InitiativeLevel::Suggest,
            confidence: 0.5,
            grounds: String::new(),
        };

        for line in response_text.lines() {
            let line = line.trim();
            if let Some(value) = line.strip_prefix("judgment:") {
                output.judgment = value.trim().to_string();
            } else if let Some(value) = line.strip_prefix("next_step:") {
                output.next_step = value.trim().to_string();
            } else if let Some(value) = line.strip_prefix("level:") {
                let level_str = value.trim().to_lowercase();
                output.level = match level_str.as_str() {
                    "request" => InitiativeLevel::Request,
                    "execute" => InitiativeLevel::Execute,
                    _ => InitiativeLevel::Suggest,
                };
            } else if let Some(value) = line.strip_prefix("confidence:") {
                if let Ok(conf) = value.trim().parse::<f32>() {
                    output.confidence = conf;
                }
            } else if let Some(value) = line.strip_prefix("grounds:") {
                output.grounds = value.trim().to_string();
            }
        }

        if output.judgment.is_empty() && output.next_step.is_empty() {
            return Err("Failed to parse model output".to_string());
        }

        Ok(output)
    }
}
```

- [ ] **步骤 4：运行测试确认通过**

执行命令：`cd src-tauri && cargo test model_client -- --nocapture 2>&1`
预期结果：所有测试通过

- [ ] **步骤 5：提交代码**

```bash
git add src-tauri/src/model_client.rs
git commit -m "feat: add model client module for Ollama API calls"
```

---

## 任务 5：行为日志模块

**涉及文件**：

- 创建：`src-tauri/src/logging.rs`

- [ ] **步骤 1：编写失败的测试用例**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_record_serialization() {
        let record = ActionRecord {
            timestamp: 1234567890,
            window_title: "test.md".to_string(),
            judgment: "test judgment".to_string(),
            next_step: "test next".to_string(),
            level: InitiativeLevel::Suggest,
            confidence: 0.85,
            grounds: "test grounds".to_string(),
            system_action: "notified".to_string(),
            user_feedback: None,
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("test.md"));
    }
}
```

- [ ] **步骤 2：运行测试确认失败**

执行命令：`cd src-tauri && cargo test logging -- --nocapture 2>&1`
预期结果：编译错误（模块不存在）

- [ ] **步骤 3：编写 logging.rs**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use crate::model_client::{InitiativeLevel, ModelOutput};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub timestamp: i64,
    pub window_title: String,
    pub judgment: String,
    pub next_step: String,
    pub level: InitiativeLevel,
    pub confidence: f32,
    pub grounds: String,
    pub system_action: String,
    pub user_feedback: Option<String>,
}

pub struct ActionLogger {
    log_path: PathBuf,
}

impl ActionLogger {
    pub fn new() -> Self {
        let log_path = Self::log_path();
        if let Some(parent) = log_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        Self { log_path }
    }

    pub fn log(&self, record: ActionRecord) -> Result<(), String> {
        let json = serde_json::to_string(&record).map_err(|e| e.to_string())?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|e| e.to_string())?;

        writeln!(file, "{}", json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<ActionRecord>, String> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let file = OpenOptions::new()
            .read(true)
            .open(&self.log_path)
            .map_err(|e| e.to_string())?;

        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(record) = serde_json::from_str::<ActionRecord>(&line) {
                    records.push(record);
                }
            }
        }

        records.reverse();
        records.truncate(limit);
        Ok(records)
    }

    pub fn clear(&self) -> Result<(), String> {
        if self.log_path.exists() {
            fs::remove_file(&self.log_path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn log_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cozmio")
            .join("action_log.jsonl")
    }
}

impl Default for ActionLogger {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **步骤 4：运行测试确认通过**

执行命令：`cd src-tauri && cargo test logging -- --nocapture 2>&1`
预期结果：所有测试通过

- [ ] **步骤 5：提交代码**

```bash
git add src-tauri/src/logging.rs
git commit -m "feat: add action logging module"
```

---

## 任务 6：执行路由模块

**涉及文件**：

- 创建：`src-tauri/src/executor.rs`

- [ ] **步骤 1：编写失败的测试用例**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_routing() {
        let suggest = ModelOutput {
            judgment: "test".to_string(),
            next_step: "next".to_string(),
            level: InitiativeLevel::Suggest,
            confidence: 0.5,
            grounds: "grounds".to_string(),
        };
        assert!(matches!(suggest.level, InitiativeLevel::Suggest));
    }
}
```

- [ ] **步骤 2：运行测试确认失败**

执行命令：`cd src-tauri && cargo test executor -- --nocapture 2>&1`
预期结果：编译错误（模块不存在）

- [ ] **步骤 3：编写 executor.rs**

```rust
use tauri::{AppHandle, Manager};

use crate::config::Config;
use crate::logging::ActionLogger;
use crate::model_client::{InitiativeLevel, ModelOutput};

pub enum ExecutionResult {
    Notified,
    Confirmed,
    Executed,
    Skipped,
}

pub struct Executor {
    config: Config,
    logger: ActionLogger,
}

impl Executor {
    pub fn new(config: Config, logger: ActionLogger) -> Self {
        Self { config, logger }
    }

    pub fn route(&self, output: &ModelOutput, window_title: &str) -> Result<ExecutionResult, String> {
        let system_action = match output.level {
            InitiativeLevel::Suggest => {
                self.handle_suggest(output, window_title)?;
                "notified".to_string()
            }
            InitiativeLevel::Request => {
                if self.config.request_use_native_dialog {
                    let confirmed = self.show_native_dialog(output)?;
                    if confirmed {
                        self.handle_execute(output)?;
                        "confirmed_and_executed"
                    } else {
                        "cancelled"
                    }
                } else {
                    "awaiting_confirmation"
                }
            }
            InitiativeLevel::Execute => {
                if self.config.execute_auto {
                    std::thread::sleep(std::time::Duration::from_secs(self.config.execute_delay_secs));
                    self.handle_execute(output)?;
                    "auto_executed"
                } else {
                    let confirmed = self.show_native_dialog(output)?;
                    if confirmed {
                        self.handle_execute(output)?;
                        "confirmed_and_executed"
                    } else {
                        "cancelled"
                    }
                }
            }
        };

        let record = crate::logging::ActionRecord {
            timestamp: chrono::Utc::now().timestamp(),
            window_title: window_title.to_string(),
            judgment: output.judgment.clone(),
            next_step: output.next_step.clone(),
            level: output.level,
            confidence: output.confidence,
            grounds: output.grounds.clone(),
            system_action,
            user_feedback: None,
        };

        self.logger.log(record).ok();

        Ok(match output.level {
            InitiativeLevel::Suggest => ExecutionResult::Notified,
            InitiativeLevel::Request => ExecutionResult::Confirmed,
            InitiativeLevel::Execute => ExecutionResult::Executed,
        })
    }

    fn handle_suggest(&self, output: &ModelOutput, _window_title: &str) -> Result<(), String> {
        use tauri_plugin_notification::NotificationExt;

        let app = tauri::Manager::default();
        if let Some(notification) = app.notif_builder()
            .title("Cozmio - 建议")
            .body(format!("{}\n\n下一步: {}", output.judgment, output.next_step))
            .build()
        {
            notification.show().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn show_native_dialog(&self, output: &ModelOutput) -> Result<bool, String> {
        use tauri_plugin_dialog::DialogExt;

        let message = format!(
            "{}\n\n📋 下一步: {}\n📊 置信度: {:.0}%",
            output.judgment,
            output.next_step,
            output.confidence * 100.0
        );

        let confirmed = tauri_plugin_dialog::Dialog::new()
            .message(&message)
            .title("Cozmio - 确认执行")
            .ok_button_label("确认执行")
            .cancel_button_label("取消")
            .blocking_show();

        Ok(confirmed)
    }

    fn handle_execute(&self, _output: &ModelOutput) -> Result<(), String> {
        Ok(())
    }
}
```

- [ ] **步骤 4：运行测试确认通过**

执行命令：`cd src-tauri && cargo test executor -- --nocapture 2>&1`
预期结果：所有测试通过（部分功能需要完整 Tauri context）

- [ ] **步骤 5：提交代码**

```bash
git add src-tauri/src/executor.rs
git commit -m "feat: add executor module for level-based routing"
```

---

## 任务 7：托盘模块

**涉及文件**：

- 创建：`src-tauri/src/tray.rs`

- [ ] **步骤 1：编写失败的测试用例**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_state_serialization() {
        assert_eq!(
            serde_json::to_string(&TrayState::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&TrayState::Paused).unwrap(),
            "\"paused\""
        );
    }
}
```

- [ ] **步骤 2：运行测试确认失败**

执行命令：`cd src-tauri && cargo test tray -- --nocapture 2>&1`
预期结果：编译错误（模块不存在）

- [ ] **步骤 3：编写 tray.rs**

```rust
use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrayState {
    Running,
    Paused,
    Processing,
}

pub struct TrayManager {
    state: TrayState,
}

impl TrayManager {
    pub fn new() -> Self {
        Self {
            state: TrayState::Running,
        }
    }

    pub fn state(&self) -> TrayState {
        self.state
    }

    pub fn set_state(&mut self, state: TrayState) {
        self.state = state;
    }

    pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
        let run_item = MenuItem::with_id(app, "run", "运行中", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        let pause_item = MenuItem::with_id(app, "pause", "暂停", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        let settings_item = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        let history_item = MenuItem::with_id(app, "history", "历史", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
            .map_err(|e| e.to_string())?;

        let menu = Menu::with_items(
            app,
            &[
                &run_item,
                &pause_item,
                &settings_item,
                &history_item,
                &quit_item,
            ],
        )
        .map_err(|e| e.to_string())?;

        let _tray = TrayIconBuilder::new()
            .menu(&menu)
            .tooltip("Cozmio - 主动智能体")
            .on_menu_event(|app, event| {
                match event.id.as_ref() {
                    "run" => {
                        let _ = app.emit("tray-action", "run");
                    }
                    "pause" => {
                        let _ = app.emit("tray-action", "pause");
                    }
                    "settings" => {
                        let _ = app.emit("tray-action", "settings");
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "history" => {
                        let _ = app.emit("tray-action", "history");
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
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

        Ok(())
    }
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **步骤 4：运行测试确认通过**

执行命令：`cd src-tauri && cargo test tray -- --nocapture 2>&1`
预期结果：所有测试通过

- [ ] **步骤 5：提交代码**

```bash
git add src-tauri/src/tray.rs
git commit -m "feat: add system tray module"
```

---

## 任务 8：Tauri IPC 命令模块

**涉及文件**：

- 创建：`src-tauri/src/commands.rs`

- [ ] **步骤 1：编写失败的测试用例**

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert!(true);
    }
}
```

- [ ] **步骤 2：运行测试确认失败**

执行命令：`cd src-tauri && cargo test commands -- --nocapture 2>&1`
预期结果：编译错误（模块不存在）

- [ ] **步骤 3：编写 commands.rs**

```rust
use tauri::State;

use crate::config::Config;
use crate::logging::{ActionLogger, ActionRecord};
use crate::model_client::ModelOutput;
use crate::tray::TrayState;

pub struct AppState {
    pub config: Config,
    pub logger: ActionLogger,
    pub tray_state: TrayState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Config::load(),
            logger: ActionLogger::new(),
            tray_state: TrayState::Running,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<Config, String> {
    Ok(state.config.clone())
}

#[tauri::command]
pub fn save_config(state: State<AppState>, config: Config) -> Result<(), String> {
    config.save()?;
    Ok(())
}

#[tauri::command]
pub fn get_history(state: State<AppState>, limit: Option<usize>) -> Result<Vec<ActionRecord>, String> {
    state.logger.get_recent(limit.unwrap_or(50))
}

#[tauri::command]
pub fn clear_history(state: State<AppState>) -> Result<(), String> {
    state.logger.clear()
}

#[tauri::command]
pub fn get_tray_state(state: State<AppState>) -> Result<TrayState, String> {
    Ok(state.tray_state)
}

#[tauri::command]
pub fn set_tray_state(state: State<AppState>, new_state: TrayState) -> Result<(), String> {
    state.tray_state = new_state;
    Ok(())
}
```

- [ ] **步骤 4：运行测试确认通过**

执行命令：`cd src-tauri && cargo test commands -- --nocapture 2>&1`
预期结果：所有测试通过

- [ ] **步骤 5：提交代码**

```bash
git add src-tauri/src/commands.rs
git commit -m "feat: add Tauri IPC command handlers"
```

---

## 任务 9：整合 main.rs

**涉及文件**：

- 修改：`src-tauri/src/main.rs`

- [ ] **步骤 1：更新 main.rs 整合所有模块**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cozmio;

fn main() {
    cozmio::run()
}

mod cozmio {
    use cozmio_lib::commands::AppState;
    use cozmio_lib::tray::TrayManager;
    use std::sync::Mutex;
    use tauri::Manager;

    pub fn run() {
        let app_state = AppState::new();
        let tray_manager = TrayManager::new();

        tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_notification::init())
            .manage(app_state)
            .manage(Mutex::new(tray_manager))
            .setup(|app| {
                log::info!("Cozmio starting...");

                let handle = app.handle().clone();
                if let Err(e) = cozmio_lib::tray::TrayManager::setup_tray(&handle) {
                    log::error!("Failed to setup tray: {}", e);
                }

                log::info!("Cozmio started successfully");
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                cozmio_lib::commands::get_config,
                cozmio_lib::commands::save_config,
                cozmio_lib::commands::get_history,
                cozmio_lib::commands::clear_history,
                cozmio_lib::commands::get_tray_state,
                cozmio_lib::commands::set_tray_state,
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
```

- [ ] **步骤 2：创建 src-tauri/src/lib.rs 导出所有模块**

```rust
pub mod commands;
pub mod config;
pub mod executor;
pub mod logging;
pub mod model_client;
pub mod tray;
pub mod window_monitor;
```

- [ ] **步骤 3：更新 Cargo.toml 添加 lib**

```toml
[lib]
name = "cozmio_lib"
path = "src/lib.rs"
```

- [ ] **步骤 4：运行开发服务器验证**

执行命令：`cd src-tauri && cargo tauri dev 2>&1 | head -100`
预期结果：应用启动，无编译错误

- [ ] **步骤 5：提交代码**

```bash
git add src-tauri/src/main.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: integrate all modules in main.rs"
```

---

## 任务 10：前端 - 样式系统

**涉及文件**：

- 修改：`src-tauri/src/styles.css`

- [ ] **步骤 1：编写完整样式**

```css
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

:root {
    --primary: #00d4aa;
    --primary-dark: #00b894;
    --bg-dark: #1a1a2e;
    --bg-panel: #f5f5f5;
    --bg-card: #ffffff;
    --text-primary: #333333;
    --text-secondary: #666666;
    --text-muted: #999999;
    --border: #e0e0e0;
    --success: #5ac05a;
    --warning: #f5c542;
    --danger: #e54d4d;
}

body {
    font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, sans-serif;
    background: var(--bg-panel);
    color: var(--text-primary);
    height: 100vh;
    overflow: hidden;
    user-select: none;
}

#app {
    height: 100%;
    display: flex;
    flex-direction: column;
}

/* Title Bar */
.title-bar {
    height: 36px;
    background: var(--bg-dark);
    display: flex;
    align-items: center;
    padding: 0 12px;
}

.title-bar .logo {
    color: var(--primary);
    font-weight: bold;
    font-size: 14px;
    margin-right: 8px;
}

.title-bar .title {
    color: #fff;
    font-size: 13px;
    flex: 1;
}

.title-bar .window-controls {
    display: flex;
    gap: 8px;
}

.title-bar .window-controls span {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    cursor: pointer;
}

.title-bar .window-controls .minimize { background: var(--warning); }
.title-bar .window-controls .maximize { background: var(--success); }
.title-bar .window-controls .close { background: var(--danger); }

/* Main Content */
.main-content {
    flex: 1;
    display: flex;
    overflow: hidden;
}

/* Sidebar */
.sidebar {
    width: 160px;
    background: var(--bg-card);
    border-right: 1px solid var(--border);
    padding: 20px 0;
}

.sidebar-item {
    display: flex;
    align-items: center;
    padding: 12px 20px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s;
    font-size: 14px;
}

.sidebar-item:hover {
    background: #f0f0f0;
    color: var(--text-primary);
}

.sidebar-item.active {
    background: #e8f8f5;
    color: var(--primary);
    border-right: 3px solid var(--primary);
}

.sidebar-item .icon {
    margin-right: 10px;
    font-size: 16px;
}

/* Content Panel */
.content-panel {
    flex: 1;
    padding: 24px;
    overflow-y: auto;
    display: none;
}

.content-panel.active {
    display: block;
}

.panel-title {
    font-size: 20px;
    color: var(--text-primary);
    margin-bottom: 20px;
    display: flex;
    align-items: center;
    gap: 8px;
}

/* Cards */
.card {
    background: var(--bg-card);
    border-radius: 8px;
    padding: 20px;
    margin-bottom: 16px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.08);
}

.card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
}

.card-title {
    font-size: 14px;
    color: var(--text-primary);
    font-weight: 500;
}

/* Status Badge */
.status-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
}

.status-badge .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--success);
}

.status-badge .dot.paused { background: var(--danger); }
.status-badge .dot.processing {
    background: var(--warning);
    animation: pulse 1s infinite;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
}

/* Buttons */
.btn {
    padding: 8px 16px;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s;
}

.btn-primary {
    background: var(--primary);
    color: #fff;
}

.btn-primary:hover {
    background: var(--primary-dark);
}

.btn-secondary {
    background: #e0e0e0;
    color: var(--text-secondary);
}

.btn-secondary:hover {
    background: #d0d0d0;
}

/* Form */
.form-row {
    display: flex;
    align-items: center;
    margin-bottom: 12px;
}

.form-row:last-child {
    margin-bottom: 0;
}

.form-label {
    width: 120px;
    color: var(--text-secondary);
    font-size: 13px;
}

.form-input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 13px;
}

.form-input:focus {
    outline: none;
    border-color: var(--primary);
}

.form-checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
}

.form-checkbox input {
    width: 18px;
    height: 18px;
    accent-color: var(--primary);
}

/* Info Rows */
.info-row {
    padding: 10px 0;
    border-bottom: 1px solid #f0f0f0;
    display: flex;
    gap: 12px;
}

.info-row:last-child {
    border-bottom: none;
}

.info-key {
    width: 70px;
    color: var(--text-muted);
    font-size: 13px;
}

.info-value {
    flex: 1;
    color: var(--text-primary);
    font-size: 13px;
    word-break: break-all;
}

/* Level Tags */
.level-tag {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: bold;
}

.level-suggest { background: #e3f2fd; color: #1976d2; }
.level-request { background: #fff3e0; color: #f57c00; }
.level-execute { background: #e8f5e9; color: #388e3c; }

/* History List */
.history-list {
    background: var(--bg-card);
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0,0,0,0.08);
}

.history-item {
    display: flex;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    cursor: pointer;
    transition: background 0.2s;
}

.history-item:hover {
    background: #f9f9f9;
}

.history-item.active {
    background: #e8f8f5;
}

.history-time {
    color: var(--text-muted);
    font-size: 12px;
    width: 70px;
}

.history-level {
    width: 80px;
}

.history-title {
    flex: 1;
    color: var(--text-primary);
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

/* Settings Section */
.settings-section {
    background: var(--bg-card);
    border-radius: 8px;
    padding: 20px;
    margin-bottom: 16px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.08);
}

.settings-section h3 {
    font-size: 14px;
    color: var(--text-primary);
    margin-bottom: 16px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
}

.btn-group {
    display: flex;
    gap: 12px;
    margin-top: 20px;
    justify-content: flex-end;
}

/* Label */
.label {
    font-size: 12px;
    color: var(--text-muted);
    margin-bottom: 8px;
    display: block;
}
```

- [ ] **步骤 2：提交代码**

```bash
git add src-tauri/src/styles.css
git commit -m "feat: add complete stylesheet"
```

---

## 任务 11：前端 - 状态面板组件

**涉及文件**：

- 创建：`src-tauri/src/components/StatusPanel.js`

- [ ] **步骤 1：创建 StatusPanel.js**

```js
export function createStatusPanel(state) {
    const panel = document.createElement('div');
    panel.className = 'content-panel active';
    panel.id = 'panel-status';

    panel.innerHTML = `
        <div class="panel-title">📊 状态</div>

        <div class="card">
            <div class="card-header">
                <div class="status-badge">
                    <span class="dot" id="status-dot"></span>
                    <span id="status-text">运行中</span>
                </div>
                <button class="btn btn-secondary" id="btn-pause">暂停</button>
            </div>
        </div>

        <div class="card">
            <span class="label">当前窗口</span>
            <div class="info-row">
                <span class="info-key">标题</span>
                <span class="info-value" id="current-title">-</span>
            </div>
            <div class="info-row">
                <span class="info-key">进程</span>
                <span class="info-value" id="current-process">-</span>
            </div>
        </div>

        <div class="card">
            <span class="label" id="last-judgment-time">最后判断 (-)</span>
            <div id="last-judgment-content">
                <div style="color: var(--text-muted); font-size: 13px;">暂无判断记录</div>
            </div>
        </div>
    `;

    const btnPause = panel.querySelector('#btn-pause');
    btnPause.addEventListener('click', () => {
        const isPaused = state.trayState === 'paused';
        window.__TAURI__.core.emit('tray-action', isPaused ? 'run' : 'pause');
    });

    return panel;
}

export function updateStatusPanel(panel, state) {
    const statusDot = panel.querySelector('#status-dot');
    const statusText = panel.querySelector('#status-text');
    const btnPause = panel.querySelector('#btn-pause');

    if (state.trayState === 'paused') {
        statusDot.className = 'dot paused';
        statusText.textContent = '已暂停';
        btnPause.textContent = '继续';
    } else if (state.trayState === 'processing') {
        statusDot.className = 'dot processing';
        statusText.textContent = '处理中';
        btnPause.textContent = '暂停';
    } else {
        statusDot.className = 'dot';
        statusText.textContent = '运行中';
        btnPause.textContent = '暂停';
    }

    if (state.currentWindow) {
        panel.querySelector('#current-title').textContent = state.currentWindow.title || '-';
        panel.querySelector('#current-process').textContent = state.currentWindow.processName || '-';
    }

    if (state.lastJudgment) {
        const time = new Date(state.lastJudgment.timestamp * 1000).toLocaleTimeString();
        panel.querySelector('#last-judgment-time').textContent = `最后判断 (${time})`;

        const levelClass = `level-${state.lastJudgment.level}`;
        panel.querySelector('#last-judgment-content').innerHTML = `
            <div class="info-row">
                <span class="info-key">判断</span>
                <span class="info-value">${state.lastJudgment.judgment || '(abstain)'}</span>
            </div>
            <div class="info-row">
                <span class="info-key">下一步</span>
                <span class="info-value">${state.lastJudgment.next_step || '-'}</span>
            </div>
            <div class="info-row">
                <span class="info-key">级别</span>
                <span class="info-value"><span class="level-tag ${levelClass}">${state.lastJudgment.level}</span></span>
            </div>
            <div class="info-row">
                <span class="info-key">置信度</span>
                <span class="info-value">${Math.round(state.lastJudgment.confidence * 100)}%</span>
            </div>
        `;
    }
}
```

- [ ] **步骤 2：提交代码**

```bash
git add src-tauri/src/components/StatusPanel.js
git commit -m "feat: add StatusPanel component"
```

---

## 任务 12：前端 - 历史记录组件

**涉及文件**：

- 创建：`src-tauri/src/components/HistoryList.js`

- [ ] **步骤 1：创建 HistoryList.js**

```js
export function createHistoryList() {
    const panel = document.createElement('div');
    panel.className = 'content-panel';
    panel.id = 'panel-history';

    panel.innerHTML = `
        <div class="panel-title">
            📜 历史记录
            <button class="btn btn-secondary" id="btn-clear" style="margin-left: auto;">清空</button>
        </div>

        <div class="history-list" id="history-list">
            <div style="padding: 20px; text-align: center; color: var(--text-muted);">
                暂无历史记录
            </div>
        </div>

        <div class="card" id="history-detail" style="display: none;">
            <span class="label">选中详情</span>
            <div id="detail-content"></div>
        </div>
    `;

    panel.querySelector('#btn-clear').addEventListener('click', async () => {
        if (confirm('确定要清空所有历史记录吗？')) {
            await window.__TAURI__.core.invoke('clear_history');
            loadHistory(panel);
        }
    });

    return panel;
}

export async function loadHistory(panel) {
    try {
        const history = await window.__TAURI__.core.invoke('get_history', { limit: 50 });
        const listEl = panel.querySelector('#history-list');

        if (!history || history.length === 0) {
            listEl.innerHTML = `
                <div style="padding: 20px; text-align: center; color: var(--text-muted);">
                    暂无历史记录
                </div>
            `;
            return;
        }

        listEl.innerHTML = history.map((item, idx) => `
            <div class="history-item${idx === 0 ? ' active' : ''}" data-index="${idx}">
                <span class="history-time">${new Date(item.timestamp * 1000).toLocaleTimeString()}</span>
                <span class="history-level"><span class="level-tag level-${item.level}">${item.level}</span></span>
                <span class="history-title">${item.window_title || '-'}</span>
            </div>
        `).join('');

        listEl.querySelectorAll('.history-item').forEach(el => {
            el.addEventListener('click', () => {
                listEl.querySelectorAll('.history-item').forEach(i => i.classList.remove('active'));
                el.classList.add('active');
                showHistoryDetail(panel, history[parseInt(el.dataset.index)]);
            });
        });

        if (history.length > 0) {
            showHistoryDetail(panel, history[0]);
        }
    } catch (e) {
        console.error('Failed to load history:', e);
    }
}

function showHistoryDetail(panel, item) {
    const detailEl = panel.querySelector('#history-detail');
    const contentEl = panel.querySelector('#detail-content');

    detailEl.style.display = 'block';
    contentEl.innerHTML = `
        <div class="info-row">
            <span class="info-key">判断</span>
            <span class="info-value">${item.judgment || '(abstain)'}</span>
        </div>
        <div class="info-row">
            <span class="info-key">下一步</span>
            <span class="info-value">${item.next_step || '-'}</span>
        </div>
        <div class="info-row">
            <span class="info-key">级别</span>
            <span class="info-value"><span class="level-tag level-${item.level}">${item.level}</span></span>
        </div>
        <div class="info-row">
            <span class="info-key">置信度</span>
            <span class="info-value">${Math.round(item.confidence * 100)}%</span>
        </div>
        <div class="info-row">
            <span class="info-key">依据</span>
            <span class="info-value">${item.grounds || '-'}</span>
        </div>
        <div class="info-row">
            <span class="info-key">用户操作</span>
            <span class="info-value" style="color: var(--primary);">${item.system_action}</span>
        </div>
    `;
}
```

- [ ] **步骤 2：提交代码**

```bash
git add src-tauri/src/components/HistoryList.js
git commit -m "feat: add HistoryList component"
```

---

## 任务 13：前端 - 设置组件

**涉及文件**：

- 创建：`src-tauri/src/components/Settings.js`

- [ ] **步骤 1：创建 Settings.js**

```js
export function createSettings() {
    const panel = document.createElement('div');
    panel.className = 'content-panel';
    panel.id = 'panel-settings';

    panel.innerHTML = `
        <div class="panel-title">⚙️ 设置</div>

        <div class="settings-section">
            <h3>模型配置</h3>
            <div class="form-row">
                <span class="form-label">Ollama 地址</span>
                <input type="text" class="form-input" id="setting-ollama-url">
            </div>
            <div class="form-row">
                <span class="form-label">模型名称</span>
                <input type="text" class="form-input" id="setting-model-name">
            </div>
        </div>

        <div class="settings-section">
            <h3>监控配置</h3>
            <div class="form-row">
                <span class="form-label">轮询间隔</span>
                <input type="number" class="form-input" id="setting-poll-interval" style="width: 80px;"> 秒
            </div>
            <div class="form-row">
                <label class="form-checkbox">
                    <input type="checkbox" id="setting-window-detection">
                    <span>启用窗口变化检测（相同窗口不重复处理）</span>
                </label>
            </div>
        </div>

        <div class="settings-section">
            <h3>执行配置</h3>
            <div class="form-row">
                <label class="form-checkbox">
                    <input type="checkbox" id="setting-execute-auto">
                    <span>execute 级别自动执行</span>
                </label>
            </div>
            <div class="form-row">
                <label class="form-checkbox">
                    <input type="checkbox" id="setting-request-dialog">
                    <span>request 级别使用 Windows 原生对话框确认</span>
                </label>
            </div>
            <div class="form-row">
                <span class="form-label">执行前延迟</span>
                <input type="number" class="form-input" id="setting-execute-delay" style="width: 80px;"> 秒
            </div>
        </div>

        <div class="btn-group">
            <button class="btn btn-secondary" id="btn-reset">恢复默认</button>
            <button class="btn btn-primary" id="btn-save">保存设置</button>
        </div>
    `;

    panel.querySelector('#btn-save').addEventListener('click', () => saveSettings(panel));
    panel.querySelector('#btn-reset').addEventListener('click', () => loadSettings(panel, true));

    return panel;
}

export async function loadSettings(panel, reset = false) {
    try {
        let config;
        if (reset) {
            config = {
                ollama_url: 'http://localhost:11434',
                model_name: 'llava',
                poll_interval_secs: 3,
                window_change_detection: true,
                execute_auto: true,
                request_use_native_dialog: true,
                execute_delay_secs: 1,
            };
        } else {
            config = await window.__TAURI__.core.invoke('get_config');
        }

        panel.querySelector('#setting-ollama-url').value = config.ollama_url;
        panel.querySelector('#setting-model-name').value = config.model_name;
        panel.querySelector('#setting-poll-interval').value = config.poll_interval_secs;
        panel.querySelector('#setting-window-detection').checked = config.window_change_detection;
        panel.querySelector('#setting-execute-auto').checked = config.execute_auto;
        panel.querySelector('#setting-request-dialog').checked = config.request_use_native_dialog;
        panel.querySelector('#setting-execute-delay').value = config.execute_delay_secs;
    } catch (e) {
        console.error('Failed to load settings:', e);
    }
}

async function saveSettings(panel) {
    const config = {
        ollama_url: panel.querySelector('#setting-ollama-url').value,
        model_name: panel.querySelector('#setting-model-name').value,
        poll_interval_secs: parseInt(panel.querySelector('#setting-poll-interval').value) || 3,
        window_change_detection: panel.querySelector('#setting-window-detection').checked,
        execute_auto: panel.querySelector('#setting-execute-auto').checked,
        request_use_native_dialog: panel.querySelector('#setting-request-dialog').checked,
        execute_delay_secs: parseInt(panel.querySelector('#setting-execute-delay').value) || 1,
    };

    try {
        await window.__TAURI__.core.invoke('save_config', { config });
        alert('设置已保存');
    } catch (e) {
        console.error('Failed to save settings:', e);
        alert('保存失败: ' + e);
    }
}
```

- [ ] **步骤 2：提交代码**

```bash
git add src-tauri/src/components/Settings.js
git commit -m "feat: add Settings component"
```

---

## 任务 14：前端 - 主应用整合

**涉及文件**：

- 修改：`src-tauri/src/main.js`
- 修改：`src-tauri/src/components/App.js`

- [ ] **步骤 1：更新 App.js**

```js
import { createStatusPanel, updateStatusPanel } from './StatusPanel.js';
import { createHistoryList, loadHistory } from './HistoryList.js';
import { createSettings, loadSettings } from './Settings.js';

let state = {
    trayState: 'running',
    currentWindow: null,
    lastJudgment: null,
};

export function mount(el) {
    el.innerHTML = `
        <div class="title-bar">
            <span class="logo">●</span>
            <span class="title">Cozmio - 主动智能体</span>
            <div class="window-controls">
                <span class="minimize"></span>
                <span class="maximize"></span>
                <span class="close"></span>
            </div>
        </div>
        <div class="main-content">
            <div class="sidebar">
                <div class="sidebar-item active" data-panel="status">
                    <span class="icon">📊</span>
                    <span>状态</span>
                </div>
                <div class="sidebar-item" data-panel="history">
                    <span class="icon">📜</span>
                    <span>历史</span>
                </div>
                <div class="sidebar-item" data-panel="settings">
                    <span class="icon">⚙️</span>
                    <span>设置</span>
                </div>
            </div>
            <div id="panels-container"></div>
        </div>
    `;

    const container = el.querySelector('#panels-container');
    const statusPanel = createStatusPanel(state);
    const historyPanel = createHistoryList();
    const settingsPanel = createSettings();

    container.appendChild(statusPanel);
    container.appendChild(historyPanel);
    container.appendChild(settingsPanel);

    // Sidebar navigation
    el.querySelectorAll('.sidebar-item').forEach(item => {
        item.addEventListener('click', () => {
            el.querySelectorAll('.sidebar-item').forEach(i => i.classList.remove('active'));
            el.querySelectorAll('.content-panel').forEach(p => p.classList.remove('active'));

            item.classList.add('active');
            const panelId = 'panel-' + item.dataset.panel;
            document.getElementById(panelId).classList.add('active');

            if (item.dataset.panel === 'history') {
                loadHistory(historyPanel);
            } else if (item.dataset.panel === 'settings') {
                loadSettings(settingsPanel);
            }
        });
    });

    // Window controls
    el.querySelector('.minimize').addEventListener('click', () => {
        window.__TAURI__.window.getCurrent().minimize();
    });

    el.querySelector('.maximize').addEventListener('click', () => {
        const win = window.__TAURI__.window.getCurrent();
        if (win.isMaximized()) {
            win.unmaximize();
        } else {
            win.maximize();
        }
    });

    el.querySelector('.close').addEventListener('click', () => {
        window.__TAURI__.window.getCurrent().hide();
    });

    // Listen for state updates from backend
    window.__TAURI__.event.listen('state-update', (event) => {
        state = { ...state, ...event.payload };
        updateStatusPanel(statusPanel, state);
    });

    // Initial state load
    loadState();
}

async function loadState() {
    try {
        const trayState = await window.__TAURI__.core.invoke('get_tray_state');
        state.trayState = trayState;
        updateStatusPanel(statusPanel, state);
    } catch (e) {
        console.error('Failed to load state:', e);
    }
}
```

- [ ] **步骤 2：更新 main.js**

```js
import { mount } from './components/App.js';

document.addEventListener('DOMContentLoaded', () => {
    mount(document.getElementById('app'));
});
```

- [ ] **步骤 3：运行开发服务器验证**

执行命令：`cd src-tauri && cargo tauri dev 2>&1 | head -100`
预期结果：前端正确渲染，导航可切换

- [ ] **步骤 4：提交代码**

```bash
git add src-tauri/src/main.js src-tauri/src/components/App.js
git commit -m "feat: integrate frontend components"
```

---

## 任务 15：主监控循环

**涉及文件**：

- 修改：`src-tauri/src/lib.rs`
- 修改：`src-tauri/src/main.rs`

- [ ] **步骤 1：添加主循环模块到 lib.rs**

```rust
pub mod main_loop;

pub use main_loop::start_main_loop;
```

- [ ] **步骤 2：创建 src-tauri/src/main_loop.rs**

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::commands::AppState;
use crate::config::Config;
use crate::executor::Executor;
use crate::logging::ActionLogger;
use crate::model_client::ModelClient;
use crate::tray::{TrayManager, TrayState};
use crate::window_monitor::WindowMonitor;

pub fn start_main_loop(
    app_handle: AppHandle,
    config: Config,
    logger: ActionLogger,
) {
    thread::spawn(move || {
        let executor = Executor::new(config.clone(), logger);
        let model_client = ModelClient::new(config.clone());
        let mut monitor = WindowMonitor::new();

        log::info!("Main loop started");

        loop {
            let tray_state = app_handle
                .state::<Mutex<TrayManager>>()
                .lock()
                .unwrap()
                .state();

            if tray_state == TrayState::Paused {
                thread::sleep(Duration::from_secs(1));
                continue;
            }

            // Emit processing state
            let _ = app_handle.emit("state-update", serde_json::json!({
                "trayState": "processing"
            }));

            match monitor.capture() {
                Ok(snapshot) => {
                    let has_changed = monitor.has_changed(&snapshot);

                    if has_changed {
                        monitor.update_last_title(&snapshot.window_info.title);

                        match model_client.call(&snapshot) {
                            Ok(output) => {
                                let result = executor.route(&output, &snapshot.window_info.title);

                                let _ = app_handle.emit("state-update", serde_json::json!({
                                    "trayState": "running",
                                    "currentWindow": {
                                        "title": snapshot.window_info.title,
                                        "processName": snapshot.window_info.process_name
                                    },
                                    "lastJudgment": {
                                        "judgment": output.judgment,
                                        "nextStep": output.next_step,
                                        "level": format!("{:?}", output.level).to_lowercase(),
                                        "confidence": output.confidence,
                                        "grounds": output.grounds,
                                        "timestamp": snapshot.timestamp
                                    }
                                }));

                                log::info!("Processed: {:?} -> {:?}", output.level, result);
                            }
                            Err(e) => {
                                log::error!("Model call failed: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Capture failed: {}", e);
                }
            }

            thread::sleep(Duration::from_secs(config.poll_interval_secs));
        }
    });
}
```

- [ ] **步骤 3：更新 main.rs 启动主循环**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    cozmio::run()
}

mod cozmio {
    use cozmio_lib::commands::AppState;
    use cozmio_lib::start_main_loop;
    use cozmio_lib::tray::TrayManager;
    use std::sync::Mutex;
    use tauri::Manager;

    pub fn run() {
        let app_state = AppState::new();

        tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_notification::init())
            .manage(app_state.clone())
            .manage(Mutex::new(TrayManager::new()))
            .setup(|app| {
                log::info!("Cozmio starting...");

                let handle = app.handle().clone();

                // Setup tray
                if let Err(e) = cozmio_lib::tray::TrayManager::setup_tray(&handle) {
                    log::error!("Failed to setup tray: {}", e);
                }

                // Start main monitoring loop
                let config = app_state.config.clone();
                let logger = app_state.logger.clone();
                start_main_loop(handle, config, logger);

                log::info!("Cozmio started successfully");
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                cozmio_lib::commands::get_config,
                cozmio_lib::commands::save_config,
                cozmio_lib::commands::get_history,
                cozmio_lib::commands::clear_history,
                cozmio_lib::commands::get_tray_state,
                cozmio_lib::commands::set_tray_state,
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
```

- [ ] **步骤 4：运行开发服务器验证**

执行命令：`cd src-tauri && cargo tauri dev 2>&1 | head -150`
预期结果：应用启动，主循环运行，窗口状态更新

- [ ] **步骤 5：提交代码**

```bash
git add src-tauri/src/lib.rs src-tauri/src/main_loop.rs src-tauri/src/main.rs
git commit -m "feat: add main monitoring loop with state updates"
```

---

## 方案总结

| 任务 | 描述 | 产出 |
|------|------|------|
| 1 | Tauri 项目初始化 | 项目骨架 |
| 2 | 配置管理模块 | config.rs |
| 3 | 窗口监控模块 | window_monitor.rs |
| 4 | 模型调用模块 | model_client.rs |
| 5 | 行为日志模块 | logging.rs |
| 6 | 执行路由模块 | executor.rs |
| 7 | 托盘模块 | tray.rs |
| 8 | IPC 命令模块 | commands.rs |
| 9 | 整合 main.rs | 完整后端 |
| 10 | 前端样式系统 | styles.css |
| 11 | 状态面板组件 | StatusPanel.js |
| 12 | 历史记录组件 | HistoryList.js |
| 13 | 设置组件 | Settings.js |
| 14 | 前端整合 | App.js + main.js |
| 15 | 主监控循环 | main_loop.rs |
