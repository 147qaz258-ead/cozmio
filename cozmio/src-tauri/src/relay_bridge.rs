use crate::commands;
use crate::logging::ActionRecord;
use crate::notification_manager;
use crate::ui_state::{RelayExecutionInfo, RelayProgressInfo};
use relay_client::proto::{ExecutionResult as RelayExecutionResult, ProgressEvent};
use relay_client::RelayClient;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const DEFAULT_RELAY_ADDR: &str = "127.0.0.1:7890";
const MAX_PROGRESS_ITEMS: usize = 12;
const MAX_RESULT_OUTPUT_CHARS: usize = 4000;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone)]
pub struct RelayDispatchRequest {
    pub trace_id: String,
    pub original_suggestion: String,
    pub dispatched_task: String,
    pub window_title: String,
    pub process_name: String,
    pub runtime_context: Option<String>,
}

impl RelayDispatchRequest {
    pub fn from_task_text(
        trace_id: &str,
        task_text: &str,
        window_title: &str,
        process_name: &str,
    ) -> Self {
        Self::from_task_text_with_context(trace_id, task_text, window_title, process_name, None)
    }

    pub fn from_task_text_with_context(
        trace_id: &str,
        task_text: &str,
        window_title: &str,
        process_name: &str,
        runtime_context: Option<&str>,
    ) -> Self {
        let runtime_context_block = runtime_context
            .filter(|context| !context.trim().is_empty())
            .map(|context| {
                format!(
                    "\n\n运行时事实与已准入记忆上下文:\n{}\n",
                    clip_relay_context(context, 2200)
                )
            })
            .unwrap_or_default();
        let dispatched_task = format!(
            "你是 Cozmio 的执行接力层。\n\n请基于下面这条已经被用户确认的原始任务继续执行。\n\n原始任务文本:\n{task_text}\n\n来源窗口: {window_title}\n来源进程: {process_name}{runtime_context_block}\n\n要求:\n- 保持任务文本原意，不要改写用户任务目标。\n- 运行时上下文只是事实材料、原文材料、反馈事实和已准入记忆，不是用户意图或任务阶段结论。\n- 回复使用中文。\n- 不要臆造截图中不存在的上下文。\n- 如果任务完成，请给出清晰结果；如果失败，请明确失败原因。"
        );

        Self {
            trace_id: trace_id.to_string(),
            original_suggestion: task_text.to_string(),
            dispatched_task,
            window_title: window_title.to_string(),
            process_name: process_name.to_string(),
            runtime_context: runtime_context.map(str::to_string),
        }
    }
}

fn clip_relay_context(value: &str, max_chars: usize) -> String {
    let mut clipped: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        clipped.push_str("...");
    }
    clipped
}

pub fn dispatch_confirmed_intervention(
    app_handle: AppHandle,
    request: RelayDispatchRequest,
) -> Result<String, String> {
    log::info!(
        "Desktop host dispatching Relay request for '{}' ({})",
        request.window_title,
        request.process_name
    );
    let app_handle = Arc::new(app_handle);
    let publish_handle = app_handle.clone();
    let publish: Arc<dyn Fn(RelayExecutionInfo) + Send + Sync> = Arc::new(move |info| {
        commands::store_relay_execution(publish_handle.as_ref(), Some(info));
        commands::emit_state_update(publish_handle.as_ref());
    });

    start_relay_session(app_handle, request, publish)
}

fn trace_ref(request: &RelayDispatchRequest) -> Option<&str> {
    if request.trace_id.is_empty() {
        None
    } else {
        Some(request.trace_id.as_str())
    }
}

fn record_relay_failure(
    request: &RelayDispatchRequest,
    session_id: Option<&str>,
    event_kind: &str,
    detail: &str,
) {
    crate::experience_recorder::record_relay_dispatch(
        trace_ref(request),
        session_id,
        event_kind,
        &request.window_title,
        &request.process_name,
        &request.original_suggestion,
        Some(detail),
    );
}

