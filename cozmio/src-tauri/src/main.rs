#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_running;
mod commands;
mod config;
#[cfg(test)]
mod executor;
mod experience_recorder;
mod logging;
mod main_loop;
mod memory_commands;
mod memory_consolidation;
mod mini_window;
mod model_client;
mod notification_manager;
mod prompt_context;
mod protocol_handler;
mod relay_bridge;
mod runtime_state;
mod toast_verification;
mod tray;
mod types;
mod ui_state;
mod updater;
mod window_monitor;

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

fn get_log_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cozmio")
        .join("cozmio.log")
}

fn setup_file_logging() {
    // 创建日志目录
    if let Some(log_dir) = get_log_path().parent() {
        let _ = std::fs::create_dir_all(log_dir);
    }

    // 使用 Mutex 包装文件句柄，确保线程安全
    static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

    // 打开文件用于写入（截断模式，每次启动清空）
    if let Ok(file) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&get_log_path())
    {
        LOG_FILE.set(Mutex::new(file)).ok();
    }

    // 初始化 env_logger，将所有日志写入文件
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format(move |buf, record| {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let msg = format!(
                "{} [{}] {} - {}:{} | {}\n",
                timestamp,
                record.level(),
                record.target(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            );

            // 同时写入文件（如果文件可用）
            if let Some(file) = LOG_FILE.get() {
                if let Ok(mut f) = file.lock() {
                    let _ = f.write_all(msg.as_bytes());
                    let _ = f.flush();
                }
            }

            // 同时输出到 stderr（开发时可见）
            writeln!(buf, "{}", msg.trim())
        })
        .init();

    log::info!("File logging initialized at {:?}", get_log_path());
}

pub fn get_app_handle() -> Option<&'static tauri::AppHandle> {
    APP_HANDLE.get()
}

use commands::AppState;

fn main() {
    // Set up panic hook to log panics to file before process terminates
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        let location = if let Some(loc) = panic_info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "unknown location".to_string()
        };
        let log_msg = format!("[PANIC] {} at {}", msg, location);
        // Try to write to log file directly
        if let Some(log_path) = dirs::data_local_dir() {
            let log_file = log_path.join("cozmio").join("cozmio.log");
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file)
            {
                use std::io::Write;
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                let _ = writeln!(file, "{} [PANIC] {} at {}", timestamp, msg, location);
            }
        }
        eprintln!("{}", log_msg);
    }));

    setup_file_logging(); // 初始化文件日志
    log::info!("[MAIN] File logging setup complete, starting cozmio::run()");
    cozmio::run()
}

use crate::config::Config;
use crate::logging::ActionLogger;
use crate::tray::TrayManager;

mod cozmio {
    use super::*;
    use commands::{
        cancel_pending_task, cancel_pending_task_by_token, check_for_updates, clear_history,
        confirm_pending_task, confirm_pending_task_by_token, dismiss_pending_task,
        dismiss_update_reminder, get_config, get_history, get_running_state, get_runtime_state,
        get_tray_state, get_ui_state, get_update_state, get_verification_result, hide_main_window,
        interrupt_current_task, list_models, mini_action, reset_verification, restart_application,
        run_relay_demo, save_config, send_verification_toast, set_tray_state, show_main_window,
        start_running, stop_running,
    };
    use memory_commands::{
        add_decision, build_activity_context, get_decision_memory, get_memory_stats,
        get_skill_memory, get_task_threads, import_existing_logs, run_suggestion_replay,
        search_memory, update_task_thread,
    };
    use memory_consolidation::{
        apply_memory_operation, build_memory_consolidation_prompt, build_replay_comparison_report,
        get_hot_context, get_memory_inspector_snapshot, get_privacy_routing_status,
        get_procedure_recall_stats, increment_procedure_use_count, list_agent_memories,
        prepare_memory_consolidation, reject_agent_memory, replay_traces,
        run_manual_memory_consolidation, set_hot_context,
    };
    use tauri::Manager;
    use tauri_plugin_deep_link::DeepLinkExt;

