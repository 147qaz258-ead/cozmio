//! Tauri IPC commands for cozmio_memory integration.
//!
//! These commands expose the Suggestion Memory Core functionality to the frontend
//! via Tauri's IPC mechanism.

use chrono::Utc;
use cozmio_memory::search::{SearchEngine, SearchResult, SearchResults};
use cozmio_memory::{
    ActivityNote, ContextSlicesStore, Decision, DecisionMemoryStore, MemoryCompetition, MemoryCore,
    MemoryEventsStore, SearchQuery, SkillMemoryStore, TaskThreadUpdate, TaskThreadsStore,
};

// ---------------------------------------------------------------------------
// DTO types for IPC boundary (must implement serde::Serialize)
// ---------------------------------------------------------------------------

/// Memory statistics returned by get_memory_stats
#[derive(Debug, serde::Serialize)]
pub struct MemoryStats {
    pub event_count: usize,
    pub slice_count: usize,
    pub thread_count: usize,
    pub decision_count: usize,
    pub skill_count: usize,
    pub time_range: Option<(String, String)>,
    pub source_distribution: Vec<(String, usize)>,
}

/// Result of importing existing logs
#[derive(Debug, serde::Serialize)]
pub struct ImportResult {
    pub events_imported: usize,
    pub slices_created: usize,
    pub errors: Vec<String>,
}

/// Search result entry (IPC-friendly version)
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResultDto {
    pub event_id: i64,
    pub score: f32,
    pub source: String,
    pub content: String,
    pub window_title: Option<String>,
    pub timestamp: String,
    pub evidence_source: String,
    pub thread_id: Option<i64>,
}

/// Combined search results (IPC-friendly)
#[derive(Debug, serde::Serialize)]
pub struct SearchResultsDto {
    pub events: Vec<SearchResultDto>,
    pub total_fts: usize,
    pub total_vec: usize,
}

/// Evidence reference (IPC-friendly)
#[derive(Debug, Clone, serde::Serialize)]
pub struct EvidenceRefDto {
    pub source: String,
    pub memory_type: String,
    pub id: i64,
    pub content_snippet: String,
    pub timestamp: Option<String>,
}

/// Reminder context built from activity (IPC-friendly)
#[derive(Debug, serde::Serialize)]
pub struct ReminderContextDto {
    pub current_activity: String,
    pub recent_context: String,
    pub related_decisions: String,
    pub relevant_skills: String,
    pub task_state: Option<String>,
    pub evidence_refs: Vec<EvidenceRefDto>,
}

/// Task thread (IPC-friendly)
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskThreadDto {
    pub id: Option<i64>,
    pub name: String,
    pub current_state: Option<String>,
    pub open_questions: Vec<String>,
    pub decisions: Vec<String>,
    pub recent_slice_ids: Vec<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Decision memory entry (IPC-friendly)
#[derive(Debug, Clone, serde::Serialize)]
pub struct DecisionDto {
    pub id: Option<i64>,
    pub memory_type: String,
    pub content: String,
    pub evidence: Option<String>,
    pub related_thread_id: Option<i64>,
    pub evidence_source: String,
}

/// Skill memory entry (IPC-friendly)
#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillDto {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub procedure: String,
    pub success_context: Option<String>,
    pub usage_count: i64,
    pub last_used_at: Option<String>,
    pub evidence_source: String,
}

// ---------------------------------------------------------------------------
// Helper: Initialize Database path
// ---------------------------------------------------------------------------

fn memory_db_path() -> std::path::PathBuf {
    use cozmio_memory::Database;
    Database::memory_dir().join("cozmio.db")
}

