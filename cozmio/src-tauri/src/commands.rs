use crate::app_running::{is_running, set_running};
use crate::config::Config;
use crate::logging::{ActionLogger, ActionRecord};
use crate::relay_bridge::{self, send_inference_request_with_context, RelayDispatchRequest};
use crate::tray::update_tray_icon;
use crate::tray::{TrayIcon, TrayState};
use crate::ui_state::{
    CurrentTaskInfo, JudgmentInfo, PendingConfirmationInfo, RelayExecutionInfo, StateUpdate,
    WindowInfo,
};
use std::sync::RwLock;
use tauri::{Emitter, Manager, State};

/// Represents a single execution session built from ledger records
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionSession {
    pub session_id: String,
    pub trace_id: Option<String>,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub task_summary: String,
    pub status: String, // "pending" | "running" | "completed" | "failed"
    pub progress_count: u32,
    pub result_summary: Option<String>,
}

/// A single progress event within a session
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProgressEvent {
    pub timestamp: i64,
    pub event_type: String,
    #[serde(alias = "level")]
    pub status_label: String,
    pub message: String,
}

/// Application global state managed by Tauri
pub struct AppState {
    pub config: Config,
    pub logger: ActionLogger,
    pub tray_state: RwLock<TrayState>,
    pub tray_handle: RwLock<Option<TrayIcon>>,
    pub current_window: RwLock<Option<WindowInfo>>,
    pub last_judgment: RwLock<Option<JudgmentInfo>>,
    pub pending_confirmation: RwLock<Option<PendingConfirmationInfo>>,
    pub current_task: RwLock<Option<CurrentTaskInfo>>,
    pub relay_execution: RwLock<Option<RelayExecutionInfo>>,
    /// Source label for last inference result: "Local Agent Box" or "Local Mock"
    pub inference_source: RwLock<Option<String>>,
    pub update_state: RwLock<crate::updater::UpdateState>,
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            logger: ActionLogger::new(),
            tray_state: RwLock::new(TrayState::default()),
            tray_handle: RwLock::new(None),
            current_window: RwLock::new(None),
            last_judgment: RwLock::new(None),
            pending_confirmation: RwLock::new(None),
            current_task: RwLock::new(None),
            relay_execution: RwLock::new(None),
            inference_source: RwLock::new(None),
            update_state: RwLock::new(crate::updater::UpdateState::None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the current configuration
#[tauri::command]
pub fn get_config(_state: State<AppState>) -> Result<Config, String> {
    Config::load()
}

/// Save the configuration
#[tauri::command]
pub fn save_config(
    app: tauri::AppHandle,
    _state: State<AppState>,
    config: Config,
) -> Result<(), String> {
    config.save()?;
    emit_state_update(&app);
    Ok(())
}

/// Get action history with optional limit
#[tauri::command]
pub fn get_history(
    state: State<AppState>,
    limit: Option<usize>,
) -> Result<Vec<ActionRecord>, String> {
    let limit = limit.unwrap_or(100);
    state.logger.get_recent(limit)
}

/// Clear all action history
#[tauri::command]
pub fn clear_history(state: State<AppState>) -> Result<(), String> {
    state.logger.clear()
}

/// Get the current tray state
#[tauri::command]
pub fn get_tray_state(state: State<AppState>) -> Result<TrayState, String> {
    Ok(*state.tray_state.read().unwrap())
}

/// Set the tray state
#[tauri::command]
pub fn set_tray_state(state: State<AppState>, new_state: TrayState) -> Result<(), String> {
    *state.tray_state.write().unwrap() = new_state;
    Ok(())
}

/// Start the running state
#[tauri::command]
pub fn start_running(app: tauri::AppHandle) -> Result<String, String> {
    set_running(true);
    let _ = app.emit("running-state-changed", "Running");
    emit_state_update(&app);
    Ok("Running".to_string())
}

/// Stop the running state
#[tauri::command]
pub fn stop_running(app: tauri::AppHandle) -> Result<String, String> {
    set_running(false);
    let _ = app.emit("running-state-changed", "Stopped");
    emit_state_update(&app);
    Ok("Stopped".to_string())
}

/// Get the current running state
#[tauri::command]
pub fn get_running_state() -> String {
    if is_running() { "Running" } else { "Stopped" }.to_string()
}

/// Show the main window and focus it
#[tauri::command]
pub fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Hide the main window
#[tauri::command]
pub fn hide_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Get the current runtime state
#[tauri::command]
pub fn get_runtime_state() -> Result<crate::runtime_state::RuntimeState, String> {
    Ok(crate::runtime_state::read_state())
}

/// Get the current UI snapshot, including cached window and judgment data.
#[tauri::command]
pub fn get_ui_state(state: State<AppState>) -> Result<StateUpdate, String> {
    Ok(build_state_update(state.inner()))
}

#[tauri::command]
pub fn confirm_pending_task(app: tauri::AppHandle) -> Result<String, String> {
    let pending = {
        let state = app.state::<AppState>();
        let pending = state.pending_confirmation.read().unwrap().clone();
        pending
    }
    .ok_or_else(|| String::from("No pending confirmation task"))?;

    store_pending_confirmation(&app, None);
    crate::notification_manager::clear_pending_notification();

    // Step 1: Send inference request to Box Worker via Relay
    // This uses the new REQ_INFERENCE (type 7) path for Box model inference
    let trace_id = pending.trace_id.clone();
    let window_title = pending.source_window.clone();
    let process_name = pending.source_process.clone();

    crate::experience_recorder::record_popup_user_action(
        "popup_confirmed",
        &trace_id,
        &window_title,
        &process_name,
        &pending.task_text,
    );
    crate::memory_consolidation::schedule_consolidation_after_event("popup_confirmed");

    log::info!(
        "Confirm pending task - sending inference to Box Worker, trace_id={}",
        trace_id
    );

    match send_inference_request_with_context(
        &trace_id,
        &window_title,
        &process_name,
        &[], // recent_actions empty for now
        Some(&pending.task_text),
        pending.runtime_context.as_deref(),
        120, // timeout 120s
    ) {
        Ok(result) => {
            // Box inference succeeded - dispatch payload_text via relay
            // so Claude Code/execution side receives it and can perform real actions
            log::info!(
                "Box inference succeeded trace_id={}, dispatching payload_text via relay (length={})",
                trace_id,
                result.payload_text.len()
            );

            // Store inference source label so UI shows "Local Agent Box"
            {
                let state = app.state::<AppState>();
                *state.inference_source.write().unwrap() = Some(String::from("Local Agent Box"));
            }

            // Build relay dispatch request using Box's payload_text as the task text
            store_current_task(
                &app,
                Some(CurrentTaskInfo {
                    trace_id: trace_id.clone(),
                    task_text: result.payload_text.clone(),
                    source_window: pending.source_window.clone(),
                    source_process: pending.source_process.clone(),
                    created_at: pending.created_at,
                    task_state: String::from("dispatching"),
                }),
            );
            store_relay_execution(&app, None);

            let request = RelayDispatchRequest::from_task_text_with_context(
                &trace_id,
                &result.payload_text, // Use Box-generated payload_text as the task
                &pending.source_window,
                &pending.source_process,
                pending.runtime_context.as_deref(),
            );

            match relay_bridge::dispatch_confirmed_intervention(app.clone(), request) {
                Ok(session_id) => {
                    log::info!(
                        "Box payload_text dispatched via relay session={} for trace_id={}",
                        session_id,
                        trace_id
                    );
                    store_last_judgment(
                        &app,
                        Some(JudgmentInfo {
                            judgment: String::from("CONTINUE"),
                            model_text: result.payload_text.clone(),
                            status_label: String::from("CONTINUE"),
                            confidence_score: 0.0,
                            grounds: format!("Box inference for trace_id={}", trace_id),
                            system_action: format!("box-inference:relay:{session_id}"),
                            process_context: None,
                        }),
                    );
                    log_task_action(
                        &app,
                        ActionRecord {
                            timestamp: chrono::Utc::now().timestamp(),
                            trace_id: Some(trace_id.clone()),
                            session_id: Some(session_id.clone()),
                            window_title: pending.source_window,
                            judgment: String::from("CONTINUE"),
                            model_text: format!(
                                "box inference dispatched relay session={}",
                                session_id
                            ),
                            status_label: String::from("CONTINUE"),
                            confidence_score: 0.0,
                            grounds: String::from("Box inference result"),
                            system_action: String::from("box-inference"),
                            content_text: Some(pending.task_text),
                            result_text: Some(result.payload_text),
                            error_text: result.error,
                            user_feedback: None,
                            model_name: None,
                            captured_at: None,
                            call_started_at: None,
                            call_duration_ms: None,
                        },
                    );
                    emit_state_update(&app);
                    Ok(session_id)
                }
                Err(e) => {
                    log::error!("Failed to dispatch Box payload_text: {}", e);
                    store_current_task_state(&app, "dispatch_error");
                    store_last_judgment(
                        &app,
                        Some(JudgmentInfo {
                            judgment: String::from("CONTINUE"),
                            model_text: result.payload_text.clone(),
                            status_label: String::from("ERROR"),
                            confidence_score: 0.0,
                            grounds: format!(
                                "Box inference succeeded but relay dispatch failed: {}",
                                e
                            ),
                            system_action: String::from("box-inference:dispatch-error"),
                            process_context: None,
                        }),
                    );
                    log_task_action(
                        &app,
                        ActionRecord {
                            timestamp: chrono::Utc::now().timestamp(),
                            trace_id: Some(trace_id.clone()),
                            session_id: None,
                            window_title: pending.source_window.clone(),
                            judgment: String::from("CONTINUE"),
                            model_text: String::from(
                                "box inference succeeded but relay dispatch failed",
                            ),
                            status_label: String::from("ERROR"),
                            confidence_score: 0.0,
                            grounds: format!(
                                "Box inference succeeded but relay dispatch failed: {}",
                                e
                            ),
                            system_action: String::from("box-inference:dispatch-error"),
                            content_text: Some(result.payload_text.clone()),
                            result_text: None,
                            error_text: Some(e.clone()),
                            user_feedback: None,
                            model_name: None,
                            captured_at: None,
                            call_started_at: None,
                            call_duration_ms: None,
                        },
                    );
                    emit_state_update(&app);
                    Err(e)
                }
            }
        }
        Err(inference_err) => {
            // Box inference failed - fall back to relay dispatch
            log::warn!(
                "Box inference failed for trace_id={}, falling back to relay dispatch: {}",
                trace_id,
                inference_err
            );

            // Store inference source label so UI shows "Local Mock" on failure
            {
                let state = app.state::<AppState>();
                *state.inference_source.write().unwrap() = Some(String::from("Local Mock"));
            }

            // Proceed with relay dispatch as fallback
            store_current_task(
                &app,
                Some(CurrentTaskInfo {
                    trace_id: trace_id.clone(),
                    task_text: pending.task_text.clone(),
                    source_window: pending.source_window.clone(),
                    source_process: pending.source_process.clone(),
                    created_at: pending.created_at,
                    task_state: String::from("dispatching"),
                }),
            );
            store_relay_execution(&app, None);

            let request = RelayDispatchRequest::from_task_text_with_context(
                &trace_id,
                &pending.task_text,
                &pending.source_window,
                &pending.source_process,
                pending.runtime_context.as_deref(),
            );

            match relay_bridge::dispatch_confirmed_intervention(app.clone(), request) {
                Ok(session_id) => {
                    log::info!(
                        "Desktop user confirmed task, relay session started session={} window='{}' process='{}'",
                        session_id,
                        pending.source_window,
                        pending.source_process
                    );
                    store_last_judgment(
                        &app,
                        Some(JudgmentInfo {
                            judgment: String::from("CONTINUE"),
                            model_text: pending.task_text.clone(),
                            status_label: String::from("CONTINUE"),
                            confidence_score: 0.0,
                            grounds: format!("Manual confirm for trace_id={}", trace_id),
                            system_action: format!("relay-dispatched:{session_id}"),
                            process_context: None,
                        }),
                    );
                    log_task_action(
                        &app,
                        ActionRecord {
                            timestamp: chrono::Utc::now().timestamp(),
                            trace_id: Some(trace_id.clone()),
                            session_id: Some(session_id.clone()),
                            window_title: pending.source_window,
                            judgment: String::from("CONTINUE"),
                            model_text: String::from("manual confirm dispatched relay"),
                            status_label: String::from("CONTINUE"),
                            confidence_score: 0.0,
                            grounds: String::from("Manual confirm dispatch"),
                            system_action: String::from("relay-dispatched"),
                            content_text: Some(pending.task_text),
                            result_text: None,
                            error_text: None,
                            user_feedback: None,
                            model_name: None,
                            captured_at: None,
                            call_started_at: None,
                            call_duration_ms: None,
                        },
                    );
                    emit_state_update(&app);
                    Ok(session_id)
                }
                Err(error) => {
                    store_current_task_state(&app, "dispatch_error");
                    store_last_judgment(
                        &app,
                        Some(JudgmentInfo {
                            judgment: String::from("CONTINUE"),
                            model_text: String::from("用户已确认，但 Relay 派发失败"),
                            status_label: String::from("ERROR"),
                            confidence_score: 0.0,
                            grounds: error.clone(),
                            system_action: String::from("relay-dispatch-error"),
                            process_context: None,
                        }),
                    );
                    log_task_action(
                        &app,
                        ActionRecord {
                            timestamp: chrono::Utc::now().timestamp(),
                            trace_id: Some(trace_id.clone()),
                            session_id: None,
                            window_title: pending.source_window.clone(),
                            judgment: String::from("CONTINUE"),
                            model_text: String::from("relay dispatch failed"),
                            status_label: String::from("ERROR"),
                            confidence_score: 0.0,
                            grounds: error.clone(),
                            system_action: String::from("relay-dispatch-error"),
                            content_text: Some(pending.task_text),
                            result_text: None,
                            error_text: Some(error.clone()),
                            user_feedback: None,
                            model_name: None,
                            captured_at: None,
                            call_started_at: None,
                            call_duration_ms: None,
                        },
                    );
                    emit_state_update(&app);
                    Err(error)
                }
            }
        }
    }
}

#[tauri::command]
pub fn cancel_pending_task(app: tauri::AppHandle) -> Result<(), String> {
    let pending = {
        let state = app.state::<AppState>();
        let pending = state.pending_confirmation.read().unwrap().clone();
        pending
    };

    store_pending_confirmation(&app, None);
    crate::notification_manager::clear_pending_notification();
    if let Some(task) = pending {
        let task_text = task.task_text.clone();
        let source_window = task.source_window.clone();
        let source_process = task.source_process.clone();
        let trace_id = task.trace_id.clone();
        crate::experience_recorder::record_popup_user_action(
            "popup_cancelled",
            &trace_id,
            &source_window,
            &source_process,
            &task_text,
        );
        crate::memory_consolidation::schedule_consolidation_after_event("popup_cancelled");
        store_last_judgment(
            &app,
            Some(JudgmentInfo {
                judgment: String::from("USER_ACTION"),
                model_text: task_text.clone(),
                status_label: String::from("INFO"),
                confidence_score: 0.0,
                grounds: String::from("pending confirmation cancelled through UI"),
                system_action: String::from("cancelled"),
                process_context: None,
            }),
        );
        log_task_action(
            &app,
            ActionRecord {
                timestamp: chrono::Utc::now().timestamp(),
                trace_id: Some(task.trace_id),
                session_id: None,
                window_title: source_window,
                judgment: String::from("USER_ACTION"),
                model_text: String::from("user cancelled pending task"),
                status_label: String::from("INFO"),
                confidence_score: 0.0,
                grounds: String::from("pending confirmation cancelled through UI"),
                system_action: String::from("cancelled"),
                content_text: Some(task_text),
                result_text: None,
                error_text: None,
                user_feedback: Some(String::from("ui_cancelled")),
                model_name: None,
                captured_at: None,
                call_started_at: None,
                call_duration_ms: None,
            },
        );
    }
    emit_state_update(&app);
    Ok(())
}

#[tauri::command]
pub fn dismiss_pending_task(app: tauri::AppHandle) -> Result<(), String> {
    let pending = {
        let state = app.state::<AppState>();
        let pending = state.pending_confirmation.read().unwrap().clone();
        pending
    };

    store_pending_confirmation(&app, None);
    crate::notification_manager::clear_pending_notification();
    if let Some(task) = pending {
        let task_text = task.task_text.clone();
        let source_window = task.source_window.clone();
        let source_process = task.source_process.clone();
        let trace_id = task.trace_id.clone();
        crate::experience_recorder::record_popup_user_action(
            "popup_dismissed",
            &trace_id,
            &source_window,
            &source_process,
            &task_text,
        );
        crate::memory_consolidation::schedule_consolidation_after_event("popup_dismissed");
        store_last_judgment(
            &app,
            Some(JudgmentInfo {
                judgment: String::from("USER_ACTION"),
                model_text: task_text.clone(),
                status_label: String::from("INFO"),
                confidence_score: 0.0,
                grounds: String::from("pending confirmation dismissed through UI"),
                system_action: String::from("dismissed"),
                process_context: None,
            }),
        );
        log_task_action(
            &app,
            ActionRecord {
                timestamp: chrono::Utc::now().timestamp(),
                trace_id: Some(task.trace_id),
                session_id: None,
                window_title: source_window,
                judgment: String::from("USER_ACTION"),
                model_text: String::from("user dismissed pending task"),
                status_label: String::from("INFO"),
                confidence_score: 0.0,
                grounds: String::from("pending confirmation dismissed through UI"),
                system_action: String::from("dismissed"),
                content_text: Some(task_text),
                result_text: None,
                error_text: None,
                user_feedback: Some(String::from("ui_dismissed")),
                model_name: None,
                captured_at: None,
                call_started_at: None,
                call_duration_ms: None,
            },
        );
    }
    emit_state_update(&app);
    Ok(())
}

#[tauri::command]
pub fn confirm_pending_task_by_token(
    app: tauri::AppHandle,
    trace_id: String,
    token: String,
) -> Result<String, String> {
    log::info!(
        "confirm_pending_task_by_token action=confirm trace_id={} token={}",
        trace_id,
        token
    );
    let _notification =
        crate::notification_manager::consume_pending_notification(&trace_id, &token)?;

    let pending_task = {
        let state = app.state::<AppState>();
        let pending = state.pending_confirmation.read().unwrap().clone();
        pending
    }
    .ok_or_else(|| String::from("No pending confirmation task found"))?;

    if pending_task.trace_id != trace_id {
        log::warn!(
            "Pending confirmation trace mismatch app_state_trace_id={} protocol_trace_id={}",
            pending_task.trace_id,
            trace_id
        );
        return Err(String::from("Pending confirmation trace mismatch"));
    }

    crate::experience_recorder::record_popup_user_action(
        "popup_confirmed",
        &trace_id,
        &pending_task.source_window,
        &pending_task.source_process,
        &pending_task.task_text,
    );
    crate::memory_consolidation::schedule_consolidation_after_event("popup_confirmed");

    store_pending_confirmation(&app, None);
    store_current_task(
        &app,
        Some(CurrentTaskInfo {
            trace_id: pending_task.trace_id.clone(),
            task_text: pending_task.task_text.clone(),
            source_window: pending_task.source_window.clone(),
            source_process: pending_task.source_process.clone(),
            created_at: pending_task.created_at,
            task_state: String::from("dispatching"),
        }),
    );
    store_relay_execution(&app, None);

    let request = RelayDispatchRequest::from_task_text_with_context(
        &pending_task.trace_id,
        &pending_task.task_text,
        &pending_task.source_window,
        &pending_task.source_process,
        pending_task.runtime_context.as_deref(),
    );

    match relay_bridge::dispatch_confirmed_intervention(app.clone(), request) {
        Ok(session_id) => {
            log::info!(
                "Protocol confirm dispatched relay trace_id={} session_id={}",
                pending_task.trace_id,
                session_id
            );
            store_last_judgment(
                &app,
                Some(JudgmentInfo {
                    judgment: String::from("CONTINUE"),
                    model_text: pending_task.task_text.clone(),
                    status_label: String::from("CONTINUE"),
                    confidence_score: 0.0,
                    grounds: format!("Protocol confirm for trace_id={}", pending_task.trace_id),
                    system_action: format!("relay-dispatched:{}", session_id),
                    process_context: None,
                }),
            );
            log_task_action(
                &app,
                ActionRecord {
                    timestamp: chrono::Utc::now().timestamp(),
                    trace_id: Some(pending_task.trace_id),
                    session_id: Some(session_id.clone()),
                    window_title: pending_task.source_window,
                    judgment: String::from("CONTINUE"),
                    model_text: String::from("toast confirm dispatched relay"),
                    status_label: String::from("CONTINUE"),
                    confidence_score: 0.0,
                    grounds: String::from("System toast confirm"),
                    system_action: String::from("relay-dispatched"),
                    content_text: Some(pending_task.task_text),
                    result_text: None,
                    error_text: None,
                    user_feedback: None,
                    model_name: None,
                    captured_at: None,
                    call_started_at: None,
                    call_duration_ms: None,
                },
            );
            emit_state_update(&app);
            Ok(session_id)
        }
        Err(error) => {
            store_current_task_state(&app, "dispatch_error");
            log_task_action(
                &app,
                ActionRecord {
                    timestamp: chrono::Utc::now().timestamp(),
                    trace_id: Some(trace_id.clone()),
                    session_id: None,
                    window_title: pending_task.source_window.clone(),
                    judgment: String::from("CONTINUE"),
                    model_text: String::from("toast confirm dispatch failed"),
                    status_label: String::from("ERROR"),
                    confidence_score: 0.0,
                    grounds: error.clone(),
                    system_action: String::from("relay-dispatch-error"),
                    content_text: Some(pending_task.task_text.clone()),
                    result_text: None,
                    error_text: Some(error.clone()),
                    user_feedback: None,
                    model_name: None,
                    captured_at: None,
                    call_started_at: None,
                    call_duration_ms: None,
                },
            );
            store_last_judgment(
                &app,
                Some(JudgmentInfo {
                    judgment: String::from("CONTINUE"),
                    model_text: String::from("Protocol confirm failed"),
                    status_label: String::from("ERROR"),
                    confidence_score: 0.0,
                    grounds: error.clone(),
                    system_action: String::from("relay-dispatch-error"),
                    process_context: None,
                }),
            );
            emit_state_update(&app);
            Err(error)
        }
    }
}

#[tauri::command]
pub fn cancel_pending_task_by_token(
    app: tauri::AppHandle,
    trace_id: String,
    token: String,
) -> Result<(), String> {
    log::info!(
        "cancel_pending_task_by_token action=cancel trace_id={} token={}",
        trace_id,
        token
    );
    let _notification =
        crate::notification_manager::consume_pending_notification(&trace_id, &token)?;

    let pending_task = {
        let state = app.state::<AppState>();
        let pending = state.pending_confirmation.read().unwrap().clone();
        pending
    };

    store_pending_confirmation(&app, None);

    if let Some(task) = pending_task {
        if task.trace_id != trace_id {
            log::warn!(
                "Pending cancel trace mismatch app_state_trace_id={} protocol_trace_id={}",
                task.trace_id,
                trace_id
            );
            return Err(String::from("Pending confirmation trace mismatch"));
        }
        crate::experience_recorder::record_popup_user_action(
            "popup_cancelled",
            &trace_id,
            &task.source_window,
            &task.source_process,
            &task.task_text,
        );
        crate::memory_consolidation::schedule_consolidation_after_event("popup_cancelled");
        store_last_judgment(
            &app,
            Some(JudgmentInfo {
                judgment: String::from("CONTINUE"),
                model_text: task.task_text.clone(),
                status_label: String::from("CONTINUE"),
                confidence_score: 0.0,
                grounds: format!("Protocol cancel for trace_id={}", trace_id),
                system_action: String::from("cancelled_by_protocol"),
                process_context: None,
            }),
        );
        log_task_action(
            &app,
            ActionRecord {
                timestamp: chrono::Utc::now().timestamp(),
                trace_id: Some(trace_id),
                session_id: None,
                window_title: task.source_window,
                judgment: String::from("CONTINUE"),
                model_text: String::from("toast cancel dismissed dispatch"),
                status_label: String::from("CONTINUE"),
                confidence_score: 0.0,
                grounds: String::from("System toast cancel"),
                system_action: String::from("cancelled_by_protocol"),
                content_text: Some(task.task_text),
                result_text: None,
                error_text: None,
                user_feedback: Some(String::from("ui_cancelled")),
                model_name: None,
                captured_at: None,
                call_started_at: None,
                call_duration_ms: None,
            },
        );
    }

    emit_state_update(&app);
    Ok(())
}

#[tauri::command]
pub fn interrupt_current_task(app: tauri::AppHandle) -> Result<(), String> {
    let relay_execution = {
        let state = app.state::<AppState>();
        let relay_execution = state.relay_execution.read().unwrap().clone();
        relay_execution
    };
    let session_id = relay_execution
        .as_ref()
        .and_then(|execution| execution.session_id.clone())
        .ok_or_else(|| String::from("No active relay session"))?;

    log::info!(
        "Desktop user requested interrupt for relay session {}",
        session_id
    );
    crate::experience_recorder::record_relay_dispatch(
        relay_execution
            .as_ref()
            .and_then(|execution| execution.trace_id.as_deref()),
        Some(&session_id),
        "user_interrupted_execution",
        "",
        "",
        "",
        Some("desktop user requested relay interrupt"),
    );
    store_current_task_state(&app, "interrupting");
    mark_relay_interrupting(&app);
    emit_state_update(&app);

    relay_bridge::interrupt_session(app.clone(), &session_id)
}

#[tauri::command]
pub fn run_relay_demo(app: tauri::AppHandle) -> Result<String, String> {
    let current_window = {
        let state = app.state::<AppState>();
        let current_window = state.current_window.read().unwrap().clone();
        current_window
    };

    let window_title = current_window
        .as_ref()
        .map(|window| window.title.as_str())
        .filter(|title| !title.trim().is_empty())
        .unwrap_or("[Manual Relay Demo]");
    let process_name = current_window
        .as_ref()
        .map(|window| window.process_name.as_str())
        .filter(|process| !process.trim().is_empty())
        .unwrap_or("cozmio.exe");
    let reason = String::from(
        "User requested a desktop-host Relay verification run from the Cozmio status panel.",
    );
    let runtime_context = {
        let state = app.state::<AppState>();
        crate::prompt_context::build_popup_context(
            &state.logger,
            window_title,
            process_name,
            &crate::window_monitor::ProcessContext::default(),
        )
    };
    let request = RelayDispatchRequest::from_task_text_with_context(
        "",
        &reason,
        window_title,
        process_name,
        Some(&runtime_context),
    );
    log::info!(
        "Desktop relay demo confirmed from status panel window='{}' process='{}'",
        window_title,
        process_name
    );

    match relay_bridge::dispatch_confirmed_intervention(app.clone(), request) {
        Ok(session_id) => {
            store_last_judgment(
                &app,
                Some(JudgmentInfo {
                    judgment: String::from("CONTINUE"),
                    model_text: String::from("桌面端已确认并自动派发到 Relay"),
                    status_label: String::from("CONTINUE"),
                    confidence_score: 0.0,
                    grounds: reason,
                    system_action: format!("relay-dispatched:{session_id}"),
                    process_context: None,
                }),
            );
            emit_state_update(&app);
            Ok(session_id)
        }
        Err(error) => {
            store_last_judgment(
                &app,
                Some(JudgmentInfo {
                    judgment: String::from("CONTINUE"),
                    model_text: String::from("桌面端发起了 Relay 验证，但派发失败"),
                    status_label: String::from("CONTINUE"),
                    confidence_score: 0.0,
                    grounds: error.clone(),
                    system_action: String::from("relay-dispatch-error"),
                    process_context: None,
                }),
            );
            emit_state_update(&app);
            Err(error)
        }
    }
}

pub fn store_current_window(app: &tauri::AppHandle, current_window: Option<WindowInfo>) {
    let state = app.state::<AppState>();
    *state.current_window.write().unwrap() = current_window;
}

pub fn store_last_judgment(app: &tauri::AppHandle, last_judgment: Option<JudgmentInfo>) {
    let state = app.state::<AppState>();
    *state.last_judgment.write().unwrap() = last_judgment;
}

pub fn store_pending_confirmation(
    app: &tauri::AppHandle,
    pending_confirmation: Option<PendingConfirmationInfo>,
) {
    let state = app.state::<AppState>();
    *state.pending_confirmation.write().unwrap() = pending_confirmation;
}

pub fn store_current_task(app: &tauri::AppHandle, current_task: Option<CurrentTaskInfo>) {
    let state = app.state::<AppState>();
    *state.current_task.write().unwrap() = current_task;
}

pub fn log_task_action(app: &tauri::AppHandle, record: ActionRecord) {
    let state = app.state::<AppState>();
    if let Err(error) = state.logger.log(record) {
        log::error!("Failed to log task action: {}", error);
    }
}

pub fn store_current_task_state(app: &tauri::AppHandle, task_state: &str) {
    let state = app.state::<AppState>();
    let mut current_task_guard = state.current_task.write().unwrap();
    if let Some(current_task) = current_task_guard.as_mut() {
        current_task.task_state = task_state.to_string();
    }
}

pub fn store_relay_execution(app: &tauri::AppHandle, relay_execution: Option<RelayExecutionInfo>) {
    let state = app.state::<AppState>();
    if let Some(current_task) = state.current_task.write().unwrap().as_mut() {
        if let Some(execution) = relay_execution.as_ref() {
            current_task.task_state = execution.relay_status.clone();
        }
    }
    *state.relay_execution.write().unwrap() = relay_execution;
}

/// Safe variants that return Result instead of panicking on lock errors
/// These are used in the main loop to prevent thread death from lock poison

pub fn store_pending_confirmation_safe(
    app: &tauri::AppHandle,
    pending_confirmation: Option<PendingConfirmationInfo>,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let guard_result = state.pending_confirmation.write();
    match guard_result {
        Ok(mut guard) => {
            *guard = pending_confirmation;
            Ok(())
        }
        Err(poisoned) => {
            let msg = format!("pending_confirmation lock poisoned: {:?}", poisoned);
            log::error!("[STORE_ERROR] {}", msg);
            Err(msg)
        }
    }
}

pub fn store_current_task_safe(
    app: &tauri::AppHandle,
    current_task: Option<CurrentTaskInfo>,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let guard_result = state.current_task.write();
    match guard_result {
        Ok(mut guard) => {
            *guard = current_task;
            Ok(())
        }
        Err(poisoned) => {
            let msg = format!("current_task lock poisoned: {:?}", poisoned);
            log::error!("[STORE_ERROR] {}", msg);
            Err(msg)
        }
    }
}

pub fn store_relay_execution_safe(
    app: &tauri::AppHandle,
    relay_execution: Option<RelayExecutionInfo>,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    // First update current_task.task_state if needed
    let task_guard_result = state.current_task.write();
    if let Ok(mut task_guard) = task_guard_result {
        if let Some(current_task) = task_guard.as_mut() {
            if let Some(ref execution) = relay_execution {
                current_task.task_state = execution.relay_status.clone();
            }
        }
    } else {
        log::error!("[STORE_ERROR] current_task lock poisoned");
    }
    // Then update relay_execution
    let relay_guard_result = state.relay_execution.write();
    match relay_guard_result {
        Ok(mut guard) => {
            *guard = relay_execution;
            Ok(())
        }
        Err(poisoned) => {
            let msg = format!("relay_execution lock poisoned: {:?}", poisoned);
            log::error!("[STORE_ERROR] {}", msg);
            Err(msg)
        }
    }
}

pub fn has_pending_confirmation(app: &tauri::AppHandle) -> bool {
    let state = app.state::<AppState>();
    let has_pending = state.pending_confirmation.read().unwrap().is_some();
    has_pending
}

pub fn has_active_task(app: &tauri::AppHandle) -> bool {
    let state = app.state::<AppState>();
    let has_active = state
        .current_task
        .read()
        .unwrap()
        .as_ref()
        .map(|task| {
            matches!(
                task.task_state.as_str(),
                "connecting" | "dispatching" | "running" | "waiting" | "interrupting"
            )
        })
        .unwrap_or(false);
    has_active
}

fn mark_relay_interrupting(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    let mut relay_execution_guard = state.relay_execution.write().unwrap();
    if let Some(relay_execution) = relay_execution_guard.as_mut() {
        relay_execution.relay_status = String::from("interrupting");
        relay_execution.updated_at = chrono::Utc::now().timestamp();
        relay_execution
            .progress
            .push(crate::ui_state::RelayProgressInfo {
                timestamp: chrono::Utc::now().timestamp(),
                status_label: String::from("warn"),
                message: String::from(
                    "Desktop host requested interrupt for the current relay session",
                ),
            });
        if relay_execution.progress.len() > 12 {
            let drop_count = relay_execution.progress.len() - 12;
            relay_execution.progress.drain(0..drop_count);
        }
    }
}

pub fn emit_state_update(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    let update = build_state_update(state.inner());

    if let Err(e) = app.emit("state-update", &update) {
        log::error!("Failed to emit state-update event: {}", e);
    }

    // Also update tray icon to match the current mini state
    let mini_state = compute_mini_state(&update);
    update_tray_icon(app, &mini_state);
}

fn build_state_update(state: &AppState) -> StateUpdate {
    let config = Config::load().unwrap_or_else(|_| state.config.clone());

    StateUpdate {
        running_state: if is_running() { "Running" } else { "Stopped" }.to_string(),
        tray_state: tray_state_to_string(&state.tray_state.read().unwrap()),
        current_window: state.current_window.read().unwrap().clone(),
        last_judgment: state.last_judgment.read().unwrap().clone(),
        pending_confirmation: state.pending_confirmation.read().unwrap().clone(),
        current_task: state.current_task.read().unwrap().clone(),
        relay_execution: state.relay_execution.read().unwrap().clone(),
        poll_interval_secs: config.poll_interval_secs,
        ollama_url: config.ollama_url,
        model_name: config.model_name,
        inference_source: state.inference_source.read().unwrap().clone(),
    }
}

fn tray_state_to_string(state: &TrayState) -> String {
    match state {
        TrayState::Idle => "idle".to_string(),
        TrayState::Processing => "processing".to_string(),
    }
}

/// Compute the mini panel state string from a StateUpdate.
/// Mirrors the frontend computeMiniState() logic in MiniPanel.js.
fn compute_mini_state(update: &StateUpdate) -> String {
    // Priority: error > confirm > executing > analyzing > monitoring > idle
    if let Some(ref relay) = update.relay_execution {
        let status = relay.relay_status.as_str();
        if status == "failed" || status == "error" {
            return "error".to_string();
        }
        if status == "completed" || status == "done" {
            return "done".to_string();
        }
        if ["running", "waiting", "dispatching"].contains(&status) {
            return "executing".to_string();
        }
    }

    if update.pending_confirmation.is_some() {
        return "confirm".to_string();
    }

    if update.tray_state == "processing" {
        return "analyzing".to_string();
    }

    if update.running_state == "Running" {
        return "monitoring".to_string();
    }

    "idle".to_string()
}

/// List available models from Ollama
#[tauri::command]
pub fn list_models() -> Result<Vec<String>, String> {
    let config = Config::load().map_err(|e| e.to_string())?;
    let url = config.ollama_url.clone();
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(format!("{}/api/tags", url))
        .send()
        .map_err(|e| format!("Failed to connect to {}: {}", url, e))?;

    #[derive(serde::Deserialize)]
    struct TagsResponse {
        models: Vec<ModelInfo>,
    }
    #[derive(serde::Deserialize)]
    struct ModelInfo {
        name: String,
    }

    let tags: TagsResponse = response
        .json()
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(tags.models.into_iter().map(|m| m.name).collect())
}

/// Send a verification toast with action buttons (for technical verification only)
#[tauri::command]
pub fn send_verification_toast(trace_id: String) -> Result<String, String> {
    crate::toast_verification::send_verification_toast(&trace_id)?;
    Ok(format!("Toast sent for trace_id={}", trace_id))
}

/// Get verification result
#[tauri::command]
pub fn get_verification_result() -> Result<crate::toast_verification::VerificationResult, String> {
    crate::toast_verification::get_verification_result()
        .ok_or_else(|| String::from("No verification result yet"))
}

/// Handle mini panel actions: confirm, interrupt, toggle
#[tauri::command]
pub fn mini_action(app: tauri::AppHandle, action: String) -> Result<(), String> {
    match action.as_str() {
        "confirm" => {
            confirm_pending_task(app)?;
            Ok(())
        }
        "interrupt" => {
            interrupt_current_task(app)?;
            Ok(())
        }
        "toggle" => {
            if is_running() {
                set_running(false);
                let _ = app.emit("running-state-changed", "Stopped");
            } else {
                set_running(true);
                let _ = app.emit("running-state-changed", "Running");
            }
            emit_state_update(&app);
            Ok(())
        }
        _ => Err(format!("unknown action: {}", action)),
    }
}

/// Reset verification state
#[tauri::command]
pub fn reset_verification() -> Result<String, String> {
    crate::toast_verification::reset_verification();
    Ok("Verification state reset".to_string())
}

/// Get the current update state
#[tauri::command]
pub fn get_update_state(state: State<AppState>) -> Result<crate::updater::UpdateState, String> {
    Ok(state.update_state.read().unwrap().clone())
}

/// Trigger update check manually
#[tauri::command]
pub async fn check_for_updates(app_handle: tauri::AppHandle) -> Result<bool, String> {
    use crate::updater::{save_update_state, UpdateChecker};

    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let checker = UpdateChecker::new(current_version.clone());
    let response = checker.check().await?;

    if response.needs_update {
        log::info!("Update available: {}", response.latest_version);
        let download_url = response.download_url.clone();
        let signature = response.signature.clone();
        let latest_version = response.latest_version.clone();
        let app = app_handle.clone();
        let version_for_install = current_version.clone();

        tauri::async_runtime::spawn(async move {
            match checker.download(&download_url, &signature).await {
                Ok(msi_path) => {
                    // Run MSI install in a blocking task since it uses blocking I/O
                    let msi_path_owned = msi_path.clone();
                    let install_result = tokio::task::spawn_blocking(move || {
                        let checker = UpdateChecker::new(version_for_install.clone());
                        checker.install(&msi_path_owned)
                    })
                    .await;

                    match install_result {
                        Ok(Ok(_)) => {
                            log::info!("Update installed, state set to Pending");
                            let state = app.state::<AppState>();
                            let update_state = crate::updater::UpdateState::Pending {
                                version: latest_version.clone(),
                                installed_at: chrono::Utc::now(),
                            };
                            *state.update_state.write().unwrap() = update_state.clone();
                            let _ = save_update_state(&update_state);
                            let _ = app.emit("update-ready", &latest_version);
                        }
                        Ok(Err(e)) => log::error!("Update install failed: {}", e),
                        Err(e) => log::error!("Update install task failed: {}", e),
                    }
                }
                Err(e) => log::error!("Update download failed: {}", e),
            }
        });
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Restart the application
#[tauri::command]
pub fn restart_application() -> Result<(), String> {
    log::info!("Restarting application...");
    std::process::Command::new("shutdown")
        .args(&["/r", "/t", "5", "/c", "Cozmio 更新重启"])
        .spawn()
        .map_err(|e| format!("Failed to spawn shutdown: {}", e))?;
    Ok(())
}

/// Dismiss update reminder (user clicked "later")
#[tauri::command]
pub fn dismiss_update_reminder(state: State<AppState>) -> Result<(), String> {
    let mut update_state = state.update_state.write().unwrap();
    *update_state = crate::updater::UpdateState::None;
    crate::updater::save_update_state(&crate::updater::UpdateState::None)?;
    log::info!("User dismissed update reminder");
    Ok(())
}

/// Get execution sessions from ledger grouped by session_id
#[tauri::command]
pub fn get_execution_sessions(
    state: State<AppState>,
    limit: Option<u32>,
) -> Result<Vec<ExecutionSession>, String> {
    let limit = limit.unwrap_or(20) as usize;
    let records = state.logger.get_recent(limit * 3)?; // fetch more to group

    // Group records by session_id that have valid session_id
    // Skip records with no session_id
    let mut sessions_map: std::collections::HashMap<String, Vec<ActionRecord>> =
        std::collections::HashMap::new();

    for record in records {
        if let Some(ref sid) = record.session_id {
            sessions_map.entry(sid.clone()).or_default().push(record);
        }
    }

    // Build ExecutionSession from grouped records
    let mut sessions: Vec<ExecutionSession> = sessions_map
        .into_iter()
        .map(|(session_id, recs)| {
            // Sort by timestamp ascending
            let mut recs = recs;
            recs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            let started_at = recs.first().map(|r| r.timestamp).unwrap_or(0);
            let ended_at = recs.last().map(|r| r.timestamp);

            // Determine status from system_action (last record's system_action)
            let last_action = recs.last().map(|r| r.system_action.as_str()).unwrap_or("");
            let status = match last_action {
                "executed" | "completed" => "completed",
                "failed" | "error" => "failed",
                "running" | "dispatching" => "running",
                _ => "pending",
            };

            // task_summary = first record's next_step or window_title
            let task_summary = recs
                .first()
                .map(|r| {
                    if !r.judgment.is_empty() {
                        r.judgment.clone()
                    } else if !r.window_title.is_empty() {
                        r.window_title.clone()
                    } else {
                        "unknown task".to_string()
                    }
                })
                .unwrap_or_else(|| "unknown task".to_string());

            let progress_count = recs.len() as u32;

            ExecutionSession {
                session_id,
                trace_id: recs.first().and_then(|r| r.trace_id.clone()),
                started_at,
                ended_at,
                task_summary,
                status: status.to_string(),
                progress_count,
                result_summary: recs.last().and_then(|r| r.result_text.clone()),
            }
        })
        .collect();

    // Sort by started_at descending (newest first)
    sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    sessions.truncate(limit);

    Ok(sessions)
}

/// Get progress events for a specific session
#[tauri::command]
pub fn get_session_progress(
    state: State<AppState>,
    session_id: String,
) -> Result<Vec<ProgressEvent>, String> {
    let records = state.logger.get_recent(200)?;

    let events: Vec<ProgressEvent> = records
        .into_iter()
        .filter(|r| r.session_id.as_ref() == Some(&session_id))
        .map(|r| {
            let event_type = match r.system_action.as_str() {
                "dispatching" => "dispatching",
                "running" => "running",
                "completed" => "completed",
                "failed" => "failed",
                "error" => "error",
                _ => "progress",
            };
            ProgressEvent {
                timestamp: r.timestamp,
                event_type: event_type.to_string(),
                status_label: r.status_label,
                message: r.result_text.or(r.error_text).unwrap_or_default(),
            }
        })
        .collect();

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert_eq!(state.config.ollama_url, "http://localhost:11434");
        assert_eq!(*state.tray_state.read().unwrap(), TrayState::Idle);
    }

    #[test]
    fn test_app_state_with_rwlock() {
        let state = AppState::new();
        *state.tray_state.write().unwrap() = TrayState::Processing;
        assert_eq!(*state.tray_state.read().unwrap(), TrayState::Processing);
    }

    #[test]
    fn test_tray_state_serialize() {
        use serde_json;
        let state = TrayState::Idle;
        let json = serde_json::to_string(&state).unwrap();
        let parsed: TrayState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, TrayState::Idle);
    }
}
