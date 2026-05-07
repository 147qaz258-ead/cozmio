# Desktop UI Commercialization — Implementation Plan

> **智能执行体须知**：使用 `superpowers:subagent-driven-development` 调度子智能体逐任务落地。步骤使用复选框（`- [ ]`）语法跟踪。

**目标**：商业化改造 Cozmio 桌面端 UI，修复 MiniDot 状态同步 + 重构 History 为会话流 + 补充 Practice 入口

**设计文档**：`docs/superpowers/specs/2026-05-06-desktop-ui-commercialization-design.md`

---

## Flywheel Bootstrap

- `claude-progress.txt`: 存在 → 更新为当前任务
- `feature_list.json`: 不存在 → 跳过
- `verification/last_result.json`: 不存在 → 将在首次验证后创建

---

## Current Truth

### 已检查文件

| 文件 | 行 | 当前实现 |
|------|-----|---------|
| `cozmio/src-tauri/src/commands.rs` | ~1023 | `emit_state_update()` 构建 StateUpdate 并 emit；调用 `compute_mini_state()` 更新 tray icon |
| `cozmio/src-tauri/src/commands.rs` | ~1036 | `build_state_update()` 从 AppState 读取状态字段 |
| `cozmio/src-tauri/src/commands.rs` | ~1063 | `compute_mini_state()` 基于 relay_status/tray_state/running_state 计算 visual state |
| `cozmio/src-tauri/src/components/MiniDot.js` | ~110 | `agentVisualState()` 优先级：pending > failed > done > active > processing > running > offline > idle |
| `cozmio/src-tauri/src/components/MiniDot.js` | ~180 | `init()` 监听 `state-update` 事件并调用 `renderMiniWorkstation()` |
| `cozmio/src-tauri/src/components/MiniDot.js` | ~48 | `buildExecutionPresence()` 从 `running_state`/`tray_state` 计算 targets |
| `cozmio/src-tauri/src/components/HistoryList.js` | ~11 | `createHistoryList()` 返回只有列表+详情的两段式结构 |
| `cozmio/src-tauri/src/components/HistoryList.js` | ~69 | `loadHistory()` 调用 `get_history` 返回 flat record 列表 |
| `cozmio/src-tauri/src/components/App.js` | ~40 | 侧边栏只有 3 个 nav item：status / history / config |
| `cozmio/src-tauri/src/mini.html` | ~41 | `.mini-agent-img` 尺寸 76x76，position absolute left:-7px top:-6px |

### 已知不一致

1. **MiniDot vs tray icon 状态源不同**：后端 `compute_mini_state` 基于 `StateUpdate`（来自 AppState），MiniDot 的 `agentVisualState` 基于前端接收的 event payload，逻辑基本一致但无统一状态源
2. **App.js header dot 和 MiniDot 可能不同步**：App.js 用 `buildExecutionPresence(presence)` 计算 overall_status，但 MiniDot 用 `agentVisualState()` 直接算 visual state
3. **History 无 session 概念**：现有 `get_history` 返回 flat 记录，无会话分组概念

---

## Implementation Shape

### RP-1: MiniDot 状态同步修复

**文件**: `cozmio/src-tauri/src/components/MiniDot.js:110`

**当前真相**:
```javascript
// agentVisualState at ~110
function agentVisualState(state, presence) {
  if (state.visual_state_override) return state.visual_state_override;
  const relayExecution = state.relay_execution || state.relayExecution || null;
  const relayStatus = relayExecution && relayExecution.relay_status || '';
  const runningState = state.running_state || state.agentState || 'Stopped';
  const trayState = state.tray_state || '';
  if (state.pending_confirmation || state.pendingConfirmation) return 'confirm';
  if (FAILED_RELAY_STATUSES.has(relayStatus) || ATTENTION_RELAY_STATUSES.has(relayStatus)) return 'error';
  if (DONE_RELAY_STATUSES.has(relayStatus) && relayStatus !== 'interrupted') return 'done';
  if (ACTIVE_RELAY_STATUSES.has(relayStatus)) return 'executing';
  if (trayState === 'processing') return 'analyzing';
  if (runningState === 'Running') return 'monitoring';
  if (presence.overall_status === 'offline') return 'offline';
  return 'idle';
}
```

