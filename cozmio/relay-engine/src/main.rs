mod agent;
mod agents;
mod error;
mod inference_router;
mod proto;
mod session;
mod transport;
mod worker_registry;
mod worker_session;

use crate::agent::AgentRegistry;
use crate::agents::ClaudeCodeConnector;
use crate::error::Error;
use crate::inference_router::{InferenceRouter, RoutingStrategy, WorkerCommand};
use crate::proto::{
    DispatchResponse, InferenceResponse, InterruptRequest, InterruptResponse, ProgressResponse,
    ResultResponse, StatusResponse,
};
use crate::session::{
    ExecutionResult, ExecutionTask, ProgressEntry, SessionId, SessionStatus, SessionSubscriber,
};
use crate::transport::windows::WindowsPipeConnection;
use crate::transport::Connection;
use crate::worker_registry::WorkerRegistry;
use crate::worker_session::WorkerSessionManager;
use prost::Message;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::mpsc;
use uuid::Uuid;

const REQ_DISPATCH: u8 = 1;
const REQ_STATUS: u8 = 2;
const REQ_PROGRESS: u8 = 3;
const REQ_RESULT: u8 = 4;
const REQ_INTERRUPT: u8 = 5;
const REQ_SUBSCRIBE: u8 = 6;
const REQ_INFERENCE: u8 = 7;
const MAX_MESSAGE_SIZE: usize = 1_000_000;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Relay Engine starting...");

    let sessions = Arc::new(crate::session::SessionManager::new());
    let agents = Arc::new(AgentRegistry::new());
    let claude_cli = std::env::var("COZMIO_CLAUDE_CLI").unwrap_or_else(|_| {
        if cfg!(windows) {
            String::from(r"C:\Users\29913\AppData\Roaming\npm\claude.cmd")
        } else {
            String::from("claude")
        }
    });

    let claude_connector = Arc::new(ClaudeCodeConnector::new(claude_cli, sessions.clone()));
    agents.register(claude_connector);

    log::info!("Registered agents: {:?}", agents.list());

    // Create worker registry for Box Worker management
    let worker_registry = Arc::new(WorkerRegistry::new());
    let worker_session_manager = Arc::new(WorkerSessionManager::new(worker_registry.clone()));

    // Create inference router with channel for worker commands
    let (worker_tx, worker_rx) = mpsc::channel::<WorkerCommand>(100);
    let inference_router = Arc::new(InferenceRouter::new(worker_registry.clone(), worker_tx));

    log::info!("Transport address: 127.0.0.1:7890");

    // Create TCP listener directly (not through transport abstraction for clearer control)
    let listener =
        std::net::TcpListener::bind("127.0.0.1:7890").expect("Failed to bind to 127.0.0.1:7890");
    listener
        .set_nonblocking(true)
        .expect("Failed to set non-blocking mode");
    log::info!("TCP listener bound to 127.0.0.1:7890");

    // Create subscription listener on port 7891
    let sub_listener =
        std::net::TcpListener::bind("127.0.0.1:7891").expect("Failed to bind to 127.0.0.1:7891");
    sub_listener
        .set_nonblocking(true)
        .expect("Failed to set non-blocking mode for subscription listener");
    log::info!("Subscription listener bound to 127.0.0.1:7891");

    // Create worker listener on port 7892
    let worker_listener =
        std::net::TcpListener::bind("127.0.0.1:7892").expect("Failed to bind to 127.0.0.1:7892");
    worker_listener
        .set_nonblocking(true)
        .expect("Failed to set non-blocking mode for worker listener");
    log::info!("Worker listener bound to 127.0.0.1:7892");

    // Wait for shutdown signal
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // Spawn a separate thread for the accept loop
            let sessions_clone = sessions.clone();
            let agents_clone = agents.clone();
            let inference_router_clone = inference_router.clone();
            let handle = std::thread::spawn(move || {
                accept_loop(
                    listener,
                    sessions_clone,
                    agents_clone,
                    inference_router_clone,
                );
            });

            // Spawn a separate thread for the subscription accept loop
            let sessions_sub = sessions.clone();
            let handle_sub = std::thread::spawn(move || {
                subscription_accept_loop(sub_listener, sessions_sub);
            });

            // Spawn a separate thread for the worker accept loop
            let worker_registry_clone = worker_registry.clone();
            let worker_session_clone = worker_session_manager.clone();
            let inference_router_worker = inference_router.clone();
            let handle_worker = std::thread::spawn(move || {
                worker_accept_loop(
                    worker_listener,
                    worker_registry_clone,
                    worker_session_clone,
                    inference_router_worker,
                );
            });

            // Spawn async task to process worker commands
            tokio::spawn(process_worker_commands(
                worker_rx,
                worker_session_manager.clone(),
                inference_router.clone(),
            ));

            // Spawn periodic task to check worker heartbeats and mark offline workers
            let worker_manager_for_heartbeat = worker_session_manager.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
                loop {
                    interval.tick().await;
                    worker_manager_for_heartbeat.check_workers_connected();
                }
            });

            log::info!("Relay Engine running: ports 7890, 7891, 7892");

            signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
            log::info!("Shutdown signal received, exiting...");
            drop(handle);
            drop(handle_sub);
            drop(handle_worker);
        });
}

