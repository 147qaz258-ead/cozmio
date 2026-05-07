use crate::proto::{ModelStatus, WorkerHeartbeat};
use chrono::Utc;
use parking_lot::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerState {
    Online,
    Idle,
    Busy,
    Offline,
}

impl From<&str> for WorkerState {
    fn from(s: &str) -> Self {
        match s {
            "online" => WorkerState::Online,
            "idle" => WorkerState::Idle,
            "busy" => WorkerState::Busy,
            _ => WorkerState::Offline,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkerInfo {
    pub worker_id: String,
    pub worker_type: String,
    pub endpoint: String,
    pub registration_id: i64,
    pub heartbeat_interval_secs: i64,
    pub state: WorkerState,
    pub model_status: Option<ModelStatus>,
    pub last_heartbeat: chrono::DateTime<Utc>,
}

impl WorkerInfo {
    pub fn new(
        worker_id: String,
        worker_type: String,
        endpoint: String,
        heartbeat_interval_secs: i64,
        registration_id: i64,
    ) -> Self {
        WorkerInfo {
            worker_id,
            worker_type,
            endpoint,
            registration_id,
            heartbeat_interval_secs,
            state: WorkerState::Online,
            model_status: None,
            last_heartbeat: Utc::now(),
        }
    }

    pub fn update_heartbeat(&mut self, heartbeat: WorkerHeartbeat) {
        self.state = WorkerState::from(heartbeat.status.as_str());
        self.model_status = heartbeat.model_status;
        self.last_heartbeat = Utc::now();
    }

    pub fn is_online(&self) -> bool {
        self.state != WorkerState::Offline
    }

    pub fn is_idle(&self) -> bool {
        self.state == WorkerState::Idle
    }

    pub fn is_busy(&self) -> bool {
        self.state == WorkerState::Busy
    }
}

/// Thread-safe worker registry
pub struct WorkerRegistry {
    workers: RwLock<HashMap<String, WorkerInfo>>,
    next_registration_id: RwLock<i64>,
}

impl WorkerRegistry {
    pub fn new() -> Self {
        WorkerRegistry {
            workers: RwLock::new(HashMap::new()),
            next_registration_id: RwLock::new(1),
        }
    }

    /// Register a new worker
    pub fn register(
        &self,
        worker_id: String,
        worker_type: String,
        endpoint: String,
        heartbeat_interval_secs: i64,
    ) -> i64 {
        let registration_id = {
            let mut next_id = self.next_registration_id.write();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let worker_info = WorkerInfo::new(
            worker_id.clone(),
            worker_type,
            endpoint,
            heartbeat_interval_secs,
            registration_id,
        );

        self.workers.write().insert(worker_id, worker_info);
        log::info!("Worker registered with registration_id={}", registration_id);
        registration_id
    }

    /// Unregister a worker
    pub fn unregister(&self, worker_id: &str) -> Option<WorkerInfo> {
        self.workers.write().remove(worker_id)
    }

    /// Update worker heartbeat
    pub fn update_heartbeat(&self, worker_id: &str, heartbeat: WorkerHeartbeat) -> bool {
        if let Some(worker) = self.workers.write().get_mut(worker_id) {
            worker.update_heartbeat(heartbeat);
            true
        } else {
            false
        }
    }

    /// Update only the last_heartbeat timestamp (used by simplified heartbeat monitor)
    pub fn update_heartbeat_timestamp(&self, worker_id: &str) -> bool {
        if let Some(worker) = self.workers.write().get_mut(worker_id) {
            worker.last_heartbeat = Utc::now();
            true
        } else {
            false
        }
    }

    /// Get worker info by worker_id
    pub fn get(&self, worker_id: &str) -> Option<WorkerInfo> {
        self.workers.read().get(worker_id).cloned()
    }

    /// Get all online workers
    pub fn get_online_workers(&self) -> Vec<WorkerInfo> {
        self.workers
            .read()
            .values()
            .filter(|w| w.is_online())
            .cloned()
            .collect()
    }

    /// Get all idle workers (suitable for inference)
    pub fn get_idle_workers(&self) -> Vec<WorkerInfo> {
        self.workers
            .read()
            .values()
            .filter(|w| w.is_idle())
            .cloned()
            .collect()
    }

    /// Get worker by registration_id
    pub fn get_by_registration_id(&self, registration_id: i64) -> Option<WorkerInfo> {
        self.workers
            .read()
            .values()
            .find(|w| w.registration_id == registration_id)
            .cloned()
    }

    /// Mark worker as busy
    pub fn mark_busy(&self, worker_id: &str) -> bool {
        if let Some(worker) = self.workers.write().get_mut(worker_id) {
            worker.state = WorkerState::Busy;
            true
        } else {
            false
        }
    }

    /// Mark worker as idle
    pub fn mark_idle(&self, worker_id: &str) -> bool {
        if let Some(worker) = self.workers.write().get_mut(worker_id) {
            worker.state = WorkerState::Idle;
            true
        } else {
            false
        }
    }

    /// Get count of online workers
    pub fn online_count(&self) -> usize {
        self.workers
            .read()
            .values()
            .filter(|w| w.is_online())
            .count()
    }

    /// Get count of idle workers
    pub fn idle_count(&self) -> usize {
        self.workers.read().values().filter(|w| w.is_idle()).count()
    }
}

impl Default for WorkerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_state_from_str() {
        assert_eq!(WorkerState::from("online"), WorkerState::Online);
        assert_eq!(WorkerState::from("idle"), WorkerState::Idle);
        assert_eq!(WorkerState::from("busy"), WorkerState::Busy);
        assert_eq!(WorkerState::from("unknown"), WorkerState::Offline);
    }
}
