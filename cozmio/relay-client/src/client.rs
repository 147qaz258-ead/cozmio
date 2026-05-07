use prost::Message;
use std::io::{Read, Write};
use std::net::TcpStream;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("not connected")]
    NotConnected,
    #[error("connection error: {0}")]
    Connection(String),
    #[error("protocol error: {0}")]
    Protocol(String),
}

pub type Result<T> = std::result::Result<T, ClientError>;

const REQ_DISPATCH: u8 = 1;
const REQ_STATUS: u8 = 2;
const REQ_PROGRESS: u8 = 3;
const REQ_RESULT: u8 = 4;
const REQ_INTERRUPT: u8 = 5;
const REQ_SUBSCRIBE: u8 = 6;
const REQ_INFERENCE: u8 = 7;

pub struct RelayClient {
    address: String,
}

impl RelayClient {
    pub fn connect(address: &str) -> Result<Self> {
        let stream =
            TcpStream::connect(address).map_err(|e| ClientError::Connection(e.to_string()))?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(30)))
            .map_err(|e| ClientError::Connection(e.to_string()))?;
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(30)))
            .map_err(|e| ClientError::Connection(e.to_string()))?;
        drop(stream);

        Ok(RelayClient {
            address: address.to_string(),
        })
    }

    pub fn is_connected(&self) -> bool {
        true
    }

    pub fn dispatch(&self, agent_name: &str, suggestion: &str, task: &str) -> Result<String> {
        let req = crate::proto::DispatchRequest {
            agent_name: agent_name.to_string(),
            original_suggestion: suggestion.to_string(),
            dispatched_task: task.to_string(),
        };

        let response =
            self.send_request::<crate::proto::DispatchResponse>(REQ_DISPATCH, req.encode_to_vec())?;
        Ok(response.session_id)
    }

    pub fn status(&self, session_id: &str) -> Result<crate::proto::StatusResponse> {
        let req = crate::proto::StatusRequest {
            session_id: session_id.to_string(),
        };

        self.send_request::<crate::proto::StatusResponse>(REQ_STATUS, req.encode_to_vec())
    }

    pub fn progress(&self, session_id: &str) -> Result<crate::proto::ProgressResponse> {
        let req = crate::proto::ProgressRequest {
            session_id: session_id.to_string(),
        };

        self.send_request::<crate::proto::ProgressResponse>(REQ_PROGRESS, req.encode_to_vec())
    }

    pub fn result(&self, session_id: &str) -> Result<crate::proto::ResultResponse> {
        let req = crate::proto::ResultRequest {
            session_id: session_id.to_string(),
        };

        self.send_request::<crate::proto::ResultResponse>(REQ_RESULT, req.encode_to_vec())
    }

    pub fn interrupt(&self, session_id: &str) -> Result<bool> {
        let req = crate::proto::InterruptRequest {
            session_id: session_id.to_string(),
        };

        let response = self
            .send_request::<crate::proto::InterruptResponse>(REQ_INTERRUPT, req.encode_to_vec())?;
        Ok(response.success)
    }

    pub fn subscribe(&self, session_id: &str) -> Result<RelaySubscription> {
        let address = self.subscription_address();
        let mut stream =
            TcpStream::connect(&address).map_err(|e| ClientError::Connection(e.to_string()))?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(120)))
            .map_err(|e| ClientError::Connection(e.to_string()))?;
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(30)))
            .map_err(|e| ClientError::Connection(e.to_string()))?;

        let req = crate::proto::SubscribeRequest {
            session_id: session_id.to_string(),
        };
        Self::send_request_to_stream(&mut stream, REQ_SUBSCRIBE, &req.encode_to_vec())?;
        Ok(RelaySubscription { stream })
    }

    pub fn inference(
        &self,
        context_bundle: &str,
        timeout_secs: u64,
    ) -> Result<crate::proto::InferenceResponse> {
        let req = crate::proto::InferenceRequest {
            context_bundle: context_bundle.to_string(),
            timeout_secs,
        };
        self.send_request::<crate::proto::InferenceResponse>(REQ_INFERENCE, req.encode_to_vec())
    }

    fn send_request<T: Message + Default>(&self, kind: u8, payload: Vec<u8>) -> Result<T> {
        let mut stream = TcpStream::connect(&self.address)
            .map_err(|e| ClientError::Connection(e.to_string()))?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(30)))
            .map_err(|e| ClientError::Connection(e.to_string()))?;
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(30)))
            .map_err(|e| ClientError::Connection(e.to_string()))?;

        Self::send_request_to_stream(&mut stream, kind, &payload)?;
        Self::recv_response_from_stream::<T>(&mut stream)
    }

    fn send_request_to_stream(stream: &mut TcpStream, kind: u8, payload: &[u8]) -> Result<()> {
        let mut frame = Vec::with_capacity(1 + payload.len());
        frame.push(kind);
        frame.extend_from_slice(payload);
        let len = frame.len() as u32;
        stream
            .write_all(&len.to_be_bytes())
            .map_err(|e| ClientError::Connection(e.to_string()))?;
        stream
            .write_all(&frame)
            .map_err(|e| ClientError::Connection(e.to_string()))?;
        stream
            .flush()
            .map_err(|e| ClientError::Connection(e.to_string()))
    }

    fn recv_response_from_stream<T: Message + Default>(stream: &mut TcpStream) -> Result<T> {
        let mut header = [0u8; 4];
        stream
            .read_exact(&mut header)
            .map_err(|e| ClientError::Connection(e.to_string()))?;
        let len = u32::from_be_bytes(header) as usize;

        // Read message bytes
        let mut msg_bytes = vec![0u8; len];
        stream
            .read_exact(&mut msg_bytes)
            .map_err(|e| ClientError::Connection(e.to_string()))?;

        // Decode protobuf
        T::decode(&msg_bytes[..]).map_err(|e| ClientError::Protocol(e.to_string()))
    }

    fn subscription_address(&self) -> String {
        match self.address.rsplit_once(':') {
            Some((host, _)) => format!("{host}:7891"),
            None => String::from("127.0.0.1:7891"),
        }
    }
}

pub struct RelaySubscription {
    stream: TcpStream,
}

impl RelaySubscription {
    pub fn recv_event(&mut self) -> Result<crate::proto::ProgressEvent> {
        RelayClient::recv_response_from_stream::<crate::proto::ProgressEvent>(&mut self.stream)
    }
}
