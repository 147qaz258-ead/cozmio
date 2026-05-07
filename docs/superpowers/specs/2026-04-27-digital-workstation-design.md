# 数字工位 / 浮动迷你面板 设计文档

**日期**: 2026-04-27
**类型**: UI 重构 / 新功能设计
**状态**: 已批准，待实现

---

## 背景与问题

当前 StatusPanel 有 6 个模块卡，以"系统内部结构"（last_judgment / current_task / relay_session / progress / result）组织信息。用户（甚至开发者本人）面对 `relay_status: running` + `"Desktop host received Relay event"` 这类信息，无法回答"agent 在干什么、我需要做什么"。

核心问题：
- **语义不透明**：relay_status 是开发者视角的状态名，不是用户能理解的
- **信息过载**：6 个模块卡平铺，用户需要自己理解哪些重要
- **被动感知**：用户需要主动打开主面板才能感知状态，后台运行时完全 blind

---

## 设计目标

1. 用户一眼感知：当前有没有事
2. 用户一眼知道：有事的话具体要做什么
3. 后台运行时也能感知：不打开主面板也能知道状态变化

---

## 设计方案

### 整体架构

```
┌─────────────────────────────────┐
│  主面板 StatusPanel              │  ← 降级为详情面板，保留全部技术细节
│  6 模块卡结构（现有）            │    用户主动打开时查看
└─────────────────────────────────┘

┌─────────────────────┐
│  浮动迷你面板        │  ← 新增，始终可见，固定右下角
│  280 × 360 px       │
│  无装饰暗色主题      │
└─────────────────────┘

┌──┐ 托盘图标（三色）
```

### 浮动迷你面板

**视觉风格**
- 暗色主题（#0d0d0f 背景），接近专业终端工具感
- 无边框、无标题栏（`decorations: false`）
- 圆角 16px，微投影

**布局结构（从上到下）**

```
┌──────────────────────────┐  ← 顶部：状态环 + 状态名
│  [状态环]  状态名        │
│   ●      空闲中          │
├──────────────────────────┤  ← 主状态卡：行动召唤
│  ● 空闲中                │
│  工位空闲，随时准备监控    │
│  ───────────────────────  │
│  [  开始监控  ] [历史]   │
├──────────────────────────┤  ← 任务卡（无任务时隐藏）
│  当前任务                 │
│  "切换到 Claude Code"    │
│  来源：Chrome - 新标签页  │
│  [  去确认  ]            │
├──────────────────────────┤  ← 系统状态（始终显示）
│  ● Ollama   ● Relay     │
└──────────────────────────┘
```

**状态环（Progress Ring）**

| 状态 | 颜色 | 中心图标 | 含义 |
|------|------|----------|------|
| idle | 灰 #636366 | 空心圆 | 待命 |
| monitoring | 蓝 #0a84ff | 圆点 | 监控中 |
| analyzing | 蓝 #0a84ff | 进度环 | 分析中 |
| confirm | 橙 #ff9f0a | 问号 | 需要确认 |
| executing | 橙 #ff9f0a | 圆点 | 执行中 |
| done | 绿 #34c759 | 勾 | 完成 |
| error | 红 #ff3b30 | 叉 | 错误 |

**颜色语义统一**
- 绿 = 正常 / 完成
- 蓝 = 进行中 / 分析
- 橙 = 等待用户 / 警告
- 红 = 错误

**任务卡激活条件**
- `pending_confirmation` 非空 → 显示"去确认"
- `current_task` 非空且 task_state 为活跃 → 显示任务内容
- `relay_status` 为 executing/running → 显示进度 + 中断按钮

**操作按钮语义**
- 空闲：开始监控
- 待确认：去确认（触发系统通知）
- 执行中：中断任务
- 完成：查看结果
- 错误：重试

### 托盘图标联动

| 系统状态 | 图标颜色 |
|----------|----------|
| idle + 无错误 | 绿色 |
| monitoring / analyzing | 蓝色 |
| confirm / executing | 橙色 |
| error | 红色 |

图标通过 `TrayIconBuilder::set_icon` 动态切换。
托盘 tooltip 显示当前状态文字（如"空闲中"、"分析中"、"执行中: 切换到 Claude Code"）。

