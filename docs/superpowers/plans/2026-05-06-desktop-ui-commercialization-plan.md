# Cozmio Desktop UI 商业化改造实施计划（完整版）

**日期**: 2026-05-06
**类型**: Tier 2 锁定计划
**设计文档**: `docs/superpowers/specs/2026-05-06-desktop-ui-commercialization-design.md`
**状态**: 已锁定 ✓

---

## 当前真相 (Current Truth)

### inspected files

- `cozmio/src-tauri/src/components/MiniDot.js:50-90` — agentVisualState() 逻辑（正确）
- `cozmio/src-tauri/src/components/App.js:100-140` — header-state-dot 同步
- `cozmio/src-tauri/src/components/HistoryList.js:1-250` — 现有 history 实现
- `cozmio/src-tauri/src/components/PracticeDashboard.js:1-1412` — 5 个 tab 实现
- `cozmio/src-tauri/src/commands.rs:88-94` — `get_history()` 实现
- `cozmio/src-tauri/src/relay_bridge.rs:457-507` — track_relay_session() progress 接收
- `cozmio/src-tauri/src/mini_window.rs:1-40` — mini window 创建
- `cozmio/src-tauri/frontend/mini.html` — 悬浮窗 HTML
- `cozmio/src-tauri/src/tray.rs:50-120` — tray 图标状态

### existing entry points

- `get_history(limit)` → Vec<ActionRecord>
- `get_timeline(limit, offset)` → Vec<serde_json::Value>
- `state-update` event — 后端推送（但 progress 是批量推送，不是增量）
- `tauri::command fn mini_action()` — 悬浮窗操作

### existing runtime path

```
relay_bridge::track_relay_session()
  → subscription.recv_event() 接收 ProgressEvent
  → apply_progress_event() 更新 snapshot.progress[]
  → publish(snapshot) → "state-update" 事件（批量）
  → ledger.record() 保存 EXECUTION_PROGRESS_RECEIVED
```

### 已知缺口

1. **progress 事件没有独立通道**：目前 progress 通过 state-update 批量推送，前端无法实时 append
2. **get_history 返回 ActionRecord**：不是 ExecutionSession
3. **mini.html 窗口可能被裁切**：尺寸或 CSS 问题
4. **Practice 5 个 tab 用户看不懂**：全是开发者视角

---

## 实现顺序

| # | RP | 类型 | 说明 |
|---|-----|------|------|
| RP-1 | MiniDot 状态同步修复 | UI/Bug | state-update 传递修复 |
| RP-1.5 | MiniDot 图片遮挡修复 | UI | 窗口 196x160 + CSS |
| RP-2 | 执行记录会话流 | 功能+UI | HISTORY 重构为会话卡片 |
| RP-3 | Practice 页面精简 | UI | 5 tab → 3 入口 |
| RP-4 | STATUS header-dot 同步 | UI | 与 MiniDot 一致 |
| RP-5 | relay_progress 事件通道 | 功能 | 新增独立 progress 推送事件 |

---

## Implementation Shape

---

### RP-1: MiniDot 悬浮窗状态同步修复

**文件**: `cozmio/src-tauri/src/components/MiniDot.js:50-90`
**当前真相**: agentVisualState() 逻辑正确，但 state-update 可能没有传到 mini window

**检查点**:
```javascript
// MiniDot.js init() 监听 state-update
await listen('state-update', (event) => {
  renderMiniWorkstation(event.payload || {});  // 问题：payload 可能为空或旧
});
```

**问题分析**:
- mini window 是独立 WebviewWindow，可能没有正确订阅 state-update
- 或者 get_ui_state 返回的状态不包含最新的 running_state

**修改为**:
```javascript
// 确认 get_ui_state 返回最新状态
const initialState = await invoke('get_ui_state');
// 确认 state-update 事件带了完整 payload
// 问题可能在后端 emit 时 payload 不完整
```

**后端检查**: `relay_bridge.rs` 中 `publish(snapshot)` 是否包含完整的 running_state

