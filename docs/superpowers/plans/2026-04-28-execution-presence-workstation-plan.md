# 执行端存活工位实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标**：将桌面主界面和悬浮窗升级为“执行端存活工位”，让用户一眼看到 Cozmio、观察循环、Relay、Claude Code 是否活着。

**架构思路**：H1 不改 Relay 协议，先在前端从现有 `StateUpdate` 推导 `ExecutionPresence`。主界面 STATUS 首页新增工位总览和执行端卡片，mini window 恢复为固定尺寸像素工位窗，二者共享同一个 presence 计算模块。

**技术栈**：Tauri v2、Rust、vanilla JavaScript、CSS

---

## 产品类型

- `traditional_software`
- 验证重点：Rust 编译通过、JS 模块引用正确、UI 状态推导不误导执行端在线性。

---

## 文件结构

- 创建：`cozmio/src-tauri/src/components/PresenceModel.js`
  - 从后端 `StateUpdate` 推导 `ExecutionPresence`。
  - 导出状态颜色、标签、主动作选择函数。
- 修改：`cozmio/src-tauri/src/components/StatusPanel.js`
  - 引入 presence model。
  - STATUS 首屏新增 workstation hero、execution target cards、current handoff。
  - 保留现有 detail cards。
- 修改：`cozmio/src-tauri/src/components/App.js`
  - header dot 改用 presence model，避免和 mini 窗口重复维护状态逻辑。
- 修改：`cozmio/src-tauri/src/mini.html`
  - 从 dot HTML 改为固定尺寸像素工位窗。
- 修改：`cozmio/src-tauri/src/components/MiniDot.js`
  - 改为渲染 mini workstation，复用 presence model。
- 修改：`cozmio/src-tauri/src/mini_window.rs`
  - 窗口尺寸从 `24x24` 改为 `196x148`，位置预留边距。
- 修改：`cozmio/src-tauri/src/main.rs`
  - 恢复 `create_mini_window()` 调用。
- 修改：`cozmio/src-tauri/src/styles.css`
  - 添加主界面工位/presence 样式。

---

## 任务1：新增 Execution Presence 前端模型

**涉及文件**：

- 创建：`cozmio/src-tauri/src/components/PresenceModel.js`

- [x] **步骤1：创建状态推导模块**

实现函数：
- `buildExecutionPresence(state)`
- `presenceStatusLabel(status)`
- `presenceToneClass(status)`
- `pickPrimaryTarget(presence)`
- `primaryActionForPresence(state, presence)`

关键规则：
- `desktop-host` 固定 online，表示 UI 已接收状态。
- `monitor-loop` 根据 `running_state` 判断 online/offline。
- `relay-engine` 根据 `relay_execution.relay_status` 判断 working/attention/degraded/online/unknown。
- `claude-code` 只有存在 `session_id` 时才可显示 working/online/degraded；否则 unknown。

- [x] **步骤2：人工检查状态误导风险**

确认没有 session 时 `Claude Code` 不显示为 working；Relay 没有 session 时显示 standby/unknown。

---

## 任务2：升级主界面 STATUS 首页

**涉及文件**：

- 修改：`cozmio/src-tauri/src/components/StatusPanel.js`
- 修改：`cozmio/src-tauri/src/styles.css`

- [x] **步骤1：导入 presence model**

在 `StatusPanel.js` 顶部引入：

```javascript
import { buildExecutionPresence, presenceStatusLabel, presenceToneClass, pickPrimaryTarget } from './PresenceModel.js';
```

- [x] **步骤2：在 `renderPanel(state)` 顶部渲染工位总览**

在现有技术模块卡之前添加：
- `renderWorkstationHero(state, presence)`
- `renderExecutionPresence(presence)`
- `renderCurrentHandoff(state, presence)`

- [x] **步骤3：保留现有细节模块**

原有 AGENT STATE、LAST JUDGMENT、CURRENT TASK、RELAY SESSION、RECENT PROGRESS、RESULT 仍保留，但标题从 `TASK MONITOR` 改为 `WORKSTATION`，下方细节区用 `DETAILS` 分隔。

- [x] **步骤4：添加 CSS**

在 `styles.css` 末尾添加：
- `.workstation-hero`
- `.pixel-agent`
- `.presence-grid`
- `.presence-card`
- `.handoff-card`
- `.tone-online/.tone-working/.tone-attention/.tone-degraded/.tone-offline/.tone-unknown`

---

## 任务3：升级悬浮窗为像素工位

**涉及文件**：

- 修改：`cozmio/src-tauri/src/mini.html`
- 修改：`cozmio/src-tauri/src/components/MiniDot.js`
- 修改：`cozmio/src-tauri/src/mini_window.rs`
- 修改：`cozmio/src-tauri/src/main.rs`

- [x] **步骤1：重写 mini HTML 结构与 CSS**

`mini.html` 使用固定 `196x148` 画布，包含：
- `#mini-workstation`
- 像素工位区域
- 状态文本
- targets summary
- hover actions

- [x] **步骤2：重写 MiniDot.js 渲染逻辑**

复用 `PresenceModel.js`：
- 初始调用 `get_ui_state`
- 监听 `state-update`
- 调用 `renderMiniWorkstation(state)`
- 按 presence primary target 显示主要状态
- 快捷按钮继续调用 `mini_action`

- [x] **步骤3：修改原生 mini window 尺寸**

`mini_window.rs`：
- `.inner_size(196.0, 148.0)`
- 右下角位置改为 `width - 220`, `height - 180`

- [x] **步骤4：恢复启动时创建 mini window**

`main.rs` setup 中取消注释 `mini_window::create_mini_window(app.handle())`。

---

## 任务4：验证

**涉及命令**：

- `cargo build -p cozmio`

- [x] **步骤1：运行构建**

执行命令：

```bash
cd D:/C_Projects/Agent/cozmio/cozmio && cargo build -p cozmio
```

预期结果：退出码 0。

- [x] **步骤2：检查关键文件引用**

确认：
- `StatusPanel.js` 能导入 `PresenceModel.js`
- `MiniDot.js` 能导入 `PresenceModel.js`
- `main.rs` 只注册一次 `mini_action` 时不引入新重复项
- `mini_window.rs` 不再创建 24x24 小圆点窗口

- [x] **步骤3：记录未完成的运行时验证**

如果未启动 Tauri dev，则明确说明未做人工截图验证。

执行记录：JS 语法检查通过；`cargo build -p cozmio` 通过；`cargo test -p cozmio` 58/58 通过。未启动 `tauri dev` 做人工截图验证。
