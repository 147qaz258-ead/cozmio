use crate::app_running::is_running;
use crate::commands::{self, AppState};
use crate::config::Config;
use crate::logging::{
    ActionLogger, ActionRecord, FactualActionRecord, FactualEventType, SystemRoute,
};
use crate::model_client::ModelClient;
use crate::prompt_context::build_popup_context;
use crate::tray::TrayState;
use crate::ui_state::{JudgmentInfo, PendingConfirmationInfo, WindowInfo};
use crate::updater::{load_update_state, save_update_state, UpdateChecker, UpdateState};
use crate::window_monitor::WindowMonitor;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Start the main monitoring loop
///
/// This function runs indefinitely, polling for window changes and processing them.
pub fn start_main_loop(app_handle: AppHandle, config: Config, logger: ActionLogger) {
    log::info!("Starting main monitoring loop");

    let mut monitor = WindowMonitor::new();
    let mut fallback_config = config;
    let mut last_error_signature: Option<String> = None;
    let mut startup_update_check_done = false;

    loop {
        let active_config = Config::load().unwrap_or_else(|e| {
            log::warn!("Failed to reload config, using last known config: {}", e);
            fallback_config.clone()
        });
        fallback_config = active_config.clone();
        let poll_interval = Duration::from_secs(active_config.poll_interval_secs.max(1));

        // === Startup update check (once per startup) ===
        if !startup_update_check_done {
            let last_check = active_config.last_check_at.as_ref();
            let should_check = last_check
                .map(|t| {
                    if let Ok(last) = chrono::DateTime::parse_from_rfc3339(t) {
                        let elapsed = chrono::Utc::now()
                            .signed_duration_since(last.with_timezone(&chrono::Utc));
                        elapsed.num_seconds() > 24 * 3600
                    } else {
                        true
                    }
                })
                .unwrap_or(true);

            if should_check {
                log::info!("Checking for updates on startup...");
                let app = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = run_update_check(&app).await {
                        log::error!("Startup update check failed: {}", e);
                    }
                });
            }
            startup_update_check_done = true;
        }
        // === Update check end ===

        // Step 1: Check running state - if not running, wait
        // Heartbeat log - every iteration, whether waiting or processing
        let pending = commands::has_pending_confirmation(&app_handle);
        let active = commands::has_active_task(&app_handle);
        let is_waiting = pending || active;
        log::info!(
            "[MAIN_LOOP] Heartbeat is_running={} pending={} active={} waiting={}",
            is_running(),
            pending,
            active,
            is_waiting
        );

        if !is_running() {
            log::debug!("Main loop paused, waiting...");
            thread::sleep(poll_interval);
            continue;
        }

        if !is_waiting {
            crate::memory_consolidation::schedule_idle_consolidation_if_due();
        }

        if is_waiting {
            // Check for stale pending confirmations - auto-expire after 60 seconds
            if pending {
                let state = app_handle.state::<AppState>();
                let pending_confirmation = state.pending_confirmation.read().unwrap().clone();
                drop(state);
                if let Some(pc) = pending_confirmation.as_ref() {
                    let age_secs = chrono::Utc::now().timestamp() - pc.created_at;
                    if age_secs > 60 {
                        log::warn!(
                            "[MAIN_LOOP] Pending confirmation expired after {}s (trace_id={}), clearing",
                            age_secs,
                            pc.trace_id
                        );
                        crate::experience_recorder::record_popup_user_action(
                            "confirmation_expired",
                            &pc.trace_id,
                            &pc.source_window,
                            &pc.source_process,
                            &pc.task_text,
                        );
                        crate::memory_consolidation::schedule_consolidation_after_event(
                            "confirmation_expired",
                        );
                        let _ = commands::store_pending_confirmation_safe(&app_handle, None);
                        let _ = commands::store_current_task_safe(&app_handle, None);
                        let _ = commands::store_relay_execution_safe(&app_handle, None);
                        thread::sleep(poll_interval);
                        continue;
                    }
                }
            }
            log::info!("Main loop waiting for pending confirmation or active relay task");
            set_tray_state(&app_handle, TrayState::Idle);
            commands::emit_state_update(&app_handle);
            thread::sleep(poll_interval);
            continue;
        }

        // Record loop iteration tick
        crate::runtime_state::increment_tick();

        // Set processing state while working
        {
            let state = app_handle.state::<AppState>();
            let mut guard = state.tray_state.write().unwrap();
            *guard = TrayState::Processing;
        }

        // Step 2: Capture current window
        let snapshot = match monitor.capture() {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Failed to capture window: {}", e);
                // Reset to idle state
                set_tray_state(&app_handle, TrayState::Idle);
                store_error_judgment(
                    &app_handle,
                    &logger,
                    &mut last_error_signature,
                    "CAPTURE_ERROR",
                    "-",
                    &e,
                    None,
                );
                commands::emit_state_update(&app_handle);
                thread::sleep(poll_interval);
                continue;
            }
        };

        let current_window = WindowInfo {
            title: snapshot.window_info.title.clone(),
            process_name: snapshot.window_info.process_name.clone(),
            process_id: snapshot.window_info.process_id,
            monitor_index: snapshot.window_info.monitor_index,
        };
        crate::experience_recorder::record_observation(
            None,
            &snapshot.window_info.title,
            &snapshot.window_info.process_name,
            snapshot.timestamp,
        );
        commands::store_current_window(&app_handle, Some(current_window));
        commands::emit_state_update(&app_handle);

        // Step 3: Noise filtering (standalone - happens before compute_context)
        {
            let process_name = &snapshot.window_info.process_name;
            let window_title = &snapshot.window_info.title;
            if process_name == "cozmio.exe" {
                log::debug!("Skipping cozmio.exe itself");
                set_tray_state(&app_handle, TrayState::Idle);
                commands::emit_state_update(&app_handle);
                thread::sleep(poll_interval);
                continue;
            }
            if (process_name == "cmd.exe" || process_name == "powershell.exe")
                && window_title.contains("cozmio")
            {
                log::debug!("Skipping cozmio-related console window");
                set_tray_state(&app_handle, TrayState::Idle);
                commands::emit_state_update(&app_handle);
                thread::sleep(poll_interval);
                continue;
            }
        }

        // Step 4: Check for window change (if detection enabled)
        if active_config.window_change_detection && !monitor.has_changed(&snapshot) {
            log::debug!("No window change detected, skipping");
            set_tray_state(&app_handle, TrayState::Idle);
            commands::emit_state_update(&app_handle);
            thread::sleep(poll_interval);
            continue;
        }

        log::info!("Window changed: {}", snapshot.window_info.title);

        // Step 5: Compute process context (BEFORE pushing current snapshot to buffer)
        let process_context = monitor.compute_context(
            &snapshot.window_info.title,
            &snapshot.window_info.process_name,
            snapshot.timestamp,
        );

        // Step 6: Build compact local context and call model - returns raw output without parsing
        let popup_context = build_popup_context(
            &logger,
            &snapshot.window_info.title,
            &snapshot.window_info.process_name,
            &process_context,
        );
        let popup_context_preview: String = popup_context.chars().take(240).collect();
        log::debug!(
            "Popup context built ({} chars): {}",
            popup_context.len(),
            popup_context_preview
        );

        let model_client = ModelClient::new(active_config.clone());
        let raw_output = match model_client.call_raw_with_context(
            &snapshot,
            &process_context,
            Some(&popup_context),
        ) {
            Ok(output) => output,
            Err(e) => {
                log::error!("Model call failed: {}", e);
                set_tray_state(&app_handle, TrayState::Idle);
                store_error_judgment(
                    &app_handle,
                    &logger,
                    &mut last_error_signature,
                    "MODEL_ERROR",
                    &snapshot.window_info.title,
                    &e,
                    Some(process_context.clone()),
                );
                commands::emit_state_update(&app_handle);
                thread::sleep(poll_interval);
                continue;
            }
        };

        last_error_signature = None;
        monitor.update_last_title(&snapshot.window_info.title);

        log::info!(
            "Model raw output ({} chars, {}ms, model={}): {}",
            raw_output.raw_text.len(),
            raw_output.call_duration_ms,
            raw_output.model_name,
            &raw_output.raw_text[..raw_output.raw_text.len().min(200)]
        );

        // Step 5: Product contract: non-empty raw text creates a pending notification.
        let window_title = snapshot.window_info.title.clone();
        let raw_output_present = !raw_output.raw_text.trim().is_empty();

        crate::experience_recorder::record_model_output(
            &raw_output.trace_id,
            &window_title,
            &snapshot.window_info.process_name,
            &raw_output.model_name,
            &raw_output.raw_text,
            raw_output.captured_at,
            raw_output.call_duration_ms,
        );

        let original_judgment = if raw_output_present {
            "raw_output_present=true"
        } else {
            "raw_output_present=false"
        }
        .to_string();
        let execution_result_str = if raw_output_present {
            "notification_created"
        } else {
            "notification_not_created"
        }
        .to_string();
        let state = app_handle.state::<AppState>();
        if let Err(error) = state.logger.log_factual(FactualActionRecord {
            timestamp: chrono::Utc::now().timestamp(),
            trace_id: Some(raw_output.trace_id.clone()),
            session_id: None,
            window_title: window_title.clone(),
            event_type: FactualEventType::ModelOutput,
            system_route: SystemRoute::Unknown,
            original_judgment,
            execution_result_str,
            raw_model_text: Some(raw_output.raw_text.clone()),
            model_name: Some(raw_output.model_name.clone()),
            captured_at: Some(raw_output.captured_at),
            call_started_at: Some(raw_output.call_started_at),
            call_duration_ms: Some(raw_output.call_duration_ms),
            execution_result: None,
            error_text: None,
            user_feedback: None,
        }) {
            log::error!("Failed to log raw model output: {}", error);
        }

        let system_action: String;
        let pending_confirmation: Option<PendingConfirmationInfo>;

        if raw_output_present {
            // Create pending notification with raw_text as task content
            let trace_id = raw_output.trace_id.clone();
            let content_text = raw_output.raw_text.clone();

            let notification_pending = crate::types::NotificationPending::new(
                trace_id.clone(),
                content_text.clone(),
                None,
            );

            if let Err(e) =
                crate::notification_manager::send_confirmation_notification(&notification_pending)
            {
                log::error!("Failed to send confirmation notification: {}", e);
            } else {
                crate::experience_recorder::record_popup_displayed(
                    &trace_id,
                    &window_title,
                    &snapshot.window_info.process_name,
                    &content_text,
                );
            }

            system_action = "awaiting-confirmation".to_string();
            pending_confirmation = Some(PendingConfirmationInfo {
                trace_id,
                task_text: content_text,
                user_how: None,
                source_window: window_title.clone(),
                source_process: snapshot.window_info.process_name.clone(),
                created_at: chrono::Utc::now().timestamp(),
                process_context: Some(process_context.clone()),
                runtime_context: Some(popup_context.clone()),
            });

            if let Some(ref pc) = pending_confirmation {
                log::debug!(
                    "[MAIN_LOOP] Storing pending confirmation, trace_id={}",
                    pc.trace_id
                );
                let store_result = commands::store_current_task_safe(&app_handle, None);
                log::debug!("[MAIN_LOOP] store_current_task result: {:?}", store_result);
                let relay_result = commands::store_relay_execution_safe(&app_handle, None);
                log::debug!(
                    "[MAIN_LOOP] store_relay_execution result: {:?}",
                    relay_result
                );
                let confirm_result =
                    commands::store_pending_confirmation_safe(&app_handle, Some(pc.clone()));
                log::debug!(
                    "[MAIN_LOOP] store_pending_confirmation result: {:?}",
                    confirm_result
                );
            }
        } else {
            system_action = "notification_not_created".to_string();
            pending_confirmation = None;
            log::debug!("[MAIN_LOOP] Notification not created because raw_text was empty");
        }

        // Step 7: Reset to running state
        set_tray_state(&app_handle, TrayState::Idle);

        // Step 8: Send state update to frontend
        let last_judgment = JudgmentInfo {
            judgment: if raw_output_present {
                "raw_output_present=true"
            } else {
                "raw_output_present=false"
            }
            .to_string(),
            model_text: raw_output.raw_text.clone(),
            status_label: if raw_output_present {
                "notification_created"
            } else {
                "notification_not_created"
            }
            .to_string(),
            confidence_score: 0.0,
            grounds: raw_output.raw_text.clone(),
            system_action,
            process_context: Some(process_context.clone()),
        };
        commands::store_last_judgment(&app_handle, Some(last_judgment));
        commands::emit_state_update(&app_handle);

        // Step 9: Push snapshot to buffer (AFTER successful iteration completes)
        monitor.push_snapshot(
            snapshot.window_info.title.clone(),
            snapshot.window_info.process_name.clone(),
            snapshot.timestamp,
        );

        log::debug!(
            "[MAIN_LOOP] Iteration complete, entering sleep for {:?}",
            poll_interval
        );

        // Step 10: Wait for poll interval
        thread::sleep(poll_interval);
    }
}

