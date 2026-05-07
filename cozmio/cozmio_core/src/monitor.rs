use crate::error::CaptureError;
use crate::types::MonitorInfo;
use xcap::Monitor;

pub fn get_monitors() -> Result<Vec<MonitorInfo>, CaptureError> {
    let monitors = Monitor::all()?;

    monitors
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let x = m.x();
            let y = m.y();
            let width = m.width() as u32;
            let height = m.height() as u32;

            Ok(MonitorInfo {
                index: i as u32 + 1,
                width,
                height,
                x,
                y,
                name: format!("Monitor {}", i + 1),
            })
        })
        .collect()
}

pub fn get_monitor_by_index(index: u32) -> Result<Monitor, CaptureError> {
    let monitors = Monitor::all()?;
    let idx = index.saturating_sub(1) as usize;

    monitors
        .get(idx)
        .ok_or(CaptureError::MonitorNotFound(index))
        .map(|m| m.clone())
}
