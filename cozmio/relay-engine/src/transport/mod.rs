use crate::error::Result;

pub trait Transport: Send + Sync {
    fn listen(&self) -> Result<()>;
    fn accept(&mut self) -> Result<Box<dyn Connection>>;
    fn address(&self) -> &str;
}

pub trait Connection: Send + Sync {
    fn send(&self, data: &[u8]) -> Result<()>;
    fn recv(&self) -> Result<Vec<u8>>;
    fn try_recv(&self) -> Result<Option<Vec<u8>>>;
    fn close(&self) -> Result<()>;
}

pub mod unix;
pub mod windows;
