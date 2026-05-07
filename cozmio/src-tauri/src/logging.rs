use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

/// A record of an action logged by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub timestamp: i64,
    pub trace_id: Option<String>,
    pub session_id: Option<String>,
    pub window_title: String,
    pub judgment: String,
    #[serde(alias = "next_step")]
    pub model_text: String,
    #[serde(alias = "level")]
    pub status_label: String,
    #[serde(alias = "confidence")]
    pub confidence_score: f32,
    pub grounds: String,
    pub system_action: String,
    pub content_text: Option<String>,
    pub result_text: Option<String>,
    pub error_text: Option<String>,
    pub user_feedback: Option<String>,
    /// The actual model name used (config name or fallback)
    pub model_name: Option<String>,
    /// When the screenshot was captured (Unix timestamp)
    pub captured_at: Option<i64>,
    /// When the API call started (Unix timestamp)
    pub call_started_at: Option<i64>,
    /// How long the API call took (milliseconds)
    pub call_duration_ms: Option<u64>,
}

/// Factual action record - separates concerns that legacy ActionRecord mixed together.
///
/// Facts are provided by the system. Semantic summaries come from models and must
/// include provenance (timestamp, source path, source range, producer).
///
/// 以下字段由系统提供事实：
///   timestamp, trace_id, window_title, event_type, system_route
/// 以下字段是模型原文（只读保存，不做解析）：
///   raw_model_text, model_name
/// 以下字段是执行端返回（只读保存）：
///   execution_result, error_text
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FactualActionRecord {
    /// Unix timestamp when this record was created
    pub timestamp: i64,
    /// Unique trace ID for this action
    pub trace_id: Option<String>,
    /// Session ID if this action is part of a relay session
    pub session_id: Option<String>,
    /// Window title at the time of this record
    pub window_title: String,
    /// Type of the primary event - distinguishes system events from model outputs
    pub event_type: FactualEventType,
    /// System routing decision (not model judgment)
    pub system_route: SystemRoute,
    /// Original model judgment before execution (CONTINUE or ABSTAIN)
    /// Preserved separately from system_route which gets updated post-execution
    pub original_judgment: String,
    /// Execution result as a string (notified, confirmed, executed, skipped)
    /// This is the legacy system_action value
    pub execution_result_str: String,
    /// Raw text from the model - preserved as-is, not parsed into structured fields.
    /// Model output is natural language. If semantic summaries are needed, they
    /// must come from a model or execution agent with provenance.
    pub raw_model_text: Option<String>,
    /// Model name used for this output (config name or fallback)
    pub model_name: Option<String>,
    /// When the screenshot was captured (Unix timestamp)
    pub captured_at: Option<i64>,
    /// When the API call started (Unix timestamp)
    pub call_started_at: Option<i64>,
    /// How long the API call took (milliseconds)
    pub call_duration_ms: Option<u64>,
    /// Execution result text from the execution side
    pub execution_result: Option<String>,
    /// Error text from system or execution side
    pub error_text: Option<String>,
    /// User UI feedback (e.g. "ui_confirmed", "ui_cancelled", "ui_dismissed")
    pub user_feedback: Option<String>,
}

/// Event types that the system can factually observe
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FactualEventType {
    /// Model was called and returned output
    ModelOutput,
    /// User confirmed a pending notification via UI
    UiConfirmed,
    /// User cancelled a pending notification via UI
    UiCancelled,
    /// User dismissed a pending notification via UI
    UiDismissed,
    /// Relay session completed
    RelayCompleted,
    /// Relay session failed
    RelayFailed,
    /// Model call resulted in an error
    ModelError,
    /// System-level error (not model-related)
    SystemError,
}

/// System routing decisions - these are factual system events, not model semantics
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SystemRoute {
    /// User confirmed the action
    Confirmed,
    /// User declined or action was auto-declined
    Declined,
    /// Action was executed automatically
    AutoExecuted,
    /// No pending action (e.g. stale confirm URL replay)
    NoPendingAction,
    /// Execution completed successfully
    Completed,
    /// Execution failed
    Failed,
    /// Execution was interrupted
    Interrupted,
    /// Model output suggested continuation (CONTINUE mode)
    Continue,
    /// Model output indicated uncertainty (ABSTAIN mode)
    Abstain,
    /// Unknown system route
    Unknown,
}

