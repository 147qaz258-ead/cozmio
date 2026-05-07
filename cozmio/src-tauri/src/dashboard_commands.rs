//! Dashboard IPC Commands
//!
//! Aggregates effect signal statistics for the practice dashboard.

use crate::commands::AppState;
use cozmio_memory::Database;
use std::path::PathBuf;
use tauri::Manager;

/// Return JSON structure for get_effect_signals
#[derive(Debug, serde::Serialize)]
pub struct EffectSignals {
    pub user_feedback: UserFeedbackCounts,
    pub execution: ExecutionCounts,
    pub memory_pool: MemoryPoolCounts,
    pub competition: CompetitionStats,
}

#[derive(Debug, serde::Serialize)]
pub struct UserFeedbackCounts {
    pub confirmed: usize,
    pub cancelled: usize,
    pub dismissed: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct ExecutionCounts {
    pub success: usize,
    pub failure: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct MemoryPoolCounts {
    pub active: usize,
    pub rejected: usize,
    pub embedded: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct CompetitionStats {
    pub preview_count: usize,
    pub avg_entries_per_preview: f64,
    pub embedding_coverage: f64,
}

fn memory_db_path() -> PathBuf {
    Database::memory_dir().join("cozmio.db")
}

fn open_memory_db() -> Result<Database, String> {
    Database::new(&memory_db_path()).map_err(|e| e.to_string())
}

fn preview_file_path() -> PathBuf {
    Database::memory_dir().join("competition_preview.jsonl")
}

fn count_ledger_events_by_type(conn: &rusqlite::Connection, event_type: &str) -> usize {
    conn.query_row(
        "SELECT COUNT(*) FROM ledger_events WHERE event_type = ?1",
        [event_type],
        |row| row.get::<_, i64>(0),
    )
    .map(|c| c as usize)
    .unwrap_or(0)
}

/// Get aggregated effect signal statistics for the dashboard.
#[tauri::command]
pub async fn get_effect_signals(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let state = app.state::<AppState>();

    // 1. User Feedback counts from ledger
    let (confirmed, cancelled, dismissed) = {
        let conn = state.ledger_manager.projection().connection();
        let confirmed = count_ledger_events_by_type(conn, "user_confirmed");
        let cancelled = count_ledger_events_by_type(conn, "user_cancelled");
        let dismissed = count_ledger_events_by_type(conn, "user_dismissed");
        (confirmed, cancelled, dismissed)
    };

    // 2. Execution counts from ledger
    let (success, failure) = {
        let conn = state.ledger_manager.projection().connection();
        let success = count_ledger_events_by_type(conn, "execution_result_received");
        let failure = count_ledger_events_by_type(conn, "execution_error_received");
        (success, failure)
    };

    // 3. Memory Pool counts from memory DB
    let (active, rejected, embedded) = match open_memory_db() {
        Ok(db) => {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let active: usize = conn
                .query_row(
                    "SELECT COUNT(*) FROM memory_candidates WHERE status = 'active'",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .map(|c| c as usize)
                .unwrap_or(0);
            let rejected: usize = conn
                .query_row(
                    "SELECT COUNT(*) FROM memory_candidates WHERE status = 'rejected'",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .map(|c| c as usize)
                .unwrap_or(0);
            let embedded: usize = conn
                .query_row(
                    "SELECT COUNT(*) FROM memory_candidates WHERE embedding_ref IS NOT NULL",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .map(|c| c as usize)
                .unwrap_or(0);
            (active, rejected, embedded)
        }
        Err(_) => (0, 0, 0),
    };

    // 4. Competition stats from competition_preview.jsonl
    let (preview_count, avg_entries_per_preview, embedding_coverage) = {
        let path = preview_file_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            let mut total_entries = 0usize;
            let mut preview_count = 0usize;

            for line in content.lines() {
                if let Ok(stored) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(entries) = stored.get("entries").and_then(|e| e.as_array()) {
                        total_entries += entries.len();
                        preview_count += 1;
                    }
                }
            }

            let avg = if preview_count > 0 {
                total_entries as f64 / preview_count as f64
            } else {
                0.0
            };

            // embedding_coverage = embedded / total memory candidates (from memory DB)
            let total_candidates: usize = match open_memory_db() {
                Ok(db) => {
                    let conn = db.conn.lock().map_err(|e| e.to_string())?;
                    conn.query_row("SELECT COUNT(*) FROM memory_candidates", [], |row| {
                        row.get::<_, i64>(0)
                    })
                    .map(|c| c as usize)
                    .unwrap_or(0)
                }
                Err(_) => 0,
            };

            let coverage = if total_candidates > 0 {
                embedded as f64 / total_candidates as f64
            } else {
                0.0
            };

            (preview_count, avg, coverage)
        } else {
            (0, 0.0, 0.0)
        }
    };

    let signals = EffectSignals {
        user_feedback: UserFeedbackCounts {
            confirmed,
            cancelled,
            dismissed,
        },
        execution: ExecutionCounts { success, failure },
        memory_pool: MemoryPoolCounts {
            active,
            rejected,
            embedded,
        },
        competition: CompetitionStats {
            preview_count,
            avg_entries_per_preview,
            embedding_coverage,
        },
    };

    Ok(serde_json::to_value(signals).map_err(|e| e.to_string())?)
}
