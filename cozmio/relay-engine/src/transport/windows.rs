use crate::error::{Error, Result};
use crate::transport::{Connection, Transport};
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;

pub struct WindowsNamedPipe {
    address: String,
    listener: Option<TcpListener>,
}

impl WindowsNamedPipe {
    pub fn new(address: &str) -> Self {
        // Convert pipe address to TCP port
        // \\.\pipe\relay-engine -> port 7890
        let port = if address.contains("relay-engine") {
            7890
        } else {
            7890
        };
        WindowsNamedPipe {
            address: format!("127.0.0.1:{}", port),
            listener: None,
        }
    }
}

impl Transport for WindowsNamedPipe {
    fn listen(&self) -> Result<()> {
        log::info!(
            "WindowsNamedPipe listen called for address: {}",
            self.address
        );
        Ok(())
    }

    fn accept(&mut self) -> Result<Box<dyn Connection>> {
        // Use stored listener if available, create if not
        if self.listener.is_none() {
            let listener = TcpListener::bind(&self.address).map_err(|e| {
                Error::Transport(format!("Failed to bind to {}: {}", self.address, e))
            })?;
            listener
                .set_nonblocking(true)
                .map_err(|e| Error::Transport(format!("Failed to set non-blocking: {}", e)))?;
            self.listener = Some(listener);
        }

        let listener = self.listener.as_ref().unwrap();
        let (stream, _) = listener
            .accept()
            .map_err(|e| Error::Transport(format!("Accept failed: {}", e)))?;

        Ok(Box::new(WindowsPipeConnection::new(stream)))
    }

    fn address(&self) -> &str {
        &self.address
    }
}

// Thread-safe wrapper around TcpStream
pub struct WindowsPipeConnection {
    stream: Mutex<TcpStream>,
}

impl WindowsPipeConnection {
    pub fn new(stream: TcpStream) -> Self {
        WindowsPipeConnection {
            stream: Mutex::new(stream),
        }
    }

    /// Read exactly len bytes, blocking until all bytes are received
    pub fn recv_exact(&self, buf: &mut [u8]) -> Result<()> {
        let mut stream = self
            .stream
            .lock()
            .expect("Failed to lock stream mutex for recv_exact");
        let mut pos = 0;
        while pos < buf.len() {
            let n = stream.read(&mut buf[pos..]).map_err(|e| Error::Io(e))?;
            if n == 0 {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "unexpected EOF during read",
                )));
            }
            pos += n;
        }
        Ok(())
    }
}

impl Connection for WindowsPipeConnection {
    fn send(&self, data: &[u8]) -> Result<()> {
        let mut stream = self
            .stream
            .lock()
            .expect("Failed to lock stream mutex for send");
        use std::io::Write;
        stream.write_all(data).map_err(|e| Error::Io(e))?;
        stream.flush().map_err(|e| Error::Io(e))?;
        Ok(())
    }

    fn recv(&self) -> Result<Vec<u8>> {
        let mut stream = self
            .stream
            .lock()
            .expect("Failed to lock stream mutex for recv");
        let mut buf = vec![0u8; 65536];
        use std::io::Read;
        let n = stream.read(&mut buf).map_err(|e| Error::Io(e))?;
        buf.truncate(n);
        Ok(buf)
    }

    fn try_recv(&self) -> Result<Option<Vec<u8>>> {
        use std::io::Read;
        let mut stream = self
            .stream
            .lock()
            .expect("Failed to lock stream mutex for try_recv");
        // Peek to see if data is available
        let mut buf = [0u8; 1];
        match stream.read(&mut buf) {
            Ok(0) => Ok(None),
            Ok(_) => {
                // Data available, read it all
                let mut data = buf.to_vec();
                let mut temp = vec![0u8; 65536];
                loop {
                    match stream.read(&mut temp) {
                        Ok(0) => break,
                        Ok(n) => {
                            data.extend_from_slice(&temp[..n]);
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(e) => return Err(Error::Io(e)),
                    }
                }
                Ok(Some(data))
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(Error::Io(e)),
        }
    }

    fn close(&self) -> Result<()> {
        // TcpStream dropped automatically
        Ok(())
    }
}

impl Drop for WindowsPipeConnection {
    fn drop(&mut self) {
        // Stream will be closed when TcpStream is dropped
    }
}