pub fn interrupt_session(app_handle: AppHandle, session_id: &str) -> Result<(), String> {
    ensure_relay_available()?;
    let relay_address = relay_address();
    let client = RelayClient::connect(&relay_address)
        .map_err(|e| format!("Failed to connect Relay for interrupt: {e}"))?;

    match client.interrupt(session_id) {
        Ok(true) => {
            log::info!("Desktop host interrupted Relay session {}", session_id);
            let state = app_handle.state::<crate::commands::AppState>();
            if let Some(relay_execution) = state.relay_execution.write().unwrap().as_mut() {
                relay_execution.relay_status = String::from("interrupted");
                relay_execution.updated_at = now_ts();
                relay_execution.progress.push(RelayProgressInfo {
                    timestamp: now_ts(),
                    status_label: String::from("info"),
                    message: String::from("Desktop host successfully interrupted Relay session"),
                });
                if relay_execution.progress.len() > MAX_PROGRESS_ITEMS {
                    let drop_count = relay_execution.progress.len() - MAX_PROGRESS_ITEMS;
                    relay_execution.progress.drain(0..drop_count);
                }
            }
            commands::emit_state_update(&app_handle);
            Ok(())
        }
        Ok(false) => Err(format!(
            "Relay refused to interrupt session {} because it is not active",
            session_id
        )),
        Err(e) => {
            let error = format!("Failed to interrupt Relay session {}: {}", session_id, e);
            log::error!("{}", error);
            let state = app_handle.state::<crate::commands::AppState>();
            if let Some(relay_execution) = state.relay_execution.write().unwrap().as_mut() {
                relay_execution.relay_status = String::from("interrupt_error");
                relay_execution.error_message = Some(error.clone());
                relay_execution.updated_at = now_ts();
            }
            commands::store_current_task_state(&app_handle, "interrupt_error");
            commands::emit_state_update(&app_handle);
            Err(error)
        }
    }
}

pub fn start_relay_session(
    app_handle: Arc<AppHandle>,
    request: RelayDispatchRequest,
    publish: Arc<dyn Fn(RelayExecutionInfo) + Send + Sync>,
) -> Result<String, String> {
    let mut snapshot = RelayExecutionInfo {
        trace_id: if request.trace_id.is_empty() {
            None
        } else {
            Some(request.trace_id.clone())
        },
        transport: String::from("tcp-loopback"),
        relay_status: String::from("connecting"),
        session_id: None,
        progress: vec![],
        result_summary: None,
        result_output: None,
        error_message: None,
        updated_at: now_ts(),
    };
    push_host_progress(
        &mut snapshot,
        format!(
            "Desktop host preparing relay dispatch for '{}' ({})",
            request.window_title, request.process_name
        ),
        "info",
    );
    publish(snapshot.clone());

    if let Err(e) = ensure_relay_available() {
        log::error!("Desktop host could not reach Relay: {}", e);
        record_relay_failure(
            &request,
            None,
            "relay_unavailable",
            &format!("ensure_relay_available failed: {e}"),
        );
        snapshot.relay_status = String::from("relay_unavailable");
        snapshot.error_message = Some(e.clone());
        snapshot.updated_at = now_ts();
        publish(snapshot);
        return Err(e);
    }

    let relay_address = relay_address();
    let client = match RelayClient::connect(&relay_address) {
        Ok(client) => client,
        Err(e) => {
            let error = format!("Failed to connect Relay: {e}");
            log::error!("{}", error);
            record_relay_failure(&request, None, "relay_connect_failed", &error);
            snapshot.relay_status = String::from("connect_error");
            snapshot.error_message = Some(error.clone());
            snapshot.updated_at = now_ts();
            publish(snapshot);
            return Err(error);
        }
    };

    log::info!("Desktop host connected to Relay at {}", relay_address);
    snapshot.relay_status = String::from("dispatching");
    snapshot.updated_at = now_ts();
    publish(snapshot.clone());

    crate::experience_recorder::record_relay_dispatch(
        trace_ref(&request),
        None,
        "relay_dispatch_requested",
        &request.window_title,
        &request.process_name,
        &request.original_suggestion,
        Some("desktop host requested relay dispatch"),
    );

    let session_id = match client.dispatch(
        "claude-code",
        &request.original_suggestion,
        &request.dispatched_task,
    ) {
        Ok(session_id) => session_id,
        Err(e) => {
            let error = format!("Relay dispatch failed: {e}");
            log::error!("{}", error);
            record_relay_failure(&request, None, "relay_dispatch_failed", &error);
            snapshot.relay_status = String::from("dispatch_error");
            snapshot.error_message = Some(error.clone());
            snapshot.updated_at = now_ts();
            publish(snapshot);
            return Err(error);
        }
    };

    snapshot.session_id = Some(session_id.clone());
    snapshot.relay_status = String::from("running");
    crate::experience_recorder::record_relay_dispatch(
        trace_ref(&request),
        Some(&session_id),
        "executor_session_started",
        &request.window_title,
        &request.process_name,
        &request.original_suggestion,
        Some("relay session id assigned"),
    );
    snapshot.updated_at = now_ts();
    push_host_progress(
        &mut snapshot,
        format!("Desktop host dispatched Relay session {session_id}"),
        "info",
    );
    publish(snapshot.clone());

    let tracking_request = request.clone();
    let tracking_session_id = session_id.clone();
    let tracking_publish = publish.clone();
    let tracking_app_handle = app_handle.clone();
    thread::spawn(move || {
        track_relay_session(
            tracking_app_handle,
            tracking_session_id,
            tracking_request,
            snapshot,
            tracking_publish,
        )
    });

    Ok(session_id)
}

