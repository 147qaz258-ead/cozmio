# Cozmio Desktop UI 商业化改造设计文档

**日期**: 2026-05-06
**类型**: 桌面端 UI 重构 / 产品化改造
**状态**: 待批准
**设计师**: Claude (作为设计师角色)

---

## 1. 产品定位

### 1.1 当前产品形态

Cozmio 是一个**主动智能体调度器**：
- 观察用户屏幕 → 理解上下文 → 派发任务给执行端（Claude Code）
- 用户确认后，执行端自动完成任务

### 1.2 当前 UI 问题

用户描述的问题：
1. **桌面精灵状态不同步** - 暂停后小人状态不变
2. **小人图片遮挡** - 悬浮窗的图片被裁切
3. **执行端日志体验差** - 只有两个框，用户看不懂 agent 在干嘛
4. **Practice 页面 5 个 tab 不知道干嘛用** - 开发者调试界面，不是产品界面
5. **整体没有产品感** - 看起来像"调试面板"，不是"商业化工具"

### 1.3 目标产品形态

> **AI 操作员控制台**

用户一眼知道：
1. AI 在干什么
2. AI 发现了什么
3. 我现在可以做什么

---

## 2. 设计原则

### 2.1 核心原则：展示用户价值，不是系统结构

用户不关心：`Relay Engine` / `Claude Code` / `interval 3s`

用户只关心：
- AI 是不是在帮我做事
- 它现在发现了什么
- 我可以做什么

### 2.2 视觉原则

| 当前问题 | 改进方向 |
|---------|---------|
| 信息平铺，无主次 | 分层视觉权重，重点突出 |
| 卡片像日志面板 | 卡片像"控制台部件"，有整体感 |
| 关键动作不突出 | 主按钮显眼，次要信息收起 |
| 文本区域像调试输出 | 结构化展示，有结论/建议/行动 |

### 2.3 状态颜色语义

- **绿** = 正常 / 完成
- **蓝** = AI 行为 / 进行中
- **橙** = 等待用户 / 警告
- **红** = 错误 / 异常

---

## 3. 页面结构规划

### 3.1 现有页面 vs 目标形态

| 现有页面 | 问题 | 目标形态 |
|---------|------|---------|
| STATUS（主页） | 形态还行 | 保持不动 |
| HISTORY | 信息展示形式传统 | 重构为会话流 |
| CONFIG | 配置功能，正常 | 保持 |
| PRACTICE | 5个tab不知道干嘛用 | 精简为 2-3 个明确入口 |
| MiniDot 悬浮窗 | 状态不同步 + 图片遮挡 | 修复同步 + 改尺寸/布局 |

### 3.2 侧边栏结构（保持 4 个，但重新定义）

```
[侧边栏]
STATUS      → 保持（工位总览）
HISTORY     → 重构为「执行记录」会话流
CONFIG      → 保持
PRACTICE    → 重构为「记忆管理」/「评估」（精简）
```

---

## 4. 模块设计

### 4.1 MiniDot 悬浮窗修复

**问题 1：状态不同步**

原因分析：
- `state-update` 事件没有正确传到 mini window
- 或者 `agentVisualState()` 计算逻辑与实际状态不一致

修复方案：
```
状态计算优先级：
1. pending_confirmation 非空 → confirm
2. relay_status ∈ {failed, error, dispatch_error} → error
3. relay_status = completed → done
4. relay_status ∈ {running, waiting, dispatching, connecting} → executing
5. tray_state = processing → analyzing
6. running_state = Running → monitoring
7. running_state = Stopped → idle
```

**问题 2：小人图片遮挡**

原因分析：
- mini.html 尺寸 224x188 可能被原生窗口裁切
- CSS hover 展开超出窗口边界

修复方案：
- 固定窗口尺寸为 `196×160`（避免裁切）
- 移除 hover 展开动画，默认展示全部信息
- 图片使用 `object-fit: contain` 避免变形

**目标形态**：
```
┌──────────────────────┐
│ [小人图]  状态标签   │  ← 固定显示，不依赖 hover
│ 当前任务：xxxxx      │
│ [快捷按钮组]        │
└──────────────────────┘
```

### 4.2 执行记录会话流（重构 HISTORY）

**用户需求**：像聊天助手一样实时看 agent 做了什么，建立信任感

**Primary User Object**：`execution_session`（执行会话）

**用户可见流程**：
1. 用户点击 HISTORY tab
2. 看到执行会话列表（时间倒序）
3. 点击一个会话，展开详情（实时滚动 progress）
4. 会话结束后，显示总结

**UI States 定义**：

| State | 页面展示 | 主要按钮 | 用户下一步 |
|-------|---------|---------|-----------|
| empty | "还没有执行记录" | 开始监控 | 去 STATUS 启动 |
| pending | "等待派发中..." + 任务摘要 | 取消 | 可取消 |
| running | 实时滚动 progress 流 | 中断 | 可中断 |
| completed | 绿色完成标记 + 总结 | 查看详情 | 展开 trace |
| failed | 红色错误标记 + 原因 | 重试 / 查看 | 可重试 |

