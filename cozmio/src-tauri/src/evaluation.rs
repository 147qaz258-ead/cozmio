//! Evaluation schema and SQLite stores for model evaluation samples and results.
//!
//! This module provides:
//! - `EvaluationSample`: A captured sample with window facts, model output, and execution result
//! - `EvaluationResult`: Human evaluation of a sample with judgment and notes
//! - `EvaluationSampleStore`: Persistence for samples
//! - `EvaluationResultStore`: Persistence for evaluation results

use cozmio_memory::{Database, MemoryError};
use rusqlite::params;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// EvaluationSample
// ---------------------------------------------------------------------------

/// EvaluationSample captures a single inference sample for later evaluation.
///
/// Fields:
/// - `id`: Unique identifier (UUID)
/// - `source_trace_id`: Reference to the trace this sample was captured from
/// - `screenshot_path`: Path to the captured screenshot file
/// - `window_facts_json`: JSON string of window metadata (title, process, rect, etc.)
/// - `context_pack_summary`: Serialized summary of the context pack used
/// - `model_raw_output`: Raw output from the vision model
/// - `user_action_description`: Human description of what the user was doing
/// - `execution_result_summary`: What actually happened when the action was executed
/// - `captured_at`: ISO8601 timestamp when the sample was captured
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSample {
    pub id: String,
    pub source_trace_id: String,
    pub screenshot_path: String,
    pub window_facts_json: String,
    pub context_pack_summary: String,
    pub model_raw_output: String,
    pub user_action_description: String,
    pub execution_result_summary: String,
    pub captured_at: String,
}

// ---------------------------------------------------------------------------
// Judgment enum
// ---------------------------------------------------------------------------

/// Judgment represents the evaluator's assessment of an evaluation sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Judgment {
    Pass,
    Partial,
    Fail,
}

impl std::fmt::Display for Judgment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Judgment::Pass => write!(f, "pass"),
            Judgment::Partial => write!(f, "partial"),
            Judgment::Fail => write!(f, "fail"),
        }
    }
}

impl Judgment {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pass" => Some(Judgment::Pass),
            "partial" => Some(Judgment::Partial),
            "fail" => Some(Judgment::Fail),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// EvaluationResult
// ---------------------------------------------------------------------------

/// EvaluationResult records a human evaluator's assessment of an EvaluationSample.
///
/// Fields:
/// - `id`: Unique identifier (UUID)
/// - `sample_id`: Foreign key to the EvaluationSample being evaluated
/// - `judgment`: The evaluator's judgment (pass/partial/fail)
/// - `groundedness_notes`: Notes on whether the model's reasoning was grounded in reality
/// - `recommendation`: Recommended follow-up action or improvement
/// - `evaluated_by`: Identifier for who performed the evaluation
/// - `evaluated_at`: ISO8601 timestamp of the evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub id: String,
    pub sample_id: String,
    pub judgment: Judgment,
    pub groundedness_notes: String,
    pub recommendation: String,
    pub evaluated_by: String,
    pub evaluated_at: String,
}

// ---------------------------------------------------------------------------
// EvaluationSampleStore
// ---------------------------------------------------------------------------

/// Store for persisting and querying EvaluationSample records.
pub struct EvaluationSampleStore<'a>(&'a Database);