**问题**：后端 `compute_mini_state` 在 `commands.rs:1063` 用的是同一套逻辑但各自独立实现。暂停时 `running_state=Stopped` 应返回 `idle`，但 MiniDot 在 `buildExecutionPresence` 的 monitor target 中用了 `runningState === 'Running' ? 'online' : 'offline'`。两处对 `Stopped` 的处理不一致。

**修改为**:
```javascript
function agentVisualState(state, presence) {
  if (state.visual_state_override) return state.visual_state_override;
  const relayExecution = state.relay_execution || state.relayExecution || null;
  const relayStatus = relayExecution && relayExecution.relay_status || '';
  const runningState = state.running_state || state.agentState || 'Stopped';
  const trayState = state.tray_state || '';

  // Priority: confirm > error > done > executing > analyzing > monitoring > idle
  if (state.pending_confirmation || state.pendingConfirmation) return 'confirm';
  if (FAILED_RELAY_STATUSES.has(relayStatus) || ATTENTION_RELAY_STATUSES.has(relayStatus)) return 'error';
  if (DONE_RELAY_STATUSES.has(relayStatus) && relayStatus !== 'interrupted') return 'done';
  if (ACTIVE_RELAY_STATUSES.has(relayStatus)) return 'executing';
  if (trayState === 'processing') return 'analyzing';
  if (runningState === 'Running') return 'monitoring';
  if (runningState === 'Stopped') return 'idle';  // 明确 Stopped → idle，不再走 presence 判断
  return 'idle';
}
```

同步修改 `buildExecutionPresence` 中 monitor target 的 status 判断：
```javascript
// at ~66-69，原有：
status: trayState === 'processing' ? 'working' : runningState === 'Running' ? 'online' : 'offline',
// 改为：
status: trayState === 'processing' ? 'working' : runningState === 'Running' ? 'online' : 'unknown',  // offline 仅在 truly disconnected 时用
```

### RP-2: History 重构为会话流

**文件**: `cozmio/src-tauri/src/components/HistoryList.js:11`

**当前真相**:
```javascript
// createHistoryList 返回两段式：列表 + 详情卡片
// loadHistory 调用 get_history 返回 flat records
```

**修改为**：保留 `createHistoryList` 接口不变，内部改为：
1. 调用新命令 `get_execution_sessions(limit: 20)` 获取会话列表
2. 每个 session 显示：时间、任务摘要、状态 badge、progress 条数
3. 点击 session 展开详情，显示该 session 下的 progress 事件滚动流
4. fallback：若 `get_execution_sessions` 失败（后端未实现），降级到现有 `get_history`

**新增命令（后端）**: `cozmio/src-tauri/src/commands.rs` 新增 `get_execution_sessions` 和 `get_session_progress`

### RP-3: MiniDot 图片遮挡修复

**文件**: `cozmio/src-tauri/src/mini.html:41`

**当前真相**:
```css
.mini-agent-img {
  width: 76px;
  height: 76px;
  object-fit: contain;
  position: absolute;
  left: -7px;
  top: -6px;
  ...
}
```

**问题**：图片 76x76 超出 .mini-scene（62x62），且 absolute positioning 容易超出容器边界导致裁切

**修改为**:
```css
.mini-agent-img {
  width: 60px;
  height: 60px;
  object-fit: contain;
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  ...
}
```

### RP-4: 已废弃

**原因**：计划中"侧边栏只有 3 个 nav item"的描述与实际不符。实际 App.js 已有 4 个 nav item（STATUS/HISTORY/MEMORY/CONFIG），MEMORY tab 已存在且对应 MemoryInspector.js。

Memory Flywheel 的 Workstream G 正在实现 Memory Inspector 功能（reject/supersede/apply hot context/run consolidation/replay），与 RP-4 原定目标重叠。RP-4 不再实施。

---

## 后端新增命令（RP-2 配套）

### get_execution_sessions

**文件**: `cozmio/src-tauri/src/commands.rs`

在 `AppState` 已有 `ledger: ActionLogger`（logging.rs）。查询 `event_type` 包含 `EXECUTION_DISPATCHED` / `EXECUTION_PROGRESS` / `EXECUTION_COMPLETED` 的记录，按 `session_id` 分组返回会话列表。

