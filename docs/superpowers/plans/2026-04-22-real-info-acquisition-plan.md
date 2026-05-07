# 真实信息获取 实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标**：建立 Rust 原生的窗口信息捕获基础设施（截图 + 前台窗口元信息 + 顶层窗口列表）

**架构思路**：xcap 做截图，windows-rs 做 Win32 API 调用，serde 做 JSON 序列化。cozmio_core 封装核心库，cozmio_capture 提供 CLI 入口。

**技术栈**：Rust 1.75+, xcap 0.1, windows 0.58, serde, thiserror, anyhow

---

## 文件结构

```
cozmio/
├── Cargo.toml                    # Workspace 配置
├── cozmio_core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs               # 库入口，导出 public API
│       ├── error.rs             # CaptureError 错误类型
│       ├── monitor.rs           # 显示器信息
│       ├── screenshot.rs        # 截图功能
│       ├── window.rs            # 窗口信息捕获
│       └── types.rs             # 公共类型定义（Rect, WindowInfo 等）
│
└── cozmio_capture/
    ├── Cargo.toml
    └── src/
        └── main.rs              # CLI 入口
```

---

## Task 1：创建项目骨架

**涉及文件**：

- 创建：`cozmio/Cargo.toml`
- 创建：`cozmio/cozmio_core/Cargo.toml`
- 创建：`cozmio/cozmio_core/src/lib.rs`
- 创建：`cozmio/cozmio_capture/Cargo.toml`
- 创建：`cozmio/cozmio_capture/src/main.rs`

- [ ] **步骤1：创建 Workspace 配置**

```toml
# cozmio/Cargo.toml
[workspace]
resolver = "2"
members = ["cozmio_core", "cozmio_capture"]
```

- [ ] **步骤2：创建 cozmio_core 的 Cargo.toml**

```toml
# cozmio/cozmio_core/Cargo.toml
[package]
name = "cozmio_core"
version = "0.1.0"
edition = "2021"

[dependencies]
xcap = "0.1"
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Graphics_Gdi",
    "Win32_System_Threading",
] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
anyhow = "1.0"
```

- [ ] **步骤3：创建 cozmio_core/lib.rs**

```rust
// cozmio/cozmio_core/src/lib.rs
pub mod error;
pub mod monitor;
pub mod screenshot;
pub mod types;
pub mod window;

pub use error::CaptureError;
pub use monitor::MonitorInfo;
pub use screenshot::Screenshot;
pub use types::{Rect, WindowInfo};
pub use window::{get_all_windows, get_foreground_window, get_window_by_hwnd};
```

- [ ] **步骤4：创建 cozmio_capture 的 Cargo.toml**

```toml
# cozmio/cozmio_capture/Cargo.toml
[package]
name = "cozmio_capture"
version = "0.1.0"
edition = "2021"

[dependencies]
cozmio_core = { path = "../cozmio_core" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

- [ ] **步骤5：创建 cozmio_capture/main.rs**

```rust
// cozmio/cozmio_capture/src/main.rs
fn main() {
    println!("cozmio_capture ready");
}
```

- [ ] **步骤6：验证项目可编译**

执行命令：`cd cozmio && cargo build`
预期结果：编译成功，无错误

- [ ] **步骤7：提交代码**

```bash
cd cozmio && git add Cargo.toml cozmio_core/ cozmio_capture/
git commit -m "feat: create project skeleton with workspace structure"
```

---

## Task 2：定义类型和错误

**涉及文件**：

- 创建：`cozmio/cozmio_core/src/types.rs`
- 创建：`cozmio/cozmio_core/src/error.rs`

- [ ] **步骤1：编写 types.rs**

```rust
// cozmio/cozmio_core/src/types.rs
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub hwnd: u64,
    pub title: String,
    pub process_name: String,
    pub process_id: u32,
    pub monitor_index: u32,
    pub rect: Rect,
    pub is_foreground: bool,
    pub is_visible: bool,
    pub z_order: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonitorInfo {
    pub index: u32,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub name: String,
}
```

- [ ] **步骤2：编写 error.rs**

```rust
// cozmio/cozmio_core/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("截图失败: {0}")]
    ScreenshotFailed(String),

    #[error("窗口获取失败: {0}")]
    WindowFailed(String),

    #[error("显示器 {0} 不存在")]
    MonitorNotFound(u32),

    #[error("Win32 API 调用失败: {0}")]
    Win32Error(String),

    #[error("编码失败: {0}")]
    EncodingError(String),
}
```

- [ ] **步骤3：验证类型定义**

执行命令：`cd cozmio && cargo check -p cozmio_core`
预期结果：编译成功，无错误

- [ ] **步骤4：提交代码**

```bash
cd cozmio && git add cozmio_core/src/types.rs cozmio_core/src/error.rs
git commit -m "feat(cozmio_core): add types and error definitions"
```

---

## Task 3：实现显示器信息获取

**涉及文件**：

- 创建：`cozmio/cozmio_core/src/monitor.rs`

- [ ] **步骤1：编写 monitor.rs**

```rust
// cozmio/cozmio_core/src/monitor.rs
use crate::error::CaptureError;
use crate::types::MonitorInfo;
use xcap::Monitor;

