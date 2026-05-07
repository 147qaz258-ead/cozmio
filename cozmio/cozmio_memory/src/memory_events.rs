use crate::db::Database;
use crate::error::MemoryError;
use rusqlite::params;

#[derive(Debug, Clone)]
pub struct MemoryEvent {
    pub id: Option<i64>,
    pub timestamp: String,
    pub source: String,
    pub window_title: Option<String>,
    pub content: String,
    pub raw_ref: Option<String>,
    pub embedding: Option<Vec<u8>>,
    pub thread_id: Option<i64>,
    pub evidence_source: String,
}

pub struct MemoryEventsStore<'a>(&'a Database);

impl<'a> MemoryEventsStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self(db)
    }

    pub fn insert(&self, event: &MemoryEvent) -> Result<i64, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO memory_events (timestamp, source, window_title, content, raw_ref, embedding, thread_id, evidence_source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                event.timestamp,
                event.source,
                event.window_title,
                event.content,
                event.raw_ref,
                event.embedding,
                event.thread_id,
                event.evidence_source,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<MemoryEvent>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, source, window_title, content, raw_ref, embedding, thread_id, evidence_source
             FROM memory_events WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(MemoryEvent {
                id: Some(row.get(0)?),
                timestamp: row.get(1)?,
                source: row.get(2)?,
                window_title: row.get(3)?,
                content: row.get(4)?,
                raw_ref: row.get(5)?,
                embedding: row.get(6)?,
                thread_id: row.get(7)?,
                evidence_source: row.get(8)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn count(&self) -> Result<i64, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM memory_events", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn count_by_source(&self, source: &str) -> Result<i64, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memory_events WHERE source = ?1",
            params![source],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn count_by_evidence_source(&self, evidence_source: &str) -> Result<i64, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memory_events WHERE evidence_source = ?1",
            params![evidence_source],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<MemoryEvent>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, source, window_title, content, raw_ref, embedding, thread_id, evidence_source
             FROM memory_events ORDER BY id DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(MemoryEvent {
                id: Some(row.get(0)?),
                timestamp: row.get(1)?,
                source: row.get(2)?,
                window_title: row.get(3)?,
                content: row.get(4)?,
                raw_ref: row.get(5)?,
                embedding: row.get(6)?,
                thread_id: row.get(7)?,
                evidence_source: row.get(8)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(MemoryError::from)
    }

    pub fn get_in_time_range(
        &self,
        start: &str,
        end: &str,
    ) -> Result<Vec<MemoryEvent>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, source, window_title, content, raw_ref, embedding, thread_id, evidence_source
             FROM memory_events WHERE timestamp >= ?1 AND timestamp <= ?2 ORDER BY id ASC"
        )?;
        let rows = stmt.query_map(params![start, end], |row| {
            Ok(MemoryEvent {
                id: Some(row.get(0)?),
                timestamp: row.get(1)?,
                source: row.get(2)?,
                window_title: row.get(3)?,
                content: row.get(4)?,
                raw_ref: row.get(5)?,
                embedding: row.get(6)?,
                thread_id: row.get(7)?,
                evidence_source: row.get(8)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(MemoryError::from)
    }

    pub fn get_time_range(&self) -> Result<(String, String), MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let range: (String, String) = conn.query_row(
            "SELECT COALESCE(MIN(timestamp), ''), COALESCE(MAX(timestamp), '') FROM memory_events",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        Ok(range)
    }

    pub fn link_to_thread(&self, event_id: i64, thread_id: i64) -> Result<(), MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "UPDATE memory_events SET thread_id = ?1 WHERE id = ?2",
            params![thread_id, event_id],
        )?;
        Ok(())
    }
}
