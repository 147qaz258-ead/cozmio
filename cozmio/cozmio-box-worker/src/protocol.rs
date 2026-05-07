use prost::Message;
use std::io::{Read, Write};
use std::net::TcpStream;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Protobuf encode error: {0}")]
    Encode(#[from] prost::EncodeError),
    #[error("Protobuf decode error: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),
}

pub const REQ_DISPATCH: u8 = 1;
pub const REQ_STATUS: u8 = 2;
pub const REQ_PROGRESS: u8 = 3;
pub const REQ_RESULT: u8 = 4;
pub const REQ_INTERRUPT: u8 = 5;
pub const REQ_SUBSCRIBE: u8 = 6;

pub const WORKER_REGISTER: u8 = 100;
pub const WORKER_HEARTBEAT: u8 = 101;
pub const INFERENCE_REQUEST: u8 = 102;
pub const INFERENCE_RESPONSE: u8 = 103;

pub fn send_response(
    stream: &mut TcpStream,
    response_type: u8,
    payload: &[u8],
) -> Result<(), ProtocolError> {
    let total_len = (payload.len() as u32).to_be_bytes();
    stream.write_all(&total_len)?;
    stream.write_all(&[response_type])?;
    stream.write_all(payload)?;
    stream.flush()?;
    Ok(())
}

pub fn send_request(
    stream: &mut TcpStream,
    request_type: u8,
    payload: &[u8],
) -> Result<Vec<u8>, ProtocolError> {
    let total_len = (payload.len() as u32).to_be_bytes();
    stream.write_all(&total_len)?;
    stream.write_all(&[request_type])?;
    stream.write_all(payload)?;
    stream.flush()?;
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    if len > 10_000_000 {
        return Err(ProtocolError::InvalidFrame(format!(
            "Response too large: {} bytes",
            len
        )));
    }
    let mut response = vec![0u8; len];
    stream.read_exact(&mut response)?;
    Ok(response)
}

pub fn build_request(request_type: u8, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(5 + payload.len());
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.push(request_type);
    frame.extend_from_slice(payload);
    frame
}

#[derive(Clone, PartialEq, Message)]
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

#[derive(Clone, PartialEq, Message)]
pub struct WorkerRegisterResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub error: String,
    #[prost(int64, tag = "3")]
    pub registration_id: i64,
}

#[derive(Clone, PartialEq, Message)]
pub struct WorkerHeartbeat {
    #[prost(string, tag = "1")]
    pub worker_id: String,
    #[prost(string, tag = "2")]
    pub status: String,
    #[prost(message, tag = "3")]
    pub model_status: Option<ModelStatus>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ModelStatus {
    #[prost(string, tag = "1")]
    pub model_name: String,
    #[prost(bool, tag = "2")]
    pub loaded: bool,
    #[prost(int32, tag = "3")]
    pub memory_used_mb: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct InferenceRequest {
    #[prost(string, tag = "1")]
    pub request_id: String,
    #[prost(string, tag = "2")]
    pub worker_id: String,
    #[prost(string, tag = "3")]
    pub context_bundle: String,
    #[prost(int64, tag = "4")]
    pub timeout_secs: i64,
    #[prost(bytes, tag = "5")]
    pub image_data: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
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
