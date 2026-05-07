//! Evaluation IPC commands.
//!
//! Provides commands for saving and evaluating inference samples.
//!
//! - `save_evaluation_sample` — extract from trace_id, generate EvaluationSample, write to store
//! - `evaluate_sample` — call relay_bridge dispatch, store as EvaluationResult
//! - `get_evaluation_samples` — return sample list
//! - `get_evaluation_results` — return result list

use crate::commands::AppState;
use crate::evaluation::{
    EvaluationFeedback, EvaluationFeedbackStore, EvaluationResult, EvaluationResultStore,
    EvaluationSample, EvaluationSampleStore,
};
use crate::relay_bridge::{send_inference_request, BoxInferenceResult};
use chrono::Utc;
use cozmio_memory::Database;
use tauri::Manager;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Implementation helpers
// ---------------------------------------------------------------------------

/// Open the memory database from app state.
fn open_memory_db(app: &tauri::AppHandle) -> Result<Database, String> {
    let _state = app.state::<AppState>();
    let db_path = Database::memory_dir().join("cozmio.db");
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open memory DB: {}", e))?;

    // Run base memory migrations
    if let Err(e) = cozmio_memory::run_migrations(&db) {
        log::warn!("Memory migrations failed: {}", e);
    }

    // Run evaluation table migrations
    if let Err(e) = crate::evaluation::run_evaluation_migrations(&db) {
        log::warn!("Evaluation migrations failed: {}", e);
    }

    Ok(db)
}

