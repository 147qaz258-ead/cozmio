//! Floating mini window module
//! Creates a small always-on-top panel that shows the current agent state

use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

/// Creates the floating mini window as a borderless always-on-top panel.
/// Fails silently if the window already exists.
pub fn create_mini_window(app: &AppHandle) -> Result<(), String> {
    // Check if mini window already exists
    if app.get_webview_window("mini").is_some() {
        log::info!("[MINI_WINDOW] Already exists, skipping");
        return Ok(());
    }

    log::info!("[MINI_WINDOW] Creating floating mini window");

    let window = WebviewWindowBuilder::new(app, "mini", WebviewUrl::App("mini.html".into()))
        .title("Cozmio 工位")
        .inner_size(196.0, 148.0)
        .decorations(false)
        .always_on_top(true)
        .resizable(false)
        .visible(true)
        .skip_taskbar(true)
        .shadow(false)
        .build()
        .map_err(|e| format!("Failed to create mini window: {}", e))?;

    // Position window at bottom-right of the primary monitor
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let monitor_size = monitor.size();
        let x = monitor_size.width.saturating_sub(220) as i32;
        let y = monitor_size.height.saturating_sub(180) as i32;
        let _ = window.set_position(PhysicalPosition { x, y });
    }

    log::info!("[MINI_WINDOW] Created successfully");
    Ok(())
}
