use crate::db::Database;
use crate::error::MemoryError;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMemory {
    pub id: Option<i64>,
    pub body: String,
    pub layer: String,
    pub lifecycle: String,
    pub source_refs_json: String,
    pub supersedes_id: Option<i64>,
    pub producer: String,
    pub used_count: i64,
    pub last_used_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub rejected_reason: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryOperation {
    pub id: Option<i64>,
    pub memory_id: i64,
    pub operation: String,
    pub layer: String,
    pub lifecycle: String,
    pub body: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExperienceSource {
    pub source_ref: String,
    pub timestamp: Option<String>,
    pub trace_id: Option<String>,
    pub session_id: Option<String>,
    pub event_kind: String,
    pub window_title: Option<String>,
    pub process_name: Option<String>,
    pub factual_text: String,
    pub raw_ref: Option<String>,
    pub artifact_refs_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsolidationRun {
    pub id: Option<i64>,
    pub trigger_kind: String,
    pub route: String,
    pub packet_json: String,
    pub model_name: Option<String>,
    pub output_text: Option<String>,
    pub status: String,
    pub error_text: Option<String>,
    pub created_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryOperationRecord {
    pub id: Option<i64>,
    pub operation_type: String,
    pub target_memory_id: Option<i64>,
    pub resulting_memory_id: Option<i64>,
    pub body: Option<String>,
    pub layer: Option<String>,
    pub source_refs_json: String,
    pub status: String,
    pub error_text: Option<String>,
    pub producer: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryWrite {
    pub body: String,
    pub layer: String,
    pub lifecycle: String,
    pub source_refs: Vec<String>,
    pub supersedes_id: Option<i64>,
    pub producer: String,
    pub expires_at: Option<String>,
}

pub struct AgentMemoryStore<'a>(&'a Database);

impl<'a> AgentMemoryStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self(db)
    }

    /// Legacy/bootstrap insert API. Keeps compatibility with earlier callers that did not yet
    /// provide provenance. New memory writes should use `remember_memory` with source refs.
    pub fn insert(&self, memory: &AgentMemory) -> Result<i64, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let source_refs_json = if memory.source_refs_json.trim().is_empty() {
            "[]".to_string()
        } else {
            memory.source_refs_json.clone()
        };
        let producer = if memory.producer.trim().is_empty() {
            "legacy".to_string()
        } else {
            memory.producer.clone()
        };
        conn.execute(
            "INSERT INTO agent_memories (
                body, layer, lifecycle, source_refs_json, supersedes_id, producer,
                used_count, last_used_at, rejected_reason, expires_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                memory.body,
                memory.layer,
                memory.lifecycle,
                source_refs_json,
                memory.supersedes_id,
                producer,
                memory.used_count,
                memory.last_used_at,
                memory.rejected_reason,
                memory.expires_at,
            ],
        )?;
        let id = conn.last_insert_rowid();
        insert_operation_row(
            &conn,
            "inserted",
            Some(id),
            Some(id),
            Some(&memory.body),
            Some(&memory.layer),
            Some(&source_refs_json),
            "applied",
            None,
            &producer,
        )?;
        Ok(id)
    }

    pub fn remember_memory(&self, write: MemoryWrite) -> Result<i64, MemoryError> {
        validate_non_empty("memory body", &write.body)?;
        validate_non_empty("memory layer", &write.layer)?;
        validate_non_empty("memory lifecycle", &write.lifecycle)?;
        validate_non_empty("memory producer", &write.producer)?;
        validate_source_refs(&write.source_refs)?;
        let source_refs_json = source_refs_json(&write.source_refs)?;

        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO agent_memories (
                body, layer, lifecycle, source_refs_json, supersedes_id, producer, expires_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                write.body,
                write.layer,
                write.lifecycle,
                source_refs_json,
                write.supersedes_id,
                write.producer,
                write.expires_at,
            ],
        )?;
        let id = conn.last_insert_rowid();
        insert_operation_row(
            &conn,
            "remembered",
            write.supersedes_id,
            Some(id),
            Some(&write.body),
            Some(&write.layer),
            Some(&source_refs_json),
            "applied",
            None,
            &write.producer,
        )?;
        Ok(id)
    }

    pub fn list_active(&self) -> Result<Vec<AgentMemory>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, body, layer, lifecycle, source_refs_json, supersedes_id, producer,
                    used_count, last_used_at, created_at, updated_at, rejected_reason, expires_at
             FROM agent_memories
             WHERE lifecycle = 'active'
               AND (expires_at IS NULL OR expires_at > datetime('now'))
             ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([], read_agent_memory)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(MemoryError::from)
    }

    pub fn get(&self, id: i64) -> Result<Option<AgentMemory>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, body, layer, lifecycle, source_refs_json, supersedes_id, producer,
                    used_count, last_used_at, created_at, updated_at, rejected_reason, expires_at
             FROM agent_memories WHERE id = ?1",
        )?;
        stmt.query_row(params![id], read_agent_memory)
            .optional()
            .map_err(MemoryError::from)
    }

    pub fn reject(&self, id: i64) -> Result<(), MemoryError> {
        self.reject_with_reason(id, None, "system")
    }

    pub fn reject_with_reason(
        &self,
        id: i64,
        reason: Option<&str>,
        producer: &str,
    ) -> Result<(), MemoryError> {
        validate_non_empty("producer", producer)?;
        let conn = self.0.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE agent_memories
             SET lifecycle = 'rejected', rejected_reason = ?2, updated_at = datetime('now')
             WHERE id = ?1",
            params![id, reason],
        )?;
        ensure_rows_affected(rows, "agent memory", id)?;
        self.log_current_with_type(&conn, id, "rejected", producer)
    }

    pub fn supersede_memory(&self, old_id: i64, write: MemoryWrite) -> Result<i64, MemoryError> {
        validate_non_empty("memory body", &write.body)?;
        validate_non_empty("memory layer", &write.layer)?;
        validate_non_empty("memory lifecycle", &write.lifecycle)?;
        validate_non_empty("memory producer", &write.producer)?;
        validate_source_refs(&write.source_refs)?;
        let source_refs_json = source_refs_json(&write.source_refs)?;

        let mut conn = self.0.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let old_exists: Option<i64> = tx
            .query_row(
                "SELECT id FROM agent_memories WHERE id = ?1",
                params![old_id],
                |row| row.get(0),
            )
            .optional()?;
        if old_exists.is_none() {
            return Err(MemoryError::NotFound(format!("agent memory {old_id}")));
        }

        tx.execute(
            "INSERT INTO agent_memories (
                body, layer, lifecycle, source_refs_json, supersedes_id, producer, expires_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                write.body,
                write.layer,
                write.lifecycle,
                source_refs_json,
                old_id,
                write.producer,
                write.expires_at,
            ],
        )?;
        let new_id = tx.last_insert_rowid();
        insert_operation_row(
            &tx,
            "remembered",
            Some(old_id),
            Some(new_id),
            Some(&write.body),
            Some(&write.layer),
            Some(&source_refs_json),
            "applied",
            None,
            &write.producer,
        )?;

        let rows = tx.execute(
            "UPDATE agent_memories
             SET lifecycle = 'superseded', updated_at = datetime('now')
             WHERE id = ?1",
            params![old_id],
        )?;
        ensure_rows_affected(rows, "agent memory", old_id)?;
        self.log_current_with_type(&tx, old_id, "superseded", "system")?;
        tx.commit()?;
        Ok(new_id)
    }

    pub fn mark_used(&self, id: i64) -> Result<(), MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE agent_memories
             SET used_count = used_count + 1,
                 last_used_at = datetime('now'),
                 updated_at = datetime('now')
             WHERE id = ?1",
            params![id],
        )?;
        ensure_rows_affected(rows, "agent memory", id)?;
        self.log_current_with_type(&conn, id, "used", "system")
    }

    pub fn recall_active_by_text(
        &self,
        text: &str,
        limit: usize,
    ) -> Result<Vec<AgentMemory>, MemoryError> {
        let mut memories = self.list_active()?;
        memories.retain(|memory| recall_score(&memory.body, text) > 0);
        memories.sort_by(|left, right| {
            recall_score(&right.body, text)
                .cmp(&recall_score(&left.body, text))
                .then_with(|| right.used_count.cmp(&left.used_count))
                .then_with(|| left.id.cmp(&right.id))
        });
        memories.truncate(limit);
        Ok(memories)
    }

    pub fn operations(&self, memory_id: i64) -> Result<Vec<MemoryOperation>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, memory_id, operation, layer, lifecycle, body, created_at
             FROM memory_operations WHERE memory_id = ?1 ORDER BY id ASC",
        )?;
        let rows = stmt.query_map(params![memory_id], |row| {
            Ok(MemoryOperation {
                id: Some(row.get(0)?),
                memory_id: row.get(1)?,
                operation: row.get(2)?,
                layer: row.get(3)?,
                lifecycle: row.get(4)?,
                body: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(MemoryError::from)
    }

    pub fn record_operation(&self, operation: MemoryOperationRecord) -> Result<i64, MemoryError> {
        validate_non_empty("operation type", &operation.operation_type)?;
        validate_non_empty("operation status", &operation.status)?;
        validate_non_empty("producer", &operation.producer)?;
        let conn = self.0.conn.lock().unwrap();
        insert_operation_row(
            &conn,
            &operation.operation_type,
            operation.target_memory_id,
            operation.resulting_memory_id,
            operation.body.as_deref(),
            operation.layer.as_deref(),
            Some(&operation.source_refs_json),
            &operation.status,
            operation.error_text.as_deref(),
            &operation.producer,
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn insert_experience_source(
        &self,
        source: &ExperienceSource,
    ) -> Result<String, MemoryError> {
        validate_non_empty("source ref", &source.source_ref)?;
        validate_non_empty("event kind", &source.event_kind)?;
        validate_non_empty("factual text", &source.factual_text)?;
        let artifact_refs_json = if source.artifact_refs_json.trim().is_empty() {
            "[]".to_string()
        } else {
            source.artifact_refs_json.clone()
        };
        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO experience_sources (
                source_ref, timestamp, trace_id, session_id, event_kind, window_title,
                process_name, factual_text, raw_ref, artifact_refs_json
             ) VALUES (?1, COALESCE(?2, datetime('now')), ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                source.source_ref,
                source.timestamp,
                source.trace_id,
                source.session_id,
                source.event_kind,
                source.window_title,
                source.process_name,
                source.factual_text,
                source.raw_ref,
                artifact_refs_json,
            ],
        )?;
        Ok(source.source_ref.clone())
    }

    pub fn recent_experience_sources(
        &self,
        limit: usize,
    ) -> Result<Vec<ExperienceSource>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT source_ref, timestamp, trace_id, session_id, event_kind, window_title,
                    process_name, factual_text, raw_ref, artifact_refs_json
             FROM experience_sources ORDER BY timestamp DESC, source_ref DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], read_experience_source)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(MemoryError::from)
    }

    pub fn create_consolidation_run(
        &self,
        trigger_kind: &str,
        route: &str,
        packet_json: &str,
        model_name: Option<&str>,
    ) -> Result<i64, MemoryError> {
        validate_non_empty("trigger kind", trigger_kind)?;
        validate_non_empty("route", route)?;
        validate_non_empty("packet json", packet_json)?;
        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO consolidation_runs (trigger_kind, route, packet_json, model_name, status)
             VALUES (?1, ?2, ?3, ?4, 'running')",
            params![trigger_kind, route, packet_json, model_name],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn complete_consolidation_run(
        &self,
        run_id: i64,
        output_text: &str,
    ) -> Result<(), MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE consolidation_runs
             SET output_text = ?2, status = 'completed', error_text = NULL, completed_at = datetime('now')
             WHERE id = ?1",
            params![run_id, output_text],
        )?;
        ensure_rows_affected(rows, "consolidation run", run_id)?;
        Ok(())
    }

    pub fn fail_consolidation_run(&self, run_id: i64, error_text: &str) -> Result<(), MemoryError> {
        validate_non_empty("error text", error_text)?;
        let conn = self.0.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE consolidation_runs
             SET status = 'failed', error_text = ?2, completed_at = datetime('now')
             WHERE id = ?1",
            params![run_id, error_text],
        )?;
        ensure_rows_affected(rows, "consolidation run", run_id)?;
        Ok(())
    }

    pub fn get_consolidation_run(
        &self,
        run_id: i64,
    ) -> Result<Option<ConsolidationRun>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, trigger_kind, route, packet_json, model_name, output_text, status,
                    error_text, created_at, completed_at
             FROM consolidation_runs WHERE id = ?1",
        )?;
        stmt.query_row(params![run_id], read_consolidation_run)
            .optional()
            .map_err(MemoryError::from)
    }

    fn log_current_with_type(
        &self,
        conn: &rusqlite::Connection,
        id: i64,
        operation_type: &str,
        producer: &str,
    ) -> Result<(), MemoryError> {
        conn.execute(
            "INSERT INTO memory_operations (
                memory_id, operation, operation_type, target_memory_id, resulting_memory_id,
                layer, lifecycle, body, source_refs_json, status, producer
             )
             SELECT id, ?1, ?1, id, id, layer, lifecycle, body, source_refs_json, 'applied', ?3
             FROM agent_memories WHERE id = ?2",
            params![operation_type, id, producer],
        )?;
        Ok(())
    }
}