impl<'a> EvaluationSampleStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self(db)
    }

    /// Insert a new evaluation sample into the store.
    pub fn save_sample(&self, sample: &EvaluationSample) -> Result<(), MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO evaluation_samples (
                id, source_trace_id, screenshot_path, window_facts_json,
                context_pack_summary, model_raw_output, user_action_description,
                execution_result_summary, captured_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                sample.id,
                sample.source_trace_id,
                sample.screenshot_path,
                sample.window_facts_json,
                sample.context_pack_summary,
                sample.model_raw_output,
                sample.user_action_description,
                sample.execution_result_summary,
                sample.captured_at,
            ],
        )?;
        Ok(())
    }

    /// Retrieve a single evaluation sample by its ID.
    pub fn get_sample(&self, id: &str) -> Result<Option<EvaluationSample>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_trace_id, screenshot_path, window_facts_json,
                    context_pack_summary, model_raw_output, user_action_description,
                    execution_result_summary, captured_at
             FROM evaluation_samples WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_sample(row)?))
        } else {
            Ok(None)
        }
    }

    /// List evaluation samples ordered by captured_at DESC, limited to `limit`.
    pub fn get_samples(&self, limit: usize) -> Result<Vec<EvaluationSample>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_trace_id, screenshot_path, window_facts_json,
                    context_pack_summary, model_raw_output, user_action_description,
                    execution_result_summary, captured_at
             FROM evaluation_samples ORDER BY captured_at DESC LIMIT ?1",
        )?;
        let mut samples = Vec::new();
        let mut rows = stmt.query(params![limit as i64])?;
        while let Some(row) = rows.next()? {
            samples.push(self.row_to_sample(row)?);
        }
        Ok(samples)
    }

    /// Check whether a sample with the given source_trace_id already exists.
    pub fn sample_exists(&self, trace_id: &str) -> Result<bool, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT 1 FROM evaluation_samples WHERE source_trace_id = ?1 LIMIT 1")?;
        let exists = stmt.query(params![trace_id])?.next()?.is_some();
        Ok(exists)
    }

    fn row_to_sample(&self, row: &rusqlite::Row) -> Result<EvaluationSample, MemoryError> {
        Ok(EvaluationSample {
            id: row.get(0)?,
            source_trace_id: row.get(1)?,
            screenshot_path: row.get(2)?,
            window_facts_json: row.get(3)?,
            context_pack_summary: row.get(4)?,
            model_raw_output: row.get(5)?,
            user_action_description: row.get(6)?,
            execution_result_summary: row.get(7)?,
            captured_at: row.get(8)?,
        })
    }
}

// ---------------------------------------------------------------------------
// EvaluationResultStore
// ---------------------------------------------------------------------------

/// Store for persisting and querying EvaluationResult records.
pub struct EvaluationResultStore<'a>(&'a Database);

impl<'a> EvaluationResultStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self(db)
    }

    /// Insert a new evaluation result into the store.
    pub fn save_result(&self, result: &EvaluationResult) -> Result<(), MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO evaluation_results (
                id, sample_id, judgment, groundedness_notes,
                recommendation, evaluated_by, evaluated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                result.id,
                result.sample_id,
                result.judgment.to_string(),
                result.groundedness_notes,
                result.recommendation,
                result.evaluated_by,
                result.evaluated_at,
            ],
        )?;
        Ok(())
    }

    /// Retrieve a single evaluation result by its sample_id.
    pub fn get_result(&self, sample_id: &str) -> Result<Option<EvaluationResult>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, sample_id, judgment, groundedness_notes,
                    recommendation, evaluated_by, evaluated_at
             FROM evaluation_results WHERE sample_id = ?1",
        )?;
        let mut rows = stmt.query(params![sample_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_result(row)?))
        } else {
            Ok(None)
        }
    }

    /// List evaluation results ordered by evaluated_at DESC, limited to `limit`.
    pub fn get_results(&self, limit: usize) -> Result<Vec<EvaluationResult>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, sample_id, judgment, groundedness_notes,
                    recommendation, evaluated_by, evaluated_at
             FROM evaluation_results ORDER BY evaluated_at DESC LIMIT ?1",
        )?;
        let mut results = Vec::new();
        let mut rows = stmt.query(params![limit as i64])?;
        while let Some(row) = rows.next()? {
            results.push(self.row_to_result(row)?);
        }
        Ok(results)
    }

    fn row_to_result(&self, row: &rusqlite::Row) -> Result<EvaluationResult, MemoryError> {
        let judgment_str: String = row.get(2)?;
        let judgment = Judgment::from_str(&judgment_str).ok_or_else(|| {
            MemoryError::NotFound("Invalid judgment value in database".to_string())
        })?;
        Ok(EvaluationResult {
            id: row.get(0)?,
            sample_id: row.get(1)?,
            judgment,
            groundedness_notes: row.get(3)?,
            recommendation: row.get(4)?,
            evaluated_by: row.get(5)?,
            evaluated_at: row.get(6)?,
        })
    }
}

// ---------------------------------------------------------------------------
// EvaluationFeedback
// ---------------------------------------------------------------------------

/// EvaluationFeedback records evaluator notes and recommendations for human review.
/// This is evidence for improvement suggestions - does NOT modify runtime behavior automatically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationFeedback {
    pub id: String,
    pub sample_id: String,
    pub source_trace_id: String,
    pub producer: String,
    pub groundedness_notes: String,
    pub recommendation: String,
    pub created_at: String,
    pub status: String,
}

// ---------------------------------------------------------------------------
// EvaluationFeedbackStore
// ---------------------------------------------------------------------------

/// Store for persisting and querying EvaluationFeedback records.
pub struct EvaluationFeedbackStore<'a>(&'a Database);