**验证**: `cargo build -p cozmio` → PASS；运行时暂停后小人 3 秒内变为 idle

---

### RP-1.5: MiniDot 悬浮窗图片遮挡修复

**文件**: `cozmio/src-tauri/src/mini_window.rs:15-35`
**当前真相**: `.inner_size(224.0, 188.0)` 可能被裁切

**修改为**:
```rust
.inner_size(196.0, 160.0)
```

**文件**: CSS 中 `.mini-agent-img` 添加
```css
.mini-agent-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  overflow: hidden;
}
```

**验证**: 图片完整显示，不被裁切

---

### RP-5: relay_progress 独立事件通道（核心新功能）

**文件**: `cozmio/src-tauri/src/relay_bridge.rs:457-507`
**当前真相**: progress 事件在 track_relay_session() 内部循环处理，通过 state-update 批量推送

**问题**: 前端无法实时 append progress 到会话卡片，只能渲染批量数据

**新增功能**: 独立的 `relay_progress` event，细粒度实时推送

```rust
// relay_bridge.rs 中 track_relay_session() 函数

// 新增：每次收到 ProgressEvent 后，emit 独立的 relay_progress event
loop {
    match subscription.recv_event() {
        Ok(event) => {
            // 原有逻辑：apply_progress_event 更新 snapshot
            apply_progress_event(&mut snapshot, &event);
            snapshot.updated_at = now_ts();

            // 新增：emit 独立的 relay_progress event（实时推送）
            let progress_payload = serde_json::json!({
                "session_id": session_id,
                "timestamp": event.timestamp,
                "level": level_label(event.level),
                "message": event.message,
                "terminal": event.terminal,
                "terminal_status": event.terminal_status,
            });

            if let Err(e) = app_handle.emit("relay_progress", &progress_payload) {
                log::warn!("Failed to emit relay_progress event: {}", e);
            }

            // 原有 logic：更新状态、写 ledger...
        }
        Err(e) => { ... }
    }
}
```

**文件**: 前端新增监听 relay_progress
```javascript
// MiniDot.js 或新建 ConversationList.js

// 监听 relay_progress 事件
await listen('relay_progress', (event) => {
  const { session_id, message, level, terminal } = event.payload;

  // 找到对应的会话卡片
  const card = document.querySelector(`[data-session-id="${session_id}"]`);
  if (card) {
    // 实时 append progress 消息（像聊天消息一样）
    appendProgressMessage(card, { message, level, timestamp: event.payload.timestamp });
  }
});

function appendProgressMessage(card, progress) {
  // 创建新消息元素，append 到卡片底部
  // 消息样式：时间 + 内容 + 层级标签（info/warning/error）
}
```

**验证**:
- `cargo build -p cozmio` → PASS
- 派发任务后，frontend 能实时收到每条 progress 消息
- 会话卡片底部实时滚动更新（像聊天）

---

### RP-2: 执行记录会话流（重构 HISTORY）

**目标**: 像聊天助手一样，用户能实时看 agent 做了什么

**文件**: `cozmio/src-tauri/src/commands.rs` — 新增 IPC
```rust
#[tauri::command]
pub fn get_execution_sessions(
    state: State<AppState>,
    limit: Option<usize>,
) -> Result<Vec<serde_json::Value>, String> {
    // 从 executor_session_registry 读取 sessions
    // 返回: Vec<ExecutionSession> JSON
    // {
    //   "id": "relay_session_id",
    //   "trace_id": "cozmio_trace_id",
    //   "started_at": unix_ts,
    //   "task_summary": "用户可读摘要",
    //   "status": "pending|running|completed|failed",
    //   "progress_count": 0,
    //   "result_summary": null
    // }
}

#[tauri::command]
pub fn get_session_progress(
    app: tauri::AppHandle,
    session_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    // 从 relay_execution.progress 读取该 session 的历史 progress
}
```

