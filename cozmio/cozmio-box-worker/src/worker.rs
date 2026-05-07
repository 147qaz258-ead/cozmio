use crate::box_model_runtime::BoxModelRuntime;
use crate::config::Config;
use crate::heartbeat::HeartbeatManager;
use crate::protocol::{
    send_request, send_response, InferenceRequest, InferenceResponse, WorkerRegisterRequest,
    WorkerRegisterResponse, INFERENCE_REQUEST, INFERENCE_RESPONSE, WORKER_REGISTER,
};
use crate::status::{WorkerStatus, WorkerStatusManager};
use prost::Message;
use std::io::Read;
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Worker state machine
pub struct Worker {
    config: Config,
    status: Arc<WorkerStatusManager>,
    heartbeat_manager: HeartbeatManager,
    box_model_runtime: BoxModelRuntime,
    stream: Option<TcpStream>,
}

impl Worker {
    pub fn new(config: Config) -> Self {
        let status = Arc::new(WorkerStatusManager::new());
        let heartbeat_manager = HeartbeatManager::new(
            config.worker_id.clone(),
            status.clone(),
            config.heartbeat_interval_secs,
        );
        let box_model_runtime = BoxModelRuntime::new();

        Self {
            config,
            status,
            heartbeat_manager,
            box_model_runtime,
            stream: None,
        }
    }

    /// Connect to relay and register
    pub fn connect(&mut self) -> Result<(), WorkerError> {
        log::info!("Connecting to relay at {}...", self.config.relay_addr);
        self.status.set_status(WorkerStatus::Connecting);

        // Try to connect
        let mut stream = TcpStream::connect(&self.config.relay_addr)
            .map_err(|e| WorkerError::ConnectionFailed(e.to_string()))?;

        log::info!("Connected to relay, registering...");
        self.status.set_status(WorkerStatus::Registering);

        // Build registration request
        let register_req = WorkerRegisterRequest {
            worker_id: self.config.worker_id.clone(),
            worker_type: self.config.worker_type.clone(),
            endpoint: format!("{}:0", "0.0.0.0"), // Placeholder endpoint
            heartbeat_interval_secs: self.config.heartbeat_interval_secs as i64,
        };

        let mut payload = Vec::new();
        register_req.encode(&mut payload).map_err(|e| {
            WorkerError::Protocol(format!("Failed to encode register request: {}", e))
        })?;

        // Send registration request
        let response_bytes = send_request(&mut stream, WORKER_REGISTER, &payload)
            .map_err(|e| WorkerError::Protocol(format!("Send request failed: {}", e)))?;

        // Decode response
        let response = WorkerRegisterResponse::decode(&response_bytes[..]).map_err(|e| {
            WorkerError::Protocol(format!("Failed to decode register response: {}", e))
        })?;

        if !response.success {
            return Err(WorkerError::RegistrationFailed(response.error));
        }

        log::info!(
            "Worker registered successfully, registration_id={}",
            response.registration_id
        );

        self.status.set_registration_id(response.registration_id);
        self.status.set_status(WorkerStatus::Online);

        // Load the model via BoxModelRuntime
        if let Err(e) = self.box_model_runtime.load() {
            log::error!("Failed to load model: {}", e);
            return Err(WorkerError::Config(format!("Model load failed: {}", e)));
        }

        // Set model status from BoxModelRuntime
        self.heartbeat_manager
            .set_model_status(self.box_model_runtime.status());

        // Start heartbeat loop
        self.heartbeat_manager
            .start(self.config.relay_addr.clone(), response.registration_id);

        // Store the stream for receiving inference requests
        self.stream = Some(stream);

        Ok(())
    }

    pub fn status(&self) -> Arc<WorkerStatusManager> {
        self.status.clone()
    }

    /// Run inference via BoxModelRuntime (Phase 2 uses MockProvider)
    /// Returns (output_text, duration_ms)
    pub fn infer(
        &self,
        context: &str,
        image_data: Option<&[u8]>,
        timeout_secs: u64,
    ) -> Result<(String, u64), crate::model_provider::ProviderError> {
        self.box_model_runtime.infer(context, image_data, timeout_secs)
    }

