use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceId(pub String);

impl TraceId {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random = (now as u64 ^ (now >> 64) as u64).wrapping_mul(0x5DEECE66D);
        Self(format!("{:016x}-{:016x}", now as u64, random))
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Dispatching,
    Running,
    Completed,
    Failed,
    Interrupted,
    Cancelled,
    Dismissed,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Dispatching => write!(f, "dispatching"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Interrupted => write!(f, "interrupted"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
            TaskStatus::Dismissed => write!(f, "dismissed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub trace_id: String,
    pub status: TaskStatus,
    pub session_id: Option<String>,
    pub content_text: String,
    pub result_text: Option<String>,
    pub error_text: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TaskState {
    pub fn new(trace_id: String, content_text: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            trace_id,
            status: TaskStatus::Pending,
            session_id: None,
            content_text,
            result_text: None,
            error_text: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmToken(pub String);

impl ConfirmToken {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random = (now as u64 ^ (now >> 64) as u64).wrapping_mul(0x5DEECE66D);
        Self(format!("{:016x}{:016x}", now as u64, random))
    }
}

impl Default for ConfirmToken {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPending {
    pub trace_id: String,
    pub token: ConfirmToken,
    pub content_text: String,
    pub user_how: Option<String>,
    pub created_at: i64,
}

impl NotificationPending {
    pub fn new(trace_id: String, content_text: String, user_how: Option<String>) -> Self {
        Self {
            trace_id,
            token: ConfirmToken::new(),
            content_text,
            user_how,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn to_protocol_url(&self, action: &str) -> String {
        format!(
            "cozmio://{action}?trace_id={}&token={}",
            urlencoding_encode(&self.trace_id),
            urlencoding_encode(&self.token.0),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceCard {
    pub source: String,
    pub ref_id: String,
    pub age_label: Option<String>,
    pub short_summary: String,
    pub why_maybe_relevant: Option<String>,
    pub similarity_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextAdmissionLineage {
    pub candidate_pool_size: usize,
    pub evidence_cards_selected: usize,
    pub selected_card_refs: Vec<String>,
    pub not_selected_reason: String,
    pub model_input_packet_summary: String,
}

fn urlencoding_encode(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => ch.to_string(),
            _ => format!("%{:02X}", ch as u8),
        })
        .collect()
}
