use crate::error::{Error, Result};
use crate::proto::{
    WorkerHeartbeat, WorkerRegisterRequest, WorkerRegisterResponse, INFERENCE_REQUEST,
    WORKER_HEARTBEAT, WORKER_REGISTER,
};
use crate::worker_registry::WorkerRegistry;
use chrono::Utc;
use prost::Message;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;

/// Manages worker sessions and their TCP connections
/// Thread-safe wrapper using interior mutability
pub struct WorkerSessionManager {
    registry: Arc<WorkerRegistry>,
    /// Map of worker_id -> Arc<Mutex<TcpStream>> for active connections
    worker_streams: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,
    /// Map of worker_id -> heartbeat task handle for cancellation
    heartbeat_tasks: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl WorkerSessionManager {
    pub fn new(registry: Arc<WorkerRegistry>) -> Self {
        WorkerSessionManager {
            registry,
            worker_streams: Arc::new(Mutex::new(HashMap::new())),
            heartbeat_tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Handle a new worker connection - register and start heartbeat monitoring
    pub fn handle_new_worker(
        &self,
        mut stream: TcpStream,
        registry: &Arc<WorkerRegistry>,
    ) -> Result<String> {
        // Read registration request
        let (request_type, payload) = read_typed_request(&mut stream)?;

        if request_type != WORKER_REGISTER {
            return Err(Error::Protocol(format!(
                "Expected WORKER_REGISTER (100), got {}",
                request_type
            )));
        }

        let register_req = WorkerRegisterRequest::decode(&payload[..]).map_err(|e| {
            Error::Protocol(format!("Failed to decode WorkerRegisterRequest: {}", e))
        })?;

        log::info!(
            "Worker registration request: worker_id={}, worker_type={}",
            register_req.worker_id,
            register_req.worker_type
        );

        // Register the worker (without stream - we store it separately)
        let registration_id = registry.register(
            register_req.worker_id.clone(),
            register_req.worker_type.clone(),
            register_req.endpoint.clone(),
            register_req.heartbeat_interval_secs,
        );

        // Store the stream wrapped in Arc<Mutex<...>>
        let stream_arc = Arc::new(Mutex::new(stream));
        self.worker_streams
            .lock()
            .map_err(|e| Error::Protocol(format!("Mutex lock error: {}", e)))?
            .insert(register_req.worker_id.clone(), stream_arc.clone());

        // Send registration response
        let response = WorkerRegisterResponse {
            success: true,
            error: String::new(),
            registration_id,
        };

        let mut response_bytes = Vec::new();
        response.encode(&mut response_bytes).map_err(|e| {
            Error::Protocol(format!("Failed to encode WorkerRegisterResponse: {}", e))
        })?;

        // Lock stream and send response
        let mut stream_lock = stream_arc
            .lock()
            .map_err(|e| Error::Protocol(format!("Mutex lock error: {}", e)))?;

        // Send response using the same framing: 4 bytes length + payload
        let len_bytes = (response_bytes.len() as u32).to_be_bytes();
        stream_lock
            .write_all(&len_bytes)
            .map_err(|e| Error::Io(e))?;
        stream_lock
            .write_all(&response_bytes)
            .map_err(|e| Error::Io(e))?;
        stream_lock.flush().map_err(|e| Error::Io(e))?;

        drop(stream_lock); // Release lock before returning

        log::info!(
            "Worker {} registered successfully, registration_id={}",
            register_req.worker_id,
            registration_id
        );

        Ok(register_req.worker_id)
    }

    /// Start heartbeat monitoring for a worker
    /// Simplified: Only tracks last_heartbeat timestamp via the registry's update_heartbeat method.
    /// Does NOT read from the stream to avoid race conditions with the message loop reader.
    /// Workers are considered offline if they miss 3 consecutive heartbeat intervals.
    pub fn start_heartbeat_monitor(
        &self,
        worker_id: String,
        _stream_arc: Arc<Mutex<TcpStream>>,
        interval_secs: i64,
    ) {
        let registry = self.registry.clone();
        let worker_id_clone = worker_id.clone();
        let heartbeat_tasks = self.heartbeat_tasks.clone();

        let handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(interval_secs as u64));
            loop {
                interval.tick().await;

                // Simply update heartbeat timestamp via registry.
                // The registry tracks last_heartbeat and check_workers_connected() will
                // mark workers offline if they miss 3 consecutive intervals.
                if registry.update_heartbeat_timestamp(&worker_id_clone) {
                    log::debug!("Worker {} heartbeat timestamp updated", worker_id_clone);
                } else {
                    log::warn!(
                        "Worker {} not found in registry for heartbeat update",
                        worker_id_clone
                    );
                    // Worker not found means it was unregistered, stop monitoring
                    break;
                }
            }

            // Remove heartbeat task when done
            if let Ok(mut tasks) = heartbeat_tasks.lock() {
                tasks.remove(&worker_id_clone);
            }
        });

        if let Ok(mut tasks) = self.heartbeat_tasks.lock() {
            tasks.insert(worker_id.clone(), handle);
        }
        log::info!("Started heartbeat monitor for worker: {}", worker_id);
    }

