# 执行端存活工位设计文档

**日期**: 2026-04-28
**类型**: 桌面端 UI 重构 / 执行端存活感知
**状态**: 已由用户确认进入“设计 -> 计划 -> 执行”流程

---

## 1. 背景与核心问题

当前 Cozmio 已经具备桌面观察、确认、Relay 派发、Claude Code 执行链路，但用户无法稳定判断执行端是否仍然活着。之前的 mini dot 方案只把黑色迷你窗缩小成状态灯，无法回答核心问题：

> Cozmio、Relay、执行端、当前任务到底有没有活着？

现有设计文档中已经规划过 Relay Engine 的 `AgentRegistry` 和“代理心跳”，但桌面端尚未把这些状态产品化。主界面仍偏技术日志，悬浮窗也无法表达多个执行端在线状态。

---

## 2. 产品目标

1. 用户一眼知道系统是否活着：桌面端、监控循环、Relay、执行端、当前 session。
2. 用户一眼知道哪个执行端在岗：当前先支持 `Claude Code`，未来可扩展 Browser Agent、Remote Runner、Local Box Worker。
3. 用户一眼知道任务是否仍在推进：执行中、等待中、卡住、完成、失败。
4. 视觉上回到“智能体工位”隐喻：像素小人/工位灯表达生命体征，而不是抽象圆点。

---

## 3. 产品类型

- type: `traditional_software`
- core risk: 状态语义错误、UI 误导在线性、悬浮窗占用/裁切问题、未来多执行端扩展困难
- verification style: `cargo build` + UI 结构检查 + 运行时人工截图验证

---

## 4. 状态模型

新增前端可消费的执行端 presence model，由现有运行状态与 Relay 状态推导，不阻塞 H1 实现真实 AgentRegistry 心跳。

```text
ExecutionPresence
├── overall_status: online | working | attention | degraded | offline
├── summary: 用户可读摘要
├── targets: ExecutionTarget[]
└── updated_at: unix seconds

ExecutionTarget
├── id: desktop-host | monitor-loop | relay-engine | claude-code
├── label: 桌面宿主 / 观察循环 / Relay Engine / Claude Code
├── kind: host | monitor | relay | executor
├── status: online | working | attention | degraded | offline | unknown
├── detail: 用户可读状态说明
├── heartbeat_age_secs: number | null
├── session_id: string | null
└── task: string | null
```

H1 采用派生心跳：
- desktop-host: 只要 UI 能收到 `state-update`，视为 online。
- monitor-loop: `running_state=Running` 为 online；Stopped 为 offline。
- relay-engine: 有活动 `relay_execution` 时按 relay 状态推导；没有活动 session 时先显示 unknown/standby。
- claude-code: 有 `session_id` 且 relay 状态为 running/waiting/dispatching 时显示 working；completed 为 online；failed/error 为 degraded。

未来 H2 接入真实心跳：Relay Engine 暴露 `agents/list` 或 heartbeat IPC 后，替换派生逻辑。

---

## 5. 主界面设计

主界面 STATUS 首页从“技术模块卡”升级为“执行端工位总览”。技术详情仍保留在下方，避免丢失调试能力。

首屏结构：

```text
WORKSTATION
┌────────────────────────────────────────────┐
│  像素工位总览                              │
│  [像素小人]  系统在线 / 正在执行 / 需要确认 │
│  Cozmio 正在观察，Relay 已连接...           │
└────────────────────────────────────────────┘

EXECUTION PRESENCE
┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐
│ 桌面宿主   │ │ 观察循环   │ │ Relay      │ │ Claude Code│
│ online     │ │ online     │ │ working    │ │ working    │
│ heartbeat  │ │ 3s poll    │ │ session... │ │ task...    │
└────────────┘ └────────────┘ └────────────┘ └────────────┘

CURRENT HANDOFF
- 当前任务 / 待确认 / 最近进展

DETAILS
- 原 LAST JUDGMENT / RELAY SESSION / PROGRESS / RESULT
```

视觉语言：
- 像素小人作为状态图标，使用 CSS 像素块实现 H1，避免先引入不可控图片资产。
- 使用暖纸色主 UI 体系，不走黑底终端面板。
- 状态颜色语义统一：绿 online，蓝 working，橙 attention，红 degraded/offline，灰 unknown。

---

## 6. 悬浮窗设计

恢复独立 mini window，但不使用 24×24 dot。窗口尺寸改为 `196×148`，避免 CSS hover 展开被原生窗口裁切。

默认状态即显示一个小工位：

```text
┌──────────────────────┐
│ [像素小人+桌子]  3/4 │
│ Claude Code working  │
│ Relay session active │
└──────────────────────┘
```

交互：
- 悬浮窗始终显示在线总览，不依赖 hover 才可读。
- hover 时显示快捷动作：开始/暂停、确认、中断。
- 多执行端时优先显示 attention/degraded，其次 working，其次 online。

---

## 7. H1 范围

### In

- 新增前端 presence 计算模块，统一主界面和 mini 窗口状态语义。
- 主界面 STATUS 首页新增工位总览和执行端卡片。
- 重写 mini.html/MiniDot.js 为像素工位悬浮窗。
- 恢复 `create_mini_window()`，尺寸改为固定可读小窗。
- 保留现有 commands 与 relay dispatch 逻辑，不改执行链路。

### Out

- Relay Engine 真实 AgentRegistry heartbeat API。
- 多进程执行端真实发现。
- AI 生成 PNG sprite 资产接入。
- 拖动悬浮窗。

---

## 8. 验收标准

- `cargo build -p cozmio` 通过。
- 主界面 STATUS 首屏出现 Execution Presence 总览，而不是只有技术模块卡。
- 悬浮窗启动时可见，显示像素工位和执行端状态，不是 24×24 圆点。
- 状态推导不误导：没有 Relay session 时不显示 Claude Code working。
- 当前任务/Relay progress 仍可在主界面查看。
