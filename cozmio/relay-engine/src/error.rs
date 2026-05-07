use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("agent error: {0}")]
    Agent(String),
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