fn track_relay_session(
    app_handle: Arc<AppHandle>,
    session_id: String,
    request: RelayDispatchRequest,
    mut snapshot: RelayExecutionInfo,
    publish: Arc<dyn Fn(RelayExecutionInfo) + Send + Sync>,
) {
    let relay_address = relay_address();
    let client = match RelayClient::connect(&relay_address) {
        Ok(client) => client,
        Err(e) => {
            log::error!("Desktop host could not reconnect for subscription: {}", e);
            let error = format!("Failed to reconnect Relay for subscription: {e}");
            record_relay_failure(
                &request,
                Some(&session_id),
                "relay_subscription_reconnect_failed",
                &error,
            );
            snapshot.relay_status = String::from("subscription_error");
            snapshot.error_message = Some(error);
            snapshot.updated_at = now_ts();
            publish(snapshot);
            return;
        }
    };

    let mut subscription = match client.subscribe(&session_id) {
        Ok(subscription) => subscription,
        Err(e) => {
            log::error!(
                "Desktop host could not subscribe Relay session {}: {}",
                session_id,
                e
            );
            let error = format!("Failed to subscribe Relay session: {e}");
            record_relay_failure(
                &request,
                Some(&session_id),
                "relay_subscription_failed",
                &error,
            );
            snapshot.relay_status = String::from("subscription_error");
            snapshot.error_message = Some(error);
            snapshot.updated_at = now_ts();
            publish(snapshot);
            return;
        }
    };
    log::info!(
        "Desktop host subscribed to Relay session {} for '{}' ({})",
        session_id,
        request.window_title,
        request.process_name
    );

    loop {
        match subscription.recv_event() {
            Ok(event) => {
                log::info!(
                    "Desktop host received Relay event session={} terminal={} status={} level={} message={}",
                    session_id,
                    event.terminal,
                    event.terminal_status,
                    level_label(event.level),
                    truncate_for_log(&event.message)
                );
                apply_progress_event(&mut snapshot, &event);
                crate::experience_recorder::record_relay_dispatch(
                    trace_ref(&request),
                    Some(&session_id),
                    "executor_progress_summary",
                    &request.window_title,
                    &request.process_name,
                    &request.original_suggestion,
                    Some(&event.message),
                );
                snapshot.updated_at = now_ts();
                publish(snapshot.clone());

                if event.terminal {
                    break;
                }
            }
            Err(e) => {
                log::error!(
                    "Desktop host Relay progress stream failed for {}: {}",
                    session_id,
                    e
                );
                let error = format!("Relay progress stream failed: {e}");
                record_relay_failure(
                    &request,
                    Some(&session_id),
                    "relay_progress_stream_failed",
                    &error,
                );
                snapshot.relay_status = String::from("subscription_error");
                snapshot.error_message = Some(error);
                snapshot.updated_at = now_ts();
                publish(snapshot);
                return;
            }
        }
    }

    match wait_for_result(&session_id) {
        Ok(Some(result)) => {
            log::info!(
                "Desktop host fetched Relay result session={} success={} summary={}",
                session_id,
                result.success,
                truncate_for_log(&result.summary)
            );
            snapshot.result_summary = Some(result.summary.clone());
            snapshot.result_output = Some(trim_output(&result.raw_output));
            snapshot.error_message = match result.error_message.trim() {
                "" => snapshot.error_message.take(),
                message => Some(message.to_string()),
            };
            if snapshot.relay_status == "running" {
                snapshot.relay_status = if result.success {
                    String::from("completed")
                } else {
                    String::from("failed")
                };
            }
            crate::experience_recorder::record_relay_dispatch(
                trace_ref(&request),
                Some(&session_id),
                if result.success {
                    "executor_completed"
                } else {
                    "executor_failed"
                },
                &request.window_title,
                &request.process_name,
                &request.original_suggestion,
                Some(&format!(
                    "success={} status={} summary={} error={}",
                    result.success, snapshot.relay_status, result.summary, result.error_message
                )),
            );
            crate::memory_consolidation::schedule_consolidation_after_event(if result.success {
                "executor_completed"
            } else {
                "executor_failed"
            });
            push_host_progress(
                &mut snapshot,
                format!(
                    "Desktop host received Relay result for '{}' ({})",
                    request.window_title, request.process_name
                ),
                "info",
            );
            snapshot.updated_at = now_ts();
            publish(snapshot.clone());

            // Send result notification toast
            let trace_id = snapshot.trace_id.clone().unwrap_or_default();
            if let Err(e) = notification_manager::send_result_notification(
                &trace_id,
                &request.original_suggestion,
                &snapshot.relay_status,
                snapshot.result_output.as_deref(),
                snapshot.error_message.as_deref(),
            ) {
                log::error!("Failed to send result toast trace_id={}: {}", trace_id, e);
            }
            commands::log_task_action(
                app_handle.as_ref(),
                ActionRecord {
                    timestamp: chrono::Utc::now().timestamp(),
                    trace_id: snapshot.trace_id.clone(),
                    session_id: Some(session_id.clone()),
                    window_title: request.window_title.clone(),
                    judgment: String::from("CONTINUE"),
                    model_text: format!("relay {}", snapshot.relay_status),
                    status_label: snapshot.relay_status.clone(),
                    confidence_score: 0.0,
                    grounds: format!("Relay terminal status {}", snapshot.relay_status),
                    system_action: snapshot.relay_status.clone(),
                    content_text: Some(request.original_suggestion.clone()),
                    result_text: snapshot.result_output.clone(),
                    error_text: snapshot.error_message.clone(),
                    user_feedback: None,
                    model_name: None,
                    captured_at: None,
                    call_started_at: None,
                    call_duration_ms: None,
                },
            );
        }
        Ok(None) => {
            log::warn!(
                "Desktop host saw terminal Relay state without result for session {}",
                session_id
            );
            snapshot.error_message = Some(String::from(
                "Relay session reached terminal state but result was not available",
            ));
            snapshot.updated_at = now_ts();
            publish(snapshot);
        }
        Err(e) => {
            log::error!(
                "Desktop host failed to fetch Relay result for {}: {}",
                session_id,
                e
            );
            snapshot.error_message = Some(e);
            snapshot.updated_at = now_ts();
            publish(snapshot);
        }
    }
}