    /// Stop heartbeat monitoring for a worker
    pub fn stop_heartbeat_monitor(&self, worker_id: &str) {
        if let Ok(mut tasks) = self.heartbeat_tasks.lock() {
            if let Some(handle) = tasks.remove(worker_id) {
                handle.abort();
                log::info!("Stopped heartbeat monitor for worker: {}", worker_id);
            }
        }
    }

    /// Handle worker disconnection
    pub fn handle_disconnect(&self, worker_id: &str) {
        log::info!("Worker {} disconnected", worker_id);
        self.stop_heartbeat_monitor(worker_id);
        if let Ok(mut streams) = self.worker_streams.lock() {
            streams.remove(worker_id);
        }
        if let Some(worker) = self.registry.unregister(worker_id) {
            log::info!(
                "Worker {} unregistered, was in state {:?}",
                worker_id,
                worker.state
            );
        }
    }

    /// Get a cloned Arc<Mutex<TcpStream>> for a worker
    pub fn get_stream_arc(&self, worker_id: &str) -> Option<Arc<Mutex<TcpStream>>> {
        self.worker_streams
            .lock()
            .ok()
            .and_then(|streams| streams.get(worker_id).cloned())
    }

    /// Send inference request to worker (does NOT wait for response - response is read by handle_worker_messages_for_worker)
    pub fn send_inference_request(
        &self,
        worker_id: &str,
        request_id: &str,
        context_bundle: &str,
        timeout_secs: i64,
    ) -> Result<()> {
        let stream_arc = self
            .get_stream_arc(worker_id)
            .ok_or_else(|| Error::Protocol(format!("Worker {} not connected", worker_id)))?;

        let mut stream = stream_arc
            .lock()
            .map_err(|e| Error::Protocol(format!("Mutex lock error: {}", e)))?;

        let request = crate::proto::InferenceRequest {
            request_id: request_id.to_string(),
            worker_id: worker_id.to_string(),
            context_bundle: context_bundle.to_string(),
            timeout_secs,
        };

        let mut payload = Vec::new();
        request
            .encode(&mut payload)
            .map_err(|e| Error::Protocol(e.to_string()))?;

        // Send request using the same framing: 4 bytes length + type byte + payload
        let total_len = (payload.len() as u32).to_be_bytes();
        stream.write_all(&total_len).map_err(|e| Error::Io(e))?;
        stream
            .write_all(&[INFERENCE_REQUEST])
            .map_err(|e| Error::Io(e))?;
        stream.write_all(&payload).map_err(|e| Error::Io(e))?;
        stream.flush().map_err(|e| Error::Io(e))?;

        log::debug!(
            "Sent inference request to worker {}: request_id={}",
            worker_id,
            request_id
        );

        Ok(())
    }