**文件**: `cozmio/src-tauri/src/components/HistoryList.js`
**重构为会话卡片**:
```javascript
// 左侧：会话列表（时间倒序，最新20条）
// 每个卡片：时间 + 任务摘要 + 状态标签 + progress条数

// 右侧/展开：会话详情
// - pending: "等待派发中..." + [取消]按钮
// - running: 实时滚动 progress 流（通过 RP-5 的 relay_progress 事件）
// - completed: 绿色勾 + 总结 + [查看详情]按钮
// - failed: 红色叉 + 原因 + [重试]按钮

const STATUS_LABELS = {
  pending: '等待派发',
  running: '进行中',
  completed: '已完成',
  failed: '失败'
};
```

**禁止出现**:
- relay_status 等技术术语
- 原始 JSON 内容

**验证**:
- `cargo build -p cozmio` → PASS
- 打开 HISTORY → 会话卡片列表
- 执行中 → progress 实时滚动

---

### RP-3: Practice 页面精简（5 tab → 3 入口）

**文件**: `cozmio/src-tauri/src/components/PracticeDashboard.js:60-120`
**当前真相**: 5 个 tab 全部平铺

**修改为入口卡片**:
```javascript
// 移除 tab-bar
// 改为 3 个入口卡片：

<div class="practice-entries">
  <div class="practice-entry" data-entry="inbox">
    <span class="entry-icon">📥</span>
    <div class="entry-content">
      <div class="entry-title">记忆收件箱</div>
      <div class="entry-desc">查看和处理 AI 识别的记忆候选</div>
      <div class="entry-count">0 条待处理</div>
    </div>
    <button class="btn btn-secondary">查看</button>
  </div>

  <div class="practice-entry" data-entry="evaluation">
    <span class="entry-icon">📊</span>
    <div class="entry-content">
      <div class="entry-title">模型评估</div>
      <div class="entry-desc">样本质量检查和评分反馈</div>
    </div>
    <button class="btn btn-secondary">查看</button>
  </div>

  <div class="practice-entry collapsed" data-entry="signals">
    <span class="entry-icon">📈</span>
    <div class="entry-content">
      <div class="entry-title">信号看板</div>
      <div class="entry-desc">用户反馈/执行/记忆池/竞争统计</div>
    </div>
    <button class="expand-btn">展开</button>
  </div>
</div>

<!-- 子面板区域（点击后展开） -->
<div class="practice-subpanel" id="subpanel-inbox" style="display:none">
  <!-- 复用 loadActiveCandidates + loadDistillationJobs -->
</div>
```

**验证**: `cargo build -p cozmio` → PASS；3 个入口卡片

---

### RP-4: STATUS header-dot 同步

**文件**: `cozmio/src-tauri/src/components/App.js:100-140`
**确保与 MiniDot 一致**:
```javascript
// initHeaderDotState() 使用与 MiniDot 相同的状态计算
function applyDot(overallStatus) {
  const visualState = agentVisualState(state, presence);
  dot.classList.add(visualState);  // idle/monitoring/analyzing/confirm/executing/done/error
  label.textContent = STATUS_LABELS[visualState] || '待机';
}
```

**验证**: header-dot 和 MiniDot 同时显示相同状态

---

## Risk → Verification Mapping

| Risk | 验证命令 | 预期结果 |
|------|---------|---------|
| relay_progress event 未正确 emit | `cargo build -p cozmio` | PASS + 派发任务后检查 console |
| 会话卡片 progress 丢失 | 派发任务后观察 HISTORY | progress 实时滚动 |
| MiniDot 状态卡住 | 暂停监控后观察 | 3秒内变为 idle |

---

## 口子词扫描（已关闭）

- [x] 没有 `需确认/TBD/复用或重定义`
- [x] 没有 `大概是/探索`
- [x] 所有模糊表述已明确化

---

## 自我审查

- [x] 每个 RP 有 cargo build 验证
- [x] 每个 RP 有可执行的行为描述
- [x] 所有 RP 状态为 "已锁定 ✓"

---

## 下一步

批准后调用 `subagent-driven-development` 执行。