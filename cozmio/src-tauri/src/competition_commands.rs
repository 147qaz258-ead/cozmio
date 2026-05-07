//! Competition IPC Commands
//!
//! Exposes competition functionality to the frontend via Tauri IPC.

use chrono::Utc;
use cozmio_memory::{
    ActivityNote, CompetitionResultEntry, CompetitionTrace, Database, EmbeddingProvider, MemoryCore,
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use crate::memory_commands::{CompetitionResultEntryDto, CompetitionTraceDto};

// ---------------------------------------------------------------------------
// DTO types for competition results
// ---------------------------------------------------------------------------

/// Top-level competition result (IPC-friendly)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionResultDto {
    pub entries: Vec<CompetitionResultEntryDto>,
    pub trace: CompetitionTraceDto,
}

/// Preview entry for recent competition results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionPreviewEntryDto {
    pub timestamp: String,
    pub window_title: String,
    pub entry_count: usize,
    pub total_tokens: usize,
    pub candidate_pool_size: usize,
    pub vector_available: bool,
}

/// Raw competition result stored for preview
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredCompetitionResult {
    timestamp: String,
    window_title: String,
    entries: Vec<CompetitionResultEntry>,
    trace: CompetitionTrace,
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn memory_db_path() -> PathBuf {
    Database::memory_dir().join("cozmio.db")
}

fn open_memory_db() -> Result<Database, String> {
    Database::new(&memory_db_path()).map_err(|e| e.to_string())
}

fn preview_file_path() -> PathBuf {
    Database::memory_dir().join("competition_preview.jsonl")
}

/// Save a competition result to the preview file (one line per result)
fn save_competition_result(
    window_title: &str,
    entries: &[CompetitionResultEntry],
    trace: &CompetitionTrace,
) -> Result<(), String> {
    let stored = StoredCompetitionResult {
        timestamp: Utc::now().to_rfc3339(),
        window_title: window_title.to_string(),
        entries: entries.to_vec(),
        trace: trace.clone(),
    };

    let json = serde_json::to_string(&stored).map_err(|e| e.to_string())?;
    let path = preview_file_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    // Append to file
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())?;

    Ok(())
}

/// Load recent competition results from preview file
fn load_recent_previews(limit: usize) -> Result<Vec<CompetitionPreviewEntryDto>, String> {
    let path = preview_file_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    for line in content.lines().rev().take(limit) {
        if let Ok(stored) = serde_json::from_str::<StoredCompetitionResult>(line) {
            let total_tokens: usize = stored.entries.iter().map(|e| e.token_estimate).sum();
            results.push(CompetitionPreviewEntryDto {
                timestamp: stored.timestamp,
                window_title: stored.window_title,
                entry_count: stored.entries.len(),
                total_tokens,
                candidate_pool_size: stored.trace.candidate_pool_size,
                vector_available: stored.trace.vector_available,
            });
        }
    }

    Ok(results)
}

/// Create embedding provider (FastEmbed if available, else fallback)
fn create_embedding_provider() -> Result<Arc<dyn EmbeddingProvider>, String> {
    cozmio_memory::embed_provider::create_provider(
        cozmio_memory::embed_provider::ProviderType::FastEmbed,
    )
    .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// IPC Commands
// ---------------------------------------------------------------------------

/// Run memory competition for a given note and window context.
/// Returns the competition result with selected memory entries.
#[tauri::command]
pub async fn compete_for_context(
    _app: tauri::AppHandle,
    note_text: String,
    window_title: String,
    token_budget: usize,
) -> Result<CompetitionResultDto, String> {
    let db = open_memory_db()?;

    // Create provider and pass to MemoryCore so SearchEngine also has real provider
    let provider = create_embedding_provider().ok();
    let embed_provider = provider.as_ref().and_then(|p| {
        if p.is_available() {
            Some(p.clone())
        } else {
            None
        }
    });
    let core = MemoryCore::new(&db, embed_provider);
    let competition = core.competition();

    // Backfill missing candidate embeddings when provider is available
    if let Some(ref p) = provider {
        if p.is_available() {
            let count =
                cozmio_memory::competition::backfill_candidate_embeddings(&db, p.as_ref(), 20)
                    .map_err(|e| e.to_string())?;
            log::info!("Backfilled {} candidate embeddings", count);
        }
    }

    // 3. Build ActivityNote
    let note = ActivityNote {
        window_title: window_title.clone(),
        content_text: note_text,
        timestamp: Utc::now(),
        current_thread_id: None,
    };

    // 4. Run competition - returns ReminderContext with competition_entries and competition_trace
    let ctx = competition
        .run_competition_and_build_context(&note, token_budget)
        .map_err(|e| e.to_string())?;

    // Extract entries and trace from ReminderContext
    let entries: Vec<CompetitionResultEntry> = ctx.competition_entries;
    let trace: CompetitionTrace = ctx.competition_trace.unwrap_or_else(|| CompetitionTrace {
        query_facts: serde_json::json!({
            "window_title": note.window_title,
            "content_text": note.content_text,
        }),
        candidate_pool_size: 0,
        skipped_reasons: vec!["no_trace".to_string()],
        vector_available: false,
        vector_provider: None,
    });

    // Build trace dto for return value
    let trace_dto = CompetitionTraceDto {
        query_facts: trace.query_facts.clone(),
        candidate_pool_size: trace.candidate_pool_size,
        skipped_reasons: trace.skipped_reasons.clone(),
        vector_available: trace.vector_available,
        vector_provider: trace.vector_provider.clone(),
    };

    // 5. Save latest preview/trace
    if let Err(e) = save_competition_result(&window_title, &entries, &trace) {
        log::warn!("Failed to save competition preview: {}", e);
    }

    // 6. Return CompetitionResultDto
    Ok(CompetitionResultDto {
        entries: entries
            .into_iter()
            .map(|e| CompetitionResultEntryDto {
                memory_id: e.memory_id,
                memory_text: e.memory_text,
                memory_kind: e.memory_kind,
                vector_score: e.vector_score,
                fact_trace: e.fact_trace,
                selection_reason_facts: e.selection_reason_facts,
                token_estimate: e.token_estimate,
                source_event_ids: e.source_event_ids,
                source_paths: e.source_paths,
                source_ranges: e.source_ranges,
                producer: e.producer,
            })
            .collect(),
        trace: trace_dto,
    })
}

/// Get recent competition preview entries.
#[tauri::command]
pub async fn get_competition_preview(
    _app: tauri::AppHandle,
    limit: usize,
) -> Result<Vec<CompetitionPreviewEntryDto>, String> {
    load_recent_previews(limit)
}

/// Backfill candidate embeddings for memory candidates without embeddings.
/// Returns the count of successfully backfilled candidates.
#[tauri::command]
pub async fn backfill_candidate_embeddings(
    _app: tauri::AppHandle,
    limit: usize,
) -> Result<usize, String> {
    let db = open_memory_db()?;
    let provider = create_embedding_provider()?;

    if !provider.is_available() {
        return Ok(0);
    }

    cozmio_memory::competition::backfill_candidate_embeddings(&db, provider.as_ref(), limit)
        .map_err(|e| e.to_string())
}