/// Run update check in background
async fn run_update_check(app_handle: &AppHandle) -> Result<(), String> {
    use crate::updater::UpdateChecker;

    let current_version = env!("CARGO_PKG_VERSION");
    let checker = UpdateChecker::new(current_version.to_string());

    let response = checker.check().await?;

    // Update last_check_at in config
    let state = app_handle.state::<AppState>();
    let mut config = state.config.clone();
    config.last_check_at = Some(chrono::Utc::now().to_rfc3339());
    if let Err(e) = config.save() {
        log::warn!("Failed to save config after update check: {}", e);
    }

    if response.needs_update {
        log::info!(
            "Update available: {}, downloading...",
            response.latest_version
        );

        let download_url = response.download_url.clone();
        let signature = response.signature.clone();
        let app = app_handle.clone();

        tauri::async_runtime::spawn(async move {
            match checker.download(&download_url, &signature).await {
                Ok(msi_path) => match checker.install(&msi_path) {
                    Ok(_) => {
                        log::info!("Update installed, state set to Pending");
                        let state = app.state::<AppState>();
                        let update_state = UpdateState::Pending {
                            version: response.latest_version.clone(),
                            installed_at: chrono::Utc::now(),
                        };
                        *state.update_state.write().unwrap() = update_state.clone();
                        let _ = save_update_state(&update_state);
                        let _ = app.emit("update-ready", &response.latest_version);
                    }
                    Err(e) => log::error!("Update install failed: {}", e),
                },
                Err(e) => log::error!("Update download failed: {}", e),
            }
        });
    }

    Ok(())
}

