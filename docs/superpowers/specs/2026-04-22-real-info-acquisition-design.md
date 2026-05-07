# 真实信息获取基础 设计文档

## 1. 目标

建立 Rust 原生的窗口信息捕获基础设施，为后续所有阶段提供可靠的真实数据源。

核心原则：**没有真实输入，任何输出都是假的。**

---

## 2. 输出内容定义

### 2.1 全屏截图

| 字段 | 类型 | 说明 |
|------|------|------|
| `image_base64` | string | PNG 格式图片的 Base64 编码 |
| `monitor_index` | u32 | 显示器编号 |
| `monitor_name` | string | 显示器名称/描述 |
| `width` | u32 | 宽度像素 |
| `height` | u32 | 高度像素 |
| `timestamp` | i64 | Unix 时间戳（毫秒） |

**压缩要求**：PNG 格式（无损压缩，适合文本和 UI 元素）

### 2.2 前台窗口元信息

| 字段 | 类型 | 说明 |
|------|------|------|
| `hwnd` | u64 | 窗口句柄 |
| `title` | string | 窗口标题 |
| `process_name` | string | 进程 exe 名 |
| `process_id` | u32 | 进程 PID |
| `monitor_index` | u32 | 所在显示器 |
| `rect` | Rect | 窗口位置和大小 |
| `is_foreground` | bool | 是否前台窗口 |
| `is_visible` | bool | 是否可见 |
| `timestamp` | i64 | Unix 时间戳（毫秒） |

**Rect 定义**：
```rust
struct Rect {
    x: i32,      // 相对于屏幕的 X 坐标
    y: i32,      // 相对于屏幕的 Y 坐标
    width: u32,  // 宽度
    height: u32, // 高度
}
```

### 2.3 顶层窗口列表

| 字段 | 类型 | 说明 |
|------|------|------|
| `windows` | Vec\<WindowInfo\> | 所有顶层窗口数组 |
| `count` | usize | 窗口数量 |
| `timestamp` | i64 | Unix 时间戳（毫秒） |

**WindowInfo 结构**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `hwnd` | u64 | 窗口句柄 |
| `title` | string | 窗口标题 |
| `process_name` | string | 进程 exe 名 |
| `process_id` | u32 | 进程 PID |
| `rect` | Rect | 窗口位置和大小 |
| `is_visible` | bool | 是否可见 |
| `is_foreground` | bool | 是否前台窗口 |
| `z_order` | u32 | Z 轴顺序（0 = 最底层） |

---

## 3. 项目结构

```
cozmio/
├── cozmio_core/           # 核心库
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs         # 库入口，导出 public API
│       ├── screenshot.rs   # 截图功能
│       ├── window.rs      # 窗口信息捕获
│       ├── monitor.rs     # 显示器信息
│       └── error.rs       # 错误类型
│
├── cozmio_capture/        # CLI 工具
│   ├── Cargo.toml
│   └── src/
│       └── main.rs        # CLI 入口，调用 cozmio_core
│
└── Cargo.toml             # Workspace 配置
```

### 3.1 cozmio_core 公共 API

```rust
// 截图
pub fn capture_screen(monitor_index: u32) -> Result<Screenshot, CaptureError>
pub fn capture_all_monitors() -> Result<Vec<Screenshot>, CaptureError>

// 窗口信息
pub fn get_foreground_window() -> Result<WindowInfo, CaptureError>
pub fn get_all_windows() -> Result<Vec<WindowInfo>, CaptureError>
pub fn get_window_by_hwnd(hwnd: isize) -> Result<WindowInfo, CaptureError>

// 显示器信息
pub fn get_monitors() -> Result<Vec<MonitorInfo>, CaptureError>

// 一次性获取全部信息（CLI 用的组合接口）
pub fn capture_all() -> Result<CaptureResult, CaptureError>
```

### 3.2 CLI 输出格式

```json
{
  "screenshot": {
    "image_base64": "...",
    "monitor_index": 1,
    "width": 1920,
    "height": 1080,
    "timestamp": 1745323200000
  },
  "foreground_window": {
    "hwnd": 12345678,
    "title": "Visual Studio Code",
    "process_name": "Code.exe",
    "process_id": 1234,
    "monitor_index": 1,
    "rect": {"x": 0, "y": 0, "width": 1920, "height": 1040},
    "is_foreground": true,
    "is_visible": true,
    "timestamp": 1745323200000
  },
  "all_windows": {
    "count": 42,
    "windows": [...],
    "timestamp": 1745323200000
  },
  "timestamp": 1745323200000
}
```

---

## 4. 技术选型