fn open_memory_db() -> Result<cozmio_memory::Database, String> {
    cozmio_memory::Database::new(&memory_db_path()).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// IPC Commands
// ---------------------------------------------------------------------------

/// Get memory statistics: record counts, time range, source distribution
#[tauri::command]
pub async fn get_memory_stats() -> Result<MemoryStats, String> {
    let db = open_memory_db()?;

    let event_count = MemoryEventsStore::new(&db)
        .count()
        .map_err(|e| e.to_string())? as usize;

    let slice_count = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM context_slices", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        count as usize
    };

    let thread_count = TaskThreadsStore::new(&db)
        .get_all()
        .map_err(|e| e.to_string())?
        .len();

    let decision_count = DecisionMemoryStore::new(&db)
        .get_all()
        .map_err(|e| e.to_string())?
        .len();

    let skill_count = SkillMemoryStore::new(&db)
        .get_all()
        .map_err(|e| e.to_string())?
        .len();

    let time_range = MemoryEventsStore::new(&db)
        .get_time_range()
        .ok()
        .filter(|(s, e)| !s.is_empty() && !e.is_empty());

    let source_distribution = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT source, COUNT(*) FROM memory_events GROUP BY source")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })
            .map_err(|e| e.to_string())?;
        rows.filter_map(|r| r.ok()).collect()
    };

    Ok(MemoryStats {
        event_count,
        slice_count,
        thread_count,
        decision_count,
        skill_count,
        time_range,
        source_distribution,
    })
}

/// Import existing logs from action_log.jsonl (if present)
/// Returns import result with counts and any errors
#[tauri::command]
pub async fn import_existing_logs() -> Result<ImportResult, String> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let db = open_memory_db()?;
    let memory_dir = cozmio_memory::Database::memory_dir();
    let log_path = memory_dir.join("action_log.jsonl");

    let mut result = ImportResult {
        events_imported: 0,
        slices_created: 0,
        errors: Vec::new(),
    };

    if !log_path.exists() {
        return Ok(result);
    }

    let file = File::open(&log_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    let events_store = MemoryEventsStore::new(&db);
    let slices_store = ContextSlicesStore::new(&db);

    for (line_num, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                result
                    .errors
                    .push(format!("Line {}: read error: {}", line_num + 1, e));
                continue;
            }
        };

        #[derive(serde::Deserialize)]
        struct LogEntry {
            timestamp: Option<String>,
            window_title: Option<String>,
            content: Option<String>,
            source: Option<String>,
        }

        let entry: LogEntry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(e) => {
                result
                    .errors
                    .push(format!("Line {}: parse error: {}", line_num + 1, e));
                continue;
            }
        };

        let timestamp = entry.timestamp.unwrap_or_else(|| Utc::now().to_rfc3339());
        let source = entry.source.unwrap_or_else(|| "imported".to_string());

        let event = cozmio_memory::MemoryEvent {
            id: None,
            timestamp: timestamp.clone(),
            source: source.clone(),
            window_title: entry.window_title,
            content: entry.content.unwrap_or_default(),
            raw_ref: None,
            embedding: None,
            thread_id: None,
            evidence_source: source,
        };

        match events_store.insert(&event) {
            Ok(_) => result.events_imported += 1,
            Err(e) => {
                result
                    .errors
                    .push(format!("Line {}: insert error: {}", line_num + 1, e));
            }
        }
    }

    // Create a context slice summarizing the import
    if result.events_imported > 0 {
        let now = Utc::now();
        let slice = cozmio_memory::ContextSlice {
            id: None,
            start_time: now.to_rfc3339(),
            end_time: now.to_rfc3339(),
            summary: format!(
                "Imported {} events from action_log.jsonl",
                result.events_imported
            ),
            entities: vec![],
            topics: vec![],
            raw_refs: vec![],
            evidence_source: "imported".to_string(),
        };
        if let Err(e) = slices_store.insert(&slice) {
            result.errors.push(format!("Slice creation error: {}", e));
        } else {
            result.slices_created += 1;
        }
    }

    Ok(result)
}