/// Internal implementation of save_evaluation_sample.
///
/// Extracts data from ledger events for the given trace_id and saves an EvaluationSample.
pub fn save_evaluation_sample_impl(
    app: &tauri::AppHandle,
    trace_id: &str,
) -> Result<EvaluationSample, String> {
    let state = app.state::<AppState>();

    // Query ledger events for this trace_id
    let events = state
        .ledger_manager
        .get_trace(trace_id)
        .map_err(|e| format!("Failed to query ledger for trace_id={}: {}", trace_id, e))?;

    if events.is_empty() {
        return Err(format!("No events found for trace_id={}", trace_id));
    }

    // Extract screenshot_path from first screenshot event if present
    let screenshot_path = events
        .iter()
        .find(|e| {
            e.content_ref
                .as_ref()
                .map(|c| c.content_type == "screenshot")
                .unwrap_or(false)
        })
        .and_then(|e| e.content_ref.as_ref())
        .map(|c| c.path_or_key.clone())
        .unwrap_or_default();

    // Extract window_facts from process_snapshot / observation events
    let window_facts: Vec<serde_json::Value> = events
        .iter()
        .filter(|e| e.event_type == crate::ledger::event_type::OBSERVATION_CAPTURED)
        .map(|e| {
            serde_json::json!({
                "window_title": e.window_title,
                "process_name": e.process_name,
                "source": e.source,
            })
        })
        .collect();
    let window_facts_json =
        serde_json::to_string(&window_facts).unwrap_or_else(|_| "{}".to_string());

    // Build context_pack_summary - prefer real context pack from ledger
    let context_pack_summary = events
        .iter()
        .find(|e| e.event_type == crate::ledger::event_type::CONTEXT_PACK_BUILT)
        .and_then(|e| e.raw_text.clone())
        .unwrap_or_else(|| {
            events
                .iter()
                .rev()
                .take(10)
                .map(|e| {
                    format!(
                        "[{}] {}",
                        e.event_type,
                        e.window_title.as_deref().unwrap_or("")
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ")
        });

    // Build model_raw_output from model_called events
    let model_outputs: Vec<String> = events
        .iter()
        .filter(|e| {
            e.event_type == crate::ledger::event_type::MODEL_CALLED
                || e.event_type == crate::ledger::event_type::MODEL_OUTPUT_RECEIVED
        })
        .filter_map(|e| e.raw_text.clone())
        .collect();
    let model_raw_output = model_outputs.join("\n---\n");

    // Build user_action_description from relay_execution_result events
    let user_action_description = events
        .iter()
        .filter(|e| {
            e.event_type == crate::ledger::event_type::USER_CONFIRMED
                || e.event_type == crate::ledger::event_type::USER_CANCELLED
                || e.event_type == crate::ledger::event_type::USER_DISMISSED
        })
        .map(|e| format!("User action: {} on trace {}", e.event_type, trace_id))
        .collect::<Vec<_>>()
        .join("; ");

    // Build execution_result_summary from execution result events
    let execution_summaries: Vec<String> = events
        .iter()
        .filter(|e| {
            e.event_type == crate::ledger::event_type::EXECUTION_RESULT_RECEIVED
                || e.event_type == crate::ledger::event_type::EXECUTION_ERROR_RECEIVED
        })
        .filter_map(|e| e.raw_text.clone())
        .collect();
    let execution_result_summary = execution_summaries.join("\n");

    // Build captured_at from first event timestamp
    let captured_at = events
        .iter()
        .next()
        .map(|e| {
            chrono::DateTime::from_timestamp(e.timestamp, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| Utc::now().to_rfc3339())
        })
        .unwrap_or_else(|| Utc::now().to_rfc3339());

    let sample = EvaluationSample {
        id: Uuid::new_v4().to_string(),
        source_trace_id: trace_id.to_string(),
        screenshot_path,
        window_facts_json,
        context_pack_summary,
        model_raw_output,
        user_action_description,
        execution_result_summary,
        captured_at,
    };

    // Save to store
    let db = open_memory_db(app)?;
    let store = EvaluationSampleStore::new(&db);
    store
        .save_sample(&sample)
        .map_err(|e| format!("Failed to save evaluation sample: {}", e))?;

    log::info!(
        "Saved evaluation sample id={} trace_id={}",
        sample.id,
        trace_id
    );

    // H3: Generate evaluation artifact non-blocking after sample save.
    // Artifact is evidence only - does NOT auto-tune competition/prompt.
    let sample_for_artifact = sample.clone();
    let app_for_artifact = app.clone();
    std::thread::spawn(move || {
        match generate_evaluation_artifact(&app_for_artifact, &sample_for_artifact) {
            Ok(artifact) => {
                log::info!(
                    "Auto-generated evaluation artifact id={} for sample_id={}",
                    artifact.id,
                    sample_for_artifact.id
                );
            }
            Err(e) => {
                log::warn!(
                    "Failed to generate evaluation artifact for sample_id={}: {}",
                    sample_for_artifact.id,
                    e
                );
            }
        }
    });

    Ok(sample)
}

/// Internal implementation of evaluate_sample.
///
/// Calls relay_bridge::send_inference_request with the sample's source_trace_id,
/// stores the result as an EvaluationResult.
pub fn evaluate_sample_impl(
    app: &tauri::AppHandle,
    sample_id: &str,
) -> Result<EvaluationResult, String> {
    let db = open_memory_db(app)?;
    let store = EvaluationSampleStore::new(&db);

    // Get sample from store
    let sample = store
        .get_sample(sample_id)
        .map_err(|e| format!("Failed to get sample {}: {}", sample_id, e))?
        .ok_or_else(|| format!("Sample not found: {}", sample_id))?;

    // Call relay_bridge inference with the source trace_id
    let trace_id = &sample.source_trace_id;

    // Gather context from ledger events for the inference call
    let events = app
        .state::<AppState>()
        .ledger_manager
        .get_trace(trace_id)
        .map_err(|e| format!("Failed to query ledger for trace_id={}: {}", trace_id, e))?;

    let window_title = events
        .iter()
        .find_map(|e| e.window_title.clone())
        .unwrap_or_default();

    let process_name = events
        .iter()
        .find_map(|e| e.process_name.clone())
        .unwrap_or_default();

    let recent_actions: Vec<String> = events
        .iter()
        .rev()
        .take(5)
        .filter_map(|e| e.raw_text.clone())
        .collect();

    // Call inference with 60 second timeout
    let inference_result: BoxInferenceResult =
        send_inference_request(trace_id, &window_title, &process_name, &recent_actions, 60)
            .map_err(|e| format!("Inference call failed: {}", e))?;

    let judgment = if let Some(err) = &inference_result.error {
        log::warn!(
            "Box inference relay error for trace_id={}: {}",
            trace_id,
            err
        );
        crate::evaluation::Judgment::Fail
    } else {
        log::info!(
            "Box inference returned {} chars for trace_id={}; groundedness requires source/feedback review",
            inference_result.payload_text.chars().count(),
            trace_id
        );
        crate::evaluation::Judgment::Partial
    };

    let groundedness_notes = format!("{:?}", inference_result);

    let result = EvaluationResult {
        id: Uuid::new_v4().to_string(),
        sample_id: sample_id.to_string(),
        judgment,
        groundedness_notes: groundedness_notes.clone(),
        recommendation: "Review for groundedness".to_string(),
        evaluated_by: "cozmio_eval".to_string(),
        evaluated_at: Utc::now().to_rfc3339(),
    };

    // Store result
    let result_store = EvaluationResultStore::new(&db);
    result_store
        .save_result(&result)
        .map_err(|e| format!("Failed to save evaluation result: {}", e))?;

    // Also save an EvaluationFeedback record for human review
    let feedback = EvaluationFeedback {
        id: Uuid::new_v4().to_string(),
        sample_id: sample_id.to_string(),
        source_trace_id: trace_id.to_string(),
        producer: "cozmio_eval".to_string(),
        groundedness_notes: groundedness_notes.clone(),
        recommendation: result.recommendation.clone(),
        created_at: Utc::now().to_rfc3339(),
        status: "pending".to_string(),
    };

    let feedback_store = EvaluationFeedbackStore::new(&db);
    feedback_store
        .save_feedback(&feedback)
        .map_err(|e| format!("Failed to save evaluation feedback: {}", e))?;

    log::info!(
        "Stored evaluation result id={} sample_id={} judgment={:?}",
        result.id,
        sample_id,
        judgment
    );
    Ok(result)
}

/// Internal implementation of get_evaluation_samples.
pub fn get_evaluation_samples_impl(
    app: &tauri::AppHandle,
    limit: Option<usize>,
) -> Result<Vec<EvaluationSample>, String> {
    let db = open_memory_db(app)?;
    let store = EvaluationSampleStore::new(&db);
    store
        .get_samples(limit.unwrap_or(50))
        .map_err(|e| format!("Failed to get evaluation samples: {}", e))
}

/// Internal implementation of get_evaluation_results.
pub fn get_evaluation_results_impl(
    app: &tauri::AppHandle,
    limit: Option<usize>,
) -> Result<Vec<EvaluationResult>, String> {
    let db = open_memory_db(app)?;
    let store = EvaluationResultStore::new(&db);
    store
        .get_results(limit.unwrap_or(50))
        .map_err(|e| format!("Failed to get evaluation results: {}", e))
}

/// Check whether an evaluation sample already exists for the given trace_id.
pub fn sample_exists(app: &tauri::AppHandle, trace_id: &str) -> Result<bool, String> {
    let db = open_memory_db(app)?;
    let store = EvaluationSampleStore::new(&db);
    store
        .sample_exists(trace_id)
        .map_err(|e| format!("Failed to check sample existence: {}", e))
}

/// Generate a structured evaluation artifact from sample data.
///
/// Extracts context_pack_built and model output from ledger for the sample's trace_id,
/// records raw output size and asks for source/feedback/outcome review, then saves
/// as EvaluationFeedback with producer="cozmio_eval".
///
/// H3 boundary: artifact is evidence only, does NOT auto-tune competition/prompt.
pub fn generate_evaluation_artifact(
    app: &tauri::AppHandle,
    sample: &EvaluationSample,
) -> Result<EvaluationFeedback, String> {
    let db = open_memory_db(app)?;

    // Re-read ledger events for this trace_id to get raw context and model output
    let events = app
        .state::<AppState>()
        .ledger_manager
        .get_trace(&sample.source_trace_id)
        .map_err(|e| format!("Failed to get trace for artifact: {}", e))?;

    // Extract context_pack_built raw_text
    let _context_raw = events
        .iter()
        .find(|e| e.event_type == "context_pack_built")
        .and_then(|e| e.raw_text.clone())
        .unwrap_or_default();

    // Extract model output raw_text
    let model_output_raw = events
        .iter()
        .find(|e| {
            e.event_type == "model_output_received"
                || e.event_type == "relay_execution_result_received"
                || e.event_type == "model_called"
        })
        .and_then(|e| e.raw_text.clone())
        .unwrap_or_default();

    let output_chars = model_output_raw.chars().count();
    let groundedness_reason = format!(
        "Model output length was {output_chars} chars. Eval does not treat model control tokens as truth; compare this raw output with source events, user feedback, and executor outcome."
    );
    let suggestions =
        "Review raw output against source evidence and user/executor outcome".to_string();

    let artifact_feedback = EvaluationFeedback {
        id: Uuid::new_v4().to_string(),
        sample_id: sample.id.clone(),
        source_trace_id: sample.source_trace_id.clone(),
        producer: "cozmio_eval".to_string(),
        groundedness_notes: format!(
            "raw_output_chars={}\nreason={}\nsuggestions={}",
            output_chars, groundedness_reason, suggestions
        ),
        recommendation: suggestions,
        created_at: Utc::now().to_rfc3339(),
        status: "pending".to_string(),
    };

    let feedback_store = EvaluationFeedbackStore::new(&db);
    feedback_store
        .save_feedback(&artifact_feedback)
        .map_err(|e| format!("Failed to save evaluation artifact feedback: {}", e))?;

    log::info!(
        "Generated evaluation artifact for sample_id={}: raw_output_chars={}",
        sample.id,
        output_chars
    );
    Ok(artifact_feedback)
}

// ---------------------------------------------------------------------------
// IPC command wrappers
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn save_evaluation_sample(
    app: tauri::AppHandle,
    trace_id: String,
) -> Result<EvaluationSample, String> {
    save_evaluation_sample_impl(&app, &trace_id)
}

#[tauri::command]
pub fn evaluate_sample(
    app: tauri::AppHandle,
    sample_id: String,
) -> Result<EvaluationResult, String> {
    evaluate_sample_impl(&app, &sample_id)
}

#[tauri::command]
pub fn get_evaluation_samples(
    app: tauri::AppHandle,
    limit: Option<usize>,
) -> Result<Vec<EvaluationSample>, String> {
    get_evaluation_samples_impl(&app, limit)
}

#[tauri::command]
pub fn get_evaluation_results(
    app: tauri::AppHandle,
    limit: Option<usize>,
) -> Result<Vec<EvaluationResult>, String> {
    get_evaluation_results_impl(&app, limit)
}

/// Internal implementation of get_evaluation_feedback.
pub fn get_evaluation_feedback_impl(
    app: &tauri::AppHandle,
    limit: Option<usize>,
    status: Option<String>,
) -> Result<Vec<EvaluationFeedback>, String> {
    let db = open_memory_db(app)?;
    let store = EvaluationFeedbackStore::new(&db);
    store
        .get_feedback_list(limit.unwrap_or(50), status.as_deref())
        .map_err(|e| format!("Failed to get evaluation feedback: {}", e))
}

/// Internal implementation of mark_evaluation_feedback_reviewed.
pub fn mark_evaluation_feedback_reviewed_impl(
    app: &tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    let db = open_memory_db(app)?;
    let store = EvaluationFeedbackStore::new(&db);
    store
        .mark_reviewed(&id)
        .map_err(|e| format!("Failed to mark feedback as reviewed: {}", e))
}

#[tauri::command]
pub fn get_evaluation_feedback(
    app: tauri::AppHandle,
    limit: Option<usize>,
    status: Option<String>,
) -> Result<Vec<EvaluationFeedback>, String> {
    get_evaluation_feedback_impl(&app, limit, status)
}

#[tauri::command]
pub fn mark_evaluation_feedback_reviewed(app: tauri::AppHandle, id: String) -> Result<(), String> {
    mark_evaluation_feedback_reviewed_impl(&app, id)
}