    /// Check if workers are still connected (heartbeat-based)
    pub fn check_workers_connected(&self) {
        let now = Utc::now();
        let timeout = chrono::Duration::seconds(90); // 3x heartbeat interval (30s)

        for worker in self.registry.get_online_workers() {
            if now - worker.last_heartbeat > timeout {
                log::warn!(
                    "Worker {} missed heartbeats, marking offline",
                    worker.worker_id
                );
                self.handle_disconnect(&worker.worker_id);
            }
        }
    }

    /// Revert worker state to idle (used when send fails after mark_busy was called)
    pub fn revert_worker_state(&self, worker_id: &str) -> bool {
        self.registry.mark_idle(worker_id)
    }
}

/// Send a typed request and receive a response
fn send_request(
    stream: &mut TcpStream,
    request_type: u8,
    payload: &[u8],
) -> std::result::Result<Vec<u8>, std::io::Error> {
    // Frame format: 4 bytes big-endian length + 1 byte type + payload
    let total_len = (payload.len() as u32).to_be_bytes();

    // Write length prefix (4 bytes)
    stream.write_all(&total_len)?;

    // Write request type (1 byte)
    stream.write_all(&[request_type])?;

    // Write payload
    stream.write_all(payload)?;

    stream.flush()?;

    // Read response length prefix (4 bytes)
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    if len > 1_000_000 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Response too large: {} bytes", len),
        ));
    }

    // Read response payload
    let mut response = vec![0u8; len];
    stream.read_exact(&mut response)?;

    Ok(response)
}

/// Read a typed request from a TCP stream
fn read_typed_request(stream: &mut TcpStream) -> Result<(u8, Vec<u8>)> {
    // Read length prefix (4 bytes)
    let mut len_bytes = [0u8; 4];
    stream
        .read_exact(&mut len_bytes)
        .map_err(|e| Error::Io(e))?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    if len > 1_000_000 {
        return Err(Error::Protocol(format!("Message too large: {} bytes", len)));
    }

    // Read type byte (1 byte)
    let mut type_byte = [0u8; 1];
    stream
        .read_exact(&mut type_byte)
        .map_err(|e| Error::Io(e))?;
    let request_type = type_byte[0];

    // Read payload
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).map_err(|e| Error::Io(e))?;

    Ok((request_type, payload))
}

/// Try to read a heartbeat from a worker (non-blocking)
fn read_heartbeat(stream: &mut TcpStream) -> std::result::Result<Option<WorkerHeartbeat>, Error> {
    // Set stream to non-blocking temporarily
    stream
        .set_nonblocking(true)
        .map_err(|e| Error::Transport(e.to_string()))?;

    let result = {
        let mut type_byte = [0u8; 1];
        match stream.read_exact(&mut type_byte) {
            Ok(_) => {
                // Got type byte, now read the rest
                if type_byte[0] == WORKER_HEARTBEAT {
                    // Read length prefix
                    let mut len_bytes = [0u8; 4];
                    if stream.read_exact(&mut len_bytes).is_ok() {
                        let len = u32::from_be_bytes(len_bytes) as usize;
                        if len <= 1_000_000 {
                            let mut payload = vec![0u8; len];
                            if stream.read_exact(&mut payload).is_ok() {
                                match WorkerHeartbeat::decode(&payload[..]) {
                                    Ok(heartbeat) => Ok(Some(heartbeat)),
                                    Err(e) => Err(Error::Protocol(format!(
                                        "Failed to decode heartbeat: {}",
                                        e
                                    ))),
                                }
                            } else {
                                Ok(None)
                            }
                        } else {
                            Err(Error::Protocol(format!(
                                "Heartbeat payload too large: {}",
                                len
                            )))
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(Error::Io(e)),
        }
    };

    // Restore blocking mode
    stream
        .set_nonblocking(false)
        .map_err(|e| Error::Transport(e.to_string()))?;

    result
}
