use prost::Message;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SessionStatus {
    PENDING = 0,
    RUNNING = 1,
    WAITING = 2,
    COMPLETED = 3,
    FAILED = 4,
    INTERRUPTED = 5,
}

impl SessionStatus {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(SessionStatus::PENDING),
            1 => Some(SessionStatus::RUNNING),
            2 => Some(SessionStatus::WAITING),
            3 => Some(SessionStatus::COMPLETED),
            4 => Some(SessionStatus::FAILED),
            5 => Some(SessionStatus::INTERRUPTED),
            _ => None,
        }
    }
    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogLevel {
    INFO = 0,
    WARN = 1,
    ERROR = 2,
}

impl LogLevel {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(LogLevel::INFO),
            1 => Some(LogLevel::WARN),
            2 => Some(LogLevel::ERROR),
            _ => None,
        }
    }
    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct DispatchRequest {
    #[prost(string, tag = "1")]
    pub agent_name: String,
    #[prost(string, tag = "2")]
    pub original_suggestion: String,
    #[prost(string, tag = "3")]
    pub dispatched_task: String,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct DispatchResponse {
    #[prost(string, tag = "1")]
    pub session_id: String,
    #[prost(int32, tag = "2")]
    pub status: i32,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct StatusRequest {
    #[prost(string, tag = "1")]
    pub session_id: String,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct StatusResponse {
    #[prost(string, tag = "1")]
    pub session_id: String,
    #[prost(int32, tag = "2")]
    pub status: i32,
    #[prost(int64, tag = "3")]
    pub started_at: i64,
    #[prost(int64, tag = "4")]
    pub updated_at: i64,
    #[prost(int64, tag = "5")]
    pub duration_secs: i64,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct ProgressRequest {
    #[prost(string, tag = "1")]
    pub session_id: String,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct ProgressResponse {
    #[prost(string, tag = "1")]
    pub session_id: String,
    #[prost(message, repeated, tag = "2")]
    pub entries: Vec<ProgressEntry>,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct InterruptRequest {
    #[prost(string, tag = "1")]
    pub session_id: String,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct InterruptResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct ResultRequest {
    #[prost(string, tag = "1")]
    pub session_id: String,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct ResultResponse {
    #[prost(string, tag = "1")]
    pub session_id: String,
    #[prost(message, optional, tag = "2")]
    pub result: Option<ExecutionResult>,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct ProgressEvent {
    #[prost(string, tag = "1")]
    pub session_id: String,
    #[prost(int64, tag = "2")]
    pub timestamp: i64,
    #[prost(string, tag = "3")]
    pub message: String,
    #[prost(int32, tag = "4")]
    pub level: i32,
    #[prost(bool, tag = "5")]
    pub terminal: bool,
    #[prost(int32, tag = "6")]
    pub terminal_status: i32,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct ExecutionResult {
    #[prost(string, tag = "1")]
    pub summary: String,
    #[prost(string, tag = "2")]
    pub raw_output: String,
    #[prost(int64, tag = "3")]
    pub duration_secs: i64,
    #[prost(bool, tag = "4")]
    pub success: bool,
    #[prost(string, tag = "5")]
    pub error_message: String,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct ProgressEntry {
    #[prost(int64, tag = "1")]
    pub timestamp: i64,
    #[prost(string, tag = "2")]
    pub message: String,
    #[prost(int32, tag = "3")]
    pub level: i32,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct SubscribeRequest {
    #[prost(string, tag = "1")]
    pub session_id: String,
}

// =============================================================================
// Worker Protocol Messages (V9) - must match cozmio-box-worker
// =============================================================================

// Worker message types
pub const WORKER_REGISTER: u8 = 100;
pub const WORKER_HEARTBEAT: u8 = 101;
pub const INFERENCE_REQUEST: u8 = 102;
pub const INFERENCE_RESPONSE: u8 = 103;

#[derive(Clone, PartialEq, Eq, Message)]
pub struct WorkerRegisterRequest {
    #[prost(string, tag = "1")]
    pub worker_id: String,
    #[prost(string, tag = "2")]
    pub worker_type: String,
    #[prost(string, tag = "3")]
    pub endpoint: String,
    #[prost(int64, tag = "4")]
    pub heartbeat_interval_secs: i64,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct WorkerRegisterResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub error: String,
    #[prost(int64, tag = "3")]
    pub registration_id: i64,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct WorkerHeartbeat {
    #[prost(string, tag = "1")]
    pub worker_id: String,
    #[prost(string, tag = "2")]
    pub status: String,
    #[prost(message, tag = "3")]
    pub model_status: Option<ModelStatus>,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct ModelStatus {
    #[prost(string, tag = "1")]
    pub model_name: String,
    #[prost(bool, tag = "2")]
    pub loaded: bool,
    #[prost(int32, tag = "3")]
    pub memory_used_mb: i32,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct InferenceRequest {
    #[prost(string, tag = "1")]
    pub request_id: String,
    #[prost(string, tag = "2")]
    pub worker_id: String,
    #[prost(string, tag = "3")]
    pub context_bundle: String,
    #[prost(int64, tag = "4")]
    pub timeout_secs: i64,
}

#[derive(Clone, PartialEq, Eq, Message)]
pub struct InferenceResponse {
    #[prost(string, tag = "1")]
    pub request_id: String,
    #[prost(bool, tag = "2")]
    pub success: bool,
    #[prost(string, tag = "3")]
    pub payload_text: String,
    #[prost(string, tag = "4")]
    pub error: String,
}
