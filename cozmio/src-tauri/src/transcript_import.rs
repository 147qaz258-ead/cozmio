use crate::commands::AppState;
use crate::ledger::LedgerEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::Manager;
use uuid::Uuid;

pub const TRANSCRIPT_IMPORTED_EVENT_TYPE: &str = "transcript_imported";
pub const TRANSCRIPT_IMPORT_SOURCE: &str = "claude_code_transcript_import";

static LAST_TRANSCRIPT_IMPORT_SUMMARY: OnceLock<Mutex<Option<TranscriptImportSummary>>> =
    OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TranscriptImportSummary {
    pub files_scanned: usize,
    pub events_imported: usize,
    pub content_refs_created: usize,
    pub skipped_lines: usize,
    pub failed_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TranscriptImportRequest {
    #[serde(default)]
    pub file_paths: Vec<String>,
    pub directory_path: Option<String>,
}

#[tauri::command]
pub fn import_claude_transcripts(
    app: tauri::AppHandle,
    request: TranscriptImportRequest,
) -> Result<TranscriptImportSummary, String> {
    let state = app.state::<AppState>();
    let summary = import_claude_transcripts_impl(
        &state.ledger_manager,
        &request.file_paths,
        request.directory_path.as_deref(),
    )
    .map_err(|e| e.to_string())?;
    remember_summary(summary.clone());
    Ok(summary)
}

#[tauri::command]
pub fn get_transcript_import_summary() -> Result<Option<TranscriptImportSummary>, String> {
    let guard = summary_cell()
        .lock()
        .map_err(|e| format!("transcript import summary lock poisoned: {}", e))?;
    Ok(guard.clone())
}

pub fn import_claude_transcripts_impl(
    ledger_manager: &crate::ledger::LedgerManager,
    file_paths: &[String],
    directory_path: Option<&str>,
) -> std::io::Result<TranscriptImportSummary> {
    let mut summary = TranscriptImportSummary::default();
    let files = collect_explicit_transcript_paths(file_paths, directory_path)?;

    for path in files {
        summary.files_scanned += 1;
        if let Err(error) = import_transcript_file(ledger_manager, &path, &mut summary) {
            summary
                .failed_files
                .push(format!("{}: {}", path.display(), error));
        }
    }

    Ok(summary)
}

fn summary_cell() -> &'static Mutex<Option<TranscriptImportSummary>> {
    LAST_TRANSCRIPT_IMPORT_SUMMARY.get_or_init(|| Mutex::new(None))
}

fn remember_summary(summary: TranscriptImportSummary) {
    if let Ok(mut guard) = summary_cell().lock() {
        *guard = Some(summary);
    }
}

fn collect_explicit_transcript_paths(
    file_paths: &[String],
    directory_path: Option<&str>,
) -> std::io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for file_path in file_paths {
        paths.push(PathBuf::from(file_path));
    }

    if let Some(directory_path) = directory_path {
        let directory = PathBuf::from(directory_path);
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                paths.push(path);
            }
        }
    }

    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn import_transcript_file(
    ledger_manager: &crate::ledger::LedgerManager,
    path: &Path,
    summary: &mut TranscriptImportSummary,
) -> std::io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for (line_index, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        if line.trim().is_empty() {
            summary.skipped_lines += 1;
            continue;
        }

        let parsed = match serde_json::from_str::<Value>(&line) {
            Ok(value) => value,
            Err(_) => {
                summary.skipped_lines += 1;
                continue;
            }
        };

        let mut metadata = HashMap::new();
        metadata.insert("content_type".to_string(), "transcript".to_string());
        metadata.insert(
            "source_path".to_string(),
            path.to_string_lossy().to_string(),
        );
        metadata.insert("source_line".to_string(), (line_index + 1).to_string());
        metadata.insert("jsonl_valid".to_string(), "true".to_string());
        extract_optional_string_field(&parsed, &mut metadata, "type", "transcript_type");
        extract_optional_string_field(&parsed, &mut metadata, "role", "transcript_role");
        extract_optional_string_field(&parsed, &mut metadata, "uuid", "transcript_uuid");
        extract_optional_string_field(
            &parsed,
            &mut metadata,
            "parentUuid",
            "transcript_parent_uuid",
        );

        let event = LedgerEvent {
            event_id: Uuid::new_v4().to_string(),
            trace_id: extract_session_like_field(&parsed),
            session_id: extract_session_like_field(&parsed),
            timestamp: extract_timestamp(&parsed).unwrap_or_else(|| Utc::now().timestamp()),
            event_type: TRANSCRIPT_IMPORTED_EVENT_TYPE.to_string(),
            source: TRANSCRIPT_IMPORT_SOURCE.to_string(),
            window_title: None,
            process_name: None,
            raw_text: Some(line.clone()),
            content_ref: None,
            parent_event_id: None,
            metadata,
        };

        let (_event, _content_ref) =
            ledger_manager.record_event_with_content(event, line.as_bytes())?;
        summary.events_imported += 1;
        summary.content_refs_created += 1;
    }

    Ok(())
}

