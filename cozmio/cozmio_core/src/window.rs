use crate::error::CaptureError;
use crate::monitor::get_monitors;
use crate::types::{Rect, WindowInfo};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowLongPtrW, GetWindowRect, GetWindowTextW,
    GetWindowThreadProcessId, IsWindowVisible, GWL_STYLE, WS_CHILD,
};

fn wide_to_string(ptr: &[u16]) -> String {
    if ptr.is_empty() {
        return String::new();
    }
    let len = ptr.iter().position(|&c| c == 0).unwrap_or(ptr.len());
    OsString::from_wide(&ptr[..len])
        .to_string_lossy()
        .into_owned()
}

fn get_window_title(hwnd: HWND) -> String {
    unsafe {
        let mut buffer = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buffer);
        wide_to_string(&buffer[..len as usize])
    }
}

fn get_process_name(hwnd: HWND) -> (String, u32) {
    unsafe {
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        let process_name = if process_id == 0 {
            String::new()
        } else {
            let handle = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                BOOL::from(false),
                process_id,
            );
            match handle {
                Ok(h) => {
                    let mut buffer = [0u16; 260]; // MAX_PATH
                    let handle = windows::Win32::Foundation::HANDLE(h.0);
                    let len = GetModuleBaseNameW(handle, None, &mut buffer);
                    if len > 0 {
                        wide_to_string(&buffer[..len as usize])
                    } else {
                        format!("PID:{}", process_id)
                    }
                }
                Err(_) => format!("PID:{}", process_id),
            }
        };

        (process_name, process_id)
    }
}

fn get_window_rect(hwnd: HWND) -> Rect {
    unsafe {
        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);
        Rect {
            x: rect.left,
            y: rect.top,
            width: (rect.right - rect.left) as u32,
            height: (rect.bottom - rect.top) as u32,
        }
    }
}

fn is_window_visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd).as_bool() }
}

fn is_child_window(hwnd: HWND) -> bool {
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        (style & WS_CHILD.0) != 0
    }
}

fn monitor_index_from_point(x: i32, y: i32) -> u32 {
    let monitors = get_monitors().unwrap_or_default();
    for m in monitors {
        if x >= m.x && x < m.x + m.width as i32 && y >= m.y && y < m.y + m.height as i32 {
            return m.index;
        }
    }
    1
}

pub fn get_foreground_window_info() -> Result<WindowInfo, CaptureError> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return Err(CaptureError::WindowFailed(
                "No foreground window".to_string(),
            ));
        }

        let title = get_window_title(hwnd);
        let (process_name, process_id) = get_process_name(hwnd);
        let rect = get_window_rect(hwnd);
        let monitor_index = monitor_index_from_point(rect.x, rect.y);

        Ok(WindowInfo {
            hwnd: hwnd.0 as u64,
            title,
            process_name,
            process_id,
            monitor_index,
            rect,
            is_foreground: true,
            is_visible: is_window_visible(hwnd),
            z_order: 0,
        })
    }
}

pub fn get_window_info(hwnd: HWND, z_order: u32) -> Option<WindowInfo> {
    if hwnd.0.is_null() || is_child_window(hwnd) {
        return None;
    }

    let title = get_window_title(hwnd);
    if title.is_empty() {
        return None;
    }

    let (process_name, process_id) = get_process_name(hwnd);
    let rect = get_window_rect(hwnd);
    let is_visible = is_window_visible(hwnd);
    let monitor_index = monitor_index_from_point(rect.x, rect.y);

    Some(WindowInfo {
        hwnd: hwnd.0 as u64,
        title,
        process_name,
        process_id,
        monitor_index,
        rect,
        is_foreground: false,
        is_visible,
        z_order,
    })
}

pub fn get_all_windows() -> Result<Vec<WindowInfo>, CaptureError> {
    struct Context {
        windows: Vec<WindowInfo>,
        current_z: u32,
    }

    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let ctx = &mut *(lparam.0 as *mut Context);

        if let Some(info) = get_window_info(hwnd, ctx.current_z) {
            ctx.windows.push(info);
            ctx.current_z += 1;
        }

        BOOL::from(true)
    }

    let mut ctx = Context {
        windows: Vec::new(),
        current_z: 1,
    };

    unsafe {
        EnumWindows(
            Some(enum_callback),
            LPARAM(&mut ctx as *mut Context as isize),
        )
        .map_err(|e| CaptureError::Win32Error(e.to_string()))?;
    }

    Ok(ctx.windows)
}

pub fn get_window_by_hwnd(hwnd_val: isize) -> Result<WindowInfo, CaptureError> {
    let hwnd = HWND(hwnd_val as *mut std::ffi::c_void);
    if hwnd.0.is_null() {
        return Err(CaptureError::WindowFailed("Invalid HWND".to_string()));
    }

    get_window_info(hwnd, 0)
        .ok_or_else(|| CaptureError::WindowFailed("Window not found".to_string()))
}
