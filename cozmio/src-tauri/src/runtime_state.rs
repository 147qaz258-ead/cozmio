use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;

static RUNTIME_STATE: OnceLock<Mutex<Option<File>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeState {
    pub running_state: String,        // "Running" | "Stopped"
    pub loop_tick_count: u64,         // 主循环迭代次数
    pub last_loop_at: Option<String>, // ISO8601 时间戳
    pub last_popup_requested_at: Option<String>,
    pub popup_count: u64, // 累计弹窗次数
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            running_state: "Stopped".to_string(),
            loop_tick_count: 0,
            last_loop_at: None,
            last_popup_requested_at: None,
            popup_count: 0,
        }
    }
}

fn get_state_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cozmio")
        .join("runtime_state.json")
}

pub fn read_state() -> RuntimeState {
    let path = get_state_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        RuntimeState::default()
    }
}

pub fn write_state(state: &RuntimeState) {
    if let Some(dir) = get_state_path().parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    let json = serde_json::to_string_pretty(state).unwrap();
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&get_state_path())
    {
        let _ = file.write_all(json.as_bytes());
    }
}

pub fn increment_tick() {
    let mut state = read_state();
    state.running_state = if crate::app_running::is_running() {
        "Running".to_string()
    } else {
        "Stopped".to_string()
    };
    state.loop_tick_count += 1;
    state.last_loop_at = Some(Local::now().to_rfc3339());
    write_state(&state);
}

pub fn record_popup() {
    let mut state = read_state();
    state.last_popup_requested_at = Some(Local::now().to_rfc3339());
    state.popup_count += 1;
    write_state(&state);
}