fn extract_optional_string_field(
    value: &Value,
    metadata: &mut HashMap<String, String>,
    source_key: &str,
    metadata_key: &str,
) {
    if let Some(text) = value.get(source_key).and_then(Value::as_str) {
        metadata.insert(metadata_key.to_string(), text.to_string());
    }
}

fn extract_session_like_field(value: &Value) -> Option<String> {
    [
        "sessionId",
        "session_id",
        "conversationId",
        "conversation_id",
    ]
    .iter()
    .find_map(|key| value.get(key).and_then(Value::as_str).map(str::to_string))
}

fn extract_timestamp(value: &Value) -> Option<i64> {
    ["timestamp", "created_at", "createdAt"]
        .iter()
        .find_map(|key| value.get(key))
        .and_then(|timestamp| {
            timestamp
                .as_i64()
                .or_else(|| timestamp.as_str().and_then(parse_timestamp_str))
        })
}

fn parse_timestamp_str(timestamp: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.timestamp())
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::LedgerManager;

    fn create_test_base_dir() -> PathBuf {
        let path = std::env::temp_dir()
            .join("cozmio_transcript_import_test")
            .join(Uuid::new_v4().to_string());
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn transcript_import_imports_valid_jsonl_and_skips_invalid_line() {
        let base_dir = create_test_base_dir();
        let fixture = base_dir.join("claude-transcript.jsonl");
        let valid_message = r#"{"type":"assistant","sessionId":"session-123","timestamp":"2026-05-01T12:00:00Z","message":{"role":"assistant","content":[{"type":"text","text":"Plan accepted"}]}}"#;
        let valid_tool_result = r#"{"type":"tool_result","sessionId":"session-123","uuid":"tool-1","timestamp":"2026-05-01T12:00:01Z","toolUseResult":{"stdout":"ok"}}"#;
        fs::write(
            &fixture,
            format!("{}\nnot valid json\n{}\n", valid_message, valid_tool_result),
        )
        .unwrap();

        let ledger = LedgerManager::with_content_store(base_dir.join("ledger")).unwrap();
        let summary =
            import_claude_transcripts_impl(&ledger, &[fixture.to_string_lossy().to_string()], None)
                .unwrap();

        assert_eq!(summary.files_scanned, 1);
        assert_eq!(summary.events_imported, 2);
        assert_eq!(summary.content_refs_created, 2);
        assert_eq!(summary.skipped_lines, 1);
        assert!(summary.failed_files.is_empty());

        let events = ledger.get_trace("session-123").unwrap();
        assert_eq!(events.len(), 2);
        for event in events {
            assert_eq!(event.event_type, TRANSCRIPT_IMPORTED_EVENT_TYPE);
            assert_eq!(event.source, TRANSCRIPT_IMPORT_SOURCE);
            assert_eq!(event.session_id.as_deref(), Some("session-123"));
            assert_eq!(
                event.metadata.get("content_type"),
                Some(&"transcript".to_string())
            );
            assert!(event.content_ref.is_some());
            let resolved = ledger
                .resolve_content_ref(event.content_ref.as_ref().unwrap())
                .unwrap();
            assert!(resolved.text.starts_with('{'));
            assert!(serde_json::from_str::<Value>(&resolved.text).is_ok());
        }
    }

    #[test]
    fn transcript_import_directory_only_scans_jsonl_files_in_that_directory() {
        let base_dir = create_test_base_dir();
        let transcript_dir = base_dir.join("transcripts");
        fs::create_dir_all(&transcript_dir).unwrap();
        fs::write(transcript_dir.join("one.jsonl"), "{\"sessionId\":\"s1\"}\n").unwrap();
        fs::write(
            transcript_dir.join("ignored.txt"),
            "{\"sessionId\":\"s2\"}\n",
        )
        .unwrap();

        let ledger = LedgerManager::with_content_store(base_dir.join("ledger")).unwrap();
        let summary = import_claude_transcripts_impl(
            &ledger,
            &[],
            Some(transcript_dir.to_string_lossy().as_ref()),
        )
        .unwrap();

        assert_eq!(summary.files_scanned, 1);
        assert_eq!(summary.events_imported, 1);
        assert_eq!(summary.skipped_lines, 0);
        assert_eq!(ledger.get_trace("s1").unwrap().len(), 1);
    }
}
