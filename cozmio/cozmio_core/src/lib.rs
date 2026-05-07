use serde::Serialize;

pub mod error;
pub mod monitor;
pub mod screenshot;
pub mod types;
pub mod window;

pub use error::CaptureError;
pub use monitor::{get_monitor_by_index, get_monitors};
pub use screenshot::Screenshot;
pub use types::{MonitorInfo, Rect, WindowInfo};
pub use window::{get_all_windows, get_foreground_window_info, get_window_by_hwnd};

#[derive(Debug, Clone, Serialize)]
pub struct CaptureAllResult {
    pub screenshot: Option<Screenshot>,
    pub foreground_window: Option<WindowInfo>,
    pub all_windows: WindowListResult,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowListResult {
    pub count: usize,
    pub windows: Vec<WindowInfo>,
    pub timestamp: i64,
}

pub fn capture_all(monitor_index: u32) -> Result<CaptureAllResult, CaptureError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let screenshot = Screenshot::capture(monitor_index).ok();
    let foreground_window = get_foreground_window_info().ok();
    let all_windows = get_all_windows()?;

    Ok(CaptureAllResult {
        screenshot,
        foreground_window,
        all_windows: WindowListResult {
            count: all_windows.len(),
            windows: all_windows,
            timestamp,
        },
        timestamp,
    })
}
