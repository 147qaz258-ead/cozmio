use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub hwnd: u64,
    pub title: String,
    pub process_name: String,
    pub process_id: u32,
    pub monitor_index: u32,
    pub rect: Rect,
    pub is_foreground: bool,
    pub is_visible: bool,
    pub z_order: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonitorInfo {
    pub index: u32,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub name: String,
}
