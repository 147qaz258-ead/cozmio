use crate::error::CaptureError;
use crate::monitor::get_monitor_by_index;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::io::Cursor;

#[derive(Debug, Clone, Serialize)]
pub struct Screenshot {
    pub image_base64: String,
    pub monitor_index: u32,
    pub width: u32,
    pub height: u32,
    pub timestamp: i64,
}

impl Screenshot {
    pub fn capture(monitor_index: u32) -> Result<Self, CaptureError> {
        let monitor = get_monitor_by_index(monitor_index)?;

        let image = monitor
            .capture_image()
            .map_err(|e| CaptureError::ScreenshotFailed(e.to_string()))?;

        let mut buffer = Cursor::new(Vec::new());
        image
            .write_to(&mut buffer, ::image::ImageFormat::Png)
            .map_err(|e| CaptureError::EncodingError(e.to_string()))?;

        let image_base64 = STANDARD.encode(buffer.into_inner());
        let width = monitor.width() as u32;
        let height = monitor.height() as u32;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        Ok(Screenshot {
            image_base64,
            monitor_index,
            width,
            height,
            timestamp,
        })
    }
}