fn insert_operation_row(
    conn: &rusqlite::Connection,
    operation_type: &str,
    target_memory_id: Option<i64>,
    resulting_memory_id: Option<i64>,
    body: Option<&str>,
    layer: Option<&str>,
    source_refs_json: Option<&str>,
    status: &str,
    error_text: Option<&str>,
    producer: &str,
) -> Result<(), MemoryError> {
    let legacy_memory_id = resulting_memory_id.or(target_memory_id).unwrap_or(0);
    let legacy_operation = operation_type;
    let legacy_layer = layer.unwrap_or("");
    let legacy_body = body.unwrap_or("");
    conn.execute(
        "INSERT INTO memory_operations (
            memory_id, operation, lifecycle, operation_type, target_memory_id, resulting_memory_id,
            body, layer, source_refs_json, status, error_text, producer
         ) VALUES (?1, ?2, 'active', ?3, ?4, ?5, ?6, ?7, COALESCE(?8, '[]'), ?9, ?10, ?11)",
        params![
            legacy_memory_id,
            legacy_operation,
            operation_type,
            target_memory_id,
            resulting_memory_id,
            legacy_body,
            legacy_layer,
            source_refs_json,
            status,
            error_text,
            producer,
        ],
    )?;
    if legacy_body.is_empty() {
        conn.execute(
            "UPDATE memory_operations SET body = '' WHERE id = last_insert_rowid()",
            [],
        )?;
    }
    Ok(())
}