pub fn get_monitors() -> Result<Vec<MonitorInfo>, CaptureError> {
    let monitors = Monitor::all();

    monitors
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let x = m.x();
            let y = m.y();
            let width = m.width() as u32;
            let height = m.height() as u32;

            Ok(MonitorInfo {
                index: i as u32 + 1,
                width,
                height,
                x,
                y,
                name: format!("Monitor {}", i + 1),
            })
        })
        .collect()
}

pub fn get_monitor_by_index(index: u32) -> Result<Monitor, CaptureError> {
    let monitors = Monitor::all();
    let idx = index.saturating_sub(1) as usize;

    monitors
        .get(idx)
        .ok_or(CaptureError::MonitorNotFound(index))
        .map(|m| m.clone())
}
```

- [ ] **步骤2：更新 lib.rs 导出**

```rust
// cozmio/cozmio_core/src/lib.rs 添加
pub use monitor::{get_monitor_by_index, get_monitors};
```

- [ ] **步骤3：验证编译**

执行命令：`cd cozmio && cargo check -p cozmio_core`
预期结果：编译成功，无错误

- [ ] **步骤4：提交代码**

```bash
cd cozmio && git add cozmio_core/src/monitor.rs cozmio_core/src/lib.rs
git commit -m "feat(cozmio_core): add monitor enumeration"
```

---

## Task 4：实现截图功能

**涉及文件**：

- 创建：`cozmio/cozmio_core/src/screenshot.rs`

- [ ] **步骤1：编写 screenshot.rs**

```rust
// cozmio/cozmio_core/src/screenshot.rs
use crate::error::CaptureError;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::io::Cursor;
use xcap::Monitor;

use crate::monitor::get_monitor_by_index;

#[derive(Debug, Clone)]
pub struct Screenshot {
    pub image_base64: String,
    pub monitor_index: u32,
    pub width: u32,
    pub height: u32,
    pub timestamp: i64,
}

impl Screenshot {
    pub fn capture(monitor_index: u32) -> Result<Self, CaptureError> {
        let monitor = get_monitor_by_index(monitor_index)?;

        let image = monitor
            .capture_image()
            .map_err(|e| CaptureError::ScreenshotFailed(e.to_string()))?;

        let mut buffer = Cursor::new(Vec::new());
        image
            .write_to(&mut buffer, ::image::ImageFormat::Png)
            .map_err(|e| CaptureError::EncodingError(e.to_string()))?;

        let image_base64 = STANDARD.encode(buffer.into_inner());
        let width = monitor.width() as u32;
        let height = monitor.height() as u32;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        Ok(Screenshot {
            image_base64,
            monitor_index,
            width,
            height,
            timestamp,
        })
    }
}
```

- [ ] **步骤2：更新 lib.rs 导出**

```rust
// cozmio/cozmio_core/src/lib.rs 添加
pub use screenshot::Screenshot;
```

- [ ] **步骤3：更新 Cargo.toml 添加 base64 和 image**

```toml
# cozmio/cozmio_core/Cargo.toml
[dependencies]
base64 = "0.22"
image = "0.25"
```

- [ ] **步骤4：验证编译**

执行命令：`cd cozmio && cargo check -p cozmio_core`
预期结果：编译成功，无错误

- [ ] **步骤5：提交代码**

```bash
cd cozmio && git add cozmio_core/src/screenshot.rs cozmio_core/Cargo.toml cozmio_core/src/lib.rs
git commit -m "feat(cozmio_core): add screenshot capture with PNG output"
```

---

## Task 5：实现窗口信息获取

**涉及文件**：

- 创建：`cozmio/cozmio_core/src/window.rs`

- [ ] **步骤1：编写 window.rs**

```rust
// cozmio/cozmio_core/src/window.rs
use crate::error::CaptureError;
use crate::monitor::get_monitors;
use crate::types::{Rect, WindowInfo};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{GetWindowRect, GetWindowRgn, HRGN};
use windows::Win32::System::Threading::{
    GetProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowLongPtrW, GetWindowTextW, GetWindowThreadProcessId,
    IsWindowVisible, GWL_STYLE, WS_CHILD, WS_VISIBLE,
};