fn apply_progress_event(snapshot: &mut RelayExecutionInfo, event: &ProgressEvent) {
    let message = if event.message.trim().is_empty() {
        String::from("<empty>")
    } else {
        event.message.clone()
    };
    snapshot.progress.push(RelayProgressInfo {
        timestamp: event.timestamp,
        status_label: level_label(event.level).to_string(),
        message,
    });
    trim_progress(snapshot);

    if event.terminal {
        snapshot.relay_status = terminal_status_label(event.terminal_status).to_string();
    } else if snapshot.relay_status == "dispatching" || snapshot.relay_status == "connecting" {
        snapshot.relay_status = String::from("running");
    }
}

fn wait_for_result(session_id: &str) -> Result<Option<RelayExecutionResult>, String> {
    let relay_address = relay_address();
    let deadline = Instant::now() + Duration::from_secs(15);

    while Instant::now() < deadline {
        let client = RelayClient::connect(&relay_address)
            .map_err(|e| format!("Failed to reconnect Relay for result fetch: {e}"))?;
        match client.result(session_id) {
            Ok(response) if response.result.is_some() => return Ok(response.result),
            Ok(_) => thread::sleep(Duration::from_millis(300)),
            Err(_) => thread::sleep(Duration::from_millis(300)),
        }
    }

    Ok(None)
}