**签名**:
```rust
#[tauri::command]
pub fn get_execution_sessions(limit: Option<u32>) -> Result<Vec<ExecutionSession>, String>;
```

**ExecutionSession 结构**:
```rust
struct ExecutionSession {
    session_id: String,
    trace_id: String,
    started_at: i64,
    ended_at: Option<i64>,
    task_summary: String,
    status: String,  // pending / running / completed / failed
    progress_count: u32,
    result_summary: Option<String>,
}
```

### get_session_progress

```rust
#[tauri::command]
pub fn get_session_progress(session_id: String) -> Result<Vec<ProgressEvent>, String>;
```

---

## Key Path Tracing

**MiniDot 状态同步路径**：
```
Backend: AppState.tray_state / AppState.running_state
  → emit_state_update() at main_loop.rs:102/133/146/...
    → build_state_update() combines AppState fields into StateUpdate
      → app.emit("state-update", StateUpdate) at commands.rs:1027

Frontend (MiniDot):
  → listen('state-update') at MiniDot.js:180
    → renderMiniWorkstation(event.payload) at MiniDot.js:181
      → buildExecutionPresence(payload) at MiniDot.js:131
      → agentVisualState(payload, presence) at MiniDot.js:134
        → visualState = 'idle'|'monitoring'|'analyzing'|etc
          → imageSrc = `./assets/agent-states/agent-${visualState}.png`
```

**问题**：main_loop 每轮循环都 emit，但 `start_running` / `stop_running` 命令执行后是否立即 emit 了 state-update？需要验证命令处理路径。

**验证路径**：命令 `start_running` / `stop_running` → AppState.running_state 变更 → main_loop 下次循环时 emit（或命令内直接 emit）

---

## Risk → Detection Mapping

| Risk | 验证方式 | 预期结果 |
|------|---------|---------|
| MiniDot 暂停后状态不更新 | 人肉：点击暂停 → 观察小人图 3 秒内变为 idle | 小人从 monitoring 变为 idle |
| History 重构后后端接口未实现 | 编译 + 接口调用 | `get_execution_sessions` 返回空列表时降级不报错 |
| MiniDot 图片裁切未修复 | 人肉：打开 mini window 观察 | 图片完整显示在容器内 |

---

## 口子词扫描

- `需确认` / `待定` / `TBD` / `复用或重定义` — 无
- `大概是` / `应该在` — 无
- `探索` / `研究` — 无（在实现步骤中）

---

## 自我审查清单

### A. 执行者是否还需要判断？

- [ ] MiniDot visualState 计算逻辑已明确，暂停时走 `idle` 分支
- [ ] History 降级逻辑已写明（fallback 到 get_history）
- [ ] Practice 入口显示内容已明确（记忆收件箱 + 模型评估）

### B. 真相检查

- [ ] `agentVisualState` 函数位置确认：MiniDot.js:110
- [ ] `buildExecutionPresence` monitor target 位置确认：MiniDot.js:66-69
- [ ] `mini-agent-img` CSS 位置确认：mini.html:41
- [ ] `get_execution_sessions` 签名已基于现有 `get_history` 接口模式

### C. 关键路径检查

- [ ] MiniDot init → listen('state-update') → renderMiniWorkstation 已追踪
- [ ] 后端 emit_state_update 调用链已追踪
- [ ] RP-2 的 `get_execution_sessions` 命令签名已明确

### D. 验证检查

- [ ] 每个 RP 都有具体验证方式（人肉/编译）
- [ ] Risk → Detection 映射完整

### E. 口子词扫描

- [ ] 无模糊表述

### F. 冻结检查

- [ ] 所有 RP 状态 = "已锁定 ✓"
- [ ] 无探索性动词

---

## 执行顺序

1. **RP-1** MiniDot 状态同步修复（前端 JS 修改，不涉及后端）
2. **RP-3** MiniDot 图片遮挡修复（CSS 修改）
3. **RP-2** History 重构（前端 + 后端新命令）
4. **RP-4** 已废弃 → 由 Memory Flywheel Workstream G 覆盖

---

**状态**: 已锁定 ✓
**保存路径**: `docs/superpowers/plans/2026-05-07-desktop-ui-commercialization-plan.md`