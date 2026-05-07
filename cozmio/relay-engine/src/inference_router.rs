use crate::error::{Error, Result};
use crate::proto::{InferenceRequest, InferenceResponse};
use crate::worker_registry::WorkerRegistry;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

/// Pending inference request awaiting response
struct PendingRequest {
    request: InferenceRequest,
    created_at: chrono::DateTime<Utc>,
    response_sender: oneshot::Sender<InferenceResponse>,
}

/// Routing strategy for selecting a worker
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingStrategy {
    /// Route to any available worker
    Any,
    /// Route to specific worker by ID
    Specific,
    /// Route to worker with specific model
    ModelSpecific(String),
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        RoutingStrategy::Any
    }
}

/// Inference router - routes inference requests to Box Workers
pub struct InferenceRouter {
    registry: Arc<WorkerRegistry>,
    /// Pending requests awaiting response
    pending_requests: parking_lot::RwLock<HashMap<String, PendingRequest>>,
    /// Channel to send requests to worker handler
    worker_tx: mpsc::Sender<WorkerCommand>,
}

impl InferenceRouter {
    pub fn new(registry: Arc<WorkerRegistry>, worker_tx: mpsc::Sender<WorkerCommand>) -> Self {
        InferenceRouter {
            registry,
            pending_requests: parking_lot::RwLock::new(HashMap::new()),
            worker_tx,
        }
    }

    /// Route an inference request to an appropriate worker
    pub async fn route_inference(
        &self,
        context_bundle: String,
        timeout_secs: i64,
        strategy: RoutingStrategy,
    ) -> Result<InferenceResponse> {
        let worker_id = match strategy {
            RoutingStrategy::Any => self.select_worker()?,
            RoutingStrategy::Specific => {
                return Err(Error::Protocol("Specific worker ID required".to_string()));
            }
            RoutingStrategy::ModelSpecific(_) => {
                return Err(Error::Protocol(
                    "Model-specific routing not yet implemented".to_string(),
                ));
            }
        };

        self.send_inference_request(worker_id, context_bundle, image_data, timeout_secs)
            .await
    }

    /// Send an inference request to a specific worker
    pub async fn send_inference_request(
        &self,
        worker_id: String,
        context_bundle: String,
        timeout_secs: i64,
    ) -> Result<InferenceResponse> {
        let request_id = Uuid::new_v4().to_string();

        let request = InferenceRequest {
            request_id: request_id.clone(),
            worker_id: worker_id.clone(),
            context_bundle: context_bundle.clone(),
            timeout_secs,
        };

        // Create oneshot channel for response
        let (response_sender, response_receiver) = oneshot::channel();

        // Store pending request
        {
            let pending = PendingRequest {
                request: request.clone(),
                created_at: Utc::now(),
                response_sender,
            };
            self.pending_requests
                .write()
                .insert(request_id.clone(), pending);
        }

        // Mark worker as busy
        self.registry.mark_busy(&worker_id);

        // Send request to worker handler via channel
        // The worker_tx receiver in main.rs will forward to worker_session_manager
        let cmd = WorkerCommand::InferenceRequest {
            request_id: request_id.clone(),
            worker_id: worker_id.clone(),
            context_bundle: context_bundle.clone(),
            timeout_secs,
        };

        self.worker_tx
            .send(cmd)
            .await
            .map_err(|e| Error::Transport(e.to_string()))?;

        log::debug!(
            "Sent inference request to worker {}: request_id={}",
            worker_id,
            request_id
        );

        // Wait for response with timeout via the stored response_receiver
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs as u64),
            response_receiver,
        )
        .await;

        // Mark worker as idle again (regardless of outcome)
        self.registry.mark_idle(&worker_id);

        // Remove pending request (regardless of outcome)
        self.pending_requests.write().remove(&request_id);

        match response {
            Ok(Ok(resp)) => Ok(resp),
            Ok(Err(e)) => Err(Error::Protocol(format!("Response receiver error: {}", e))),
            Err(_) => Err(Error::Protocol("Inference request timed out".to_string())),
        }
    }

    /// Select the best worker for inference
    fn select_worker(&self) -> Result<String> {
        // First try to find an idle worker
        let idle_workers = self.registry.get_idle_workers();
        if !idle_workers.is_empty() {
            // Return the first idle worker (could implement better selection logic)
            return Ok(idle_workers[0].worker_id.clone());
        }

        // Fall back to any online worker
        let online_workers = self.registry.get_online_workers();
        if !online_workers.is_empty() {
            return Ok(online_workers[0].worker_id.clone());
        }

        Err(Error::Protocol("No available workers".to_string()))
    }

    /// Handle response from worker
    pub fn handle_response(&self, request_id: &str, response: InferenceResponse) -> bool {
        if let Some(pending) = self.pending_requests.write().remove(request_id) {
            let _ = pending.response_sender.send(response);
            true
        } else {
            log::warn!("Received response for unknown request_id: {}", request_id);
            false
        }
    }

    /// Cancel a pending request
    pub fn cancel_request(&self, request_id: &str) -> bool {
        if let Some(pending) = self.pending_requests.write().remove(request_id) {
            // Send error response
            let _ = pending.response_sender.send(InferenceResponse {
                request_id: request_id.to_string(),
                success: false,
                payload_text: String::new(),
                error: "Request cancelled".to_string(),
            });
            true
        } else {
            false
        }
    }

    /// Get count of pending requests
    pub fn pending_count(&self) -> usize {
        self.pending_requests.read().len()
    }

    /// Clean up stale pending requests
    pub fn cleanup_stale_requests(&self, max_age_secs: i64) {
        let now = Utc::now();
        let mut to_remove = Vec::new();

        {
            let pending = self.pending_requests.read();
            for (request_id, req) in pending.iter() {
                let age = now - req.created_at;
                if age.num_seconds() > max_age_secs {
                    to_remove.push(request_id.clone());
                }
            }
        }

        for request_id in to_remove {
            log::warn!("Cleaning up stale request: {}", request_id);
            self.cancel_request(&request_id);
        }
    }
}

/// Commands sent to the worker handler
#[derive(Debug)]
pub enum WorkerCommand {
    /// Send inference request to worker
    InferenceRequest {
        request_id: String,
        worker_id: String,
        context_bundle: String,
        timeout_secs: i64,
    },
    /// Handle worker disconnect
    WorkerDisconnect { worker_id: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_strategy_default() {
        assert_eq!(RoutingStrategy::default(), RoutingStrategy::Any);
    }
}