fn push_host_progress(snapshot: &mut RelayExecutionInfo, message: String, status_label: &str) {
    snapshot.progress.push(RelayProgressInfo {
        timestamp: now_ts(),
        status_label: status_label.to_string(),
        message,
    });
    trim_progress(snapshot);
}

fn trim_progress(snapshot: &mut RelayExecutionInfo) {
    if snapshot.progress.len() > MAX_PROGRESS_ITEMS {
        let drop_count = snapshot.progress.len() - MAX_PROGRESS_ITEMS;
        snapshot.progress.drain(0..drop_count);
    }
}

fn trim_output(output: &str) -> String {
    let mut trimmed = output.trim().to_string();
    if trimmed.chars().count() > MAX_RESULT_OUTPUT_CHARS {
        trimmed = trimmed
            .chars()
            .take(MAX_RESULT_OUTPUT_CHARS)
            .collect::<String>();
        trimmed.push_str("\n...[truncated]");
    }
    trimmed
}

fn truncate_for_log(message: &str) -> String {
    const MAX_LOG_CHARS: usize = 180;
    let trimmed = message.trim();
    if trimmed.chars().count() <= MAX_LOG_CHARS {
        return trimmed.to_string();
    }

    let mut truncated = trimmed.chars().take(MAX_LOG_CHARS).collect::<String>();
    truncated.push_str("...[truncated]");
    truncated
}

fn level_label(progress_code: i32) -> &'static str {
    match progress_code {
        1 => "warn",
        2 => "error",
        _ => "info",
    }
}

fn terminal_status_label(status: i32) -> &'static str {
    match status {
        3 => "completed",
        4 => "failed",
        5 => "interrupted",
        2 => "waiting",
        1 => "running",
        _ => "unknown",
    }
}

fn ensure_relay_available() -> Result<(), String> {
    if can_connect(&relay_address()) {
        return Ok(());
    }

    let relay_binary = find_local_relay_binary()
        .ok_or_else(|| String::from("Relay is not reachable and relay-engine.exe was not found"))?;

    log::info!(
        "Relay not reachable, spawning local relay-engine at {:?}",
        relay_binary
    );
    spawn_local_relay(&relay_binary)?;

    let deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < deadline {
        if can_connect(&relay_address()) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(200));
    }

    Err(String::from(
        "Relay did not become reachable after spawning local relay-engine",
    ))
}

fn can_connect(address: &str) -> bool {
    let socket_addr = match resolve_socket_addr(address) {
        Some(socket_addr) => socket_addr,
        None => return false,
    };

    TcpStream::connect_timeout(&socket_addr, Duration::from_millis(500)).is_ok()
}

fn resolve_socket_addr(address: &str) -> Option<SocketAddr> {
    address.to_socket_addrs().ok()?.next()
}

fn relay_address() -> String {
    std::env::var("COZMIO_RELAY_ADDR").unwrap_or_else(|_| DEFAULT_RELAY_ADDR.to_string())
}

