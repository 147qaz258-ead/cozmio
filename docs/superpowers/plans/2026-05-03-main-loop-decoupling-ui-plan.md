# Plan: Main Loop 5-Flow Decoupling + Desktop Embedded Confirmation Panel

**日期**: 2026-05-03
**版本**: v0.1
**依赖**: `2026-05-03-model-input-packet-core-types-plan.md`（NotificationPending 增强字段）
**并行度**: 与 Context Admission 计划互不冲突，可独立执行

---

## Current Truth

- files inspected:
  - `cozmio/src-tauri/src/main_loop.rs:82-108` — 当前 is_waiting 分支：60 秒内 `sleep + continue`，完全停止窗口检测
    ```rust
    if is_waiting {
        // 60 秒内：sleep + continue = 完全停止观察
        thread::sleep(poll_interval);
        continue;
    }
    ```
  - `cozmio/src-tauri/src/notification_manager.rs:22` — `send_confirmation_notification()` 直接发送 Toast，不区分 trigger_route / freshness
  - `cozmio/src-tauri/src/ui_state.rs:48` — `PendingConfirmationInfo { trace_id, task_text, user_how, source_window, source_process, created_at, process_context }` — 无 trigger_route / freshness / lineage_ref
  - `cozmio/src-tauri/src/types.rs:101` — `NotificationPending { trace_id, token, content_text, user_how, created_at }` — 无 trigger_route/freshness
  - 设计文档 Section 7: 桌面端内嵌确认面板设计，显示：task_text + source_window + freshness badge + trigger_route badge

- existing entry points:
  - `send_confirmation_notification()` at `notification_manager.rs:14`
  - `store_pending_confirmation_safe()` at `commands.rs`

---

## Implementation Shape

### RP-1: Replace `is_waiting` sleep+continue with observe-only loop

**文件**: `cozmio/src-tauri/src/main_loop.rs:82-108`
**当前真相**:
```rust
if is_waiting {
    if pending {
        let age_secs = chrono::Utc::now().timestamp() - pc.created_at;
        if age_secs > 60 {
            // 超时清除
            store_pending_confirmation_safe(&app_handle, None);
            continue;
        }
    }
    // 60 秒内：sleep + continue = 完全停止观察
    thread::sleep(poll_interval);
    continue;
}
```

**修改为**:
```rust
if is_waiting {
    // === 观察流继续（始终运行，不阻塞）===
    // 即使有 pending，也继续捕获窗口变化
    let snapshot = match monitor.capture() {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Pending capture failed: {}", e);
            thread::sleep(poll_interval);
            continue;
        }
    };

    // 构建 context_signals（用于后续决策和 UI 显示）
    let competition_context = build_activity_context_sync(&app_handle, &snapshot);
    let signals = build_context_signals(&snapshot, &competition_context);

    // 检查 pending 是否过期
    if pending {
        let state = app_handle.state::<AppState>();
        let pc = state.pending_confirmation.read().unwrap().clone();
        if let Some(ref p) = pc {
            let age_secs = chrono::Utc::now().timestamp() - p.created_at;
            if age_secs > 60 {
                log::warn!("[MAIN_LOOP] Pending expired after {}s, clearing", age_secs);
                let _ = commands::store_pending_confirmation_safe(&app_handle, None);
                let _ = commands::store_current_task_safe(&app_handle, None);
                let _ = commands::store_relay_execution_safe(&app_handle, None);
            }
        }
    }

    // 更新 UI 显示 pending 状态 + 窗口变化 + signals（观察流继续）
    emit_state_update_with_pending_context(&app_handle, &snapshot, &signals);
    set_tray_state(&app_handle, TrayState::Idle);
    thread::sleep(poll_interval);
    continue;
}
```

新增 `emit_state_update_with_pending_context()` 辅助函数（在 commands.rs 或 ui_state.rs）：
- 更新 `StateUpdate.current_window` 为最新捕获的 snapshot
- 更新 `StateUpdate.pending_confirmation` 显示当前 pending 状态
- 添加 `StateUpdate.context_signals` 字段（若 UI 需要）

**验证**: 启动 app，pending 时切换窗口，观察 UI 是否更新（不卡住）
**事实依据**: main_loop.rs:82-108 + 设计文档 Section 5 五流解耦
**状态**: 已锁定 ✓

---

### RP-2: Add `trigger_route` and `freshness` to `PendingConfirmationInfo`

**文件**: `cozmio/src-tauri/src/ui_state.rs:48`
**当前真相**:
```rust
pub struct PendingConfirmationInfo {
    pub trace_id: String,
    pub task_text: String,
    pub user_how: Option<String>,
    pub source_window: String,
    pub source_process: String,
    pub created_at: i64,
    pub process_context: Option<ProcessContext>,
}
```

**修改为**:
```rust
pub struct PendingConfirmationInfo {
    pub trace_id: String,
    pub task_text: String,
    pub user_how: Option<String>,
    pub source_window: String,
    pub source_process: String,
    pub created_at: i64,
    pub process_context: Option<ProcessContext>,
    /// 触发路线（用于归因 badge）
    pub trigger_route: TriggerRoute,
    /// 新鲜度
    pub freshness: Freshness,
    /// Lineage 引用
    pub lineage_ref: String,
}
```

**验证**: `cargo check -p cozmio` — 所有 `PendingConfirmationInfo` 构造处需要更新
**事实依据**: ui_state.rs:48 + 设计文档 Section 3.4 NotificationPending 增强
**状态**: 已锁定 ✓

---

