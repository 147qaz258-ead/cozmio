use cozmio_memory::{AgentMemoryStore, ExperienceSource};
use serde::{Deserialize, Serialize};
use sha256::digest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExperienceEventInput {
    pub event_kind: String,
    pub trace_id: Option<String>,
    pub session_id: Option<String>,
    pub window_title: Option<String>,
    pub process_name: Option<String>,
    pub factual_text: String,
    pub raw_ref: Option<String>,
    pub artifact_refs: Vec<String>,
    pub timestamp: Option<String>,
}

impl ExperienceEventInput {
    pub fn new(event_kind: &str, factual_text: impl Into<String>) -> Self {
        Self {
            event_kind: event_kind.to_string(),
            trace_id: None,
            session_id: None,
            window_title: None,
            process_name: None,
            factual_text: factual_text.into(),
            raw_ref: None,
            artifact_refs: Vec::new(),
            timestamp: None,
        }
    }
}

pub fn is_enabled() -> bool {
    crate::config::Config::load()
        .map(|config| config.memory_flywheel_enabled)
        .unwrap_or(true)
}

pub fn record_experience(input: ExperienceEventInput) -> Result<Option<String>, String> {
    if !is_enabled() {
        return Ok(None);
    }
    let db = crate::memory_consolidation::open_memory_db()?;
    record_experience_with_store(&AgentMemoryStore::new(&db), input).map(Some)
}

pub fn record_experience_with_store(
    store: &AgentMemoryStore<'_>,
    input: ExperienceEventInput,
) -> Result<String, String> {
    let source = build_experience_source(input)?;
    store
        .insert_experience_source(&source)
        .map_err(|error| error.to_string())
}

pub fn record_experience_with_store_if_enabled(
    store: &AgentMemoryStore<'_>,
    input: ExperienceEventInput,
    enabled: bool,
) -> Result<Option<String>, String> {
    if !enabled {
        return Ok(None);
    }
    record_experience_with_store(store, input).map(Some)
}

pub fn build_experience_source(input: ExperienceEventInput) -> Result<ExperienceSource, String> {
    let event_kind = normalize_required("event_kind", &input.event_kind)?;
    let factual_text = normalize_required("factual_text", &input.factual_text)?;
    reject_forbidden_semantic_labels(&event_kind)?;
    reject_forbidden_semantic_labels(&factual_text)?;

    let artifact_refs_json = serde_json::to_string(&input.artifact_refs)
        .map_err(|error| format!("failed to encode artifact refs: {error}"))?;
    let source_ref = stable_source_ref(
        input.trace_id.as_deref(),
        input.session_id.as_deref(),
        &event_kind,
        &factual_text,
        input.raw_ref.as_deref(),
        &artifact_refs_json,
    );

    Ok(ExperienceSource {
        source_ref,
        timestamp: input.timestamp,
        trace_id: clean_optional(input.trace_id),
        session_id: clean_optional(input.session_id),
        event_kind,
        window_title: clean_optional(input.window_title),
        process_name: clean_optional(input.process_name),
        factual_text,
        raw_ref: clean_optional(input.raw_ref),
        artifact_refs_json,
    })
}

pub fn record_and_log(input: ExperienceEventInput) {
    match record_experience(input) {
        Ok(Some(source_ref)) => log::debug!("Experience source recorded source_ref={}", source_ref),
        Ok(None) => log::debug!("Experience recording disabled"),
        Err(error) => log::warn!("Experience source recording failed: {}", error),
    }
}

pub fn record_observation(
    trace_id: Option<&str>,
    window_title: &str,
    process_name: &str,
    captured_at: i64,
) {
    let mut input = ExperienceEventInput::new(
        "foreground_observation",
        format!(
            "Foreground observation captured_at={} window_title=\"{}\" process_name=\"{}\".",
            captured_at,
            clip(window_title, 180),
            clip(process_name, 120)
        ),
    );
    input.trace_id = trace_id.map(str::to_string);
    input.window_title = Some(window_title.to_string());
    input.process_name = Some(process_name.to_string());
    record_and_log(input);
}

pub fn record_model_output(
    trace_id: &str,
    window_title: &str,
    process_name: &str,
    model_name: &str,
    output_text: &str,
    captured_at: i64,
    call_duration_ms: u64,
) {
    let mut input = ExperienceEventInput::new(
        "popup_model_output",
        format!(
            "Popup model output captured_at={} model_name=\"{}\" call_duration_ms={} output_text=\"{}\".",
            captured_at,
            clip(model_name, 120),
            call_duration_ms,
            clip(output_text, 900)
        ),
    );
    input.trace_id = Some(trace_id.to_string());
    input.window_title = Some(window_title.to_string());
    input.process_name = Some(process_name.to_string());
    input.raw_ref = Some(format!("action_log:trace:{trace_id}:model_output"));
    record_and_log(input);
}

