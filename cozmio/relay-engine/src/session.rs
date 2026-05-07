use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Pending,
    Running,
    Waiting,
    Completed,
    Failed,
    Interrupted,
}

impl SessionStatus {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            SessionStatus::Completed | SessionStatus::Failed | SessionStatus::Interrupted
        )
    }
}

impl From<i32> for SessionStatus {
    fn from(v: i32) -> Self {
        match v {
            0 => SessionStatus::Pending,
            1 => SessionStatus::Running,
            2 => SessionStatus::Waiting,
            3 => SessionStatus::Completed,
            4 => SessionStatus::Failed,
            5 => SessionStatus::Interrupted,
            _ => {
                log::warn!("Unknown SessionStatus value: {}, defaulting to Pending", v);
                SessionStatus::Pending
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl From<i32> for LogLevel {
    fn from(v: i32) -> Self {
        match v {
            0 => LogLevel::Info,
            1 => LogLevel::Warn,
            2 => LogLevel::Error,
            _ => {
                log::warn!("Unknown LogLevel value: {}, defaulting to Info", v);
                LogLevel::Info
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEntry {
    pub timestamp: i64,
    pub message: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub summary: String,
    pub raw_output: String,
    pub duration_secs: i64,
    pub success: bool,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct ExecutionTask {
    pub original_suggestion: String,
    pub dispatched_task: String,
    pub agent_name: String,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub task: ExecutionTask,
    pub status: SessionStatus,
    pub progress_logs: Vec<ProgressEntry>,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub final_result: Option<ExecutionResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    pub fn new() -> Self {
        SessionId(Uuid::new_v4().to_string())
    }

    pub fn from_string(s: String) -> Self {
        SessionId(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct SessionManager {
    sessions: parking_lot::RwLock<std::collections::HashMap<SessionId, Session>>,
    pub(crate) processes: parking_lot::RwLock<std::collections::HashMap<u32, SessionId>>,
    interrupt_requested: parking_lot::RwLock<HashSet<SessionId>>,
    subscribers: Arc<RwLock<HashMap<SessionId, Vec<Arc<dyn SessionSubscriber>>>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: parking_lot::RwLock::new(std::collections::HashMap::new()),
            processes: parking_lot::RwLock::new(std::collections::HashMap::new()),
            interrupt_requested: parking_lot::RwLock::new(HashSet::new()),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_session(&self, task: ExecutionTask) -> SessionId {
        let id = SessionId::new();
        let now = Utc::now();
        let session = Session {
            id: id.clone(),
            task,
            status: SessionStatus::Pending,
            progress_logs: Vec::new(),
            started_at: now,
            updated_at: now,
            final_result: None,
        };
        self.sessions.write().insert(id.clone(), session);
        id
    }

    pub fn get_session(&self, session_id: &SessionId) -> Option<Session> {
        self.sessions.read().get(session_id).cloned()
    }

    pub fn update_status(&self, session_id: &SessionId, status: SessionStatus) {
        let mut should_notify_terminal = false;
        if let Some(session) = self.sessions.write().get_mut(session_id) {
            if session.status != status && status.is_terminal() {
                should_notify_terminal = true;
            }
            session.status = status;
            session.updated_at = Utc::now();
        }
        if should_notify_terminal {
            self.notify_terminal(session_id, status);
        }
    }

    pub fn add_progress(&self, session_id: &SessionId, entry: ProgressEntry) {
        if let Some(session) = self.sessions.write().get_mut(session_id) {
            session.progress_logs.push(entry.clone());
            session.updated_at = Utc::now();
        }
        // Notify subscribers about new progress
        self.notify_progress(session_id, &entry);
    }

    pub fn set_result(&self, session_id: &SessionId, result: ExecutionResult) {
        if let Some(session) = self.sessions.write().get_mut(session_id) {
            session.final_result = Some(result);
            session.updated_at = Utc::now();
        }
    }

    pub fn get_status(&self, session_id: &SessionId) -> Option<SessionStatus> {
        self.sessions.read().get(session_id).map(|s| s.status)
    }

    pub fn get_progress(&self, session_id: &SessionId) -> Option<Vec<ProgressEntry>> {
        self.sessions
            .read()
            .get(session_id)
            .map(|s| s.progress_logs.clone())
    }

    pub fn get_result(&self, session_id: &SessionId) -> Option<ExecutionResult> {
        self.sessions
            .read()
            .get(session_id)
            .and_then(|s| s.final_result.clone())
    }

    pub fn register_process(&self, pid: u32, session_id: SessionId) {
        self.processes.write().insert(pid, session_id);
    }

    pub fn unregister_process(&self, pid: &u32) {
        self.processes.write().remove(pid);
    }

    pub fn mark_interrupt_requested(&self, session_id: &SessionId) {
        self.interrupt_requested.write().insert(session_id.clone());
    }

    pub fn clear_interrupt_requested(&self, session_id: &SessionId) {
        self.interrupt_requested.write().remove(session_id);
    }

    pub fn is_interrupt_requested(&self, session_id: &SessionId) -> bool {
        self.interrupt_requested.read().contains(session_id)
    }

    #[allow(dead_code)]
    pub fn get_session_by_pid(&self, pid: &u32) -> Option<SessionId> {
        self.processes.read().get(pid).cloned()
    }

    /// Subscribe to progress events for a session
    pub fn subscribe(&self, session_id: SessionId, subscriber: Arc<dyn SessionSubscriber>) {
        let (buffered_entries, terminal_status) = {
            let mut subs = self.subscribers.write().unwrap();
            let Some(session) = self.sessions.read().get(&session_id).cloned() else {
                return;
            };
            subs.entry(session_id.clone())
                .or_insert_with(Vec::new)
                .push(subscriber.clone());
            (session.progress_logs, session.status)
        };

        for entry in buffered_entries {
            subscriber.on_progress(&session_id, &entry);
        }

        if terminal_status.is_terminal() {
            subscriber.on_terminal(&session_id, terminal_status);
        }
    }

    /// Unsubscribe a specific subscriber
    pub fn unsubscribe(&self, session_id: &SessionId, subscriber_id: &str) {
        let mut subs = self.subscribers.write().unwrap();
        if let Some(session_subs) = subs.get_mut(session_id) {
            session_subs.retain(|s| s.subscriber_id() != subscriber_id);
        }
    }

    /// Notify all subscribers of a session about new progress
    pub fn notify_progress(&self, session_id: &SessionId, entry: &ProgressEntry) {
        let subscribers = {
            let subs = self.subscribers.read().unwrap();
            subs.get(session_id).cloned().unwrap_or_default()
        };

        for sub in subscribers {
            sub.on_progress(session_id, entry);
        }
    }

    fn notify_terminal(&self, session_id: &SessionId, status: SessionStatus) {
        let subscribers = {
            let subs = self.subscribers.read().unwrap();
            subs.get(session_id).cloned().unwrap_or_default()
        };

        for sub in subscribers {
            sub.on_terminal(session_id, status);
        }
    }
}

/// Trait for session progress subscribers
pub trait SessionSubscriber: Send + Sync {
    fn subscriber_id(&self) -> &str;
    fn on_progress(&self, session_id: &SessionId, entry: &ProgressEntry);
    fn on_terminal(&self, session_id: &SessionId, status: SessionStatus);
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