### RP-3: Refactor `send_confirmation_notification()` to accept enhanced `NotificationPending`

**文件**: `cozmio/src-tauri/src/notification_manager.rs:22`
**当前真相**:
```rust
pub fn send_confirmation_notification(pending: &NotificationPending) -> Result<(), String> {
    let title = escape_xml("Cozmio - 任务确认");
    let content_text = &pending.content_text;  // 旧字段名
    let body = truncate_for_notification(content_text, 200);
    // ... Toast XML 无 freshness / trigger_route badge
}
```

**修改为**:
```rust
pub fn send_confirmation_notification(pending: &NotificationPending) -> Result<(), String> {
    let title = escape_xml("Cozmio - 任务确认");
    // 使用新字段 user_facing_text
    let body = truncate_for_notification(&pending.user_facing_text, 150);
    // 添加 freshness badge 和 trigger_route badge 到 Toast XML
    let freshness_label = pending.freshness.display_label();
    let route_label = pending.trigger_route.badge_label();

    let toast_xml = format!(
        r#"<toast activationType="protocol" launch="{launch_url}" duration="long">
            <visual>
                <binding template="ToastGeneric">
                    <text>{title}</text>
                    <text>{body}</text>
                    <text>[{route_label}] {freshness_label}</text>
                </binding>
            </visual>
            <actions>
                <action content="确认" activationType="protocol" arguments="{confirm_url}" />
                <action content="取消" activationType="protocol" arguments="{cancel_url}" />
            </actions>
        </toast>"#,
        // ...
    );
    // ...
}
```

**验证**: `cargo check -p cozmio` — NotificationPending 字段映射正确
**事实依据**: notification_manager.rs:22 + 设计文档 Section 7 桌面确认面板
**状态**: 已锁定 ✓

---

### RP-4: Update main_loop.rs to construct enhanced `NotificationPending`

**文件**: `cozmio/src-tauri/src/main_loop.rs:310`
**当前真相**:
```rust
let notification_pending = crate::types::NotificationPending::new(
    trace_id.clone(),
    content_text.clone(),  // 直接用 raw_text
    None,
);
```

**修改为**:
```rust
// 从 build_context_signals() 获取 trigger_route 和 freshness
let trigger_route = signals
    .iter()
    .find_map(|s| s.trigger_route.clone())
    .unwrap_or_else(|| TriggerRoute::CurrentScreen {
        window_title: snapshot.window_info.title.clone(),
        process_name: snapshot.window_info.process_name.clone(),
        freshness: Freshness::from_timestamp(chrono::Utc::now().timestamp()),
    });

let freshness = Freshness::from_timestamp(chrono::Utc::now().timestamp());
let lineage_ref = format!("lineage-{}", trace_id);

let notification_pending = crate::types::NotificationPending::new(
    trace_id.clone(),
    user_facing_text,  // 经过 sanitize_for_user 处理后的文本
    None,
    trigger_route,
    freshness,
    lineage_ref,
);
```

**验证**: `cargo check -p cozmio` — NotificationPending 构造参数匹配
**事实依据**: main_loop.rs:310 + types.rs NotificationPending 新字段
**状态**: 已锁定 ✓

---

### RP-5: Update `PendingConfirmationInfo` construction in main_loop

**文件**: `cozmio/src-tauri/src/main_loop.rs:330`
**当前真相**:
```rust
pending_confirmation = Some(PendingConfirmationInfo {
    trace_id,
    task_text: content_text,
    user_how: None,
    source_window: window_title.clone(),
    source_process: snapshot.window_info.process_name.clone(),
    created_at: chrono::Utc::now().timestamp(),
    process_context: Some(process_context.clone()),
});
```

**修改为**:
```rust
pending_confirmation = Some(PendingConfirmationInfo {
    trace_id,
    task_text: content_text,
    user_how: None,
    source_window: window_title.clone(),
    source_process: snapshot.window_info.process_name.clone(),
    created_at: chrono::Utc::now().timestamp(),
    process_context: Some(process_context.clone()),
    trigger_route: trigger_route.clone(),
    freshness,
    lineage_ref,
});
```

**验证**: `cargo check -p cozmio` — PendingConfirmationInfo 字段完整
**事实依据**: main_loop.rs:330
**状态**: 已锁定 ✓

---

## Key Path Tracing

```
main_loop.rs:82 (is_waiting 分支)
  → monitor.capture() 持续观察
  → build_activity_context_sync() 持续构建 competition_context
  → build_context_signals() 持续构建 signals
  → emit_state_update_with_pending_context() 更新 UI（观察流继续）
  → thread::sleep(poll_interval) + continue（不阻塞）

main_loop.rs:310 (NotificationPending 构造)
  → trigger_route from signals
  → freshness = Freshness::from_timestamp(now)
  → lineage_ref = format!("lineage-{}", trace_id)
  → send_confirmation_notification(&notification_pending)

notification_manager.rs:22
  → Toast XML 中加入 [route_label] freshness_label badge
```

## Risk → Verification Mapping

| Risk | 验证命令 | 预期结果 |
|------|---------|---------|
| is_waiting 时窗口检测停止 | 手动：启动 app → pending 时切换窗口 | UI current_window 更新，不卡住 |
| NotificationPending 字段不匹配 | `cargo check -p cozmio 2>&1 | grep "field"` | 0 errors |
| Toast 不显示 trigger_route badge | Windows 通知中心查看 Toast | 显示 "[当前页面] 刚刚" badge |
| PendingConfirmationInfo 字段缺失 | `cargo check -p cozmio 2>&1 | grep "does not have field"` | 0 errors |