/// Search memory with query text, optional thread filter, and limit
#[tauri::command]
pub async fn search_memory(
    query: String,
    thread_id: Option<i64>,
    limit: usize,
) -> Result<SearchResultsDto, String> {
    let db = open_memory_db()?;
    let core = MemoryCore::new(&db, None);
    let search_engine = core.search_engine();

    let search_query = SearchQuery {
        text: Some(query),
        time_range: None,
        thread_id,
        limit,
    };

    let results = search_engine
        .search(&search_query)
        .map_err(|e| e.to_string())?;

    Ok(SearchResultsDto {
        events: results
            .events
            .into_iter()
            .map(|e| SearchResultDto {
                event_id: e.event_id,
                score: e.score,
                source: e.source,
                content: e.content,
                window_title: e.window_title,
                timestamp: e.timestamp,
                evidence_source: e.evidence_source,
                thread_id: e.thread_id,
            })
            .collect(),
        total_fts: results.total_fts,
        total_vec: results.total_vec,
    })
}

/// Build activity context from ActivityNote for toast/reply generation
#[tauri::command]
pub async fn build_activity_context(
    window_title: String,
    content_text: String,
    current_thread_id: Option<i64>,
) -> Result<ReminderContextDto, String> {
    let db = open_memory_db()?;
    let core = MemoryCore::new(&db, None);
    let competition = core.competition();

    let note = ActivityNote {
        window_title,
        content_text,
        timestamp: Utc::now(),
        current_thread_id,
    };

    let ctx = competition
        .build_reminder_context(&note)
        .map_err(|e| e.to_string())?;

    Ok(ReminderContextDto {
        current_activity: ctx.current_activity,
        recent_context: ctx.recent_context,
        related_decisions: ctx.related_decisions,
        relevant_skills: ctx.relevant_skills,
        task_state: ctx.task_state,
        evidence_refs: ctx
            .evidence_refs
            .into_iter()
            .map(|e| EvidenceRefDto {
                source: e.source,
                memory_type: e.memory_type,
                id: e.id,
                content_snippet: e.content_snippet,
                timestamp: e.timestamp,
            })
            .collect(),
    })
}

/// Run suggestion replay: generate reminder candidates for recent activity
#[tauri::command]
pub async fn run_suggestion_replay() -> Result<ReminderContextDto, String> {
    // Replay the most recent activity as a suggestion context
    let db = open_memory_db()?;

    let recent_event = {
        let store = MemoryEventsStore::new(&db);
        store.get_recent(1).ok().and_then(|mut e| e.pop())
    };

    let (window_title, content_text) = if let Some(event) = recent_event {
        (event.window_title.unwrap_or_default(), event.content)
    } else {
        (
            "No recent activity".to_string(),
            "No events in memory yet".to_string(),
        )
    };

    let core = MemoryCore::new(&db, None);
    let competition = core.competition();

    let note = ActivityNote {
        window_title,
        content_text,
        timestamp: Utc::now(),
        current_thread_id: None,
    };

    let ctx = competition
        .build_reminder_context(&note)
        .map_err(|e| e.to_string())?;

    Ok(ReminderContextDto {
        current_activity: ctx.current_activity,
        recent_context: ctx.recent_context,
        related_decisions: ctx.related_decisions,
        relevant_skills: ctx.relevant_skills,
        task_state: ctx.task_state,
        evidence_refs: ctx
            .evidence_refs
            .into_iter()
            .map(|e| EvidenceRefDto {
                source: e.source,
                memory_type: e.memory_type,
                id: e.id,
                content_snippet: e.content_snippet,
                timestamp: e.timestamp,
            })
            .collect(),
    })
}

/// Get all task threads
#[tauri::command]
pub async fn get_task_threads() -> Result<Vec<TaskThreadDto>, String> {
    let db = open_memory_db()?;
    let store = TaskThreadsStore::new(&db);

    let threads = store.get_all().map_err(|e| e.to_string())?;

    Ok(threads
        .into_iter()
        .map(|t| TaskThreadDto {
            id: t.id,
            name: t.name,
            current_state: t.current_state,
            open_questions: t.open_questions,
            decisions: t.decisions,
            recent_slice_ids: t.recent_slice_ids,
            created_at: t.created_at,
            updated_at: t.updated_at,
        })
        .collect())
}