/// Set the tray state in AppState
fn set_tray_state(app_handle: &AppHandle, state: TrayState) {
    let state_guard = app_handle.state::<AppState>();
    let mut guard = state_guard.tray_state.write().unwrap();
    *guard = state;
}

fn store_error_judgment(
    app_handle: &AppHandle,
    logger: &ActionLogger,
    last_error_signature: &mut Option<String>,
    error_kind: &str,
    window_title: &str,
    message: &str,
    process_context: Option<crate::window_monitor::ProcessContext>,
) {
    let judgment = JudgmentInfo {
        judgment: error_kind.to_string(),
        model_text: user_facing_error_text(error_kind, message),
        status_label: "ERROR".to_string(),
        confidence_score: 0.0,
        grounds: message.to_string(),
        system_action: "error".to_string(),
        process_context,
    };
    commands::store_last_judgment(app_handle, Some(judgment.clone()));

    let signature = format!("{}:{}:{}", error_kind, window_title, message);
    if last_error_signature.as_deref() == Some(signature.as_str()) {
        return;
    }

    *last_error_signature = Some(signature);
    let record = ActionRecord {
        timestamp: chrono::Utc::now().timestamp(),
        trace_id: None,
        session_id: None,
        window_title: window_title.to_string(),
        judgment: judgment.judgment,
        model_text: judgment.model_text,
        status_label: judgment.status_label,
        confidence_score: judgment.confidence_score,
        grounds: judgment.grounds,
        system_action: judgment.system_action,
        content_text: None,
        result_text: None,
        error_text: Some(message.to_string()),
        user_feedback: None,
        model_name: None,
        captured_at: None,
        call_started_at: None,
        call_duration_ms: None,
    };

    if let Err(e) = logger.log(record) {
        log::error!("Failed to log error action: {}", e);
    }
}

