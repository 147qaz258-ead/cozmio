use crate::db::Database;
use crate::error::MemoryError;
use rusqlite::params;

#[derive(Debug, Clone)]
pub struct Decision {
    pub id: Option<i64>,
    pub memory_type: String,
    pub content: String,
    pub evidence: Option<String>,
    pub related_thread_id: Option<i64>,
    pub evidence_source: String,
}

pub struct DecisionMemoryStore<'a>(&'a Database);

impl<'a> DecisionMemoryStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self(db)
    }

    pub fn insert(&self, decision: &Decision) -> Result<i64, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO decision_memory (memory_type, content, evidence, related_thread_id, evidence_source)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                decision.memory_type,
                decision.content,
                decision.evidence,
                decision.related_thread_id,
                decision.evidence_source,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all(&self) -> Result<Vec<Decision>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, memory_type, content, evidence, related_thread_id, evidence_source
             FROM decision_memory ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Decision {
                id: Some(row.get(0)?),
                memory_type: row.get(1)?,
                content: row.get(2)?,
                evidence: row.get(3)?,
                related_thread_id: row.get(4)?,
                evidence_source: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(MemoryError::from)
    }

    pub fn get_by_thread(&self, thread_id: i64) -> Result<Vec<Decision>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, memory_type, content, evidence, related_thread_id, evidence_source
             FROM decision_memory WHERE related_thread_id = ?1 ORDER BY id ASC",
        )?;
        let rows = stmt.query_map(params![thread_id], |row| {
            Ok(Decision {
                id: Some(row.get(0)?),
                memory_type: row.get(1)?,
                content: row.get(2)?,
                evidence: row.get(3)?,
                related_thread_id: row.get(4)?,
                evidence_source: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(MemoryError::from)
    }
}