| 模块 | 方案 | 理由 |
|------|------|------|
| 截图 | `xcap` crate | 跨平台，支持多显示器，输出 PNG/WebP/JPEG |
| 窗口 API | `windows` crate | 微软官方 Rust 绑定，完整覆盖 Win32 API |
| 错误处理 | `thiserror` + `anyhow` | 简洁的 Rust 错误模式 |
| 序列化 | `serde` + `serde_json` | JSON 输出 |

**依赖版本（锁定）**：
```toml
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

---

## 5. 核心实现逻辑

### 5.1 截图流程

```
1. 调用 xcap::Monitor::all() 获取所有显示器
2. 根据 monitor_index 选择目标显示器
3. 调用 monitor.capture() 获取截图
4. 转换为 PNG 字节流
5. Base64 编码
6. 记录元数据（宽高、时间戳）
```

### 5.2 前台窗口捕获流程

```
1. 调用 GetForegroundWindow() 获取前台窗口句柄
2. 调用 GetWindowThreadProcessId() 获取 PID
3. 调用 GetWindowText() 获取窗口标题
4. 调用 GetWindowRect() 获取窗口位置和大小
5. 调用 GetMonitorInfo() 确定所在显示器
6. 调用 IsWindowVisible() 判断可见性
7. 组装 WindowInfo
```

### 5.3 顶层窗口列表捕获流程

```
1. 调用 EnumWindows() 枚举所有窗口
2. 对每个窗口：
   - 跳过不可见窗口（IsWindowVisible 返回 false）
   - 跳过子窗口（GetWindowLongPtr(hwnd, GWL_STYLE) & WS_CHILD）
   - 获取 WindowInfo
3. 通过 GetWindowZOrder() 或类似方式确定 z_order
4. 按 z_order 排序输出
```

---

## 6. 错误处理

```rust
#[derive(Debug, thiserror::Error)]
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

---

## 7. 测试策略

### 7.1 单元测试

- `screenshot.rs`: 测试 PNG 编码完整性
- `window.rs`: 测试 WindowInfo 字段填充
- `monitor.rs`: 测试多显示器枚举

### 7.2 集成测试

```rust
#[test]
fn test_capture_all_returns_valid_json() {
    let result = capture_all().unwrap();
    // 验证 JSON 可解析
    // 验证 screenshot.image_base64 可解码为有效图片
    // 验证 foreground_window.hwnd > 0
    // 验证 all_windows.count > 0
}
```

### 7.3 人工验证

CLI 工具输出 JSON 后：
1. 检查 `screenshot` 是否为有效图片（粘贴到图片工具）
2. 检查 `foreground_window` 是否为当前前台窗口
3. 检查 `all_windows` 是否包含所有可见窗口

---

## 8. 验证目标

Phase 2 完成时，验证以下内容：

| 验证项 | 方法 |
|--------|------|
| 截图是真实屏幕数据 | 用图片工具打开 base64 解码后的 PNG，应为当前屏幕 |
| 前台窗口元信息正确 | 对比任务管理器中的窗口列表 |
| 窗口列表完整 | 截图内所有可见窗口都应出现在列表中 |
| JSON 输出可解析 | Python/其他工具能正常解析 |
| Z-order 正确 | 窗口顺序应与实际 Z 轴顺序一致 |
| 多显示器正确 | 切换主显示器后数据应正确更新 |

---

## 9. Phase 2 验收条件

1. `cozmio_capture --json` 输出完整 JSON，包含 screenshot + foreground_window + all_windows
2. 截图数据用图片工具验证为真实屏幕内容
3. 前台窗口信息与实际窗口一致
4. 窗口列表包含所有可见窗口
5. 错误情况（无前台窗口、无显示器）有合理错误信息
6. 核心库 cozmio_core 可独立被其他 Rust 项目引用

---

## 10. 与 Phase 1 的关系

Phase 1 验证了**模型能力**（对文字描述的判断）。
Phase 2 建立**真实信息获取基础**（捕获真实窗口数据）。

两者是独立的：
- Phase 1 可以用伪造输入快速验证模型
- Phase 2 必须用真实数据验证系统能获取什么

**Phase 3 才会将 Phase 2 的真实数据接入 Phase 1 的模型调用框架。**

---

## 11. 后续阶段预告（不写入实现计划）

- **Phase 3**: 将真实截图 + 元信息接入模型调用
- **Phase 4**: 建立 Python/Rust 桥接（如果需要 Python 调用）
- **Phase 5**: 桌面应用集成（Tauri/Electron）

Phase 2 只做信息获取，不做模型调用。