/// Logger for action records stored in JSONL format
#[derive(Clone)]
pub struct ActionLogger {
    log_path: PathBuf,
}

impl ActionLogger {
    /// Create a new ActionLogger with the default log path
    pub fn new() -> Self {
        let log_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cozmio");
        let log_path = log_dir.join("action_log.jsonl");

        // Ensure directory exists
        if let Err(e) = fs::create_dir_all(&log_dir) {
            eprintln!("Failed to create log directory: {}", e);
        }

        Self { log_path }
    }

    /// Create a new ActionLogger with a custom path (useful for testing)
    #[cfg(test)]
    pub fn with_path(log_path: PathBuf) -> Self {
        // Ensure parent directory exists
        if let Some(parent) = log_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create log directory: {}", e);
            }
        }
        Self { log_path }
    }

    /// Log a FactualActionRecord to the action log.
    ///
    /// This writes in the legacy JSONL format for backwards compatibility
    /// with existing log readers (cozmio_memory importer, etc).
    pub fn log_factual(&self, record: FactualActionRecord) -> Result<(), String> {
        // Convert FactualActionRecord to legacy ActionRecord for JSONL format
        let legacy = ActionRecord {
            timestamp: record.timestamp,
            trace_id: record.trace_id,
            session_id: record.session_id,
            window_title: record.window_title,
            content_text: record.raw_model_text,
            model_name: record.model_name,
            result_text: record.execution_result,
            error_text: record.error_text,
            user_feedback: record.user_feedback,
            captured_at: record.captured_at,
            call_started_at: record.call_started_at,
            call_duration_ms: record.call_duration_ms,
            // judgment is the original model judgment, preserved in original_judgment
            judgment: record.original_judgment,
            model_text: String::new(), // removed - was duplicate of reason
            status_label: String::new(), // removed - was duplicate of judgment
            confidence_score: 0.0,     // removed - was system-faked 1.0
            grounds: String::new(),    // removed - was duplicate of reason
            // system_action is the execution result string (notified, confirmed, executed, skipped)
            system_action: record.execution_result_str,
        };
        self.log(legacy)
    }

    /// Log an action record to the JSONL file
    pub fn log(&self, record: ActionRecord) -> Result<(), String> {
        let json =
            serde_json::to_string(&record).map_err(|e| format!("Serialization error: {}", e))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        writeln!(file, "{}", json).map_err(|e| format!("Failed to write to log file: {}", e))?;
        Ok(())
    }

    /// Get the most recent `limit` records in descending order by timestamp
    pub fn get_recent(&self, limit: usize) -> Result<Vec<ActionRecord>, String> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let file = OpenOptions::new()
            .read(true)
            .open(&self.log_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        let reader = BufReader::new(file);
        let mut records: Vec<ActionRecord> = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str(&line) {
                Ok(record) => records.push(record),
                Err(e) => {
                    log::warn!("Skipping malformed action history record: {}", e);
                }
            }
        }

        // Sort by timestamp descending (most recent first)
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Return only the requested limit
        records.truncate(limit);
        Ok(records)
    }

    /// Get recent records by reading only the tail of the JSONL file.
    ///
    /// This is intended for prompt context on the hot path. It avoids scanning
    /// a full day of logs every monitoring tick while still capturing the latest
    /// decisions, feedback, and relay results.
    pub fn get_recent_tail(
        &self,
        limit: usize,
        max_tail_bytes: u64,
    ) -> Result<Vec<ActionRecord>, String> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.log_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        let file_len = file
            .metadata()
            .map_err(|e| format!("Failed to read log metadata: {}", e))?
            .len();
        let read_len = file_len.min(max_tail_bytes.max(1));
        file.seek(SeekFrom::Start(file_len.saturating_sub(read_len)))
            .map_err(|e| format!("Failed to seek log tail: {}", e))?;

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .map_err(|e| format!("Failed to read log tail: {}", e))?;

        let mut records: Vec<ActionRecord> = Vec::new();
        for (idx, line) in buffer.lines().enumerate() {
            if file_len > read_len && idx == 0 {
                continue;
            }
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str(line) {
                Ok(record) => records.push(record),
                Err(e) => {
                    log::warn!("Skipping malformed action history tail record: {}", e);
                }
            }
        }

        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        records.truncate(limit);
        Ok(records)
    }

    /// Clear all logs by truncating the file to zero bytes
    pub fn clear(&self) -> Result<(), String> {
        let _file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.log_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        // File goes out of scope here, closed automatically
        Ok(())
    }

    /// Get the path to the log file
    #[cfg(test)]
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }
}