/// Create or update a task thread
#[tauri::command]
pub async fn update_task_thread(
    name: String,
    current_state: Option<String>,
    open_questions: Option<Vec<String>>,
    decisions: Option<Vec<String>>,
) -> Result<TaskThreadDto, String> {
    let db = open_memory_db()?;
    let store = TaskThreadsStore::new(&db);

    let update = TaskThreadUpdate {
        name,
        current_state,
        open_questions,
        decisions,
        recent_slice_ids: None,
    };

    let thread = store.upsert(&update).map_err(|e| e.to_string())?;

    Ok(TaskThreadDto {
        id: thread.id,
        name: thread.name,
        current_state: thread.current_state,
        open_questions: thread.open_questions,
        decisions: thread.decisions,
        recent_slice_ids: thread.recent_slice_ids,
        created_at: thread.created_at,
        updated_at: thread.updated_at,
    })
}

/// Get decisions, optionally filtered by thread
#[tauri::command]
pub async fn get_decision_memory(thread_id: Option<i64>) -> Result<Vec<DecisionDto>, String> {
    let db = open_memory_db()?;
    let store = DecisionMemoryStore::new(&db);

    let decisions = if let Some(tid) = thread_id {
        store.get_by_thread(tid).map_err(|e| e.to_string())?
    } else {
        store.get_all().map_err(|e| e.to_string())?
    };

    Ok(decisions
        .into_iter()
        .map(|d| DecisionDto {
            id: d.id,
            memory_type: d.memory_type,
            content: d.content,
            evidence: d.evidence,
            related_thread_id: d.related_thread_id,
            evidence_source: d.evidence_source,
        })
        .collect())
}

/// Add a new decision
#[tauri::command]
pub async fn add_decision(
    memory_type: String,
    content: String,
    evidence: Option<String>,
    related_thread_id: Option<i64>,
) -> Result<DecisionDto, String> {
    let db = open_memory_db()?;
    let store = DecisionMemoryStore::new(&db);

    let decision = Decision {
        id: None,
        memory_type,
        content,
        evidence,
        related_thread_id,
        evidence_source: "user-added".to_string(),
    };

    let id = store.insert(&decision).map_err(|e| e.to_string())?;

    Ok(DecisionDto {
        id: Some(id),
        memory_type: decision.memory_type,
        content: decision.content,
        evidence: decision.evidence,
        related_thread_id: decision.related_thread_id,
        evidence_source: decision.evidence_source,
    })
}

/// Get all skills
#[tauri::command]
pub async fn get_skill_memory() -> Result<Vec<SkillDto>, String> {
    let db = open_memory_db()?;
    let store = SkillMemoryStore::new(&db);

    let skills = store.get_all().map_err(|e| e.to_string())?;

    Ok(skills
        .into_iter()
        .map(|s| SkillDto {
            id: s.id,
            name: s.name,
            description: s.description,
            procedure: s.procedure,
            success_context: s.success_context,
            usage_count: s.usage_count,
            last_used_at: s.last_used_at,
            evidence_source: s.evidence_source,
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Integration note for main_loop.rs
// ---------------------------------------------------------------------------

// FUTURE: When toasting, call build_activity_context() to get ReminderContext
// let context = memory_core.build_activity_context(&activity_note)?;
//
// Example integration in main_loop.rs:
// ```rust
// if let Some(pending) = pending_confirmation {
//     let activity_note = ActivityNote {
//         window_title: current_window.title.clone(),
//         content_text: pending.task_text.clone(),
//         timestamp: Utc::now(),
//         current_thread_id: None,
//     };
//     match memory_core.competition().build_reminder_context(&activity_note) {
//         Ok(ctx) => {
//             log::info!("Reminder context built with {} evidence refs", ctx.evidence_refs.len());
//         }
//         Err(e) => {
//             log::warn!("Failed to build reminder context: {}", e);
//         }
//     }
// }
// ```