pub fn record_popup_displayed(
    trace_id: &str,
    window_title: &str,
    process_name: &str,
    task_text: &str,
) {
    let mut input = ExperienceEventInput::new(
        "popup_displayed",
        format!(
            "Confirmation popup displayed for trace_id={} task_text=\"{}\".",
            trace_id,
            clip(task_text, 900)
        ),
    );
    input.trace_id = Some(trace_id.to_string());
    input.window_title = Some(window_title.to_string());
    input.process_name = Some(process_name.to_string());
    record_and_log(input);
}

pub fn record_popup_user_action(
    event_kind: &str,
    trace_id: &str,
    window_title: &str,
    process_name: &str,
    task_text: &str,
) {
    let mut input = ExperienceEventInput::new(
        event_kind,
        format!(
            "Popup action event_kind={} trace_id={} task_text=\"{}\".",
            event_kind,
            trace_id,
            clip(task_text, 900)
        ),
    );
    input.trace_id = Some(trace_id.to_string());
    input.window_title = Some(window_title.to_string());
    input.process_name = Some(process_name.to_string());
    record_and_log(input);
}

pub fn build_relay_dispatch_event(
    trace_id: Option<&str>,
    session_id: Option<&str>,
    event_kind: &str,
    window_title: &str,
    process_name: &str,
    task_text: &str,
    detail: Option<&str>,
) -> ExperienceEventInput {
    let detail_text = detail.unwrap_or("");
    let mut input = ExperienceEventInput::new(
        event_kind,
        format!(
            "Relay event event_kind={} trace_id={} session_id={} task_text=\"{}\" detail=\"{}\".",
            event_kind,
            trace_id.unwrap_or(""),
            session_id.unwrap_or(""),
            clip(task_text, 900),
            clip(detail_text, 900)
        ),
    );
    input.trace_id = trace_id.map(str::to_string);
    input.session_id = session_id.map(str::to_string);
    input.window_title = Some(window_title.to_string());
    input.process_name = Some(process_name.to_string());
    input
}

pub fn record_relay_dispatch(
    trace_id: Option<&str>,
    session_id: Option<&str>,
    event_kind: &str,
    window_title: &str,
    process_name: &str,
    task_text: &str,
    detail: Option<&str>,
) {
    record_and_log(build_relay_dispatch_event(
        trace_id,
        session_id,
        event_kind,
        window_title,
        process_name,
        task_text,
        detail,
    ));
}

pub fn record_memory_feedback(
    event_kind: &str,
    memory_id: &str,
    factual_text: &str,
) -> Result<Option<String>, String> {
    record_memory_feedback_with_refs(event_kind, memory_id, factual_text, Vec::new())
}

pub fn record_memory_feedback_with_refs(
    event_kind: &str,
    memory_id: &str,
    factual_text: &str,
    artifact_refs: Vec<String>,
) -> Result<Option<String>, String> {
    if !is_enabled() {
        return Ok(None);
    }
    let db = crate::memory_consolidation::open_memory_db()?;
    record_memory_feedback_with_store(
        &AgentMemoryStore::new(&db),
        event_kind,
        memory_id,
        factual_text,
        artifact_refs,
    )
    .map(Some)
}

pub fn record_memory_feedback_with_store(
    store: &AgentMemoryStore<'_>,
    event_kind: &str,
    memory_id: &str,
    factual_text: &str,
    artifact_refs: Vec<String>,
) -> Result<String, String> {
    let mut input = ExperienceEventInput::new(
        event_kind,
        format!(
            "Memory feedback event_kind={} memory_id={} factual_text=\"{}\".",
            event_kind,
            memory_id,
            clip(factual_text, 900)
        ),
    );
    input.raw_ref = Some(format!("agent_memory:{memory_id}"));
    input.artifact_refs = artifact_refs;
    record_experience_with_store(store, input)
}

fn stable_source_ref(
    trace_id: Option<&str>,
    session_id: Option<&str>,
    event_kind: &str,
    factual_text: &str,
    raw_ref: Option<&str>,
    artifact_refs_json: &str,
) -> String {
    let trace_part = trace_id
        .map(safe_ref_part)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "no-trace".to_string());
    let session_part = session_id.map(safe_ref_part).unwrap_or_default();
    let hash = digest(format!(
        "{}:{}:{}:{}:{}:{}",
        trace_id.unwrap_or(""),
        session_id.unwrap_or(""),
        event_kind,
        factual_text,
        raw_ref.unwrap_or(""),
        artifact_refs_json
    ));
    if session_part.is_empty() {
        format!("experience:{trace_part}:{event_kind}:{}", &hash[..12])
    } else {
        format!(
            "experience:{trace_part}:{session_part}:{event_kind}:{}",
            &hash[..12]
        )
    }
}

