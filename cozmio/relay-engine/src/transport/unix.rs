use crate::error::{Error, Result};

pub struct UnixDomainSocket {
    address: String,
}

impl UnixDomainSocket {
    pub fn new(address: &str) -> Self {
        UnixDomainSocket {
            address: address.to_string(),
        }
    }
}

impl super::Transport for UnixDomainSocket {
    fn listen(&self) -> Result<()> {
        log::info!(
            "UnixDomainSocket listen called for address: {}",
            self.address
        );
        Ok(())
    }

    fn accept(&mut self) -> Result<Box<dyn super::Connection>> {
        Err(Error::Transport(
            "accept not implemented for UnixDomainSocket".into(),
        ))
    }

    fn address(&self) -> &str {
        &self.address
    }
}

pub struct UnixConnection;

impl super::Connection for UnixConnection {
    fn send(&self, _data: &[u8]) -> Result<()> {
        Err(Error::Transport(
            "send not implemented for UnixConnection".into(),
        ))
    }

    fn recv(&self) -> Result<Vec<u8>> {
        Err(Error::Transport(
            "recv not implemented for UnixConnection".into(),
        ))
    }

    fn try_recv(&self) -> Result<Option<Vec<u8>>> {
        Err(Error::Transport(
            "try_recv not implemented for UnixConnection".into(),
        ))
    }

    fn close(&self) -> Result<()> {
        Err(Error::Transport(
            "close not implemented for UnixConnection".into(),
        ))
    }
}