fn read_agent_memory(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentMemory> {
    Ok(AgentMemory {
        id: Some(row.get(0)?),
        body: row.get(1)?,
        layer: row.get(2)?,
        lifecycle: row.get(3)?,
        source_refs_json: row.get(4)?,
        supersedes_id: row.get(5)?,
        producer: row.get(6)?,
        used_count: row.get(7)?,
        last_used_at: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        rejected_reason: row.get(11)?,
        expires_at: row.get(12)?,
    })
}

fn read_experience_source(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExperienceSource> {
    Ok(ExperienceSource {
        source_ref: row.get(0)?,
        timestamp: row.get(1)?,
        trace_id: row.get(2)?,
        session_id: row.get(3)?,
        event_kind: row.get(4)?,
        window_title: row.get(5)?,
        process_name: row.get(6)?,
        factual_text: row.get(7)?,
        raw_ref: row.get(8)?,
        artifact_refs_json: row.get(9)?,
    })
}

fn read_consolidation_run(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConsolidationRun> {
    Ok(ConsolidationRun {
        id: Some(row.get(0)?),
        trigger_kind: row.get(1)?,
        route: row.get(2)?,
        packet_json: row.get(3)?,
        model_name: row.get(4)?,
        output_text: row.get(5)?,
        status: row.get(6)?,
        error_text: row.get(7)?,
        created_at: row.get(8)?,
        completed_at: row.get(9)?,
    })
}

fn ensure_rows_affected(rows: usize, entity: &str, id: i64) -> Result<(), MemoryError> {
    if rows == 0 {
        return Err(MemoryError::NotFound(format!("{entity} {id}")));
    }
    Ok(())
}

fn validate_non_empty(field: &str, value: &str) -> Result<(), MemoryError> {
    if value.trim().is_empty() {
        return Err(MemoryError::Import(format!("{field} must not be empty")));
    }
    Ok(())
}

fn validate_source_refs(source_refs: &[String]) -> Result<(), MemoryError> {
    if source_refs.is_empty()
        || source_refs
            .iter()
            .any(|source_ref| source_ref.trim().is_empty())
    {
        return Err(MemoryError::Import(
            "new memory writes require at least one non-empty source ref".to_string(),
        ));
    }
    Ok(())
}

fn source_refs_json(source_refs: &[String]) -> Result<String, MemoryError> {
    validate_source_refs(source_refs)?;
    serde_json::to_string(source_refs).map_err(MemoryError::from)
}

fn recall_score(body: &str, text: &str) -> usize {
    let body = body.to_lowercase();
    recall_terms(text)
        .iter()
        .filter(|term| body.contains(term.as_str()))
        .count()
}

fn recall_terms(text: &str) -> HashSet<String> {
    text.split(|ch: char| !(ch.is_alphanumeric() || ch == '_' || ch as u32 > 0x7f))
        .map(str::trim)
        .filter(|term| term.chars().count() >= 2)
        .map(str::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::run_migrations;
    use rusqlite::Connection;
    use std::sync::Mutex;

    fn test_db() -> Database {
        let db = Database {
            conn: Mutex::new(Connection::open_in_memory().unwrap()),
        };
        run_migrations(&db).unwrap();
        db
    }

    fn memory(body: &str, layer: &str) -> AgentMemory {
        AgentMemory {
            id: None,
            body: body.to_string(),
            layer: layer.to_string(),
            lifecycle: "active".to_string(),
            source_refs_json: "[]".to_string(),
            supersedes_id: None,
            producer: "legacy-test".to_string(),
            used_count: 0,
            last_used_at: None,
            created_at: None,
            updated_at: None,
            rejected_reason: None,
            expires_at: None,
        }
    }

    fn source(source_ref: &str) -> ExperienceSource {
        ExperienceSource {
            source_ref: source_ref.to_string(),
            timestamp: None,
            trace_id: Some("trace-1".to_string()),
            session_id: Some("session-1".to_string()),
            event_kind: "popup_displayed".to_string(),
            window_title: Some("Cozmio".to_string()),
            process_name: Some("cozmio.exe".to_string()),
            factual_text: "Popup text was displayed to the user.".to_string(),
            raw_ref: Some("raw://popup/1".to_string()),
            artifact_refs_json: "[]".to_string(),
        }
    }

    fn write(body: &str, layer: &str, source_ref: &str) -> MemoryWrite {
        MemoryWrite {
            body: body.to_string(),
            layer: layer.to_string(),
            lifecycle: "active".to_string(),
            source_refs: vec![source_ref.to_string()],
            supersedes_id: None,
            producer: "consolidation-agent".to_string(),
            expires_at: None,
        }
    }

    #[test]
    fn insert_get_and_list_active_round_trip_memory_body() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);

        let id = store
            .insert(&memory(
                "When editing this crate, keep model-facing memory as natural language.",
                "agent",
            ))
            .unwrap();

        let stored = store.get(id).unwrap().unwrap();
        assert_eq!(
            stored.body,
            "When editing this crate, keep model-facing memory as natural language."
        );
        assert_eq!(stored.layer, "agent");
        assert_eq!(stored.lifecycle, "active");
        assert_eq!(store.list_active().unwrap().len(), 1);
    }

    #[test]
    fn reject_removes_memory_from_active_recall() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);
        let id = store
            .insert(&memory(
                "Prefer targeted tests when touching cozmio_memory.",
                "agent",
            ))
            .unwrap();

        store.reject(id).unwrap();

        let stored = store.get(id).unwrap().unwrap();
        assert_eq!(stored.lifecycle, "rejected");
        assert!(store.list_active().unwrap().is_empty());
        assert!(store
            .recall_active_by_text("targeted tests", 10)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn mark_used_updates_usage_and_records_operation() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);
        let id = store
            .insert(&memory(
                "Run the memory crate tests before reporting completion.",
                "agent",
            ))
            .unwrap();

        store.mark_used(id).unwrap();

        let stored = store.get(id).unwrap().unwrap();
        assert_eq!(stored.used_count, 1);
        assert!(stored.last_used_at.is_some());

        let operations = store.operations(id).unwrap();
        assert_eq!(operations.len(), 2);
        assert_eq!(operations[0].operation, "inserted");
        assert_eq!(operations[1].operation, "used");
    }

    #[test]
    fn recall_active_by_text_returns_matching_active_memories_first_by_usage() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);
        let first = store
            .insert(&memory(
                "Use natural language memory bodies for recall.",
                "agent",
            ))
            .unwrap();
        let second = store
            .insert(&memory(
                "Natural language notes should stay readable.",
                "agent",
            ))
            .unwrap();
        store.mark_used(second).unwrap();
        store.mark_used(second).unwrap();
        store.mark_used(first).unwrap();

        let recalled = store.recall_active_by_text("language", 10).unwrap();

        assert_eq!(recalled.len(), 2);
        assert_eq!(recalled[0].id, Some(second));
        assert_eq!(recalled[1].id, Some(first));
    }

    #[test]
    fn recall_active_by_text_matches_partial_context_terms() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);
        let matched = store
            .insert(&memory(
                "Source window: Cozmio - Claude Code. Model wrote: 继续推进自动记忆飞轮。",
                "episode",
            ))
            .unwrap();
        store
            .insert(&memory("A separate note about browser tabs.", "episode"))
            .unwrap();

        let recalled = store
            .recall_active_by_text("current_window title=Cozmio process=Code.exe 自动飞轮", 10)
            .unwrap();

        assert_eq!(recalled.len(), 1);
        assert_eq!(recalled[0].id, Some(matched));
    }

    #[test]
    fn schema_migration_adds_target_columns_and_tables_idempotently() {
        let db = test_db();
        run_migrations(&db).unwrap();
        let conn = db.conn.lock().unwrap();
        let tables = [
            ("agent_memories", "source_refs_json"),
            ("agent_memories", "supersedes_id"),
            ("memory_operations", "operation_type"),
            ("memory_operations", "resulting_memory_id"),
            ("consolidation_runs", "packet_json"),
            ("experience_sources", "factual_text"),
        ];
        for (table, column) in tables {
            let found: i64 = conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = ?1"),
                    params![column],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(found, 1, "missing {table}.{column}");
        }
    }

    #[test]
    fn source_run_and_operation_rows_round_trip() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);
        store
            .insert_experience_source(&source("trace-1:event-1"))
            .unwrap();

        let recent = store.recent_experience_sources(10).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].source_ref, "trace-1:event-1");
        assert_eq!(recent[0].event_kind, "popup_displayed");

        let run_id = store
            .create_consolidation_run(
                "after_feedback",
                "local",
                r#"{"source_refs":["trace-1:event-1"]}"#,
                Some("llava"),
            )
            .unwrap();
        store.complete_consolidation_run(run_id, "abstain").unwrap();
        let run = store.get_consolidation_run(run_id).unwrap().unwrap();
        assert_eq!(run.status, "completed");
        assert_eq!(run.output_text.as_deref(), Some("abstain"));
        assert!(run.completed_at.is_some());

        let op_id = store
            .record_operation(MemoryOperationRecord {
                id: None,
                operation_type: "abstain".to_string(),
                target_memory_id: None,
                resulting_memory_id: None,
                body: None,
                layer: None,
                source_refs_json: r#"["trace-1:event-1"]"#.to_string(),
                status: "applied".to_string(),
                error_text: None,
                producer: "consolidation-agent".to_string(),
                created_at: None,
            })
            .unwrap();
        assert!(op_id > 0);
    }

    #[test]
    fn new_memory_writes_require_non_empty_source_refs() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);
        let mut write = write(
            "Remember source-backed facts only.",
            "reflection",
            "trace-1:event-1",
        );
        write.source_refs.clear();

        let err = store.remember_memory(write).unwrap_err().to_string();
        assert!(err.contains("source ref"));
    }

    #[test]
    fn reject_and_supersede_keep_memories_inspectable_but_out_of_recall() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);
        store
            .insert_experience_source(&source("trace-1:event-1"))
            .unwrap();
        let rejected = store
            .remember_memory(write(
                "Use the old popup phrasing.",
                "reflection",
                "trace-1:event-1",
            ))
            .unwrap();
        let old = store
            .remember_memory(write(
                "Use detailed memory flywheel handoffs.",
                "procedure",
                "trace-1:event-1",
            ))
            .unwrap();

        store
            .reject_with_reason(rejected, Some("user rejected"), "developer")
            .unwrap();
        let replacement = store
            .supersede_memory(
                old,
                write(
                    "Use concise memory flywheel handoffs.",
                    "procedure",
                    "trace-1:event-1",
                ),
            )
            .unwrap();

        assert_eq!(store.get(rejected).unwrap().unwrap().lifecycle, "rejected");
        assert_eq!(store.get(old).unwrap().unwrap().lifecycle, "superseded");
        assert_eq!(
            store.get(replacement).unwrap().unwrap().supersedes_id,
            Some(old)
        );

        let recalled = store
            .recall_active_by_text("memory flywheel handoffs popup", 10)
            .unwrap();
        assert_eq!(recalled.len(), 1);
        assert_eq!(recalled[0].id, Some(replacement));
    }

    #[test]
    fn supersede_failure_preserves_original_active_memory() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);
        store
            .insert_experience_source(&source("trace-1:event-1"))
            .unwrap();
        let original = store
            .remember_memory(write(
                "Keep original active when replacement is invalid.",
                "procedure",
                "trace-1:event-1",
            ))
            .unwrap();
        let mut invalid = write(
            "Replacement without source refs should fail.",
            "procedure",
            "trace-1:event-1",
        );
        invalid.source_refs.clear();

        let err = store
            .supersede_memory(original, invalid)
            .unwrap_err()
            .to_string();

        assert!(err.contains("source ref"));
        assert_eq!(store.get(original).unwrap().unwrap().lifecycle, "active");
        let recalled = store
            .recall_active_by_text("original active replacement", 10)
            .unwrap();
        assert_eq!(recalled.len(), 1);
        assert_eq!(recalled[0].id, Some(original));
    }

    #[test]
    fn partial_consolidation_and_experience_schemas_upgrade_idempotently() {
        let db = Database {
            conn: Mutex::new(Connection::open_in_memory().unwrap()),
        };
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "CREATE TABLE consolidation_runs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    status TEXT NOT NULL DEFAULT 'running'
                )",
                [],
            )
            .unwrap();
            conn.execute(
                "CREATE TABLE experience_sources (
                    source_ref TEXT PRIMARY KEY
                )",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO consolidation_runs (status) VALUES ('legacy')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO experience_sources (source_ref) VALUES ('legacy-source')",
                [],
            )
            .unwrap();
        }

        run_migrations(&db).unwrap();
        run_migrations(&db).unwrap();

        let conn = db.conn.lock().unwrap();
        for (table, column) in [
            ("consolidation_runs", "trigger_kind"),
            ("consolidation_runs", "route"),
            ("consolidation_runs", "packet_json"),
            ("consolidation_runs", "completed_at"),
            ("experience_sources", "timestamp"),
            ("experience_sources", "event_kind"),
            ("experience_sources", "factual_text"),
            ("experience_sources", "artifact_refs_json"),
        ] {
            let found: i64 = conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = ?1"),
                    params![column],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(found, 1, "missing {table}.{column}");
        }
        let packet_json: String = conn
            .query_row(
                "SELECT packet_json FROM consolidation_runs WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let factual_text: String = conn
            .query_row(
                "SELECT factual_text FROM experience_sources WHERE source_ref = 'legacy-source'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(packet_json, "{}");
        assert_eq!(factual_text, "");
    }

    #[test]
    fn missing_row_updates_return_errors() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);

        assert!(store
            .mark_used(404)
            .unwrap_err()
            .to_string()
            .contains("Not found"));
        assert!(store
            .reject_with_reason(404, Some("missing"), "test")
            .unwrap_err()
            .to_string()
            .contains("Not found"));
        assert!(store
            .complete_consolidation_run(404, "output")
            .unwrap_err()
            .to_string()
            .contains("Not found"));
        assert!(store
            .fail_consolidation_run(404, "failed")
            .unwrap_err()
            .to_string()
            .contains("Not found"));
    }
}
