# Mini Dot Indicator — 悬浮小圆点替代丑黑框

> **智能执行体须知**：本方案以运行效果、验证资产和飞轮写回为中心。

## 1. Flywheel Context

- active task: DIGITAL-WORKSTATION H3 完成，H4（E2E）待执行
- current phase: 设计替换丑黑框 mini window 为小圆点悬浮指示器
- latest verification: H3 cargo build PASS，4 tests PASS（2026-04-28）
- blocker: 用户反馈 mini window 太丑（全黑、大、无法交互）
- next expected step: 写 plan → 实现小圆点 → 更新飞轮

## 2. Goal

将 280x360 全黑无法交互的 mini window 替换为 **24x24 悬浮小圆点**，hover 展开显示状态 + 快捷按钮。视觉简洁、不占空间。

## 3. Product Type

- type: `desktop_ui_runtime`
- core risk: 用户体验可见性、hover 交互流畅性、状态同步正确性
- verification style: 截图 + 人工可见路径

## 4. Global Roadmap

| Phase | 目标 | 依赖 | 验收意图 |
|-------|------|------|---------|
| H1 | 小圆点 + hover 展开 | — | dot 显示在角落，hover 展开状态，cargo build 通过 |
| H2 | 集成到主应用状态栏 | H1 | 主应用内可见 dot，无需独立窗口 |
| H3 | 快捷操作（confirm/interrupt/toggle）| H1 | hover 展开后可点击操作按钮 |

## 5. Scope

### In（本次包含）

- mini_window.rs 重写：创建 24x24 dot 而非 280x360 窗口
- mini.html 重写：纯 CSS hover 展开（180x120 左右），无 JS 动画依赖
- 保持 state-update 订阅机制不变
- 保持 tray icon 联动不变

### Out（本次不包含）

- 主应用内嵌 dot（H2）
- 多 session 并行显示（H2/H3）
- 拖动功能（H3 如果需要）
- 原有 280x360 黑色面板样式（彻底废弃）

## 6. Current Truth

- `mini_window.rs`：创建 280x360 borderless always-on-top webview，内容为 mini.html
- `mini.html`：#0d0d0f 全黑背景，280x360，MiniPanel.js 订阅 state-update
- `MiniPanel.js`：computeMiniState() 计算状态，renderMiniPanel() 渲染完整面板
- `tray.rs`：4个图标 green/blue/orange/red，update_tray_icon() 根据状态切换
- `main.rs:240`：启动时调用 create_mini_window()

## 7. Implementation Steps

### H1 实现步骤

**Step 1 — mini_window.rs 重写**
- 将窗口尺寸从 280x360 改为 24x24（内嵌 webview 实际大小，展开用 CSS）
- 窗口位置：bottom-right corner，decorations(false)，always_on_top(true)，shadow(false)
- 背景色透明：.html background: transparent，body background: transparent
- webview URL 仍为 mini.html

**Step 2 — mini.html + MiniPanel.js 重构为 dot 样式**
- body 尺寸 24x24，border-radius 50%，overflow: hidden
- dot 显示：用纯 CSS circle（border-radius: 50%）显示当前状态色
- hover 展开：在 body 上使用 `:hover + .expand-card` 或兄弟选择器，显示一个小的卡片（180x120）覆盖在 dot 周围
- 展开卡片内容：状态标签 + 当前窗口标题（截断）+ 2个快捷按钮（确认/中断/开关）
- 快捷按钮调用 mini_action（复用现有逻辑）
- 状态色通过 `--dot-color` CSS 变量从 MiniPanel.js 设置

**Step 3 — MiniPanel.js 适配**
- renderMiniPanel() 改为只渲染 dot + 可选展开卡片
- dot 颜色通过 `--dot-color` CSS 变量注入
- 初始渲染直接设置 body.style.setProperty('--dot-color', color)
- 订阅 state-update 时更新 CSS 变量

**Step 4 — build 验证**
- `cargo build -p cozmio` 必须通过
- 启动 tauri dev，肉眼验证 dot 可见、hover 展开

## 8. Verification Asset

- verification type: `desktop_ui_runtime`
- command: `cargo build -p cozmio`
- expected evidence: 编译通过 + 截图显示右下角小圆点 + hover 展开显示状态卡片
- evidence location: 截图保存到 `verification/`
- failure condition: 编译失败 or dot 不可见
- writeback targets:
  - `verification/last_result.json`
  - `feature_list.json`
  - `claude-progress.txt`

## 9. Phase Gate

- [ ] `cargo build -p cozmio` 通过
- [ ] dot 在右下角可见（24x24 彩色圆点）
- [ ] hover dot 显示展开卡片（状态 + 按钮）
- [ ] 截图证据已保存到 verification/
- [ ] `verification/last_result.json` 已更新
- [ ] `feature_list.json` DIGITAL-WORKSTATION 已更新状态
- [ ] `claude-progress.txt` 已更新 H1 完成状态

## 10. Next Execution Step

- next phase: H1 实现（dot + hover 展开）
- entry skill: `superpowers:subagent-driven-development`
- stop condition: Phase Gate 全部满足