fn accept_loop(
    listener: std::net::TcpListener,
    sessions: Arc<crate::session::SessionManager>,
    agents: Arc<AgentRegistry>,
    inference_router: Arc<InferenceRouter>,
) {
    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                log::info!("Accepted connection from: {}", addr);
                if let Err(e) = stream.set_nonblocking(false) {
                    log::error!("Failed to set accepted stream blocking mode: {}", e);
                    continue;
                }
                let sessions = sessions.clone();
                let agents = agents.clone();
                let inference_router = inference_router.clone();
                std::thread::spawn(move || {
                    handle_connection(stream, &sessions, &agents, &inference_router);
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No connection pending, sleep briefly and try again
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => {
                log::error!("Accept error: {}", e);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}

fn subscription_accept_loop(
    listener: std::net::TcpListener,
    sessions: Arc<crate::session::SessionManager>,
) {
    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                log::info!("Accepted subscription connection from: {}", addr);
                if let Err(e) = stream.set_nonblocking(false) {
                    log::error!("Failed to set subscription stream blocking mode: {}", e);
                    continue;
                }
                let sessions = sessions.clone();
                std::thread::spawn(move || {
                    handle_subscribe(stream, sessions);
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => {
                log::error!("Subscription accept error: {}", e);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}

fn worker_accept_loop(
    listener: std::net::TcpListener,
    registry: Arc<WorkerRegistry>,
    session_manager: Arc<WorkerSessionManager>,
    inference_router: Arc<InferenceRouter>,
) {
    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                log::info!("Accepted worker connection from: {}", addr);
                if let Err(e) = stream.set_nonblocking(false) {
                    log::error!("Failed to set worker stream blocking mode: {}", e);
                    continue;
                }
                let registry_clone = registry.clone();
                let session_manager_clone = session_manager.clone();
                let inference_router_clone = inference_router.clone();
                std::thread::spawn(move || {
                    handle_worker_connection(
                        stream,
                        registry_clone,
                        session_manager_clone,
                        inference_router_clone,
                    );
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => {
                log::error!("Worker accept error: {}", e);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}

fn handle_worker_connection(
    stream: std::net::TcpStream,
    registry: Arc<WorkerRegistry>,
    session_manager: Arc<WorkerSessionManager>,
    inference_router: Arc<InferenceRouter>,
) {
    // Handle new worker registration
    // handle_new_worker stores the stream internally in worker_streams
    match session_manager.handle_new_worker(stream, &registry) {
        Ok(worker_id) => {
            // Get worker info to determine heartbeat interval
            if let Some(worker_info) = registry.get(&worker_id) {
                let interval = worker_info.heartbeat_interval_secs;

                // Get the stored stream for heartbeat monitor
                if let Some(stream_arc) = session_manager.get_stream_arc(&worker_id) {
                    session_manager.start_heartbeat_monitor(
                        worker_id.clone(),
                        stream_arc,
                        interval,
                    );
                }

                // Keep the connection alive and handle ongoing communication
                // The handle_worker_messages will access the stream via session_manager
                handle_worker_messages_for_worker(worker_id, session_manager, inference_router);
            }
        }
        Err(e) => {
            log::error!("Failed to handle worker connection: {}", e);
        }
    }
}

fn handle_worker_messages_for_worker(
    worker_id: String,
    session_manager: Arc<WorkerSessionManager>,
    inference_router: Arc<InferenceRouter>,
) {
    use prost::Message;
    use std::io::Read;

    // Continuously read messages from the worker via the stored stream
    loop {
        // Get the stream from session_manager
        let stream_arc = match session_manager.get_stream_arc(&worker_id) {
            Some(arc) => arc,
            None => {
                log::debug!("Worker {} stream not found, disconnecting", worker_id);
                session_manager.handle_disconnect(&worker_id);
                break;
            }
        };

        let mut stream = match stream_arc.lock() {
            Ok(lock) => lock,
            Err(e) => {
                log::error!("Worker {} failed to lock stream: {}", worker_id, e);
                session_manager.handle_disconnect(&worker_id);
                break;
            }
        };

        // Read frame: 4 bytes length + 1 byte type + payload
        let mut len_bytes = [0u8; 4];
        match stream.read_exact(&mut len_bytes) {
            Ok(_) => {}
            Err(e) => {
                log::debug!("Worker {} connection closed: {}", worker_id, e);
                session_manager.handle_disconnect(&worker_id);
                break;
            }
        }

        let len = u32::from_be_bytes(len_bytes) as usize;
        if len > MAX_MESSAGE_SIZE {
            log::error!("Worker {} message too large: {} bytes", worker_id, len);
            break;
        }

        let mut type_byte = [0u8; 1];
        if let Err(e) = stream.read_exact(&mut type_byte) {
            log::debug!("Worker {} read error: {}", worker_id, e);
            session_manager.handle_disconnect(&worker_id);
            break;
        }

        let msg_type = type_byte[0];

        let mut payload = vec![0u8; len];
        if let Err(e) = stream.read_exact(&mut payload) {
            log::debug!("Worker {} payload read error: {}", worker_id, e);
            session_manager.handle_disconnect(&worker_id);
            break;
        }

        // Handle message based on type
        // Note: Heartbeat messages are handled by the heartbeat monitor task
        // This function handles other message types like INFERENCE_RESPONSE
        match msg_type {
            proto::INFERENCE_RESPONSE => {
                // Decode the inference response and route it back to the requestor
                match crate::proto::InferenceResponse::decode(&payload[..]) {
                    Ok(response) => {
                        log::info!(
                            "Worker {} inference response: request_id={}, success={}",
                            worker_id,
                            response.request_id,
                            response.success
                        );
                        // Route response back to the InferenceRouter's pending request
                        if inference_router.handle_response(&response.request_id, response.clone())
                        {
                            log::debug!("Successfully routed inference response to requestor");
                        } else {
                            log::warn!(
                                "No pending request found for inference response: {}",
                                response.request_id
                            );
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to decode InferenceResponse from worker {}: {}",
                            worker_id,
                            e
                        );
                    }
                }
            }
            _ => {
                log::warn!(
                    "Worker {} sent unhandled message type: {}",
                    worker_id,
                    msg_type
                );
            }
        }
    }
}

/// Process worker commands from the channel and forward to worker session manager
async fn process_worker_commands(
    mut worker_rx: mpsc::Receiver<WorkerCommand>,
    worker_session_manager: Arc<WorkerSessionManager>,
    router: Arc<InferenceRouter>,
) {
    log::info!("Worker command processor started");
    while let Some(cmd) = worker_rx.recv().await {
        match cmd {
            WorkerCommand::InferenceRequest {
                request_id,
                worker_id,
                context_bundle,
                timeout_secs,
            } => {
                log::info!(
                    "Processing inference command: request_id={}, worker_id={}",
                    request_id,
                    worker_id
                );
                // Forward to worker session manager synchronously via blocking task
                // Note: send_inference_request only SENDS the request, does NOT wait for response
                // The response will be received by handle_worker_messages_for_worker and routed via handle_response
                let session_manager = worker_session_manager.clone();
                let request_id_clone = request_id.clone();
                let worker_id_clone = worker_id.clone();
                let result = tokio::task::spawn_blocking(move || {
                    session_manager.send_inference_request(
                        &worker_id,
                        &request_id,
                        &context_bundle,
                        timeout_secs,
                    )
                })
                .await;

                match result {
                    Ok(Ok(())) => {
                        log::debug!(
                            "Inference request sent to worker: request_id={}, worker_id={}",
                            request_id_clone,
                            worker_id_clone
                        );
                        // Response will be routed via handle_response in handle_worker_messages_for_worker
                    }
                    Ok(Err(e)) => {
                        log::error!(
                            "Failed to send inference request for request_id={}: {}",
                            request_id_clone,
                            e
                        );
                        // Clean up: cancel pending request and revert worker state
                        router.cancel_request(&request_id_clone);
                        // Revert worker state to idle since send failed
                        let _ = worker_session_manager.revert_worker_state(&worker_id_clone);
                    }
                    Err(e) => {
                        log::error!(
                            "Task join error for inference request {}: {}",
                            request_id_clone,
                            e
                        );
                    }
                }
            }
            WorkerCommand::WorkerDisconnect { worker_id } => {
                log::info!("Worker disconnect command: {}", worker_id);
                worker_session_manager.handle_disconnect(&worker_id);
            }
        }
    }
    log::info!("Worker command processor exiting");
}

fn handle_connection(
    stream: std::net::TcpStream,
    sessions: &Arc<crate::session::SessionManager>,
    agents: &Arc<AgentRegistry>,
    inference_router: &Arc<InferenceRouter>,
) {
    let mut connection = WindowsPipeConnection::new(stream);

    loop {
        let mut len_bytes = [0u8; 4];
        match connection.recv_exact(&mut len_bytes) {
            Ok(_) => {}
            Err(e) => {
                log::debug!("Connection recv error for length header: {}", e);
                break;
            }
        };

        let len = u32::from_be_bytes(len_bytes) as usize;

        if len > MAX_MESSAGE_SIZE {
            log::error!("Message too large: {} bytes (max 1MB)", len);
            break;
        }

        let mut msg_bytes = vec![0u8; len];
        match connection.recv_exact(&mut msg_bytes) {
            Ok(_) => {}
            Err(e) => {
                log::debug!("Connection recv error for message: {}", e);
                break;
            }
        };

        let response = match split_typed_request(msg_bytes) {
            Ok((REQ_DISPATCH, payload)) => match proto::DispatchRequest::decode(&payload[..]) {
                Ok(req) => handle_dispatch(req, sessions, agents),
                Err(e) => {
                    log::error!("Failed to decode DispatchRequest: {}", e);
                    build_error_response("Invalid DispatchRequest")
                }
            },
            Ok((REQ_STATUS, payload)) => match proto::StatusRequest::decode(&payload[..]) {
                Ok(req) => handle_status(req, sessions),
                Err(e) => {
                    log::error!("Failed to decode StatusRequest: {}", e);
                    build_error_response("Invalid StatusRequest")
                }
            },
            Ok((REQ_PROGRESS, payload)) => match proto::ProgressRequest::decode(&payload[..]) {
                Ok(req) => handle_progress(req, sessions),
                Err(e) => {
                    log::error!("Failed to decode ProgressRequest: {}", e);
                    build_error_response("Invalid ProgressRequest")
                }
            },
            Ok((REQ_RESULT, payload)) => match proto::ResultRequest::decode(&payload[..]) {
                Ok(req) => handle_result(req, sessions),
                Err(e) => {
                    log::error!("Failed to decode ResultRequest: {}", e);
                    build_error_response("Invalid ResultRequest")
                }
            },
            Ok((REQ_INTERRUPT, payload)) => match proto::InterruptRequest::decode(&payload[..]) {
                Ok(req) => handle_interrupt(sessions, agents, &req),
                Err(e) => {
                    log::error!("Failed to decode InterruptRequest: {}", e);
                    build_error_response("Invalid InterruptRequest")
                }
            },
            Ok((REQ_SUBSCRIBE, _)) => {
                log::warn!("SubscribeRequest received on main port 7890 - should use port 7891");
                build_error_response("SubscribeRequest not valid on main port, use port 7891")
            }
            Ok((REQ_INFERENCE, payload)) => {
                match proto::InferenceRequest::decode(&payload[..]) {
                    Ok(req) => {
                        log::info!(
                            "Inference request: request_id={}, context_bundle_len={}",
                            req.request_id,
                            req.context_bundle.len()
                        );
                        // Call the async InferenceRouter using block_on in a blocking context
                        // handle_connection runs in a spawned thread, so we can use the runtime
                        let router = inference_router.clone();
                        let response: Result<InferenceResponse, Error> =
                            tokio::runtime::Handle::current().block_on(async {
                                router
                                    .route_inference(
                                        req.context_bundle.clone(),
                                        req.timeout_secs,
                                        RoutingStrategy::Any,
                                    )
                                    .await
                            });
                        match response {
                            Ok(resp) => encode_response(&resp),
                            Err(e) => {
                                log::error!("Inference request failed: {}", e);
                                encode_response(&InferenceResponse {
                                    request_id: req.request_id,
                                    success: false,
                                    payload_text: String::new(),
                                    error: e.to_string(),
                                })
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to decode InferenceRequest: {}", e);
                        build_error_response("Invalid InferenceRequest")
                    }
                }
            }
            Ok((kind, _)) => {
                log::warn!("Unknown request kind: {}", kind);
                build_error_response("Unknown request kind")
            }
            Err(e) => {
                log::error!("Invalid request frame: {}", e);
                build_error_response("Invalid request frame")
            }
        };

        let mut prefixed = Vec::with_capacity(4 + response.len());
        let len = (response.len() as u32).to_be_bytes();
        prefixed.extend_from_slice(&len);
        prefixed.extend_from_slice(&response);
        if let Err(e) = connection.send(&prefixed) {
            log::error!("Failed to send response: {}", e);
            break;
        }

        break;
    }
}

fn handle_dispatch(
    req: proto::DispatchRequest,
    sessions: &Arc<crate::session::SessionManager>,
    agents: &Arc<AgentRegistry>,
) -> Vec<u8> {
    log::info!(
        "Dispatch request: agent={}, task={}",
        req.agent_name,
        req.dispatched_task
    );

    let dispatched_task = req.dispatched_task.clone();
    let task = ExecutionTask {
        original_suggestion: req.original_suggestion,
        dispatched_task: dispatched_task.clone(),
        agent_name: req.agent_name.clone(),
    };

    let session_id = sessions.create_session(task);

    // Get agent and dispatch
    if let Some(agent) = agents.get(&req.agent_name) {
        match agent.dispatch(session_id.clone(), req.dispatched_task) {
            Ok(_) => {
                let status = sessions
                    .get_status(&session_id)
                    .unwrap_or(crate::session::SessionStatus::Pending);
                let response = DispatchResponse {
                    session_id: session_id.to_string(),
                    status: status as i32,
                };
                encode_response(&response)
            }
            Err(e) => {
                log::error!("Dispatch failed: {}", e);
                sessions.set_result(
                    &session_id,
                    ExecutionResult {
                        summary: String::from("Dispatch failed"),
                        raw_output: String::new(),
                        duration_secs: 0,
                        success: false,
                        error_message: e.to_string(),
                    },
                );
                sessions.update_status(&session_id, SessionStatus::Failed);
                let response = DispatchResponse {
                    session_id: session_id.to_string(),
                    status: SessionStatus::Failed as i32,
                };
                encode_response(&response)
            }
        }
    } else {
        log::error!("Agent not found: {}", req.agent_name);
        sessions.set_result(
            &session_id,
            ExecutionResult {
                summary: String::from("Agent not found"),
                raw_output: String::new(),
                duration_secs: 0,
                success: false,
                error_message: format!("Agent not found: {}", req.agent_name),
            },
        );
        sessions.update_status(&session_id, SessionStatus::Failed);
        let response = DispatchResponse {
            session_id: session_id.to_string(),
            status: SessionStatus::Failed as i32,
        };
        encode_response(&response)
    }
}

fn handle_status(
    req: proto::StatusRequest,
    sessions: &Arc<crate::session::SessionManager>,
) -> Vec<u8> {
    log::info!("Status request: session_id={}", req.session_id);

    let session_id = SessionId::from_string(req.session_id.clone());
    if let Some(session) = sessions.get_session(&session_id) {
        let response = StatusResponse {
            session_id: req.session_id,
            status: session.status as i32,
            started_at: session.started_at.timestamp(),
            updated_at: session.updated_at.timestamp(),
            duration_secs: (session.updated_at - session.started_at).num_seconds(),
        };
        encode_response(&response)
    } else {
        build_error_response("Session not found")
    }
}

fn handle_progress(
    req: proto::ProgressRequest,
    sessions: &Arc<crate::session::SessionManager>,
) -> Vec<u8> {
    log::info!("Progress request: session_id={}", req.session_id);

    let session_id = SessionId::from_string(req.session_id.clone());
    let entries = sessions
        .get_progress(&session_id)
        .unwrap_or_default()
        .into_iter()
        .map(|e| proto::ProgressEntry {
            timestamp: e.timestamp,
            message: e.message,
            level: e.level as i32,
        })
        .collect();

    let response = ProgressResponse {
        session_id: req.session_id,
        entries,
    };
    encode_response(&response)
}

fn handle_result(
    req: proto::ResultRequest,
    sessions: &Arc<crate::session::SessionManager>,
) -> Vec<u8> {
    log::info!("Result request: session_id={}", req.session_id);

    let session_id = SessionId::from_string(req.session_id.clone());
    if let Some(result) = sessions.get_result(&session_id) {
        let response = ResultResponse {
            session_id: req.session_id,
            result: Some(proto::ExecutionResult {
                summary: result.summary,
                raw_output: result.raw_output,
                duration_secs: result.duration_secs,
                success: result.success,
                error_message: result.error_message,
            }),
        };
        encode_response(&response)
    } else {
        build_error_response("Result not available")
    }
}

fn handle_interrupt(
    _sessions: &Arc<crate::session::SessionManager>,
    agents: &Arc<AgentRegistry>,
    req: &InterruptRequest,
) -> Vec<u8> {
    log::info!("Interrupt request: session_id={}", req.session_id);

    let session_id = SessionId::from_string(req.session_id.clone());

    if let Some(agent) = agents.get("claude-code") {
        match agent.interrupt(&session_id) {
            Ok(()) => {
                let resp = InterruptResponse { success: true };
                let mut bytes = Vec::new();
                resp.encode(&mut bytes)
                    .map_err(|e| format!("encode error: {}", e))
                    .unwrap();
                bytes
            }
            Err(e) => {
                log::error!("Interrupt failed: {}", e);
                encode_response(&InterruptResponse { success: false })
            }
        }
    } else {
        encode_response(&InterruptResponse { success: false })
    }
}

fn encode_response<T: prost::Message>(msg: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    msg.encode(&mut bytes).expect("Failed to encode response");
    bytes
}

fn build_error_response(error: &str) -> Vec<u8> {
    // Return a proper DispatchResponse with empty session_id to indicate error
    log::error!("Error response: {}", error);
    let response = DispatchResponse {
        session_id: String::new(), // Empty session_id indicates error
        status: -1,                // Negative status indicates error
    };
    encode_response(&response)
}

fn handle_subscribe(stream: std::net::TcpStream, sessions: Arc<crate::session::SessionManager>) {
    let connection = WindowsPipeConnection::new(stream);

    let mut len_bytes = [0u8; 4];
    match connection.recv_exact(&mut len_bytes) {
        Ok(_) => {}
        Err(e) => {
            log::debug!("Subscription recv error for length header: {}", e);
            return;
        }
    };

    let len = u32::from_be_bytes(len_bytes) as usize;
    if len > MAX_MESSAGE_SIZE {
        log::error!("SubscribeRequest too large: {} bytes", len);
        return;
    }

    let mut msg_bytes = vec![0u8; len];
    match connection.recv_exact(&mut msg_bytes) {
        Ok(_) => {}
        Err(e) => {
            log::debug!("Subscription recv error for message: {}", e);
            return;
        }
    };

    let (request_kind, payload) = match split_typed_request(msg_bytes) {
        Ok(parts) => parts,
        Err(e) => {
            log::error!("Invalid subscription frame: {}", e);
            return;
        }
    };

    if request_kind != REQ_SUBSCRIBE {
        log::warn!(
            "Invalid request kind on subscription port: {}",
            request_kind
        );
        return;
    }

    let req = match proto::SubscribeRequest::decode(&payload[..]) {
        Ok(req) => req,
        Err(e) => {
            log::error!("Failed to decode SubscribeRequest: {}", e);
            return;
        }
    };

    let session_id = SessionId::from_string(req.session_id.clone());
    let subscriber_id = Uuid::new_v4().to_string();

    let subscriber: Arc<dyn SessionSubscriber> = Arc::new(TcpSessionSubscriber {
        connection,
        subscriber_id: subscriber_id.clone(),
    });

    sessions.subscribe(session_id.clone(), subscriber);
    log::info!("Client subscribed to session: {}", req.session_id);
}

fn send_progress_event(connection: &WindowsPipeConnection, event: &proto::ProgressEvent) -> bool {
    let mut bytes = Vec::new();
    if let Err(e) = event.encode(&mut bytes) {
        log::error!("Failed to encode ProgressEvent: {}", e);
        return false;
    }

    let mut prefixed = Vec::with_capacity(4 + bytes.len());
    let len = (bytes.len() as u32).to_be_bytes();
    prefixed.extend_from_slice(&len);
    prefixed.extend_from_slice(&bytes);

    if let Err(e) = connection.send(&prefixed) {
        log::error!("Failed to send ProgressEvent: {}", e);
        return false;
    }

    true
}

/// Subscriber implementation that sends progress events over a TCP connection
struct TcpSessionSubscriber {
    connection: WindowsPipeConnection,
    subscriber_id: String,
}

impl SessionSubscriber for TcpSessionSubscriber {
    fn subscriber_id(&self) -> &str {
        &self.subscriber_id
    }

    fn on_progress(&self, session_id: &SessionId, entry: &ProgressEntry) {
        let event = proto::ProgressEvent {
            session_id: session_id.to_string(),
            timestamp: entry.timestamp,
            message: entry.message.clone(),
            level: entry.level as i32,
            terminal: false,
            terminal_status: 0,
        };
        let _ = send_progress_event(&self.connection, &event);
    }

    fn on_terminal(&self, session_id: &SessionId, status: SessionStatus) {
        let event = proto::ProgressEvent {
            session_id: session_id.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            message: format!("session terminal: {:?}", status),
            level: 0,
            terminal: true,
            terminal_status: status as i32,
        };
        let _ = send_progress_event(&self.connection, &event);
    }
}

fn split_typed_request(bytes: Vec<u8>) -> Result<(u8, Vec<u8>), String> {
    if bytes.is_empty() {
        return Err(String::from("empty request frame"));
    }

    Ok((bytes[0], bytes[1..].to_vec()))
}