    /// Run the main loop - handle incoming messages from relay
    pub fn run(&mut self) -> Result<(), WorkerError> {
        let mut stream = self
            .stream
            .take()
            .ok_or_else(|| WorkerError::Protocol("No stream available".to_string()))?;

        log::info!("Worker main loop started, waiting for inference requests...");

        loop {
            // Read frame from relay
            let (msg_type, payload) = match read_frame(&mut stream) {
                Ok((t, p)) => (t, p),
                Err(e) => {
                    log::error!("Failed to read frame: {}", e);
                    break;
                }
            };

            match msg_type {
                INFERENCE_REQUEST => {
                    log::debug!("Received INFERENCE_REQUEST");
                    if let Err(e) = self.handle_inference_request(&mut stream, &payload) {
                        log::error!("Failed to handle inference request: {}", e);
                    }
                }
                _ => {
                    log::warn!("Unknown message type: {}", msg_type);
                }
            }
        }

        log::info!("Worker main loop exited");
        Ok(())
    }

    /// Handle an incoming INFERENCE_REQUEST message
    fn handle_inference_request(
        &self,
        stream: &mut TcpStream,
        payload: &[u8],
    ) -> Result<(), WorkerError> {
        // Parse InferenceRequest
        let request = InferenceRequest::decode(payload).map_err(|e| {
            WorkerError::Protocol(format!("Failed to decode inference request: {}", e))
        })?;

        let request_id = &request.request_id;
        let context_bundle = &request.context_bundle;
        let timeout_secs = request.timeout_secs as u64;

        log::info!(
            "Box inference: generating response [trace_id={}]",
            request_id
        );

        let start = Instant::now();

        // Run inference
        let image_data = if !request.image_data.is_empty() {
            Some(&request.image_data[..])
        } else {
            None
        };
        let result = self.infer(context_bundle, image_data, timeout_secs);
        let duration_ms = start.elapsed().as_millis() as u64;

        // Build response
        let response = match result {
            Ok((output_text, _duration_ms)) => {
                log::info!(
                    "Box inference: completed [trace_id={}, duration={}ms, output_chars={}]",
                    request_id,
                    duration_ms,
                    output_text.chars().count()
                );
                InferenceResponse {
                    request_id: request_id.clone(),
                    success: true,
                    payload_text: output_text,
                    error: String::new(),
                }
            }
            Err(e) => {
                log::error!(
                    "Box inference: failed [trace_id={}, duration={}ms, error={}]",
                    request_id,
                    duration_ms,
                    e
                );
                InferenceResponse {
                    request_id: request_id.clone(),
                    success: false,
                    payload_text: String::new(),
                    error: e.to_string(),
                }
            }
        };

        // Encode and send response
        let mut response_payload = Vec::new();
        response.encode(&mut response_payload).map_err(|e| {
            WorkerError::Protocol(format!("Failed to encode inference response: {}", e))
        })?;

        send_response(stream, INFERENCE_RESPONSE, &response_payload).map_err(|e| {
            WorkerError::Protocol(format!("Failed to send inference response: {}", e))
        })?;

        Ok(())
    }
}

/// Read a framed message from a TCP stream
fn read_frame(stream: &mut TcpStream) -> Result<(u8, Vec<u8>), WorkerError> {
    // Read 4-byte length prefix
    let mut len_bytes = [0u8; 4];
    stream
        .read_exact(&mut len_bytes)
        .map_err(|e| WorkerError::Protocol(format!("Failed to read length prefix: {}", e)))?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    if len > 1_000_000 {
        return Err(WorkerError::Protocol(format!(
            "Frame too large: {} bytes",
            len
        )));
    }

    // Read 1-byte message type
    let mut type_bytes = [0u8; 1];
    stream
        .read_exact(&mut type_bytes)
        .map_err(|e| WorkerError::Protocol(format!("Failed to read message type: {}", e)))?;
    let msg_type = type_bytes[0];

    // Read payload
    let mut payload = vec![0u8; len];
    stream
        .read_exact(&mut payload)
        .map_err(|e| WorkerError::Protocol(format!("Failed to read payload: {}", e)))?;

    Ok((msg_type, payload))
}