impl Default for ActionLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_client::InterventionMode;
    use serial_test::serial;
    use std::fs;

    fn create_test_logger() -> ActionLogger {
        let temp_dir = std::env::temp_dir().join("cozmio_log_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        ActionLogger::with_path(temp_dir.join("action_log.jsonl"))
    }

    #[test]
    #[serial] // These tests use a shared temp dir and must run serially
    fn test_log_and_get_recent() {
        let logger = create_test_logger();

        let record1 = ActionRecord {
            timestamp: 1000,
            trace_id: None,
            session_id: None,
            window_title: "Window 1".to_string(),
            judgment: "good".to_string(),
            model_text: "continue".to_string(),
            status_label: InterventionMode::Continue.to_string(),
            confidence_score: 0.8,
            grounds: "test".to_string(),
            system_action: "none".to_string(),
            content_text: None,
            result_text: None,
            error_text: None,
            user_feedback: None,
            model_name: None,
            captured_at: None,
            call_started_at: None,
            call_duration_ms: None,
        };

        let record2 = ActionRecord {
            timestamp: 2000,
            trace_id: None,
            session_id: None,
            window_title: "Window 2".to_string(),
            judgment: "bad".to_string(),
            model_text: "stop".to_string(),
            status_label: InterventionMode::Abstain.to_string(),
            confidence_score: 0.9,
            grounds: "test2".to_string(),
            system_action: "quit".to_string(),
            content_text: None,
            result_text: None,
            error_text: None,
            user_feedback: Some("feedback".to_string()),
            model_name: None,
            captured_at: None,
            call_started_at: None,
            call_duration_ms: None,
        };

        logger.log(record1.clone()).unwrap();
        logger.log(record2.clone()).unwrap();

        let recent = logger.get_recent(10).unwrap();
        assert_eq!(recent.len(), 2);
        // Most recent first
        assert_eq!(recent[0].timestamp, 2000);
        assert_eq!(recent[1].timestamp, 1000);

        // Check record2 fields
        assert_eq!(recent[0].window_title, "Window 2");
        assert_eq!(recent[0].user_feedback, Some("feedback".to_string()));
    }

    #[test]
    #[serial]
    fn test_get_recent_limit() {
        let logger = create_test_logger();

        for i in 0..5 {
            let record = ActionRecord {
                timestamp: (i + 1) as i64 * 1000,
                trace_id: None,
                session_id: None,
                window_title: format!("Window {}", i),
                judgment: "test".to_string(),
                model_text: "test".to_string(),
                status_label: InterventionMode::Continue.to_string(),
                confidence_score: 0.5,
                grounds: "test".to_string(),
                system_action: "none".to_string(),
                content_text: None,
                result_text: None,
                error_text: None,
                user_feedback: None,
                model_name: None,
                captured_at: None,
                call_started_at: None,
                call_duration_ms: None,
            };
            logger.log(record).unwrap();
        }

        let recent = logger.get_recent(3).unwrap();
        assert_eq!(recent.len(), 3);
        // Should be most recent first (5000, 4000, 3000)
        assert_eq!(recent[0].timestamp, 5000);
        assert_eq!(recent[1].timestamp, 4000);
        assert_eq!(recent[2].timestamp, 3000);
    }

    #[test]
    #[serial]
    fn test_clear() {
        let logger = create_test_logger();

        let record = ActionRecord {
            timestamp: 1000,
            trace_id: None,
            session_id: None,
            window_title: "Window".to_string(),
            judgment: "test".to_string(),
            model_text: "test".to_string(),
            status_label: InterventionMode::Continue.to_string(),
            confidence_score: 0.7,
            grounds: "test".to_string(),
            system_action: "none".to_string(),
            content_text: None,
            result_text: None,
            error_text: None,
            user_feedback: None,
            model_name: None,
            captured_at: None,
            call_started_at: None,
            call_duration_ms: None,
        };

        logger.log(record).unwrap();
        logger.clear().unwrap();

        let recent = logger.get_recent(10).unwrap();
        assert!(recent.is_empty());
    }

    #[test]
    #[serial]
    fn test_get_recent_empty() {
        let logger = create_test_logger();

        // Ensure file exists but is empty
        let _ = fs::File::create(logger.log_path());

        let recent = logger.get_recent(10).unwrap();
        assert!(recent.is_empty());
    }

    #[test]
    #[serial]
    fn test_get_recent_missing_file_returns_empty() {
        let temp_dir = std::env::temp_dir().join("cozmio_log_missing_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let logger = ActionLogger::with_path(temp_dir.join("missing_action_log.jsonl"));

        let recent = logger.get_recent(10).unwrap();

        assert!(recent.is_empty());
    }

    #[test]
    #[serial]
    fn test_clear_missing_file_creates_empty_log() {
        let temp_dir = std::env::temp_dir().join("cozmio_log_clear_missing_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let logger = ActionLogger::with_path(temp_dir.join("missing_action_log.jsonl"));

        logger.clear().unwrap();
        let recent = logger.get_recent(10).unwrap();

        assert!(recent.is_empty());
        assert!(logger.log_path().exists());
    }

    #[test]
    #[serial]
    fn test_get_recent_skips_malformed_lines() {
        let logger = create_test_logger();
        let record = ActionRecord {
            timestamp: 1000,
            trace_id: None,
            session_id: None,
            window_title: "Window".to_string(),
            judgment: "CONTINUE".to_string(),
            model_text: "continue".to_string(),
            status_label: InterventionMode::Continue.to_string(),
            confidence_score: 0.0,
            grounds: "test".to_string(),
            system_action: "confirmed".to_string(),
            content_text: None,
            result_text: None,
            error_text: None,
            user_feedback: None,
            model_name: None,
            captured_at: None,
            call_started_at: None,
            call_duration_ms: None,
        };

        fs::write(
            logger.log_path(),
            format!("not-json\n{}\n", serde_json::to_string(&record).unwrap()),
        )
        .unwrap();

        let recent = logger.get_recent(10).unwrap();

        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].window_title, "Window");
    }

    #[test]
    #[serial]
    fn test_get_recent_tail_reads_latest_records() {
        let logger = create_test_logger();

        for i in 0..8 {
            logger
                .log(ActionRecord {
                    timestamp: 1000 + i,
                    trace_id: None,
                    session_id: None,
                    window_title: format!("Window {}", i),
                    judgment: "CONTINUE".to_string(),
                    model_text: "continue".to_string(),
                    status_label: InterventionMode::Continue.to_string(),
                    confidence_score: 0.0,
                    grounds: "test".to_string(),
                    system_action: "confirmed".to_string(),
                    content_text: Some("content".to_string()),
                    result_text: None,
                    error_text: None,
                    user_feedback: None,
                    model_name: None,
                    captured_at: None,
                    call_started_at: None,
                    call_duration_ms: None,
                })
                .unwrap();
        }

        let recent = logger.get_recent_tail(3, 4096).unwrap();

        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].window_title, "Window 7");
        assert_eq!(recent[1].window_title, "Window 6");
        assert_eq!(recent[2].window_title, "Window 5");
    }
}