fn wide_to_string(ptr: &[u16]) -> String {
    if ptr.is_empty() {
        return String::new();
    }
    let len = ptr.iter().position(|&c| c == 0).unwrap_or(ptr.len());
    OsString::from_wide(&ptr[..len])
        .to_string_lossy()
        .into_owned()
}

fn get_window_title(hwnd: HWND) -> String {
    unsafe {
        let mut buffer = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buffer);
        wide_to_string(&buffer[..len as usize])
    }
}

fn get_process_name(hwnd: HWND) -> (String, u32) {
    unsafe {
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        let process_name = if process_id == 0 {
            String::new()
        } else {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, BOOL::from(false), process_id);
            match handle {
                Ok(h) => {
                    // Note: Getting process name on Windows is complex
                    // For now, return PID as string fallback
                    drop(h);
                    format!("PID:{}", process_id)
                }
                Err(_) => format!("PID:{}", process_id),
            }
        };

        (process_name, process_id)
    }
}

fn get_window_rect(hwnd: HWND) -> Rect {
    unsafe {
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect);
        Rect {
            x: rect.left,
            y: rect.top,
            width: (rect.right - rect.left) as u32,
            height: (rect.bottom - rect.top) as u32,
        }
    }
}

fn is_window_visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd).as_bool() }
}

fn is_child_window(hwnd: HWND) -> bool {
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        (style & WS_CHILD.0) != 0
    }
}

fn monitor_index_from_point(x: i32, y: i32) -> u32 {
    let monitors = get_monitors().unwrap_or_default();
    for m in monitors {
        if x >= m.x && x < m.x + m.width as i32 && y >= m.y && y < m.y + m.height as i32 {
            return m.index;
        }
    }
    1
}

pub fn get_foreground_window_info() -> Result<WindowInfo, CaptureError> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return Err(CaptureError::WindowFailed("No foreground window".to_string()));
        }

        let title = get_window_title(hwnd);
        let (process_name, process_id) = get_process_name(hwnd);
        let rect = get_window_rect(hwnd);
        let monitor_index = monitor_index_from_point(rect.x, rect.y);

        Ok(WindowInfo {
            hwnd: hwnd.0 as u64,
            title,
            process_name,
            process_id,
            monitor_index,
            rect,
            is_foreground: true,
            is_visible: is_window_visible(hwnd),
            z_order: 0,
        })
    }
}

pub fn get_window_info(hwnd: HWND, z_order: u32) -> Option<WindowInfo> {
    if hwnd.0.is_null() || is_child_window(hwnd) {
        return None;
    }

    let title = get_window_title(hwnd);
    if title.is_empty() {
        return None;
    }

    let (process_name, process_id) = get_process_name(hwnd);
    let rect = get_window_rect(hwnd);
    let is_visible = is_window_visible(hwnd);
    let monitor_index = monitor_index_from_point(rect.x, rect.y);

    Some(WindowInfo {
        hwnd: hwnd.0 as u64,
        title,
        process_name,
        process_id,
        monitor_index,
        rect,
        is_foreground: false,
        is_visible,
        z_order,
    })
}

