use crate::db::Database;
use crate::error::MemoryError;
use chrono::{DateTime, Utc};
use rusqlite::params;

#[derive(Debug, Clone)]
pub struct ContextSlice {
    pub id: Option<i64>,
    pub start_time: String,
    pub end_time: String,
    pub summary: String,
    pub entities: Vec<String>,
    pub topics: Vec<String>,
    pub raw_refs: Vec<i64>,
    pub evidence_source: String,
}

pub struct ContextSlicesStore<'a>(&'a Database);

impl<'a> ContextSlicesStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self(db)
    }

    pub fn insert(&self, slice: &ContextSlice) -> Result<i64, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO context_slices (start_time, end_time, summary, entities, topics, raw_refs, evidence_source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                slice.start_time,
                slice.end_time,
                slice.summary,
                serde_json::to_string(&slice.entities).unwrap_or_default(),
                serde_json::to_string(&slice.topics).unwrap_or_default(),
                serde_json::to_string(&slice.raw_refs).unwrap_or_default(),
                slice.evidence_source,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Alias for insert() - inserts a context slice and returns the row ID.
    pub fn insert_slice(&self, slice: &ContextSlice) -> Result<i64, MemoryError> {
        self.insert(slice)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<ContextSlice>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, start_time, end_time, summary, entities, topics, raw_refs, evidence_source
             FROM context_slices WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_slice(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<ContextSlice>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, start_time, end_time, summary, entities, topics, raw_refs, evidence_source
             FROM context_slices ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            ContextSlicesStore::<'_>::row_to_slice_row(row)
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(MemoryError::from)
    }

    pub fn get_since(&self, since_hours: u32) -> Result<Vec<ContextSlice>, MemoryError> {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(since_hours as i64);
        self.get_since_ts(cutoff)
    }

    /// Get context slices since a specific timestamp (used by ReminderContext to use ActivityNote.timestamp as reference).
    pub fn get_since_ts(&self, timestamp: DateTime<Utc>) -> Result<Vec<ContextSlice>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let cutoff_str = timestamp.to_rfc3339();
        let mut stmt = conn.prepare(
            "SELECT id, start_time, end_time, summary, entities, topics, raw_refs, evidence_source
             FROM context_slices WHERE start_time >= ?1 ORDER BY id ASC",
        )?;
        let rows = stmt.query_map(params![cutoff_str], |row| {
            ContextSlicesStore::<'_>::row_to_slice_row(row)
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(MemoryError::from)
    }

    fn row_to_slice(&self, row: &rusqlite::Row) -> Result<ContextSlice, MemoryError> {
        let entities_str: String = row.get(4)?;
        let topics_str: String = row.get(5)?;
        let raw_refs_str: String = row.get(6)?;
        Ok(ContextSlice {
            id: Some(row.get(0)?),
            start_time: row.get(1)?,
            end_time: row.get(2)?,
            summary: row.get(3)?,
            entities: serde_json::from_str(&entities_str).unwrap_or_default(),
            topics: serde_json::from_str(&topics_str).unwrap_or_default(),
            raw_refs: serde_json::from_str(&raw_refs_str).unwrap_or_default(),
            evidence_source: row.get(7)?,
        })
    }

    fn row_to_slice_row(row: &rusqlite::Row) -> Result<ContextSlice, rusqlite::Error> {
        let entities_str: String = row.get(4)?;
        let topics_str: String = row.get(5)?;
        let raw_refs_str: String = row.get(6)?;
        Ok(ContextSlice {
            id: Some(row.get(0)?),
            start_time: row.get(1)?,
            end_time: row.get(2)?,
            summary: row.get(3)?,
            entities: serde_json::from_str(&entities_str).unwrap_or_default(),
            topics: serde_json::from_str(&topics_str).unwrap_or_default(),
            raw_refs: serde_json::from_str(&raw_refs_str).unwrap_or_default(),
            evidence_source: row.get(7)?,
        })
    }
}
