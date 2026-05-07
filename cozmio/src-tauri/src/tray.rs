use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub type TrayIcon = tauri::tray::TrayIcon;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrayState {
    Idle,
    Processing,
}

impl Default for TrayState {
    fn default() -> Self {
        Self::Idle
    }
}

pub struct TrayManager {
    state: TrayState,
}

impl TrayManager {
    pub fn new() -> Self {
        Self {
            state: TrayState::default(),
        }
    }

    pub fn state(&self) -> TrayState {
        self.state
    }

    pub fn set_state(&mut self, state: TrayState) {
        self.state = state;
    }

    pub fn setup_tray(app: &AppHandle) -> Result<TrayIcon, String> {
        let show = MenuItem::with_id(app, "show", "打开主界面", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        let separator1 =
            tauri::menu::PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
        let status = MenuItem::with_id(app, "status", "● Stopped", false, None::<&str>)
            .map_err(|e| e.to_string())?;
        let separator2 =
            tauri::menu::PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
        let start = MenuItem::with_id(app, "start", "启动", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        let stop = MenuItem::with_id(app, "stop", "停止", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        let separator3 =
            tauri::menu::PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
        let open_log = MenuItem::with_id(app, "open_log", "打开日志目录", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        let separator4 =
            tauri::menu::PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
        let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
            .map_err(|e| e.to_string())?;

        let menu = Menu::with_items(
            app,
            &[
                &show,
                &separator1,
                &status,
                &separator2,
                &start,
                &stop,
                &separator3,
                &open_log,
                &separator4,
                &quit,
            ],
        )
        .map_err(|e| e.to_string())?;

        TrayIconBuilder::new()
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(move |app, event| match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "start" => {
                    crate::app_running::set_running(true);
                    let _ = app.emit("running-state-changed", "Running");
                }
                "stop" => {
                    crate::app_running::set_running(false);
                    let _ = app.emit("running-state-changed", "Stopped");
                }
                "open_log" => {
                    let log_dir = dirs::data_local_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("cozmio");
                    let _ = std::process::Command::new("explorer").arg(&log_dir).spawn();
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            })
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            })
            .build(app)
            .map_err(|e| e.to_string())
    }
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn update_tray_icon(_app: &AppHandle, state: &str) {
    let icon_name = match state {
        "idle" => "tray-green.png",
        "monitoring" | "analyzing" => "tray-blue.png",
        "confirm" | "executing" => "tray-orange.png",
        "error" | "failed" => "tray-red.png",
        _ => "tray-green.png",
    };
    log::debug!("[TRAY] State {} maps to {}", state, icon_name);
}