    pub fn run() {
        log::info!("[RUN] Starting tauri builder");
        tauri::Builder::default()
            .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
                // Focus existing window when another instance launches
                if let Some(url) = args.iter().find(|arg| arg.starts_with("cozmio://")) {
                    log::info!(
                        "Single-instance deep-link activation received without forcing main window: {}",
                        url
                    );
                    if let Some(action) =
                        crate::protocol_handler::PendingProtocolAction::from_url(url)
                    {
                        process_protocol_action(app.clone(), action);
                    } else {
                        log::warn!("Failed to parse single-instance deep link URL: {}", url);
                    }
                    return;
                }
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }))
            .plugin(tauri_plugin_shell::init())
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_notification::init())
            .plugin(tauri_plugin_deep_link::init())
            .manage(AppState::new())
            .invoke_handler(tauri::generate_handler![
                get_config,
                save_config,
                get_history,
                clear_history,
                get_tray_state,
                set_tray_state,
                start_running,
                stop_running,
                get_running_state,
                show_main_window,
                hide_main_window,
                get_runtime_state,
                get_ui_state,
                list_models,
                run_relay_demo,
                confirm_pending_task,
                cancel_pending_task,
                dismiss_pending_task,
                interrupt_current_task,
                send_verification_toast,
                get_verification_result,
                reset_verification,
                confirm_pending_task_by_token,
                cancel_pending_task_by_token,
                get_memory_stats,
                import_existing_logs,
                search_memory,
                build_activity_context,
                run_suggestion_replay,
                get_task_threads,
                update_task_thread,
                get_decision_memory,
                add_decision,
                get_skill_memory,
                prepare_memory_consolidation,
                build_memory_consolidation_prompt,
                run_manual_memory_consolidation,
                apply_memory_operation,
                list_agent_memories,
                get_memory_inspector_snapshot,
                get_privacy_routing_status,
                reject_agent_memory,
                replay_traces,
                build_replay_comparison_report,
                get_hot_context,
                set_hot_context,
                get_procedure_recall_stats,
                increment_procedure_use_count,
                mini_action,
                get_update_state,
                check_for_updates,
                restart_application,
                dismiss_update_reminder,
            ])
            .setup(|app| {
                log::info!("[SETUP] Starting setup block");
                APP_HANDLE.set(app.handle().clone()).ok();
                log::info!("[SETUP] App handle set");
                log::info!("Cozmio starting...");

                // Initialize runtime state file
                crate::runtime_state::write_state(&crate::runtime_state::RuntimeState::default());
                // Setup system tray - store the tray handle for dynamic icon updates
                match TrayManager::setup_tray(app.handle()) {
                    Ok(tray_icon) => {
                        log::info!("[SETUP] Tray icon created with handle");
                        // Store the tray handle in AppState for dynamic icon updates
                        let state = app.state::<crate::commands::AppState>();
                        state.tray_handle.write().unwrap().replace(tray_icon);
                    }
                    Err(e) => {
                        log::error!("Failed to setup tray: {}", e);
                    }
                }

                if let Err(e) = mini_window::create_mini_window(app.handle()) {
                    log::error!("Failed to create mini window: {}", e);
                } else {
                    log::info!("[SETUP] Mini window created");
                }

                // Register protocol handler for cozmio:// URLs
                #[cfg(desktop)]
                {
                    log::info!("[SETUP] Desktop block entered");
                    let handle = app.handle().clone();
                    log::info!("[SETUP] Handle cloned");

                    if let Err(e) = handle.deep_link().register("cozmio") {
                        log::warn!("Failed to register cozmio:// protocol: {}", e);
                    } else {
                        log::info!("Registered cozmio:// protocol handler");
                    }

                    // Listen for deep link events (protocol activation)
                    let event_handle = handle.clone();
                    handle.clone().deep_link().on_open_url(move |event| {
                        let urls = event.urls();
                        log::info!("Deep link event received: {:?}", urls);
                        for url in urls {
                            let url_str: String = url.to_string();
                            log::info!("Processing deep link URL: {}", url_str);

                            // Parse the protocol URL
                            if let Some(action) = crate::protocol_handler::PendingProtocolAction::from_url(&url_str) {
                                log::info!("Parsed action={} trace_id={} token_present={}",
                                    action.action,
                                    action.trace_id,
                                    action.token.is_some()
                                );
                                process_protocol_action(event_handle.clone(), action);
                            } else {
                                log::warn!("Failed to parse deep link URL: {}", url_str);
                            }
                        }
                    });

                    // Check if app was launched with a deep link URL (cold start case)
                    if let Ok(Some(urls)) = handle.deep_link().get_current() {
                        log::info!("App launched with deep link URLs: {:?}", urls);
                        for url in urls {
                            let url_str: String = url.to_string();
                            if let Some(action) = crate::protocol_handler::PendingProtocolAction::from_url(&url_str) {
                                log::info!(
                                    "Cold start protocol action {} for trace_id={} token_present={}",
                                    action.action,
                                    action.trace_id,
                                    action.token.is_some()
                                );
                                process_protocol_action(handle.clone(), action);
                            }
                        }
                    }
                }

                // Load config and start main loop
                let config = Config::load().unwrap_or_default();
                let logger = ActionLogger::new();

                log::info!(
                    "Main loop starting with poll_interval={}s, model={}",
                    config.poll_interval_secs,
                    config.model_name
                );

                // Spawn main loop in background thread
                let app_handle = app.handle().clone();
                let loop_handle = std::thread::spawn(move || {
                    main_loop::start_main_loop(app_handle, config, logger);
                });
                // Thread handle is dropped immediately - we don't join
                // Panic catching is done INSIDE start_main_loop via catch_unwind
                log::info!("[MAIN] Main loop thread spawned successfully");

                Ok(())
            })
            .on_window_event(|window, event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    // Hide window instead of closing (close button -> hide to tray)
                    let _ = window.hide();
                    api.prevent_close();
                }
            })
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }

    fn process_protocol_action(
        app_handle: tauri::AppHandle,
        action: crate::protocol_handler::PendingProtocolAction,
    ) {
        match action.action.as_str() {
            "confirm" => {
                let Some(token) = action.token.clone() else {
                    log::warn!(
                        "PROTOCOL: Confirm action missing token trace_id={}",
                        action.trace_id
                    );
                    return;
                };
                let trace_id = action.trace_id.clone();
                tauri::async_runtime::spawn(async move {
                    match crate::commands::confirm_pending_task_by_token(
                        app_handle,
                        trace_id.clone(),
                        token,
                    ) {
                        Ok(session_id) => log::info!(
                            "PROTOCOL: Confirm processed trace_id={} session_id={}",
                            trace_id,
                            session_id
                        ),
                        Err(error) => log::warn!(
                            "PROTOCOL: Confirm rejected trace_id={} error={}",
                            trace_id,
                            error
                        ),
                    }
                });
            }
            "cancel" => {
                let Some(token) = action.token.clone() else {
                    log::warn!(
                        "PROTOCOL: Cancel action missing token trace_id={}",
                        action.trace_id
                    );
                    return;
                };
                let trace_id = action.trace_id.clone();
                tauri::async_runtime::spawn(async move {
                    match crate::commands::cancel_pending_task_by_token(
                        app_handle,
                        trace_id.clone(),
                        token,
                    ) {
                        Ok(()) => log::info!("PROTOCOL: Cancel processed trace_id={}", trace_id),
                        Err(error) => log::warn!(
                            "PROTOCOL: Cancel rejected trace_id={} error={}",
                            trace_id,
                            error
                        ),
                    }
                });
            }
            "action" => {
                log::info!(
                    "PROTOCOL: Toast body clicked trace_id={} token_present={}",
                    action.trace_id,
                    action.token.is_some()
                );
            }
            other => {
                log::warn!("PROTOCOL: Unknown action {}", other);
            }
        }
    }
}