### 与主面板的关系

| 面板 | 定位 | 打开方式 |
|------|------|----------|
| 浮动迷你面板 | 始终可见，工位感知层 | 自动打开，不关闭 |
| 主面板 StatusPanel | 详情层，技术细节 | 用户手动打开 |

主面板不删除，保持现有结构，作为浮动面板的"详情展开"。

---

## 技术方案

### 多窗口架构

使用 Tauri v2 的 `WebviewWindowBuilder` 创建第二个无边框窗口：

```rust
// mini_window.rs
pub fn create_mini_window(app: &AppHandle) -> Result<()> {
    WebviewWindowBuilder::new(app, "mini", WebviewUrl::App("mini.html".into()))
        .title("Cozmio 工位")
        .inner_size(280.0, 360.0)
        .decorations(false)
        .always_on_top(true)
        .resizable(false)
        .visible(true)
        .skip_taskbar(true)
        .shadow(false)
        .build()?;
}
```

窗口固定在右下角（屏幕工作区边缘）。

### 前后端接口

**后端逻辑不变**：`emit_state_update` 已通过 Tauri event 系统支持多窗口订阅。

**前端 mini panel** 订阅 `state-update` 事件，渲染简化状态：

```javascript
// MiniPanel.js 接收 StateUpdate，同步渲染
await listen('state-update', (event) => {
    renderMiniPanel(event.payload);
});
```

**封装操作 command**：

```rust
#[tauri::command]
pub fn mini_action(app: AppHandle, action: String) -> Result<(), String> {
    match action.as_str() {
        "confirm" => confirm_pending_task(app),
        "interrupt" => interrupt_current_task(app),
        "toggle" => { /* toggle running */ }
        _ => Err("unknown action")
    }
}
```

### 图标文件

准备 4 张 32×32 PNG 图标：
- `tray-green.png`
- `tray-blue.png`
- `tray-orange.png`
- `tray-red.png`

放在 `src-tauri/icons/` 目录。

### 托盘图标动态切换

```rust
// tray.rs 修改
pub fn update_tray_icon(state: &str) {
    let icon = match state {
        "idle" => include_bytes!("icons/tray-green.png"),
        "monitoring" | "analyzing" => include_bytes!("icons/tray-blue.png"),
        "confirm" | "executing" => include_bytes!("icons/tray-orange.png"),
        "error" | "failed" => include_bytes!("icons/tray-red.png"),
        _ => include_bytes!("icons/tray-green.png"),
    };
    // 调用 tray.set_icon(icon)
}
```

在 `set_tray_state` 和 `main_loop` 的状态变化处调用。

---

## 实现文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/src/mini_window.rs` | 新增 | 浮动窗口创建逻辑 |
| `src-tauri/src/components/MiniPanel.js` | 新增 | 迷你面板前端组件 |
| `src-tauri/frontend/mini.html` | 新增 | 迷你面板 HTML 入口 |
| `src-tauri/icons/tray-*.png` | 新增 | 4 张托盘图标 |
| `src-tauri/src/tray.rs` | 修改 | 添加动态图标切换 |
| `src-tauri/src/main.rs` | 修改 | 注册 mini window builder |
| `src-tauri/src/commands.rs` | 修改 | 添加 mini_action command |

---

## 状态映射表（后端 → 前端）

| 后端状态字段 | 迷你面板状态 | 颜色 |
|-------------|-------------|------|
| `tray_state=Idle` + `running_state=Stopped` | idle | 灰 |
| `tray_state=Idle` + `running_state=Running` + 无事件 | monitoring | 蓝 |
| `tray_state=Processing` | analyzing | 蓝 |
| `pending_confirmation` 非空 | confirm | 橙 |
| `relay_status` ∈ {running, waiting, dispatching} | executing | 橙 |
| `relay_status` = completed | done | 绿 |
| `relay_status` = failed / error | error | 红 |

---

## 未纳入范围

- 主面板 StatusPanel 重构（保留现有 6 模块卡结构）
- 通知系统的修改（现有 toast 行为不变）
- 主面板和迷你面板之间的通信（各自独立订阅 state-update）