fn log_follow_up_action(
    logger: &ActionLogger,
    window_title: &str,
    judgment: &str,
    model_text: &str,
    system_action: &str,
) {
    let record = ActionRecord {
        timestamp: chrono::Utc::now().timestamp(),
        trace_id: None,
        session_id: None,
        window_title: window_title.to_string(),
        judgment: judgment.to_string(),
        model_text: model_text.to_string(),
        status_label: judgment.to_string(),
        confidence_score: 0.0,
        grounds: model_text.to_string(),
        system_action: system_action.to_string(),
        content_text: Some(model_text.to_string()),
        result_text: None,
        error_text: None,
        user_feedback: None,
        model_name: None,
        captured_at: None,
        call_started_at: None,
        call_duration_ms: None,
    };

    if let Err(e) = logger.log(record) {
        log::error!("Failed to log follow-up action: {}", e);
    }
}

fn user_facing_error_text(error_kind: &str, message: &str) -> String {
    match error_kind {
        "MODEL_ERROR" => format!(
            "模型调用失败，请检查 Ollama 地址、模型名称或服务状态：{}",
            message
        ),
        "CAPTURE_ERROR" => format!("窗口捕获失败，当前没有可分析的窗口信息：{}", message),
        "EXECUTOR_ERROR" => format!("结果执行失败，系统没有完成提示动作：{}", message),
        _ => message.to_string(),
    }
}
