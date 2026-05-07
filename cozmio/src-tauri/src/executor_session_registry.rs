use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutorSessionStatus {
    Dispatched,
    Running,
    Completed,
    Failed,
    Interrupted,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutorLogSource {
    pub source_type: String,
    pub locator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutorSessionSummary {
    pub cozmio_trace_id: String,
    pub relay_session_id: String,
    pub executor_target: String,
    pub dispatched_at: i64,
    pub last_activity_at: i64,
    pub task_brief: String,
    pub status: ExecutorSessionStatus,
    pub executor_native_session_id: Option<String>,
    pub log_source: Option<ExecutorLogSource>,
    pub result_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutorTurn {
    pub role: String,
    pub timestamp: i64,
    pub content: String,
    pub tool_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutorSessionDetail {
    pub summary: ExecutorSessionSummary,
    pub turns: Vec<ExecutorTurn>,
}

pub fn record_dispatched_session(
    app: &tauri::AppHandle,
    summary: ExecutorSessionSummary,
) -> Result<(), String> {
    let state = app.state::<crate::commands::AppState>();
    record_dispatched_session_in_state(state.inner(), summary)
}

pub fn update_session_status(
    app: &tauri::AppHandle,
    relay_session_id: &str,
    status: ExecutorSessionStatus,
    result_summary: Option<String>,
) -> Result<(), String> {
    let state = app.state::<crate::commands::AppState>();
    update_session_status_in_state(state.inner(), relay_session_id, status, result_summary)
}

pub fn list_tracked_sessions(app: &tauri::AppHandle, limit: usize) -> Vec<ExecutorSessionSummary> {
    let state = app.state::<crate::commands::AppState>();
    list_tracked_sessions_in_state(state.inner(), limit)
}

pub fn get_tracked_session_detail(
    app: &tauri::AppHandle,
    cozmio_trace_id: &str,
) -> Option<ExecutorSessionDetail> {
    let state = app.state::<crate::commands::AppState>();
    get_tracked_session_detail_in_state(state.inner(), cozmio_trace_id)
}

pub fn recent_executor_summary(app: &tauri::AppHandle, limit: usize) -> Option<String> {
    let state = app.state::<crate::commands::AppState>();
    recent_executor_summary_in_state(state.inner(), limit)
}

pub fn record_dispatched_session_in_state(
    state: &crate::commands::AppState,
    summary: ExecutorSessionSummary,
) -> Result<(), String> {
    if summary.relay_session_id.trim().is_empty() {
        return Err(String::from("relay_session_id is required"));
    }

    let mut sessions = state.executor_sessions.write().unwrap();
    sessions.retain(|session| session.relay_session_id != summary.relay_session_id);
    sessions.push(summary);
    sessions.sort_by(|a, b| b.last_activity_at.cmp(&a.last_activity_at));
    Ok(())
}

pub fn update_session_status_in_state(
    state: &crate::commands::AppState,
    relay_session_id: &str,
    status: ExecutorSessionStatus,
    result_summary: Option<String>,
) -> Result<(), String> {
    let mut sessions = state.executor_sessions.write().unwrap();
    let Some(session) = sessions
        .iter_mut()
        .find(|session| session.relay_session_id == relay_session_id)
    else {
        return Err(String::from("executor session not found"));
    };

    session.status = status;
    session.last_activity_at = now_ts();
    if result_summary
        .as_ref()
        .is_some_and(|summary| !summary.trim().is_empty())
    {
        session.result_summary = result_summary;
    }
    sessions.sort_by(|a, b| b.last_activity_at.cmp(&a.last_activity_at));
    Ok(())
}

pub fn list_tracked_sessions_in_state(
    state: &crate::commands::AppState,
    limit: usize,
) -> Vec<ExecutorSessionSummary> {
    state
        .executor_sessions
        .read()
        .unwrap()
        .iter()
        .take(limit)
        .cloned()
        .collect()
}

pub fn get_tracked_session_detail_in_state(
    state: &crate::commands::AppState,
    cozmio_trace_id: &str,
) -> Option<ExecutorSessionDetail> {
    state
        .executor_sessions
        .read()
        .unwrap()
        .iter()
        .find(|session| session.cozmio_trace_id == cozmio_trace_id)
        .cloned()
        .map(|summary| ExecutorSessionDetail {
            summary,
            turns: Vec::new(),
        })
}

pub fn recent_executor_summary_in_state(
    state: &crate::commands::AppState,
    limit: usize,
) -> Option<String> {
    let lines = state
        .executor_sessions
        .read()
        .unwrap()
        .iter()
        .take(limit)
        .map(|session| {
            let result = session
                .result_summary
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("尚无结果摘要");
            format!(
                "{} [{}] {} -> {}",
                session.relay_session_id,
                status_label(&session.status),
                clip(&session.task_brief, 120),
                clip(result, 160)
            )
        })
        .collect::<Vec<_>>();

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

pub fn status_from_relay_label(label: &str) -> ExecutorSessionStatus {
    match label {
        "completed" | "done" => ExecutorSessionStatus::Completed,
        "failed" | "error" | "dispatch_error" | "subscription_error" => {
            ExecutorSessionStatus::Failed
        }
        "interrupted" => ExecutorSessionStatus::Interrupted,
        "running" | "waiting" | "dispatching" | "connecting" => ExecutorSessionStatus::Running,
        _ => ExecutorSessionStatus::Unknown,
    }
}

fn status_label(status: &ExecutorSessionStatus) -> &'static str {
    match status {
        ExecutorSessionStatus::Dispatched => "dispatched",
        ExecutorSessionStatus::Running => "running",
        ExecutorSessionStatus::Completed => "completed",
        ExecutorSessionStatus::Failed => "failed",
        ExecutorSessionStatus::Interrupted => "interrupted",
        ExecutorSessionStatus::Unknown => "unknown",
    }
}

fn clip(value: &str, max_chars: usize) -> String {
    let mut clipped: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        clipped.push_str("...");
    }
    clipped.replace('\n', " ")
}

fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> crate::commands::AppState {
        let temp_dir = std::env::temp_dir().join(format!(
            "cozmio_executor_session_registry_{}",
            uuid::Uuid::new_v4()
        ));
        let ledger = crate::ledger::LedgerManager::new(temp_dir).unwrap();
        crate::commands::AppState::new(ledger)
    }

    fn summary(trace: &str, session: &str, at: i64) -> ExecutorSessionSummary {
        ExecutorSessionSummary {
            cozmio_trace_id: trace.to_string(),
            relay_session_id: session.to_string(),
            executor_target: String::from("claude-code"),
            dispatched_at: at,
            last_activity_at: at,
            task_brief: String::from("检查观察交接链路"),
            status: ExecutorSessionStatus::Dispatched,
            executor_native_session_id: None,
            log_source: None,
            result_summary: None,
        }
    }

    #[test]
    fn records_dispatched_session_in_tracked_registry() {
        let state = test_state();

        record_dispatched_session_in_state(&state, summary("trace-1", "relay-1", 10)).unwrap();

        let sessions = list_tracked_sessions_in_state(&state, 30);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].cozmio_trace_id, "trace-1");
        assert_eq!(sessions[0].relay_session_id, "relay-1");
    }

    #[test]
    fn update_by_relay_session_changes_status_and_activity() {
        let state = test_state();
        record_dispatched_session_in_state(&state, summary("trace-1", "relay-1", 1)).unwrap();

        update_session_status_in_state(
            &state,
            "relay-1",
            ExecutorSessionStatus::Completed,
            Some(String::from("Task completed successfully")),
        )
        .unwrap();

        let sessions = list_tracked_sessions_in_state(&state, 30);
        assert_eq!(sessions[0].status, ExecutorSessionStatus::Completed);
        assert!(sessions[0].last_activity_at >= sessions[0].dispatched_at);
        assert_eq!(
            sessions[0].result_summary.as_deref(),
            Some("Task completed successfully")
        );
    }

    #[test]
    fn recent_executor_summary_uses_only_tracked_sessions() {
        let state = test_state();
        record_dispatched_session_in_state(&state, summary("trace-1", "relay-1", 1)).unwrap();

        let summary = recent_executor_summary_in_state(&state, 3).unwrap();

        assert!(summary.contains("relay-1"));
        assert!(summary.contains("检查观察交接链路"));
        assert!(!summary.contains(".claude"));
        assert!(!summary.contains("projects"));
    }

    #[test]
    fn detail_returns_empty_turns_until_native_log_is_linked() {
        let state = test_state();
        record_dispatched_session_in_state(&state, summary("trace-1", "relay-1", 1)).unwrap();

        let detail = get_tracked_session_detail_in_state(&state, "trace-1").unwrap();

        assert_eq!(detail.summary.relay_session_id, "relay-1");
        assert!(detail.turns.is_empty());
    }
}