fn find_local_relay_binary() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("COZMIO_RELAY_ENGINE_BIN") {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent()?.to_path_buf();
    let candidates = [
        workspace_root
            .join("target")
            .join("release")
            .join(binary_name()),
        workspace_root
            .join("target")
            .join("debug")
            .join(binary_name()),
        manifest_dir.join(binary_name()),
    ];

    candidates.into_iter().find(|candidate| candidate.exists())
}

fn binary_name() -> &'static str {
    if cfg!(windows) {
        "relay-engine.exe"
    } else {
        "relay-engine"
    }
}

fn spawn_local_relay(binary_path: &Path) -> Result<(), String> {
    let mut command = Command::new(binary_path);
    command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to spawn relay-engine at {:?}: {}", binary_path, e))
}

fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Response from a Box inference request
#[derive(Debug, Clone)]
pub struct BoxInferenceResult {
    pub payload_text: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Send an inference request to the Box Worker via Relay REQ_INFERENCE (type 7).
///
/// Returns the natural language response from the Box model, or an error.
/// The trace_id is included in the context_bundle and logged throughout the链路.
pub fn send_inference_request(
    trace_id: &str,
    window_title: &str,
    process_name: &str,
    recent_actions: &[String],
    timeout_secs: u64,
) -> Result<BoxInferenceResult, String> {
    send_inference_request_with_context(
        trace_id,
        window_title,
        process_name,
        recent_actions,
        None,
        None,
        timeout_secs,
    )
}

pub fn send_inference_request_with_context(
    trace_id: &str,
    window_title: &str,
    process_name: &str,
    recent_actions: &[String],
    confirmed_popup_output: Option<&str>,
    runtime_context: Option<&str>,
    timeout_secs: u64,
) -> Result<BoxInferenceResult, String> {
    // Build context_bundle JSON
    let context_bundle = serde_json::json!({
        "trace_id": trace_id,
        "window_title": window_title,
        "process_name": process_name,
        "recent_actions": recent_actions,
        "confirmed_popup_output": confirmed_popup_output,
        "runtime_context": runtime_context,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    let bundle_str = context_bundle.to_string();

    log::info!(
        "Sending inference to Box Worker via Relay, trace_id={}, window='{}'",
        trace_id,
        window_title
    );

    ensure_relay_available()?;

    let relay_address = relay_address();
    let client = RelayClient::connect(&relay_address)
        .map_err(|e| format!("Failed to connect to Relay for inference: {e}"))?;

    match client.inference(&bundle_str, timeout_secs) {
        Ok(response) => {
            log::info!(
                "Box Worker inference completed trace_id={} success={}",
                trace_id,
                response.success
            );
            Ok(BoxInferenceResult {
                payload_text: response.payload_text,
                success: response.success,
                error: if response.error_message.is_empty() {
                    None
                } else {
                    Some(response.error_message)
                },
            })
        }
        Err(e) => {
            let err_msg = format!("Box inference failed: {e}");
            log::error!("{}", err_msg);
            Err(err_msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_task_text_carries_runtime_context_as_factual_material() {
        let request = RelayDispatchRequest::from_task_text_with_context(
            "trace-1",
            "请检查 Cozmio 的 handoff 链路",
            "Cozmio design review",
            "Code.exe",
            Some(
                "hot_stable_context:\n- source_type=hot_stable_context, source_ref=human_context.md\nrecall_admission:\nrecent_feedback_facts:\n- source_type=recent_feedback_fact, source_ref=action_log:1:trace",
            ),
        );

        assert!(request
            .dispatched_task
            .contains("运行时事实与已准入记忆上下文"));
        assert!(request.dispatched_task.contains("hot_stable_context:"));
        assert!(request.dispatched_task.contains("recent_feedback_facts:"));
        assert!(request
            .dispatched_task
            .contains("不是用户意图或任务阶段结论"));
        assert_eq!(
            request.runtime_context.as_deref(),
            Some(
                "hot_stable_context:\n- source_type=hot_stable_context, source_ref=human_context.md\nrecall_admission:\nrecent_feedback_facts:\n- source_type=recent_feedback_fact, source_ref=action_log:1:trace"
            )
        );
    }
}
