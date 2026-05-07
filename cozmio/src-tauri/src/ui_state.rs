use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    pub running_state: String,
    pub tray_state: String,
    pub current_window: Option<WindowInfo>,
    pub last_judgment: Option<JudgmentInfo>,
    pub pending_confirmation: Option<PendingConfirmationInfo>,
    pub current_task: Option<CurrentTaskInfo>,
    pub relay_execution: Option<RelayExecutionInfo>,
    pub poll_interval_secs: u64,
    pub ollama_url: String,
    pub model_name: String,
    /// Source of the last inference result: "Local Agent Box" or "Local Mock" or null
    pub inference_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub title: String,
    pub process_name: String,
    pub process_id: u32,
    pub monitor_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgmentInfo {
    pub judgment: String,
    #[serde(alias = "next_step")]
    pub model_text: String,
    #[serde(alias = "level")]
    pub status_label: String,
    #[serde(alias = "confidence")]
    pub confidence_score: f32,
    pub grounds: String,
    pub system_action: String,
    pub process_context: Option<crate::window_monitor::ProcessContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingConfirmationInfo {
    pub trace_id: String,
    pub task_text: String,
    pub user_how: Option<String>,
    pub source_window: String,
    pub source_process: String,
    pub created_at: i64,
    pub process_context: Option<crate::window_monitor::ProcessContext>,
    pub runtime_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentTaskInfo {
    pub trace_id: String,
    pub task_text: String,
    pub source_window: String,
    pub source_process: String,
    pub created_at: i64,
    pub task_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayExecutionInfo {
    pub trace_id: Option<String>,
    pub transport: String,
    pub relay_status: String,
    pub session_id: Option<String>,
    pub progress: Vec<RelayProgressInfo>,
    pub result_summary: Option<String>,
    pub result_output: Option<String>,
    pub error_message: Option<String>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayProgressInfo {
    pub timestamp: i64,
    #[serde(alias = "level")]
    pub status_label: String,
    pub message: String,
}
