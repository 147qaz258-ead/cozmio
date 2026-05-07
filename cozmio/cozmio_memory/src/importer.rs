use crate::db::Database;
use crate::error::MemoryError;
use crate::memory_events::{MemoryEvent, MemoryEventsStore};
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// ActionRecord from action_log.jsonl (matches src-tauri/src/logging.rs)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActionRecord {
    pub timestamp: i64,
    pub trace_id: Option<String>,
    pub session_id: Option<String>,
    pub window_title: String,
    pub system_action: String,
    pub content_text: Option<String>,
    pub result_text: Option<String>,
    pub error_text: Option<String>,
    pub user_feedback: Option<String>,
}

/// Result of an import operation
#[derive(Debug)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub errors: usize,
}

impl ImportResult {
    pub fn new() -> Self {
        Self {
            imported: 0,
            skipped: 0,
            errors: 0,
        }
    }
}

/// Get the default action_log.jsonl path
pub fn default_action_log_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cozmio")
        .join("action_log.jsonl")
}

/// Import action records from action_log.jsonl into memory_events
pub fn import_from_file(db: &Database, path: &Path) -> Result<ImportResult, MemoryError> {
    let mut result = ImportResult::new();

    if !path.exists() {
        return Err(MemoryError::Import(format!(
            "action_log.jsonl not found at {:?}",
            path
        )));
    }

    let file = File::open(path).map_err(|e| MemoryError::Import(e.to_string()))?;
    let reader = BufReader::new(file);
    let store = MemoryEventsStore::new(db);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                match serde_json::from_str::<ActionRecord>(line) {
                    Ok(record) => {
                        let timestamp = Utc
                            .timestamp_opt(record.timestamp, 0)
                            .single()
                            .map(|dt| dt.to_rfc3339())
                            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                        // Build content from available fields
                        let content = build_content(&record);

                        let event = MemoryEvent {
                            id: None,
                            timestamp,
                            source: "action_log".to_string(),
                            window_title: Some(record.window_title),
                            content,
                            raw_ref: Some(line.to_string()),
                            embedding: None,
                            thread_id: None,
                            evidence_source: "imported".to_string(),
                        };

                        if let Err(e) = store.insert(&event) {
                            eprintln!("Failed to insert event: {}", e);
                            result.errors += 1;
                        } else {
                            result.imported += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("Skipping malformed line: {}", e);
                        result.skipped += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read line: {}", e);
                result.errors += 1;
            }
        }
    }

    Ok(result)
}

fn build_content(record: &ActionRecord) -> String {
    let mut parts = Vec::new();

    if let Some(trace_id) = record.trace_id.as_deref() {
        if !trace_id.is_empty() {
            parts.push(format!("trace_id: {}", trace_id));
        }
    }
    if let Some(session_id) = record.session_id.as_deref() {
        if !session_id.is_empty() {
            parts.push(format!("session_id: {}", session_id));
        }
    }
    if !record.system_action.is_empty() {
        parts.push(format!("system_action: {}", record.system_action));
    }
    if let Some(ref ct) = record.content_text {
        if !ct.is_empty() {
            parts.push(format!("content: {}", ct));
        }
    }
    if let Some(ref rt) = record.result_text {
        if !rt.is_empty() {
            parts.push(format!("result: {}", rt));
        }
    }
    if let Some(ref et) = record.error_text {
        if !et.is_empty() {
            parts.push(format!("error: {}", et));
        }
    }
    if let Some(ref uf) = record.user_feedback {
        if !uf.is_empty() {
            parts.push(format!("feedback: {}", uf));
        }
    }

    parts.join(" | ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_import_from_file() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_memory.db");
        let log_path = dir.path().join("action_log.jsonl");

        let db = Database::new(&db_path).unwrap();
        crate::run_migrations(&db).unwrap();

        // Write test records
        let record = ActionRecord {
            timestamp: 1700000000,
            trace_id: None,
            session_id: None,
            window_title: "Test Window".to_string(),
            system_action: "none".to_string(),
            content_text: Some("test content".to_string()),
            result_text: None,
            error_text: None,
            user_feedback: None,
        };
        let json = serde_json::to_string(&record).unwrap();
        let mut file = File::create(&log_path).unwrap();
        writeln!(file, "{}", json).unwrap();

        let result = import_from_file(&db, &log_path).unwrap();
        assert_eq!(result.imported, 1);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.errors, 0);

        let store = MemoryEventsStore::new(&db);
        let count = store.count().unwrap();
        assert_eq!(count, 1);

        let events = store.get_recent(10).unwrap();
        assert_eq!(events[0].window_title.as_deref(), Some("Test Window"));
        assert_eq!(events[0].evidence_source, "imported");
    }

    #[test]
    fn test_import_nonexistent_file() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_memory.db");
        let db = Database::new(&db_path).unwrap();
        crate::run_migrations(&db).unwrap();

        let result = import_from_file(&db, &PathBuf::from("/nonexistent/action_log.jsonl"));
        assert!(result.is_err());
    }

    #[test]
    fn test_build_content() {
        let record = ActionRecord {
            timestamp: 1700000000,
            trace_id: None,
            session_id: None,
            window_title: "Window".to_string(),
            system_action: "none".to_string(),
            content_text: Some("some text".to_string()),
            result_text: None,
            error_text: None,
            user_feedback: Some("helpful".to_string()),
        };

        let content = build_content(&record);
        assert!(!content.contains("judgment: good"));
        assert!(!content.contains("next_step: continue"));
        assert!(!content.contains("confidence: 0.90"));
        assert!(!content.contains("grounds: looks fine"));
        assert!(content.contains("content: some text"));
        assert!(content.contains("feedback: helpful"));
    }
}
