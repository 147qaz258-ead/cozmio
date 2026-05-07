use prost::Message;
use std::io::{Read, Write};
use std::net::TcpStream;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let img_path = "examples/vision_test_small.png";
    let img_data = std::fs::read(img_path)?;

    let request = InferenceRequest {
        request_id: "vision-test-001".to_string(),
        worker_id: "box-h1-001".to_string(),
        context_bundle: "Describe this image in detail.".to_string(),
        timeout_secs: 1800,
        image_data: img_data,
    };

    let mut payload = Vec::new();
    request.encode(&mut payload)?;

    let mut stream = TcpStream::connect("127.0.0.1:7890")?;

    // Frame format: 4 bytes length + 1 byte type (7 for REQ_INFERENCE) + payload
    let total_len = (1 + payload.len() as u32).to_be_bytes();
    stream.write_all(&total_len)?;
    stream.write_all(&[7])?; // REQ_INFERENCE
    stream.write_all(&payload)?;
    stream.flush()?;

    println!("Sent vision inference request to relay...");

    // Read response length
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let resp_len = u32::from_be_bytes(len_bytes) as usize;

    let mut resp_payload = vec![0u8; resp_len];
    stream.read_exact(&mut resp_payload)?;

    let response = InferenceResponse::decode(&resp_payload[..])?;
    println!("Response success: {}", response.success);
    println!("Response text: {}", response.payload_text);
    if !response.error.is_empty() {
        println!("Response error: {}", response.error);
    }

    Ok(())
}