pub fn get_all_windows() -> Result<Vec<WindowInfo>, CaptureError> {
    struct Context {
        windows: Vec<WindowInfo>,
        current_z: u32,
    }

    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let ctx = &mut *(lparam.0 as *mut Context);

        if let Some(mut info) = get_window_info(hwnd, ctx.current_z) {
            // Update z_order to be actual order
            info.z_order = ctx.current_z;
            ctx.windows.push(info);
            ctx.current_z += 1;
        }

        BOOL::from(true)
    }

    let mut ctx = Context {
        windows: Vec::new(),
        current_z: 1,
    };

    unsafe {
        EnumWindows(
            Some(enum_callback),
            LPARAM(&mut ctx as *mut Context as isize),
        )
        .map_err(|e| CaptureError::Win32Error(e.to_string()))?;
    }

    // Sort by z_order (lower = bottom of stack)
    ctx.windows.sort_by_key(|w| w.z_order);

    Ok(ctx.windows)
}

pub fn get_window_by_hwnd(hwnd_val: isize) -> Result<WindowInfo, CaptureError> {
    let hwnd = HWND(hwnd_val as *mut std::ffi::c_void);
    if hwnd.0.is_null() {
        return Err(CaptureError::WindowFailed("Invalid HWND".to_string()));
    }

    get_window_info(hwnd, 0)
        .ok_or_else(|| CaptureError::WindowFailed("Window not found".to_string()))
}
```

- [ ] **步骤2：更新 lib.rs 导出**

```rust
// cozmio/cozmio_core/src/lib.rs
pub use window::{get_all_windows, get_foreground_window_info, get_window_by_hwnd};
```

- [ ] **步骤3：验证编译**

执行命令：`cd cozmio && cargo check -p cozmio_core`
预期结果：编译成功，无错误

- [ ] **步骤4：提交代码**

```bash
cd cozmio && git add cozmio_core/src/window.rs cozmio_core/src/lib.rs
git commit -m "feat(cozmio_core): add window info capture with Win32 API"
```

---

## Task 6：实现组合 API 和 JSON 输出

**涉及文件**：

- 创建：`cozmio/cozmio_core/src/lib.rs`（更新）
- 创建：`cozmio/cozmio_capture/src/main.rs`（更新）

- [ ] **步骤1：更新 lib.rs 添加 CaptureAllResult**

```rust
// cozmio/cozmio_core/src/lib.rs
#[derive(Debug, Clone, Serialize)]
pub struct CaptureAllResult {
    pub screenshot: Option<Screenshot>,
    pub foreground_window: Option<WindowInfo>,
    pub all_windows: WindowListResult,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowListResult {
    pub count: usize,
    pub windows: Vec<WindowInfo>,
    pub timestamp: i64,
}

pub fn capture_all(monitor_index: u32) -> Result<CaptureAllResult, CaptureError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let screenshot = Screenshot::capture(monitor_index).ok();
    let foreground_window = get_foreground_window_info().ok();
    let all_windows = get_all_windows()?;

