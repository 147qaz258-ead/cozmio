use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

/// Worker status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerStatus {
    Disconnected,
    Connecting,
    Registering,
    Online,
    Error,
}

impl WorkerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkerStatus::Disconnected => "disconnected",
            WorkerStatus::Connecting => "connecting",
            WorkerStatus::Registering => "registering",
            WorkerStatus::Online => "online",
            WorkerStatus::Error => "error",
        }
    }
}

/// Thread-safe worker status holder
#[derive(Debug)]
pub struct WorkerStatusManager {
    status: Arc<AtomicU8>,
    registration_id: std::sync::Mutex<Option<i64>>,
}

impl WorkerStatusManager {
    pub fn new() -> Self {
        Self {
            status: Arc::new(AtomicU8::new(WorkerStatus::Disconnected as u8)),
            registration_id: std::sync::Mutex::new(None),
        }
    }

    pub fn set_status(&self, status: WorkerStatus) {
        self.status.store(status as u8, Ordering::SeqCst);
        log::info!("Worker status changed to: {:?}", status);
    }

    pub fn get_status(&self) -> WorkerStatus {
        WorkerStatus::from_u8(self.status.load(Ordering::SeqCst))
    }

    pub fn set_registration_id(&self, id: i64) {
        let mut guard = self.registration_id.lock().unwrap();
        *guard = Some(id);
        log::info!("Worker registered with ID: {}", id);
    }

    pub fn get_registration_id(&self) -> Option<i64> {
        self.registration_id.lock().unwrap().clone()
    }
}

impl Default for WorkerStatusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerStatus {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => WorkerStatus::Disconnected,
            1 => WorkerStatus::Connecting,
            2 => WorkerStatus::Registering,
            3 => WorkerStatus::Online,
            4 => WorkerStatus::Error,
            _ => WorkerStatus::Disconnected,
        }
    }
}

/// Model runtime status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelRuntimeStatus {
    Uninitialized,
    Loading,
    Ready,
    Busy,
    Error,
}

impl ModelRuntimeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelRuntimeStatus::Uninitialized => "uninitialized",
            ModelRuntimeStatus::Loading => "loading",
            ModelRuntimeStatus::Ready => "ready",
            ModelRuntimeStatus::Busy => "busy",
            ModelRuntimeStatus::Error => "error",
        }
    }
}
