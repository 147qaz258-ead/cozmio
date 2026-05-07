use crate::protocol::{
    send_request, ModelStatus, ProtocolError, WorkerHeartbeat, WORKER_HEARTBEAT,
};
use crate::status::{ModelRuntimeStatus, WorkerStatus, WorkerStatusManager};
use prost::Message;
use std::net::TcpStream;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Heartbeat manager that periodically sends heartbeats to the relay
pub struct HeartbeatManager {
    worker_id: String,
    status: Arc<WorkerStatusManager>,
    model_status: Arc<AtomicU8>,
    heartbeat_interval_secs: u64,
}

impl HeartbeatManager {
    pub fn new(
        worker_id: String,
        status: Arc<WorkerStatusManager>,
        heartbeat_interval_secs: u64,
    ) -> Self {
        Self {
            worker_id,
            status,
            model_status: Arc::new(AtomicU8::new(ModelRuntimeStatus::Uninitialized as u8)),
            heartbeat_interval_secs,
        }
    }

    pub fn set_model_status(&self, model_status: ModelRuntimeStatus) {
        self.model_status
            .store(model_status as u8, Ordering::SeqCst);
    }

    /// Start the heartbeat loop in a background thread
    pub fn start(&self, relay_addr: String, registration_id: i64) {
        let worker_id = self.worker_id.clone();
        let interval = self.heartbeat_interval_secs;
        let status = self.status.clone();
        let model_status = self.model_status.clone();

        thread::spawn(move || {
            log::info!(
                "Heartbeat loop started (interval={}s, registration_id={})",
                interval,
                registration_id
            );

            loop {
                // Check if we're still online
                if status.get_status() != WorkerStatus::Online {
                    log::warn!("Worker status is not Online, stopping heartbeat loop");
                    break;
                }

                match Self::send_heartbeat(&relay_addr, &worker_id, &model_status) {
                    Ok(()) => {
                        log::debug!("Heartbeat sent successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to send heartbeat: {}", e);
                        status.set_status(WorkerStatus::Error);
                    }
                }

                thread::sleep(Duration::from_secs(interval));
            }

            log::info!("Heartbeat loop exited");
        });
    }

    fn send_heartbeat(
        relay_addr: &str,
        worker_id: &str,
        model_status: &Arc<AtomicU8>,
    ) -> Result<(), ProtocolError> {
        let model_status_val = ModelRuntimeStatus::from_u8(model_status.load(Ordering::SeqCst));

        let model_name = match model_status_val {
            ModelRuntimeStatus::Ready => "mock-model".to_string(),
            _ => "mock-model".to_string(),
        };

        let (loaded, memory_mb) = match model_status_val {
            ModelRuntimeStatus::Ready => (true, 0),
            ModelRuntimeStatus::Loading => (false, 0),
            _ => (false, 0),
        };

        let heartbeat = WorkerHeartbeat {
            worker_id: worker_id.to_string(),
            status: model_status_val.as_str().to_string(),
            model_status: Some(ModelStatus {
                model_name,
                loaded,
                memory_used_mb: memory_mb,
            }),
        };

        let mut payload = Vec::new();
        heartbeat.encode(&mut payload)?;

        let mut stream = TcpStream::connect(relay_addr)?;
        let _response = send_request(&mut stream, WORKER_HEARTBEAT, &payload)?;

        Ok(())
    }
}

impl ModelRuntimeStatus {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => ModelRuntimeStatus::Uninitialized,
            1 => ModelRuntimeStatus::Loading,
            2 => ModelRuntimeStatus::Ready,
            3 => ModelRuntimeStatus::Busy,
            4 => ModelRuntimeStatus::Error,
            _ => ModelRuntimeStatus::Uninitialized,
        }
    }
}