impl<'a> EvaluationFeedbackStore<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self(db)
    }

    /// Insert a new evaluation feedback record.
    pub fn save_feedback(&self, feedback: &EvaluationFeedback) -> Result<(), MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO evaluation_feedback (
                id, sample_id, source_trace_id, producer, groundedness_notes,
                recommendation, created_at, status
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                feedback.id,
                feedback.sample_id,
                feedback.source_trace_id,
                feedback.producer,
                feedback.groundedness_notes,
                feedback.recommendation,
                feedback.created_at,
                feedback.status,
            ],
        )?;
        Ok(())
    }

    /// Get feedback by ID.
    pub fn get_feedback(&self, id: &str) -> Result<Option<EvaluationFeedback>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, sample_id, source_trace_id, producer, groundedness_notes,
                    recommendation, created_at, status
             FROM evaluation_feedback WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_feedback(row)?))
        } else {
            Ok(None)
        }
    }

    /// List feedback records ordered by created_at DESC, optionally filtered by status.
    pub fn get_feedback_list(
        &self,
        limit: usize,
        status: Option<&str>,
    ) -> Result<Vec<EvaluationFeedback>, MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        let query = if status.is_some() {
            "SELECT id, sample_id, source_trace_id, producer, groundedness_notes,
                    recommendation, created_at, status
             FROM evaluation_feedback WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2"
        } else {
            "SELECT id, sample_id, source_trace_id, producer, groundedness_notes,
                    recommendation, created_at, status
             FROM evaluation_feedback ORDER BY created_at DESC LIMIT ?1"
        };

        let mut feedbacks = Vec::new();
        if let Some(s) = status {
            let mut stmt = conn.prepare(query)?;
            let mut rows = stmt.query(params![s, limit as i64])?;
            while let Some(row) = rows.next()? {
                feedbacks.push(self.row_to_feedback(row)?);
            }
        } else {
            let mut stmt = conn.prepare(query)?;
            let mut rows = stmt.query(params![limit as i64])?;
            while let Some(row) = rows.next()? {
                feedbacks.push(self.row_to_feedback(row)?);
            }
        }
        Ok(feedbacks)
    }

    /// Update feedback status to "reviewed".
    pub fn mark_reviewed(&self, id: &str) -> Result<(), MemoryError> {
        let conn = self.0.conn.lock().unwrap();
        conn.execute(
            "UPDATE evaluation_feedback SET status = 'reviewed' WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    fn row_to_feedback(&self, row: &rusqlite::Row) -> Result<EvaluationFeedback, MemoryError> {
        Ok(EvaluationFeedback {
            id: row.get(0)?,
            sample_id: row.get(1)?,
            source_trace_id: row.get(2)?,
            producer: row.get(3)?,
            groundedness_notes: row.get(4)?,
            recommendation: row.get(5)?,
            created_at: row.get(6)?,
            status: row.get(7)?,
        })
    }
}

// ---------------------------------------------------------------------------
// Schema helpers (called by migrations)
// ---------------------------------------------------------------------------

/// SQL to create the evaluation_tables.
/// Exposed so cozmio_memory::run_migrations can call it.
pub const EVALUATION_SAMPLES_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS evaluation_samples (\
        id TEXT PRIMARY KEY,\
        source_trace_id TEXT NOT NULL,\
        screenshot_path TEXT NOT NULL,\
        window_facts_json TEXT NOT NULL,\
        context_pack_summary TEXT NOT NULL,\
        model_raw_output TEXT NOT NULL,\
        user_action_description TEXT NOT NULL,\
        execution_result_summary TEXT NOT NULL,\
        captured_at TEXT NOT NULL\
    )";

pub const EVALUATION_RESULTS_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS evaluation_results (\
        id TEXT PRIMARY KEY,\
        sample_id TEXT NOT NULL,\
        judgment TEXT NOT NULL,\
        groundedness_notes TEXT NOT NULL,\
        recommendation TEXT NOT NULL,\
        evaluated_by TEXT NOT NULL,\
        evaluated_at TEXT NOT NULL,\
        FOREIGN KEY (sample_id) REFERENCES evaluation_samples(id)\
    )";

pub const EVALUATION_FEEDBACK_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS evaluation_feedback (\
        id TEXT PRIMARY KEY,\
        sample_id TEXT NOT NULL,\
        source_trace_id TEXT NOT NULL,\
        producer TEXT NOT NULL,\
        groundedness_notes TEXT NOT NULL,\
        recommendation TEXT NOT NULL,\
        created_at TEXT NOT NULL,\
        status TEXT NOT NULL\
    )";