fn normalize_required(field: &str, value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(value.replace('\n', " "))
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn safe_ref_part(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .chars()
        .take(80)
        .collect()
}

fn clip(value: &str, max_chars: usize) -> String {
    let mut clipped: String = value
        .trim()
        .replace('\n', " ")
        .chars()
        .take(max_chars)
        .collect();
    if value.trim().chars().count() > max_chars {
        clipped.push_str("...");
    }
    clipped
}

fn reject_forbidden_semantic_labels(value: &str) -> Result<(), String> {
    const FORBIDDEN: [&str; 20] = [
        "user_intent",
        "task_phase",
        "task_stage",
        "stuckness",
        "stuck",
        "poor_timing",
        "helpfulness",
        "user is stuck",
        "poorly timed",
        "semantic_user_state",
        "is_oscillating",
        "just_arrived",
        "last_switch_direction",
        "intent",
        "confidence",
        "next_step",
        "workflow_stage",
        "mode: continue",
        "mode: abstain",
        "negative_feedback",
    ];
    let lower = value.to_ascii_lowercase();
    for forbidden in FORBIDDEN {
        if lower.contains(forbidden) {
            return Err(format!(
                "forbidden semantic label in experience source: {forbidden}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cozmio_memory::{run_migrations, Database};
    use rusqlite::Connection;
    use std::sync::Mutex;

    #[test]
    fn event_construction_uses_mechanical_source_ref_and_fields() {
        let source = build_experience_source(ExperienceEventInput {
            event_kind: "popup_confirmed".to_string(),
            trace_id: Some("trace/1".to_string()),
            session_id: Some("session:1".to_string()),
            window_title: Some("Editor".to_string()),
            process_name: Some("Code.exe".to_string()),
            factual_text: "Popup action event_kind=popup_confirmed trace_id=trace/1.".to_string(),
            raw_ref: Some("action_log:trace:trace/1".to_string()),
            artifact_refs: vec!["ledger:event:1".to_string()],
            timestamp: Some("2026-05-07T00:00:00Z".to_string()),
        })
        .unwrap();

        assert!(source
            .source_ref
            .starts_with("experience:trace-1:session-1:popup_confirmed:"));
        assert_eq!(source.event_kind, "popup_confirmed");
        assert_eq!(source.trace_id.as_deref(), Some("trace/1"));
        assert_eq!(source.session_id.as_deref(), Some("session:1"));
        assert_eq!(source.artifact_refs_json, r#"["ledger:event:1"]"#);
    }

    #[test]
    fn recorder_rejects_forbidden_semantic_labels() {
        let err = build_experience_source(ExperienceEventInput::new(
            "popup_displayed",
            "This includes user_intent as a code-owned label.",
        ))
        .unwrap_err();

        assert!(err.contains("forbidden semantic label"));
    }

    #[test]
    fn store_recorder_gate_skips_persistence_when_disabled() {
        let db = Database {
            conn: Mutex::new(Connection::open_in_memory().unwrap()),
        };
        run_migrations(&db).unwrap();
        let store = AgentMemoryStore::new(&db);
        let result = record_experience_with_store_if_enabled(
            &store,
            ExperienceEventInput::new("popup_model_output", "Popup model output captured."),
            false,
        )
        .unwrap();

        assert_eq!(result, None);
        assert!(store.recent_experience_sources(10).unwrap().is_empty());
    }

    #[test]
    fn relay_failure_event_construction_is_mechanical_and_factual() {
        let event = build_relay_dispatch_event(
            Some("trace-1"),
            Some("session-1"),
            "relay_progress_stream_failed",
            "Editor",
            "Code.exe",
            "confirmed task text",
            Some("Relay progress stream failed: connection closed"),
        );
        let source = build_experience_source(event).unwrap();

        assert_eq!(source.event_kind, "relay_progress_stream_failed");
        assert_eq!(source.trace_id.as_deref(), Some("trace-1"));
        assert_eq!(source.session_id.as_deref(), Some("session-1"));
        assert!(source
            .factual_text
            .contains("Relay progress stream failed: connection closed"));
        assert!(!source.factual_text.contains("user_intent"));
        assert!(!source.factual_text.contains("task_phase"));
    }
}