**设计要点**：
- 每次 relay dispatch 生成一个会话卡片
- progress 事件实时 append 到卡片底部（像聊天消息）
- 会话列表只显示最新 20 个，更多归档
- 每个会话显示：时间、任务摘要（截断）、状态、progress 条数

**禁止出现**：
- `relay_status: running` 这样的技术术语
- 原始 JSON 内容
- 技术性 trace_id

### 4.3 Practice 页面精简

**用户需求**：记忆管理和评估，但不知道现在这些功能干嘛用

**问题分析**：
- 5 个 tab（Timeline/Inbox/Preview/Signals/Evaluation）全是开发者调试视角
- 用户看到的是"功能"，不是"价值"

**重构方案**：精简为 2 个明确入口

```
┌─────────────────────────────────────┐
│ PRACTICE LOOP                       │
│                                     │
│ ┌─────────────────────────────┐    │
│ │ 📥 记忆收件箱                │    │  ← 原来 Inbox，用户能理解"收件箱"
│ │ 3 条待处理记忆               │    │
│ │ [查看]                      │    │
│ └─────────────────────────────┘    │
│                                     │
│ ┌─────────────────────────────┐    │
│ │ 📊 模型评估                  │    │  ← 原来 Evaluation
│ │ 样本质量 / 评分 / 反馈       │    │
│ │ [查看]                      │    │
│ └─────────────────────────────┘    │
│                                     │
│ ┌─────────────────────────────┐    │
│ │ 📈 信号看板                 │    │  ← 原来 Signals（可选收起）
│ │ [展开]                      │    │
│ └─────────────────────────────┘    │
└─────────────────────────────────────┘
```

**Timeline 和 Preview 暂时收起**：
- Timeline → 可以从会话流的 trace 入口进入
- Preview → 开发者用，用户不需要

### 4.4 主页面 STATUS（保持不动）

用户确认形态还行，只做小修复：

- 确保 header-state-dot 与 MiniDot 状态同步
- 关键动作按钮（启动/暂停/确认）视觉突出
- 不改变现有的模块卡结构

---

## 5. 技术实现要点

### 5.1 MiniDot 同步修复

**关键代码位置**：
- `MiniDot.js` 的 `agentVisualState()` 函数
- `App.js` 的 `initHeaderDotState()` 函数

**修复检查点**：
```javascript
// 状态计算必须与 tray.rs 的 update_tray_icon 一致
// 暂停时：running_state=Stopped → idle（不是保持之前状态）
```

### 5.2 执行记录数据结构

```typescript
ExecutionSession {
  id: string;              // relay session_id
  trace_id: string;        // cozmio trace_id
  started_at: number;      // unix timestamp
  ended_at: number | null;
  task_summary: string;    // 任务摘要（用户可读）
  status: 'pending' | 'running' | 'completed' | 'failed';
  progress_count: number;  // progress 事件数量
  result_summary: string | null;
}
```

### 5.3 Session 列表查询

```rust
// commands.rs 新增
get_execution_sessions(limit: 20) -> Vec<ExecutionSession>
get_session_progress(session_id: string) -> Vec<ProgressEvent>
```

---

## 6. 设计验收标准

### 6.1 MiniDot 验收
- [ ] 暂停监控后，小人状态 3 秒内从 monitoring 变为 idle
- [ ] 悬浮窗不会被裁切，图片完整显示
- [ ] 运行中时显示当前任务摘要（不是"待机"）

### 6.2 执行记录验收
- [ ] 打开 HISTORY tab，一眼看到"有没有执行记录"
- [ ] 正在执行时，progress 实时滚动显示（像聊天消息）
- [ ] 会话结束显示总结，用户能回答"做了什么"
- [ ] 没有技术术语（relay_status 等）

### 6.3 Practice 验收
- [ ] 用户看到"记忆收件箱"能理解是干嘛的
- [ ] 用户看到"模型评估"能理解是干嘛的
- [ ] 每个入口都有明确价值说明

### 6.4 整体验收
- [ ] 第一次打开的人 3 秒内能回答：这东西在干嘛、发现什么、可以做什么
- [ ] 视觉有层次感，不是信息堆在一起的调试面板

---

## 7. 设计问题记录

| # | 问题描述 | 严重程度 | 建议修复 |
|---|---------|---------|---------|
| 1 | MiniDot 状态同步问题是 event 传播问题还是 state 计算问题，需进一步验证 | BLOCKER | 需要 runtime 测试确认根因 |
| 2 | 执行记录是否复用现有 Ledger 数据，还是需要新表 | 需要确认 | 复用 ledger.event_type=EXECUTION_* 的事件 |
| 3 | History tab 重构后，原有的 history 命令是否废弃 | 需要确认 | 保持兼容，或者合并到新接口 |

---

## 8. 下一步

设计文档批准后：
1. 调用 `writing-plans` 生成实施方案
2. 实施方案包含：MiniDot 修复 → 会话流重构 → Practice 精简
3. 每步验证后再继续