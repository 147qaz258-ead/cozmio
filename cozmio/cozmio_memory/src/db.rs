use crate::MemoryError;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self, MemoryError> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn memory_dir() -> std::path::PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("cozmio")
            .join("memory")
    }
}