    Ok(CaptureAllResult {
        screenshot,
        foreground_window,
        all_windows: WindowListResult {
            count: all_windows.len(),
            windows: all_windows,
            timestamp,
        },
        timestamp,
    })
}
```

- [ ] **步骤2：更新 cozmio_capture/main.rs**

```rust
// cozmio/cozmio_capture/src/main.rs
use cozmio_core::{capture_all, get_monitors};
use serde_json;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--monitors".to_string()) {
        // List monitors and exit
        match get_monitors() {
            Ok(monitors) => {
                println!("Available monitors:");
                for m in monitors {
                    println!("  {}: {}x{}+{}+{}", m.index, m.width, m.height, m.x, m.y);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Default: capture all info from monitor 1
    let monitor_index: u32 = args
        .iter()
        .position(|a| a == "--monitor")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    match capture_all(monitor_index) {
        Ok(result) => {
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
```

- [ ] **步骤3：验证编译**

执行命令：`cd cozmio && cargo build`
预期结果：编译成功，无错误

- [ ] **步骤4：运行 CLI 测试**

执行命令：`cd cozmio && cargo run -p cozmio_capture -- --monitors`
预期结果：输出可用显示器列表

- [ ] **步骤5：提交代码**

```bash
cd cozmio && git add cozmio_core/src/lib.rs cozmio_capture/src/main.rs
git commit -m "feat: add capture_all API and CLI entry point"
```

---

## Task 7：验证输出正确性

**涉及文件**：

- 创建：`cozmio/cozmio_capture/examples/verify_capture.rs`（可选测试脚本）

- [ ] **步骤1：运行 CLI 验证 JSON 输出**

执行命令：`cd cozmio && cargo run -p cozmio_capture`
预期结果：
1. 输出有效的 JSON
2. JSON 包含 screenshot.image_base64（非空）
3. JSON 包含 foreground_window（hwnd > 0）
4. JSON 包含 all_windows.windows（count > 0）

- [ ] **步骤2：验证 screenshot 可以解码为图片**

将输出的 base64 复制，用以下命令验证：
```bash
cd cozmio && cargo run -p cozmio_capture | jq -r '.screenshot.image_base64' | base64 -d > test_screenshot.png
# 用图片查看器打开 test_screenshot.png
```

预期结果：test_screenshot.png 是当前屏幕的截图

- [ ] **步骤3：验证 foreground_window 与任务管理器一致**

执行命令：在 Windows 上打开任务管理器，对比 hwnd 和窗口标题
预期结果：CLI 输出的 foreground_window 是当前前台窗口

- [ ] **步骤4：提交最终代码**

```bash
cd cozmio && git add -A && git commit -m "feat: complete cozmio_capture CLI with full window info"
```

---

## Task 8：添加单元测试

**涉及文件**：

- 创建：`cozmio/cozmio_core/tests/unit_tests.rs`

- [ ] **步骤1：编写单元测试**

```rust
// cozmio/cozmio_core/tests/unit_tests.rs
use cozmio_core::*;

#[test]
fn test_monitor_list_not_empty() {
    let monitors = get_monitors().unwrap();
    assert!(!monitors.is_empty(), "At least one monitor should exist");
}

#[test]
fn test_window_info_has_required_fields() {
    let windows = get_all_windows().unwrap();
    for w in windows {
        assert!(w.hwnd > 0, "HWND must be non-zero");
        // Note: title may be empty for some system windows, but structure is valid
    }
}

#[test]
fn test_screenshot_capture() {
    let screenshot = Screenshot::capture(1).unwrap();
    assert!(!screenshot.image_base64.is_empty(), "Screenshot base64 must not be empty");
    assert!(screenshot.width > 0, "Width must be non-zero");
    assert!(screenshot.height > 0, "Height must be non-zero");
    assert!(screenshot.timestamp > 0, "Timestamp must be non-zero");
}

#[test]
fn test_capture_all_produces_valid_json() {
    let result = capture_all(1).unwrap();
    let json = serde_json::to_string(&result).unwrap();
    assert!(!json.is_empty(), "JSON output must not be empty");
    assert!(json.contains("screenshot"), "JSON must contain screenshot field");
    assert!(json.contains("foreground_window"), "JSON must contain foreground_window field");
    assert!(json.contains("all_windows"), "JSON must contain all_windows field");
}
```

- [ ] **步骤2：运行测试**

执行命令：`cd cozmio && cargo test -p cozmio_core`
预期结果：所有测试通过

- [ ] **步骤3：提交测试**

```bash
cd cozmio && git add cozmio_core/tests/unit_tests.rs
git commit -m "test(cozmio_core): add unit tests for core functionality"
```

---

## 验收标准

| 验证项 | 预期结果 |
|--------|----------|
| `cargo build` | 编译成功，无错误 |
| `cargo run -p cozmio_capture -- --monitors` | 列出所有显示器 |
| `cargo run -p cozmio_capture` | 输出有效 JSON |
| JSON.screenshot.image_base64 | 可解码为 PNG 图片 |
| JSON.foreground_window.hwnd | > 0 |
| JSON.all_windows.count | > 0 |
| `cargo test -p cozmio_core` | 所有测试通过 |

---

## 方案总结

| Task | 描述 | 关键文件 |
|------|------|----------|
| 1 | 项目骨架 | Cargo.toml, lib.rs |
| 2 | 类型和错误 | types.rs, error.rs |
| 3 | 显示器信息 | monitor.rs |
| 4 | 截图功能 | screenshot.rs |
| 5 | 窗口信息 | window.rs |
| 6 | 组合 API | lib.rs, main.rs |
| 7 | 验证输出 | CLI 运行验证 |
| 8 | 单元测试 | unit_tests.rs |