pub const EVALUATION_SAMPLES_TRACE_IDX_SQL: &str =
    "CREATE INDEX IF NOT EXISTS idx_evaluation_samples_source_trace_id ON evaluation_samples(source_trace_id)";

pub const EVALUATION_RESULTS_SAMPLE_IDX_SQL: &str =
    "CREATE INDEX IF NOT EXISTS idx_evaluation_results_sample_id ON evaluation_results(sample_id)";

pub const EVALUATION_FEEDBACK_SAMPLE_IDX_SQL: &str =
    "CREATE INDEX IF NOT EXISTS idx_evaluation_feedback_sample_id ON evaluation_feedback(sample_id)";

/// Run evaluation table creation (idempotent).
pub fn run_evaluation_migrations(db: &Database) -> Result<(), MemoryError> {
    let conn = db.conn.lock().unwrap();
    conn.execute(EVALUATION_SAMPLES_TABLE_SQL, [])?;
    conn.execute(EVALUATION_RESULTS_TABLE_SQL, [])?;
    conn.execute(EVALUATION_FEEDBACK_TABLE_SQL, [])?;
    conn.execute(EVALUATION_SAMPLES_TRACE_IDX_SQL, [])?;
    conn.execute(EVALUATION_RESULTS_SAMPLE_IDX_SQL, [])?;
    conn.execute(EVALUATION_FEEDBACK_SAMPLE_IDX_SQL, [])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use cozmio_memory::Database;
    use uuid::Uuid;

    fn test_db() -> Database {
        let dir = std::env::temp_dir().join(format!("cozmio_eval_test_db_{}", Uuid::new_v4()));
        let _ = std::fs::remove_dir_all(&dir);
        Database::new(&dir).unwrap()
    }

    fn init_tables(db: &Database) {
        run_evaluation_migrations(db).unwrap();
    }

    #[test]
    fn test_save_and_get_sample() {
        let db = test_db();
        init_tables(&db);
        let store = EvaluationSampleStore::new(&db);

        let sample = EvaluationSample {
            id: Uuid::new_v4().to_string(),
            source_trace_id: "trace_abc123".to_string(),
            screenshot_path: "C:\\screenshots\\img_001.png".to_string(),
            window_facts_json: r#"{"title":"Notepad","process":"notepad.exe"}"#.to_string(),
            context_pack_summary: "User was editing a text file".to_string(),
            model_raw_output: "The user is typing in Notepad.".to_string(),
            user_action_description: "User opened notepad and started typing".to_string(),
            execution_result_summary: "Text was successfully captured".to_string(),
            captured_at: "2026-05-01T12:00:00Z".to_string(),
        };

        store.save_sample(&sample).unwrap();

        let retrieved = store.get_sample(&sample.id).unwrap().unwrap();
        assert_eq!(retrieved.id, sample.id);
        assert_eq!(retrieved.source_trace_id, "trace_abc123");
        assert_eq!(retrieved.screenshot_path, "C:\\screenshots\\img_001.png");
        assert_eq!(
            retrieved.window_facts_json,
            r#"{"title":"Notepad","process":"notepad.exe"}"#
        );
        assert_eq!(
            retrieved.context_pack_summary,
            "User was editing a text file"
        );
        assert_eq!(retrieved.model_raw_output, "The user is typing in Notepad.");
        assert_eq!(
            retrieved.user_action_description,
            "User opened notepad and started typing"
        );
        assert_eq!(
            retrieved.execution_result_summary,
            "Text was successfully captured"
        );
        assert_eq!(retrieved.captured_at, "2026-05-01T12:00:00Z");

        // Test get non-existent
        assert!(store.get_sample("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_get_samples_limit() {
        let db = test_db();
        init_tables(&db);
        let store = EvaluationSampleStore::new(&db);

        for i in 0..5 {
            let sample = EvaluationSample {
                id: Uuid::new_v4().to_string(),
                source_trace_id: format!("trace_{}", i),
                screenshot_path: format!("path_{}", i),
                window_facts_json: "{}".to_string(),
                context_pack_summary: format!("ctx_{}", i),
                model_raw_output: "output".to_string(),
                user_action_description: "action".to_string(),
                execution_result_summary: "result".to_string(),
                captured_at: format!("2026-05-01T12:00:{:02}Z", i),
            };
            store.save_sample(&sample).unwrap();
        }

        let all = store.get_samples(10).unwrap();
        assert_eq!(all.len(), 5);

        let limited = store.get_samples(3).unwrap();
        assert_eq!(limited.len(), 3);
    }

    #[test]
    fn test_sample_exists() {
        let db = test_db();
        init_tables(&db);
        let store = EvaluationSampleStore::new(&db);

        let sample = EvaluationSample {
            id: Uuid::new_v4().to_string(),
            source_trace_id: "trace_unique_456".to_string(),
            screenshot_path: "path".to_string(),
            window_facts_json: "{}".to_string(),
            context_pack_summary: "ctx".to_string(),
            model_raw_output: "out".to_string(),
            user_action_description: "action".to_string(),
            execution_result_summary: "result".to_string(),
            captured_at: "2026-05-01T12:00:00Z".to_string(),
        };

        assert!(!store.sample_exists("trace_unique_456").unwrap());
        store.save_sample(&sample).unwrap();
        assert!(store.sample_exists("trace_unique_456").unwrap());
        assert!(!store.sample_exists("nonexistent_trace").unwrap());
    }

    #[test]
    fn test_save_and_get_result() {
        let db = test_db();
        init_tables(&db);
        let sample_store = EvaluationSampleStore::new(&db);
        let result_store = EvaluationResultStore::new(&db);

        let sample = EvaluationSample {
            id: Uuid::new_v4().to_string(),
            source_trace_id: "trace_result_001".to_string(),
            screenshot_path: "path".to_string(),
            window_facts_json: "{}".to_string(),
            context_pack_summary: "ctx".to_string(),
            model_raw_output: "out".to_string(),
            user_action_description: "action".to_string(),
            execution_result_summary: "result".to_string(),
            captured_at: "2026-05-01T12:00:00Z".to_string(),
        };
        sample_store.save_sample(&sample).unwrap();

        let result = EvaluationResult {
            id: Uuid::new_v4().to_string(),
            sample_id: sample.id.clone(),
            judgment: Judgment::Pass,
            groundedness_notes: "Model correctly identified the window".to_string(),
            recommendation: "Keep this pattern for similar cases".to_string(),
            evaluated_by: "evaluator_001".to_string(),
            evaluated_at: "2026-05-01T14:00:00Z".to_string(),
        };

        result_store.save_result(&result).unwrap();

        let retrieved = result_store.get_result(&sample.id).unwrap().unwrap();
        assert_eq!(retrieved.id, result.id);
        assert_eq!(retrieved.sample_id, sample.id);
        assert_eq!(retrieved.judgment, Judgment::Pass);
        assert_eq!(
            retrieved.groundedness_notes,
            "Model correctly identified the window"
        );
        assert_eq!(
            retrieved.recommendation,
            "Keep this pattern for similar cases"
        );
        assert_eq!(retrieved.evaluated_by, "evaluator_001");
        assert_eq!(retrieved.evaluated_at, "2026-05-01T14:00:00Z");

        // Test get non-existent
        assert!(result_store
            .get_result("nonexistent_sample")
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_get_results_limit() {
        let db = test_db();
        init_tables(&db);
        let sample_store = EvaluationSampleStore::new(&db);
        let result_store = EvaluationResultStore::new(&db);

        for i in 0..3 {
            let sample = EvaluationSample {
                id: Uuid::new_v4().to_string(),
                source_trace_id: format!("trace_res_{}", i),
                screenshot_path: "path".to_string(),
                window_facts_json: "{}".to_string(),
                context_pack_summary: "ctx".to_string(),
                model_raw_output: "out".to_string(),
                user_action_description: "action".to_string(),
                execution_result_summary: "result".to_string(),
                captured_at: format!("2026-05-01T12:00:{:02}Z", i),
            };
            sample_store.save_sample(&sample).unwrap();

            let result = EvaluationResult {
                id: Uuid::new_v4().to_string(),
                sample_id: sample.id,
                judgment: if i == 0 {
                    Judgment::Fail
                } else {
                    Judgment::Pass
                },
                groundedness_notes: format!("notes_{}", i),
                recommendation: format!("rec_{}", i),
                evaluated_by: "eval".to_string(),
                evaluated_at: format!("2026-05-01T14:00:{:02}Z", i),
            };
            result_store.save_result(&result).unwrap();
        }

        let all = result_store.get_results(10).unwrap();
        assert_eq!(all.len(), 3);

        let limited = result_store.get_results(2).unwrap();
        assert_eq!(limited.len(), 2);
    }

    #[test]
    fn test_judgment_from_str() {
        assert_eq!(Judgment::from_str("pass"), Some(Judgment::Pass));
        assert_eq!(Judgment::from_str("partial"), Some(Judgment::Partial));
        assert_eq!(Judgment::from_str("fail"), Some(Judgment::Fail));
        assert_eq!(Judgment::from_str("invalid"), None);
    }

    #[test]
    fn test_judgment_display() {
        assert_eq!(Judgment::Pass.to_string(), "pass");
        assert_eq!(Judgment::Partial.to_string(), "partial");
        assert_eq!(Judgment::Fail.to_string(), "fail");
    }

    #[test]
    fn test_evaluation_feedback_save_and_get() {
        let db = test_db();
        init_tables(&db);
        let store = EvaluationFeedbackStore::new(&db);

        let feedback = EvaluationFeedback {
            id: Uuid::new_v4().to_string(),
            sample_id: "sample_123".to_string(),
            source_trace_id: "trace_456".to_string(),
            producer: "cozmio_eval".to_string(),
            groundedness_notes: "Model reasoning was grounded".to_string(),
            recommendation: "Keep this pattern".to_string(),
            created_at: "2026-05-01T15:00:00Z".to_string(),
            status: "pending".to_string(),
        };

        store.save_feedback(&feedback).unwrap();

        let retrieved = store.get_feedback(&feedback.id).unwrap().unwrap();
        assert_eq!(retrieved.id, feedback.id);
        assert_eq!(retrieved.sample_id, "sample_123");
        assert_eq!(retrieved.source_trace_id, "trace_456");
        assert_eq!(retrieved.producer, "cozmio_eval");
        assert_eq!(retrieved.groundedness_notes, "Model reasoning was grounded");
        assert_eq!(retrieved.recommendation, "Keep this pattern");
        assert_eq!(retrieved.status, "pending");
    }

    #[test]
    fn test_evaluation_feedback_list_with_status_filter() {
        let db = test_db();
        init_tables(&db);
        let store = EvaluationFeedbackStore::new(&db);

        // Create feedback with different statuses
        let feedback1 = EvaluationFeedback {
            id: Uuid::new_v4().to_string(),
            sample_id: "sample_1".to_string(),
            source_trace_id: "trace_1".to_string(),
            producer: "producer".to_string(),
            groundedness_notes: "notes".to_string(),
            recommendation: "rec".to_string(),
            created_at: "2026-05-01T10:00:00Z".to_string(),
            status: "pending".to_string(),
        };
        let feedback2 = EvaluationFeedback {
            id: Uuid::new_v4().to_string(),
            sample_id: "sample_2".to_string(),
            source_trace_id: "trace_2".to_string(),
            producer: "producer".to_string(),
            groundedness_notes: "notes".to_string(),
            recommendation: "rec".to_string(),
            created_at: "2026-05-01T11:00:00Z".to_string(),
            status: "reviewed".to_string(),
        };

        store.save_feedback(&feedback1).unwrap();
        store.save_feedback(&feedback2).unwrap();

        // Get all feedback
        let all = store.get_feedback_list(10, None).unwrap();
        assert_eq!(all.len(), 2);

        // Filter by pending
        let pending = store.get_feedback_list(10, Some("pending")).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].status, "pending");

        // Filter by reviewed
        let reviewed = store.get_feedback_list(10, Some("reviewed")).unwrap();
        assert_eq!(reviewed.len(), 1);
        assert_eq!(reviewed[0].status, "reviewed");
    }

    #[test]
    fn test_mark_feedback_reviewed() {
        let db = test_db();
        init_tables(&db);
        let store = EvaluationFeedbackStore::new(&db);

        let feedback = EvaluationFeedback {
            id: Uuid::new_v4().to_string(),
            sample_id: "sample_x".to_string(),
            source_trace_id: "trace_x".to_string(),
            producer: "producer".to_string(),
            groundedness_notes: "notes".to_string(),
            recommendation: "rec".to_string(),
            created_at: "2026-05-01T12:00:00Z".to_string(),
            status: "pending".to_string(),
        };

        store.save_feedback(&feedback).unwrap();
        assert_eq!(
            store.get_feedback(&feedback.id).unwrap().unwrap().status,
            "pending"
        );

        store.mark_reviewed(&feedback.id).unwrap();
        assert_eq!(
            store.get_feedback(&feedback.id).unwrap().unwrap().status,
            "reviewed"
        );
    }
}
