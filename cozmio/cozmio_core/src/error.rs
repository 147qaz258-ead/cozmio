use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("截图失败: {0}")]
    ScreenshotFailed(String),

    #[error("窗口获取失败: {0}")]
    WindowFailed(String),

    #[error("显示器 {0} 不存在")]
    MonitorNotFound(u32),

    #[error("Win32 API 调用失败: {0}")]
    Win32Error(String),

    #[error("编码失败: {0}")]
    EncodingError(String),

    #[error("显示器枚举失败: {0}")]
    MonitorEnumFailed(String),
}

impl From<xcap::XCapError> for CaptureError {
    fn from(e: xcap::XCapError) -> Self {
        CaptureError::MonitorEnumFailed(e.to_string())
    }
}
