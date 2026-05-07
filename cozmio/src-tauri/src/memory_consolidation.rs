use chrono::Utc;
use cozmio_memory::{
    run_migrations, AgentMemory, AgentMemoryStore, ConsolidationRun, Database, ExperienceSource,
    MemoryWrite,
};
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, Ordering};

const DEFAULT_PRODUCER: &str = "local-popup-agent";
const IDLE_CONSOLIDATION_COOLDOWN_SECS: i64 = 300;
static LAST_IDLE_CONSOLIDATION_AT: AtomicI64 = AtomicI64::new(0);

const REMEMBER_EPISODE: &str = "remember_episode";
const REMEMBER_REFLECTION: &str = "remember_reflection";
const REMEMBER_SKILL: &str = "remember_skill";
const UPDATE_HOT_CONTEXT: &str = "update_hot_context";
const REMOVE_OR_SUPERSEDE: &str = "remove_or_supersede";
const ABSTAIN: &str = "abstain";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsolidationSourceInput {
    pub source_kind: String,
    pub source_text: String,
    pub source_ref: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsolidationSource {
    pub source_ref: String,
    pub source_kind: String,
    pub source_text: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelatedMemory {
    pub memory_id: String,
    pub layer: String,
    pub body: String,
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub use_count: Option<i64>,
    #[serde(default)]
    pub last_used_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsolidationPacket {
    pub packet_id: String,
    pub created_at: i64,
    pub sources: Vec<ConsolidationSource>,
    #[serde(default)]
    pub related_memories: Vec<RelatedMemory>,
    #[serde(default)]
    pub current_hot_context: String,
}

/// Statistics for procedure recall - returned by get_procedure_recall_stats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcedureRecallStats {
    pub total_procedures: usize,
    pub active_procedures: usize,
    pub total_use_count: usize,
    pub top_procedures: Vec<ProcedureUsageDto>,
}

/// DTO for individual procedure usage information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcedureUsageDto {
    pub memory_id: String,
    pub body_preview: String,
    pub use_count: usize,
    pub last_used_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryOperationInput {
    pub operation_type: String,
    pub layer: Option<String>,
    pub body: Option<String>,
    pub source_refs: Vec<String>,
    pub target_memory_id: Option<String>,
    pub producer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentMemoryDto {
    pub memory_id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub layer: String,
    pub lifecycle: String,
    pub body: String,
    pub source_refs: Vec<String>,
    pub supersedes: Option<String>,
    pub last_used_at: Option<i64>,
    pub use_count: i64,
    pub producer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryOperationDto {
    pub operation_id: String,
    pub created_at: i64,
    pub operation_type: String,
    pub target_memory_id: Option<String>,
    pub resulting_memory_id: Option<String>,
    pub body: Option<String>,
    pub source_refs: Vec<String>,
    pub status: String,
    pub error_text: Option<String>,
    pub producer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplyMemoryOperationResult {
    pub operation: MemoryOperationDto,
    pub memory: Option<AgentMemoryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsolidationRunDto {
    pub run_id: String,
    pub trigger_kind: String,
    pub route: String,
    pub status: String,
    pub output_text: Option<String>,
    pub error_text: Option<String>,
    pub operations: Vec<ApplyMemoryOperationResult>,
    pub packet: ConsolidationPacket,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsolidationPrompt {
    pub packet_id: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsolidationRunSummaryDto {
    pub run_id: String,
    pub trigger_kind: String,
    pub route: String,
    pub status: String,
    pub model_name: Option<String>,
    pub output_text: Option<String>,
    pub error_text: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub packet_id: Option<String>,
    pub packet_source_count: usize,
    pub packet_related_memory_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryInspectorSnapshot {
    pub memories: Vec<AgentMemoryDto>,
    pub recent_experience_sources: Vec<ExperienceSource>,
    pub recent_operations: Vec<MemoryOperationDto>,
    pub recent_consolidation_runs: Vec<ConsolidationRunSummaryDto>,
    pub latest_runtime_recall_admission: Option<RuntimeRecallAdmissionSnapshot>,
}

/// Privacy routing status for the inspector
/// Contains factual information only - no semantic inferences about content sensitivity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivacyRoutingStatus {
    pub current_mode: String,                      // "local" | "cloud" | "executor"
    pub allowed_material_ids: Vec<String>,         // material ids approved for cloud/executor
    pub consolidation_routes: Vec<String>,         // distinct routes used in recent runs
    pub route_distribution: Vec<RouteCount>,       // count per route
    pub last_approval_source_refs: Vec<String>,     // source refs from last approval
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouteCount {
    pub route: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeRecallAdmissionBudget {
    pub max_memories: usize,
    pub max_recent_feedback_facts: usize,
    pub max_hot_context_chars: usize,
    pub max_memory_chars: usize,
    pub max_feedback_admission_chars: usize,
    pub max_memory_admission_chars: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeHotContextAdmission {
    pub source_type: String,
    pub source_ref: String,
    pub chars: usize,
    pub admitted_chars: usize,
    pub mechanical_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeFeedbackFactAdmission {
    pub source_type: String,
    pub source_ref: String,
    pub event_kind: String,
    pub age_seconds: i64,
    pub factual_text: String,
    pub mechanical_score: usize,
    pub mechanical_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeMemoryAdmission {
    pub source_type: String,
    pub memory_id: String,
    pub layer: String,
    pub last_used_at: Option<String>,
    pub source_refs: Vec<String>,
    pub mechanical_score: usize,
    pub mechanical_reason: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeRecallAdmissionSnapshot {
    pub captured_at: i64,
    pub route: String,
    pub context_surface: String,
    pub window_title: String,
    pub process_name: String,
    pub budget: RuntimeRecallAdmissionBudget,
    pub hot_context: Option<RuntimeHotContextAdmission>,
    pub feedback_facts: Vec<RuntimeFeedbackFactAdmission>,
    pub memories: Vec<RuntimeMemoryAdmission>,
}

fn memory_db_path() -> PathBuf {
    Database::memory_dir().join("cozmio.db")
}

fn runtime_recall_admission_path() -> PathBuf {
    Database::memory_dir().join("latest_runtime_recall_admission.json")
}

fn load_latest_runtime_recall_admission() -> Option<RuntimeRecallAdmissionSnapshot> {
    let path = runtime_recall_admission_path();
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Parse model JSON output into MemoryOperationInput list.
/// Returns error if JSON is malformed or operations fail validation.
/// Abstain operations with no body are valid and returned as-is.
fn parse_consolidation_model_output(
    output: &str,
) -> Result<Vec<MemoryOperationInput>, String> {
    #[derive(serde::Deserialize)]
    struct OperationsWrapper {
        operations: Vec<RawOperation>,
    }
    #[derive(serde::Deserialize)]
    struct RawOperation {
        operation_type: String,
        layer: Option<String>,
        body: Option<String>,
        source_refs: Option<Vec<String>>,
        target_memory_id: Option<String>,
    }

    let trimmed = output.trim();
    let wrapper: OperationsWrapper =
        serde_json::from_str(trimmed).map_err(|e| format!("JSON parse error: {}", e))?;

    let mut results = Vec::with_capacity(wrapper.operations.len());
    for (i, op) in wrapper.operations.into_iter().enumerate() {
        validate_operation_type(&op.operation_type)
            .map_err(|err| format!("operation[{}]: {}", i, err))?;
        if let Some(ref layer) = op.layer {
            validate_layer(layer)
                .map_err(|err| format!("operation[{}]: {}", i, err))?;
        }
        // For non-abstain, body is required by validate_memory_operation but source_refs can be empty
        // if the consolidation prompt itself provides implicit source context.
        let input = MemoryOperationInput {
            operation_type: op.operation_type,
            layer: op.layer,
            body: op.body,
            source_refs: op.source_refs.unwrap_or_default(),
            target_memory_id: op.target_memory_id,
            producer: Some("consolidation-agent".to_string()),
        };
        results.push(input);
    }
    Ok(results)
}

pub fn open_memory_db() -> Result<Database, String> {
    let path = memory_db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let db = Database::new(&path).map_err(|e| e.to_string())?;
    run_migrations(&db).map_err(|e| e.to_string())?;
    Ok(db)
}

fn now_unix() -> i64 {
    Utc::now().timestamp()
}

fn stable_id(prefix: &str, created_at: i64, material: &str) -> String {
    let hash = digest(format!("{prefix}:{created_at}:{material}"));
    format!("{prefix}_{}", &hash[..16])
}

fn normalize_source_refs(refs: &[String]) -> Vec<String> {
    refs.iter()
        .map(|source_ref| source_ref.trim())
        .filter(|source_ref| !source_ref.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn validate_operation_type(operation_type: &str) -> Result<(), String> {
    match operation_type {
        REMEMBER_EPISODE | REMEMBER_REFLECTION | REMEMBER_SKILL | UPDATE_HOT_CONTEXT
        | REMOVE_OR_SUPERSEDE | ABSTAIN => Ok(()),
        other => Err(format!("Unsupported operation_type: {other}")),
    }
}

fn validate_layer(layer: &str) -> Result<(), String> {
    match layer {
        "episode" | "reflection" | "procedure" | "hot_context_proposal" => Ok(()),
        other => Err(format!("Unsupported layer: {other}")),
    }
}

fn required_layer_for_operation(operation_type: &str) -> Option<&'static str> {
    match operation_type {
        REMEMBER_EPISODE => Some("episode"),
        REMEMBER_REFLECTION => Some("reflection"),
        REMEMBER_SKILL => Some("procedure"),
        UPDATE_HOT_CONTEXT => Some("hot_context_proposal"),
        _ => None,
    }
}

fn layer_for_operation(input: &MemoryOperationInput) -> Result<Option<String>, String> {
    match input.operation_type.as_str() {
        REMEMBER_EPISODE | REMEMBER_REFLECTION | REMEMBER_SKILL | UPDATE_HOT_CONTEXT => {
            let required = required_layer_for_operation(&input.operation_type)
                .ok_or_else(|| format!("Unsupported operation_type: {}", input.operation_type))?;
            if let Some(layer) = input
                .layer
                .as_deref()
                .map(str::trim)
                .filter(|layer| !layer.is_empty())
            {
                validate_layer(layer)?;
                if layer != required {
                    return Err(format!(
                        "operation_type {} requires layer {}, got {}",
                        input.operation_type, required, layer
                    ));
                }
            }
            Ok(Some(required.to_string()))
        }
        REMOVE_OR_SUPERSEDE => {
            if let Some(layer) = input
                .layer
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                validate_layer(layer)?;
                Ok(Some(layer.to_string()))
            } else {
                Ok(None)
            }
        }
        ABSTAIN => Ok(None),
        other => Err(format!("Unsupported operation_type: {other}")),
    }
}

fn required_body(input: &MemoryOperationInput) -> Result<Option<String>, String> {
    let body = input
        .body
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    match input.operation_type.as_str() {
        REMEMBER_EPISODE | REMEMBER_REFLECTION | REMEMBER_SKILL | UPDATE_HOT_CONTEXT => body
            .map(str::to_string)
            .ok_or_else(|| format!("body is required for {}", input.operation_type))
            .map(Some),
        REMOVE_OR_SUPERSEDE | ABSTAIN => Ok(body.map(str::to_string)),
        other => Err(format!("Unsupported operation_type: {other}")),
    }
}

fn validate_source_refs_against_packet(
    input_refs: &[String],
    packet: &ConsolidationPacket,
) -> Result<Vec<String>, String> {
    let normalized = normalize_source_refs(input_refs);
    let packet_refs = packet
        .sources
        .iter()
        .map(|source| source.source_ref.as_str())
        .collect::<BTreeSet<_>>();

    for source_ref in &normalized {
        if !packet_refs.contains(source_ref.as_str()) {
            return Err(format!("Unknown source_ref: {source_ref}"));
        }
    }

    Ok(normalized)
}

fn validate_memory_operation(
    input: &MemoryOperationInput,
    packet: &ConsolidationPacket,
) -> Result<(Option<String>, Option<String>, Vec<String>, String), String> {
    validate_operation_type(&input.operation_type)?;
    let layer = layer_for_operation(input)?;
    let body = required_body(input)?;
    let source_refs = validate_source_refs_against_packet(&input.source_refs, packet)?;

    if input.operation_type != ABSTAIN && source_refs.is_empty() {
        return Err("source_refs must be non-empty except for abstain".to_string());
    }

    if input.operation_type == REMOVE_OR_SUPERSEDE {
        let target = input
            .target_memory_id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| "target_memory_id is required for remove_or_supersede".to_string())?;
        if target.parse::<i64>().is_err() {
            return Err(format!("target_memory_id must be numeric: {target}"));
        }
    }

    let producer = input
        .producer
        .as_deref()
        .map(str::trim)
        .filter(|producer| !producer.is_empty())
        .unwrap_or(DEFAULT_PRODUCER)
        .to_string();

    Ok((layer, body, source_refs, producer))
}

fn create_memory(
    store: &AgentMemoryStore<'_>,
    body: String,
    layer: String,
    source_refs: Vec<String>,
    producer: String,
) -> Result<AgentMemoryDto, String> {
    if let Some(existing) = find_exact_active_memory(store, &layer, &body, &source_refs)? {
        return Ok(existing);
    }

    let id = store
        .remember_memory(MemoryWrite {
            body,
            layer,
            lifecycle: "active".to_string(),
            source_refs,
            supersedes_id: None,
            producer,
            expires_at: None,
        })
        .map_err(|e| e.to_string())?;
    memory_by_id(store, id)
}

fn find_exact_active_memory(
    store: &AgentMemoryStore<'_>,
    layer: &str,
    body: &str,
    source_refs: &[String],
) -> Result<Option<AgentMemoryDto>, String> {
    let normalized_source_refs = normalize_source_refs(source_refs);
    for memory in store.list_active().map_err(|e| e.to_string())? {
        if memory.layer != layer || memory.body != body {
            continue;
        }
        let stored_refs: Vec<String> = serde_json::from_str(&memory.source_refs_json)
            .map_err(|e| format!("failed to decode memory source refs: {e}"))?;
        if normalize_source_refs(&stored_refs) == normalized_source_refs {
            return Ok(Some(agent_memory_to_dto(memory)));
        }
    }
    Ok(None)
}

fn supersede_memory(
    store: &AgentMemoryStore<'_>,
    old_id: i64,
    body: String,
    layer: String,
    source_refs: Vec<String>,
    producer: String,
) -> Result<AgentMemoryDto, String> {
    let id = store
        .supersede_memory(
            old_id,
            MemoryWrite {
                body,
                layer,
                lifecycle: "active".to_string(),
                source_refs,
                supersedes_id: Some(old_id),
                producer,
                expires_at: None,
            },
        )
        .map_err(|e| e.to_string())?;
    memory_by_id(store, id)
}

fn memory_by_id(store: &AgentMemoryStore<'_>, id: i64) -> Result<AgentMemoryDto, String> {
    let stored = store
        .get(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Inserted memory was not found: {id}"))?;
    Ok(agent_memory_to_dto(stored))
}

fn agent_memory_to_dto(memory: AgentMemory) -> AgentMemoryDto {
    let created_at = parse_sqlite_time(memory.created_at.as_deref()).unwrap_or_default();
    let updated_at = parse_sqlite_time(memory.updated_at.as_deref()).unwrap_or(created_at);
    AgentMemoryDto {
        memory_id: memory.id.unwrap_or_default().to_string(),
        created_at,
        updated_at,
        layer: memory.layer,
        lifecycle: memory.lifecycle,
        body: memory.body,
        source_refs: serde_json::from_str(&memory.source_refs_json).unwrap_or_default(),
        supersedes: memory.supersedes_id.map(|id| id.to_string()),
        last_used_at: parse_sqlite_time(memory.last_used_at.as_deref()),
        use_count: memory.used_count,
        producer: memory.producer,
    }
}

fn parse_sqlite_time(value: Option<&str>) -> Option<i64> {
    value.and_then(|value| {
        chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|dt| dt.and_utc().timestamp())
    })
}

pub fn prepare_memory_consolidation_impl(
    sources: Vec<ConsolidationSourceInput>,
) -> Result<ConsolidationPacket, String> {
    if sources.is_empty() {
        return Err("sources must be non-empty".to_string());
    }

    let created_at = now_unix();
    let mut prepared = Vec::with_capacity(sources.len());
    for (idx, source) in sources.into_iter().enumerate() {
        let source_text = source.source_text.trim();
        if source_text.is_empty() {
            return Err(format!("source_text is empty at index {idx}"));
        }

        let source_kind = source.source_kind.trim();
        if source_kind.is_empty() {
            return Err(format!("source_kind is empty at index {idx}"));
        }

        let source_ref = source.source_ref.unwrap_or_else(|| {
            let hash = digest(format!("{created_at}:{idx}:{source_kind}:{source_text}"));
            format!("source_{}", &hash[..16])
        });

        prepared.push(ConsolidationSource {
            source_ref,
            source_kind: source_kind.to_string(),
            source_text: source_text.to_string(),
            timestamp: source.timestamp.unwrap_or_else(|| Utc::now().to_rfc3339()),
        });
    }

    let material = serde_json::to_string(&prepared).map_err(|e| e.to_string())?;
    Ok(ConsolidationPacket {
        packet_id: stable_id("packet", created_at, &material),
        created_at,
        sources: prepared,
        related_memories: Vec::new(),
        current_hot_context: String::new(),
    })
}

pub fn build_consolidation_packet_from_store(
    store: &AgentMemoryStore<'_>,
    limit: usize,
    current_hot_context: String,
) -> Result<ConsolidationPacket, String> {
    let sources = store
        .recent_experience_sources(limit)
        .map_err(|e| e.to_string())?
        .into_iter()
        .rev()
        .map(source_to_packet_input)
        .collect::<Vec<_>>();
    let mut packet = prepare_memory_consolidation_impl(sources)?;
    packet.current_hot_context = current_hot_context;
    packet.related_memories = related_memories_for_packet(store, &packet, 8)?;
    Ok(packet)
}

fn source_to_packet_input(source: ExperienceSource) -> ConsolidationSourceInput {
    let source_ref = source.source_ref.clone();
    let timestamp = source.timestamp.clone();
    let event_kind = source.event_kind.clone();
    ConsolidationSourceInput {
        source_kind: event_kind.clone(),
        source_text: format!(
            "source_ref={} timestamp={} trace_id={} session_id={} event_kind={} window_title={} process_name={} factual_text={} raw_ref={} artifact_refs={}",
            source_ref,
            timestamp.clone().unwrap_or_default(),
            source.trace_id.unwrap_or_default(),
            source.session_id.unwrap_or_default(),
            event_kind,
            source.window_title.unwrap_or_default(),
            source.process_name.unwrap_or_default(),
            source.factual_text,
            source.raw_ref.unwrap_or_default(),
            source.artifact_refs_json
        ),
        source_ref: Some(source_ref),
        timestamp,
    }
}

fn related_memories_for_packet(
    store: &AgentMemoryStore<'_>,
    packet: &ConsolidationPacket,
    limit: usize,
) -> Result<Vec<RelatedMemory>, String> {
    let query = packet
        .sources
        .iter()
        .map(|source| source.source_text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let recalled_memories = store
        .recall_active_by_text(&query, limit)
        .map_err(|e| e.to_string())?;

    // Increment use_count for procedure memories and collect results
    let mut memories = Vec::with_capacity(recalled_memories.len());
    for memory in recalled_memories {
        let memory_id = memory.id.unwrap_or_default();
        let is_procedure = memory.layer == "procedure";

        // Increment use_count for procedure memories when recalled
        if is_procedure && memory_id > 0 {
            let _ = store.mark_used(memory_id);
        }

        let last_used_at = parse_sqlite_time(memory.last_used_at.as_deref());
        memories.push(RelatedMemory {
            memory_id: memory_id.to_string(),
            layer: memory.layer,
            body: memory.body,
            source_refs: serde_json::from_str(&memory.source_refs_json).unwrap_or_default(),
            use_count: if is_procedure { Some(memory.used_count) } else { None },
            last_used_at,
        });
    }
    memories.sort_by(|left, right| left.memory_id.cmp(&right.memory_id));
    Ok(memories)
}

fn load_current_hot_context_from_path(path: &Path) -> String {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                "[hot_context_unavailable: human_context.md is empty]".to_string()
            } else {
                trimmed.to_string()
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => format!(
            "[hot_context_unavailable: human_context.md not found at {}]",
            path.display()
        ),
        Err(error) => format!(
            "[hot_context_unavailable: failed to read {}: {}]",
            path.display(),
            error
        ),
    }
}

fn default_hot_context_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|base| base.join("cozmio").join("human_context.md"))
}

fn load_current_hot_context_for_consolidation() -> String {
    if let Some(path) = default_hot_context_path() {
        load_current_hot_context_from_path(&path)
    } else {
        "[hot_context_unavailable: failed to resolve local data directory]".to_string()
    }
}

/// Read the current hot context file content (human_context.md).
/// Returns empty string if file does not exist.
#[tauri::command]
pub fn get_hot_context() -> String {
    default_hot_context_path()
        .map(|p| std::fs::read_to_string(&p).unwrap_or_default())
        .unwrap_or_default()
}

/// Write new content to the hot context file (human_context.md).
/// Creates the file and parent directories if they do not exist.
#[tauri::command]
pub fn set_hot_context(content: String) -> Result<(), String> {
    let path = default_hot_context_path()
        .ok_or_else(|| "failed to resolve local data directory".to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create directory: {}", e))?;
    }
    std::fs::write(&path, &content)
        .map_err(|e| format!("failed to write hot context: {}", e))?;
    Ok(())
}

pub fn build_consolidation_prompt(
    packet: &ConsolidationPacket,
) -> Result<ConsolidationPrompt, String> {
    let packet_json = serde_json::to_string_pretty(packet).map_err(|e| e.to_string())?;
    let prompt = format!(
        r#"You are Cozmio's local-only memory consolidation agent.

INPUTS:
- recent factual packet from experience_sources
- related existing active memories
- current hot context
- operation tool descriptions
- privacy/routing limits

BOUNDARIES:
- Do not invent motives, intent, task phase, or hidden user state.
- Preserve uncertainty in memory bodies when evidence is ambiguous.
- Write only if future Cozmio behavior should change.
- Cite source_refs for every non-abstain operation.
- Prefer feedback-bearing experience such as confirm, cancel, dismiss, expiration, executor completed, executor failed, user correction, or memory rejection.
- Avoid stable conclusions from screenshot-only evidence.
- Local-only route: do not request cloud or external execution for consolidation.

TOOLS:
- remember_episode: create layer episode for what happened.
- remember_reflection: create layer reflection for what should change next time.
- remember_skill: create layer procedure for a reusable helping or handoff pattern.
- update_hot_context: create layer hot_context_proposal only; it is a draft and must not be auto-applied.
- remove_or_supersede: reject or supersede an existing memory; target_memory_id is required.
- abstain: record that no memory update is warranted.

Return JSON with shape:
{{"operations":[{{"operation_type":"remember_episode|remember_reflection|remember_skill|update_hot_context|remove_or_supersede|abstain","layer":"episode|reflection|procedure|hot_context_proposal","body":"natural language memory text","source_refs":["..."],"target_memory_id":"optional numeric id"}}]}}

FACTUAL_PACKET:
{packet_json}
"#
    );
    Ok(ConsolidationPrompt {
        packet_id: packet.packet_id.clone(),
        prompt,
    })
}

pub fn run_manual_consolidation_impl(
    db: &Database,
    trigger_kind: &str,
    operations: Vec<MemoryOperationInput>,
) -> Result<ConsolidationRunDto, String> {
    let store = AgentMemoryStore::new(db);
    let packet = build_consolidation_packet_from_store(
        &store,
        50,
        load_current_hot_context_for_consolidation(),
    )?;
    run_manual_consolidation_with_packet(db, trigger_kind, packet, operations)
}

pub fn run_manual_consolidation_with_packet(
    db: &Database,
    trigger_kind: &str,
    packet: ConsolidationPacket,
    operations: Vec<MemoryOperationInput>,
) -> Result<ConsolidationRunDto, String> {
    if operations.is_empty() {
        return Err("operations must be non-empty".to_string());
    }
    let store = AgentMemoryStore::new(db);
    let packet_json = serde_json::to_string(&packet).map_err(|e| e.to_string())?;
    let run_id = store
        .create_consolidation_run(trigger_kind, "local-manual", &packet_json, None)
        .map_err(|e| e.to_string())?;

    let mut applied = Vec::new();
    for input in operations {
        match apply_memory_operation_impl(db, &packet, input) {
            Ok(result) => applied.push(result),
            Err(error) => {
                let _ = store.fail_consolidation_run(run_id, &error);
                return Err(error);
            }
        }
    }

    let output_text = serde_json::to_string(&applied).map_err(|e| e.to_string())?;
    store
        .complete_consolidation_run(run_id, &output_text)
        .map_err(|e| e.to_string())?;
    Ok(run_to_dto(
        store
            .get_consolidation_run(run_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("consolidation run not found: {run_id}"))?,
        packet,
        applied,
    ))
}

fn run_to_dto(
    run: ConsolidationRun,
    packet: ConsolidationPacket,
    operations: Vec<ApplyMemoryOperationResult>,
) -> ConsolidationRunDto {
    ConsolidationRunDto {
        run_id: run.id.unwrap_or_default().to_string(),
        trigger_kind: run.trigger_kind,
        route: run.route,
        status: run.status,
        output_text: run.output_text,
        error_text: run.error_text,
        operations,
        packet,
    }
}

pub fn schedule_consolidation_after_event(trigger_kind: &str) {
    if !crate::experience_recorder::is_enabled() {
        return;
    }
    let trigger_kind = trigger_kind.to_string();
    std::thread::spawn(move || {
        // Load config inside thread to get consolidation_model_name
        let config = match crate::config::Config::load() {
            Ok(c) => c,
            Err(e) => {
                log::warn!("Consolidation skipped: failed to load config: {}", e);
                return;
            }
        };

        // Constraint check: cloud/executor consolidation requires explicit allowed material
        if config.memory_maintenance_mode != "local" {
            log::warn!(
                "cloud/executor consolidation requires explicit allowed material: mode={}",
                config.memory_maintenance_mode
            );
            // Note: Does not block execution - full allowed material system not yet implemented
        }

        match open_memory_db().and_then(|db| {
            let store = AgentMemoryStore::new(&db);
            let packet = build_consolidation_packet_from_store(
                &store,
                50,
                load_current_hot_context_for_consolidation(),
            )?;
            let prompt = build_consolidation_prompt(&packet)?;
            let packet_json = serde_json::to_string(&packet).map_err(|e| e.to_string())?;
            let model_client = crate::model_client::ModelClient::new(config);
            let model_name = model_client.consolidation_model_name().to_string();
            let run_id = store
                .create_consolidation_run(&trigger_kind, "local-live", &packet_json, Some(&model_name))
                .map_err(|e| e.to_string())?;
            let (response_text, used_model, _call_duration_ms) =
                model_client.call_consolidation_model(&prompt.prompt).map_err(|e| {
                    store
                        .fail_consolidation_run(run_id, &format!("model call failed: {}", e))
                        .ok();
                    e
                })?;
            if response_text.trim().is_empty() {
                store
                    .complete_consolidation_run(run_id, "abstain: empty model response")
                    .map_err(|e| e.to_string())?;
                return Ok(());
            }
            let operations = parse_consolidation_model_output(&response_text).map_err(|e| {
                store
                    .fail_consolidation_run(run_id, &format!("parse error: {}", e))
                    .ok();
                e
            })?;
            let operations_count = operations.len();
            for op_input in operations {
                let result = apply_memory_operation_impl(&db, &packet, op_input);
                if let Err(apply_err) = result {
                    log::warn!("Consolidation op failed: {}", apply_err);
                }
            }
            store
                .complete_consolidation_run(
                    run_id,
                    &format!(
                        "model={} operations_applied={}\n{}",
                        used_model,
                        operations_count,
                        response_text
                    ),
                )
                .map_err(|e| e.to_string())
        }) {
            Ok(()) => log::debug!(
                "Consolidation live run completed trigger={}",
                trigger_kind
            ),
            Err(error) => log::warn!(
                "Consolidation live run failed trigger={} error={}",
                trigger_kind,
                error
            ),
        }
    });
}

pub fn schedule_idle_consolidation_if_due() {
    if !crate::experience_recorder::is_enabled() {
        return;
    }
    let now = now_unix();
    let previous = LAST_IDLE_CONSOLIDATION_AT.load(Ordering::Relaxed);
    if now - previous < IDLE_CONSOLIDATION_COOLDOWN_SECS {
        return;
    }
    if LAST_IDLE_CONSOLIDATION_AT
        .compare_exchange(previous, now, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {
        return;
    }
    schedule_consolidation_after_event("idle_unconsolidated_events");
}

pub fn apply_memory_operation_impl(
    db: &Database,
    packet: &ConsolidationPacket,
    input: MemoryOperationInput,
) -> Result<ApplyMemoryOperationResult, String> {
    let (layer, body, source_refs, producer) = validate_memory_operation(&input, packet)?;
    let created_at = now_unix();
    let operation_id = stable_id(
        "op",
        created_at,
        &format!(
            "{}:{:?}:{:?}:{:?}",
            input.operation_type, input.target_memory_id, body, source_refs
        ),
    );

    let mut operation = MemoryOperationDto {
        operation_id,
        created_at,
        operation_type: input.operation_type.clone(),
        target_memory_id: input.target_memory_id.clone(),
        resulting_memory_id: None,
        body: body.clone(),
        source_refs: source_refs.clone(),
        status: "accepted".to_string(),
        error_text: None,
        producer: producer.clone(),
    };

    if input.operation_type == ABSTAIN {
        return Ok(ApplyMemoryOperationResult {
            operation,
            memory: None,
        });
    }

    let store = AgentMemoryStore::new(db);
    if input.operation_type == REMOVE_OR_SUPERSEDE {
        let target = input
            .target_memory_id
            .as_deref()
            .ok_or_else(|| "target_memory_id is required for remove_or_supersede".to_string())?
            .parse::<i64>()
            .map_err(|e| e.to_string())?;
        if let (Some(layer), Some(body)) = (layer, body) {
            let memory = supersede_memory(
                &store,
                target,
                body,
                layer,
                source_refs.clone(),
                producer.clone(),
            )?;
            operation.resulting_memory_id = Some(memory.memory_id.clone());
            crate::experience_recorder::record_memory_feedback_with_store(
                &store,
                "memory_superseded",
                &target.to_string(),
                &format!(
                    "remove_or_supersede operation superseded memory_id={} resulting_memory_id={}",
                    target, memory.memory_id
                ),
                source_refs.clone(),
            )?;
            return Ok(ApplyMemoryOperationResult {
                operation,
                memory: Some(memory),
            });
        }
        store
            .reject_with_reason(
                target,
                Some("remove_or_supersede operation without successor body"),
                &producer,
            )
            .map_err(|e| e.to_string())?;
        crate::experience_recorder::record_memory_feedback_with_store(
            &store,
            "memory_rejected",
            &target.to_string(),
            &format!(
                "remove_or_supersede operation rejected memory_id={} without successor body",
                target
            ),
            source_refs.clone(),
        )?;
        return Ok(ApplyMemoryOperationResult {
            operation,
            memory: None,
        });
    }

    let layer = layer.ok_or_else(|| "layer is required for memory creation".to_string())?;
    let body = body.ok_or_else(|| "body is required for memory creation".to_string())?;
    let memory = create_memory(&store, body, layer, source_refs.clone(), producer.clone())?;
    operation.resulting_memory_id = Some(memory.memory_id.clone());
    Ok(ApplyMemoryOperationResult {
        operation,
        memory: Some(memory),
    })
}

pub fn list_agent_memories_impl(
    db: &Database,
    limit: usize,
) -> Result<Vec<AgentMemoryDto>, String> {
    let store = AgentMemoryStore::new(db);
    let mut memories = store
        .list_active()
        .map_err(|e| e.to_string())?
        .into_iter()
        .rev()
        .take(limit)
        .map(agent_memory_to_dto)
        .collect::<Vec<_>>();
    memories.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(memories)
}

fn list_all_agent_memories_impl(
    db: &Database,
    limit: usize,
) -> Result<Vec<AgentMemoryDto>, String> {
    let conn = db.conn.lock().unwrap();
    let sql = format!(
        "SELECT id, body, layer, lifecycle, source_refs_json, supersedes_id, producer,
                used_count, last_used_at, created_at, updated_at, rejected_reason, expires_at
         FROM agent_memories
         ORDER BY updated_at DESC, id DESC
         LIMIT {}",
        limit
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
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
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
        .map(|memories| memories.into_iter().map(agent_memory_to_dto).collect())
}

fn list_recent_memory_operations_impl(
    db: &Database,
    limit: usize,
) -> Result<Vec<MemoryOperationDto>, String> {
    let conn = db.conn.lock().unwrap();
    let sql = format!(
        "SELECT id, operation_type, target_memory_id, resulting_memory_id, body,
                source_refs_json, status, error_text, producer, created_at
         FROM memory_operations
         ORDER BY id DESC
         LIMIT {}",
        limit
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let source_refs_json: String = row.get(5)?;
            let created_at: Option<String> = row.get(9)?;
            Ok(MemoryOperationDto {
                operation_id: row.get::<_, i64>(0)?.to_string(),
                created_at: parse_sqlite_time(created_at.as_deref()).unwrap_or_default(),
                operation_type: row.get(1)?,
                target_memory_id: row.get::<_, Option<i64>>(2)?.map(|id| id.to_string()),
                resulting_memory_id: row.get::<_, Option<i64>>(3)?.map(|id| id.to_string()),
                body: row.get(4)?,
                source_refs: serde_json::from_str(&source_refs_json).unwrap_or_default(),
                status: row.get(6)?,
                error_text: row.get(7)?,
                producer: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn list_recent_consolidation_runs_impl(
    db: &Database,
    limit: usize,
) -> Result<Vec<ConsolidationRunSummaryDto>, String> {
    let conn = db.conn.lock().unwrap();
    let sql = format!(
        "SELECT id, trigger_kind, route, packet_json, model_name, output_text, status,
                error_text, created_at, completed_at
         FROM consolidation_runs
         ORDER BY id DESC
         LIMIT {}",
        limit
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let packet_json: String = row.get(3)?;
            let packet: Option<ConsolidationPacket> = serde_json::from_str(&packet_json).ok();
            let created_at: Option<String> = row.get(8)?;
            let completed_at: Option<String> = row.get(9)?;
            Ok(ConsolidationRunSummaryDto {
                run_id: row.get::<_, i64>(0)?.to_string(),
                trigger_kind: row.get(1)?,
                route: row.get(2)?,
                status: row.get(6)?,
                model_name: row.get(4)?,
                output_text: row.get(5)?,
                error_text: row.get(7)?,
                created_at: parse_sqlite_time(created_at.as_deref()).unwrap_or_default(),
                completed_at: parse_sqlite_time(completed_at.as_deref()),
                packet_id: packet.as_ref().map(|packet| packet.packet_id.clone()),
                packet_source_count: packet
                    .as_ref()
                    .map(|packet| packet.sources.len())
                    .unwrap_or(0),
                packet_related_memory_count: packet
                    .as_ref()
                    .map(|packet| packet.related_memories.len())
                    .unwrap_or(0),
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn get_memory_inspector_snapshot_impl(
    db: &Database,
    limit: usize,
) -> Result<MemoryInspectorSnapshot, String> {
    let limit = limit.clamp(1, 100);
    let store = AgentMemoryStore::new(db);
    Ok(MemoryInspectorSnapshot {
        memories: list_all_agent_memories_impl(db, limit)?,
        recent_experience_sources: store
            .recent_experience_sources(limit)
            .map_err(|e| e.to_string())?,
        recent_operations: list_recent_memory_operations_impl(db, limit)?,
        recent_consolidation_runs: list_recent_consolidation_runs_impl(db, limit)?,
        latest_runtime_recall_admission: load_latest_runtime_recall_admission(),
    })
}

pub fn auto_remember_model_output_with_db(
    _db: &Database,
    _window_title: &str,
    _process_name: &str,
    _trace_id: Option<&str>,
    model_text: &str,
    _captured_at: i64,
) -> Result<Option<i64>, String> {
    let model_text = model_text.trim();
    if model_text.is_empty() {
        return Ok(None);
    }

    log::debug!(
        "Direct model-output memory writes are disabled; consolidation owns memory synthesis"
    );
    Ok(None)
}

pub fn auto_remember_model_output_with_db_if_enabled(
    db: &Database,
    memory_flywheel_enabled: bool,
    window_title: &str,
    process_name: &str,
    trace_id: Option<&str>,
    model_text: &str,
    captured_at: i64,
) -> Result<Option<i64>, String> {
    if !memory_flywheel_enabled {
        return Ok(None);
    }
    auto_remember_model_output_with_db(
        db,
        window_title,
        process_name,
        trace_id,
        model_text,
        captured_at,
    )
}

pub fn auto_remember_model_output(
    window_title: &str,
    process_name: &str,
    trace_id: Option<&str>,
    model_text: &str,
    captured_at: i64,
) -> Result<Option<i64>, String> {
    if !crate::experience_recorder::is_enabled() {
        return Ok(None);
    }
    let db = open_memory_db()?;
    auto_remember_model_output_with_db(
        &db,
        window_title,
        process_name,
        trace_id,
        model_text,
        captured_at,
    )
}

fn clip(value: &str, max_chars: usize) -> String {
    let mut clipped: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        clipped.push_str("...");
    }
    clipped.replace('\n', " ")
}

#[tauri::command]
pub fn prepare_memory_consolidation(
    sources: Vec<ConsolidationSourceInput>,
) -> Result<ConsolidationPacket, String> {
    prepare_memory_consolidation_impl(sources)
}

#[tauri::command]
pub fn build_memory_consolidation_prompt() -> Result<ConsolidationPrompt, String> {
    let db = open_memory_db()?;
    let store = AgentMemoryStore::new(&db);
    let packet = build_consolidation_packet_from_store(
        &store,
        50,
        load_current_hot_context_for_consolidation(),
    )?;
    build_consolidation_prompt(&packet)
}

#[tauri::command]
pub fn run_manual_memory_consolidation(
    trigger_kind: String,
    operations: Vec<MemoryOperationInput>,
) -> Result<ConsolidationRunDto, String> {
    let db = open_memory_db()?;
    run_manual_consolidation_impl(&db, &trigger_kind, operations)
}

pub fn apply_memory_operation_command_path<F>(
    open_db: F,
    packet: ConsolidationPacket,
    input: MemoryOperationInput,
) -> Result<ApplyMemoryOperationResult, String>
where
    F: FnOnce() -> Result<Database, String>,
{
    let db = open_db()?;
    apply_memory_operation_impl(&db, &packet, input)
}

#[tauri::command]
pub fn apply_memory_operation(
    packet: ConsolidationPacket,
    input: MemoryOperationInput,
) -> Result<ApplyMemoryOperationResult, String> {
    apply_memory_operation_command_path(open_memory_db, packet, input)
}

#[tauri::command]
pub fn list_agent_memories(limit: Option<usize>) -> Result<Vec<AgentMemoryDto>, String> {
    let db = open_memory_db()?;
    list_agent_memories_impl(&db, limit.unwrap_or(20))
}

#[tauri::command]
pub fn get_memory_inspector_snapshot(
    limit: Option<usize>,
) -> Result<MemoryInspectorSnapshot, String> {
    let db = open_memory_db()?;
    get_memory_inspector_snapshot_impl(&db, limit.unwrap_or(30))
}

/// Returns privacy routing status for the inspector.
/// All returned information is factual boundary data - no semantic inference about content sensitivity.
#[tauri::command]
pub fn get_privacy_routing_status() -> Result<PrivacyRoutingStatus, String> {
    let db = open_memory_db()?;

    // Get current mode from config
    let config = crate::config::Config::load().map_err(|e| e.to_string())?;
    let current_mode = config.memory_maintenance_mode;

    // Get recent consolidation runs to analyze route distribution
    let recent_runs = list_recent_consolidation_runs_impl(&db, 100)
        .map_err(|e| e.to_string())?;

    // Build route distribution
    let mut route_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut consolidation_routes: Vec<String> = Vec::new();
    for run in &recent_runs {
        *route_counts.entry(run.route.clone()).or_insert(0) += 1;
        if !consolidation_routes.contains(&run.route) {
            consolidation_routes.push(run.route.clone());
        }
    }

    let route_distribution: Vec<RouteCount> = route_counts
        .into_iter()
        .map(|(route, count)| RouteCount { route, count })
        .collect();

    // For now, allowed_material_ids and last_approval_source_refs are empty vectors
    // until the full allowed material system is implemented
    let allowed_material_ids: Vec<String> = Vec::new();
    let last_approval_source_refs: Vec<String> = Vec::new();

    Ok(PrivacyRoutingStatus {
        current_mode,
        allowed_material_ids,
        consolidation_routes,
        route_distribution,
        last_approval_source_refs,
    })
}

pub fn reject_agent_memory_impl(
    db: &Database,
    memory_id: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    let id = memory_id
        .trim()
        .parse::<i64>()
        .map_err(|error| format!("memory_id must be numeric: {error}"))?;
    AgentMemoryStore::new(db)
        .reject_with_reason(id, reason, "user")
        .map_err(|error| error.to_string())?;
    crate::experience_recorder::record_memory_feedback_with_store(
        &AgentMemoryStore::new(db),
        "memory_rejected",
        memory_id,
        reason.unwrap_or("agent memory rejected"),
        vec![],
    )?;
    Ok(())
}

pub fn reject_agent_memory_command_path<F>(
    open_db: F,
    memory_id: &str,
    reason: Option<&str>,
) -> Result<(), String>
where
    F: FnOnce() -> Result<Database, String>,
{
    let db = open_db()?;
    reject_agent_memory_impl(&db, memory_id, reason)
}

#[tauri::command]
pub fn reject_agent_memory(memory_id: String, reason: Option<String>) -> Result<(), String> {
    reject_agent_memory_command_path(open_memory_db, &memory_id, reason.as_deref())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayTracesResult {
    pub packet_id: String,
    pub sources_count: usize,
    pub prompt_preview: String,
}

#[tauri::command]
pub fn replay_traces(source_refs: Vec<String>) -> Result<ReplayTracesResult, String> {
    if source_refs.is_empty() {
        return Err("source_refs must be non-empty".to_string());
    }
    let db = open_memory_db()?;
    let store = AgentMemoryStore::new(&db);
    let all_sources = store
        .recent_experience_sources(200)
        .map_err(|e| e.to_string())?;
    let matching: Vec<ExperienceSource> = all_sources
        .into_iter()
        .filter(|s| source_refs.contains(&s.source_ref))
        .collect();
    if matching.is_empty() {
        return Err("no matching sources found for the given source_refs".to_string());
    }
    let inputs: Vec<ConsolidationSourceInput> = matching
        .into_iter()
        .map(source_to_packet_input)
        .collect();
    let packet = prepare_memory_consolidation_impl(inputs)?;
    let prompt = build_consolidation_prompt(&packet)?;
    Ok(ReplayTracesResult {
        packet_id: packet.packet_id,
        sources_count: packet.sources.len(),
        prompt_preview: clip(&prompt.prompt, 500),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayComparisonReport {
    pub with_memory_preview: String,
    pub without_memory_preview: String,
    pub source_count: usize,
    pub admitted_memory_count: usize,
    pub hot_context_chars: usize,
}

#[tauri::command]
pub fn build_replay_comparison_report(source_refs: Vec<String>) -> Result<ReplayComparisonReport, String> {
    if source_refs.is_empty() {
        return Err("source_refs must be non-empty".to_string());
    }
    let db = open_memory_db()?;
    let store = AgentMemoryStore::new(&db);

    // Get all sources matching the given source_refs
    let all_sources = store
        .recent_experience_sources(200)
        .map_err(|e| e.to_string())?;
    let matching: Vec<ExperienceSource> = all_sources
        .into_iter()
        .filter(|s| source_refs.contains(&s.source_ref))
        .collect();
    if matching.is_empty() {
        return Err("no matching sources found for the given source_refs".to_string());
    }

    // 1. WITH MEMORY: build packet with memories using prepare_memory_consolidation_impl
    let inputs_with_memories: Vec<ConsolidationSourceInput> = matching
        .clone()
        .into_iter()
        .map(source_to_packet_input)
        .collect();
    let mut packet_with_memory = prepare_memory_consolidation_impl(inputs_with_memories)?;
    let current_hot_context = load_current_hot_context_for_consolidation();
    let hot_context_chars = current_hot_context.len();
    packet_with_memory.current_hot_context = current_hot_context;
    // Attach related memories (admitted memories)
    let admitted_memories = related_memories_for_packet(&store, &packet_with_memory, 8)?;
    let admitted_memory_count = admitted_memories.len();
    packet_with_memory.related_memories = admitted_memories;
    let prompt_with_memory = build_consolidation_prompt(&packet_with_memory)?;
    let with_memory_preview = clip(&prompt_with_memory.prompt, 500);

    // 2. WITHOUT MEMORY: build a factual-only packet (no admitted memories, no hot context)
    let inputs_without_memories: Vec<ConsolidationSourceInput> = matching
        .into_iter()
        .map(source_to_packet_input)
        .collect();
    let mut packet_without_memory = prepare_memory_consolidation_impl(inputs_without_memories)?;
    // Ensure no admitted memories and no hot context
    packet_without_memory.related_memories = vec![];
    packet_without_memory.current_hot_context = String::new();
    let prompt_without_memory = build_consolidation_prompt(&packet_without_memory)?;
    let without_memory_preview = clip(&prompt_without_memory.prompt, 500);

    Ok(ReplayComparisonReport {
        with_memory_preview,
        without_memory_preview,
        source_count: packet_without_memory.sources.len(),
        admitted_memory_count,
        hot_context_chars,
    })
}

/// Get procedure recall statistics - returns total, active count and top procedures by use_count
#[tauri::command]
pub fn get_procedure_recall_stats() -> Result<ProcedureRecallStats, String> {
    let db = open_memory_db()?;

    // Query all procedure memories
    let conn = db.conn.lock().unwrap();
    let sql = format!(
        "SELECT id, body, used_count, last_used_at
         FROM agent_memories
         WHERE layer = 'procedure'
         ORDER BY used_count DESC"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    // Collect all rows first to release the statement before dropping conn
    let raw_rows: Vec<Result<(i64, String, i64, Option<String>), _>> = stmt
        .query_map([], |row| {
            let last_used_at_str: Option<String> = row.get(3)?;
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                last_used_at_str,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect();

    drop(stmt);
    drop(conn);

    let mut procedures: Vec<ProcedureUsageDto> = Vec::new();
    let mut total_use_count: usize = 0;
    let mut active_count: usize = 0;

    for row_result in raw_rows {
        let (id, body, used_count, last_used_at_str) = row_result.map_err(|e| e.to_string())?;
        total_use_count += used_count as usize;
        active_count += 1;

        let body_preview = body.chars().take(50).collect::<String>();
        let last_used_at = parse_sqlite_time(last_used_at_str.as_deref()).unwrap_or(0);

        procedures.push(ProcedureUsageDto {
            memory_id: id.to_string(),
            body_preview,
            use_count: used_count as usize,
            last_used_at,
        });
    }

    let total_procedures = procedures.len();
    let top_procedures: Vec<ProcedureUsageDto> = procedures.into_iter().take(5).collect();

    Ok(ProcedureRecallStats {
        total_procedures,
        active_procedures: active_count,
        total_use_count,
        top_procedures,
    })
}

/// Increment the use_count for a procedure memory
#[tauri::command]
pub fn increment_procedure_use_count(memory_id: String) -> Result<i64, String> {
    let id = memory_id
        .trim()
        .parse::<i64>()
        .map_err(|error| format!("memory_id must be numeric: {}", error))?;

    let db = open_memory_db()?;
    let store = AgentMemoryStore::new(&db);

    // Verify the memory exists and has layer = 'procedure'
    let memory = store
        .get(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Memory not found: {}", memory_id))?;

    if memory.layer != "procedure" {
        return Err(format!(
            "Memory {} has layer '{}', expected 'procedure'",
            memory_id, memory.layer
        ));
    }

    // Increment use_count
    store.mark_used(id).map_err(|e| e.to_string())?;

    // Return the new use_count
    let updated_memory = store
        .get(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Memory not found after update: {}", memory_id))?;

    Ok(updated_memory.used_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_db_path(prefix: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let id = TEST_DB_COUNTER.fetch_add(1, Ordering::Relaxed);
        path.push(format!("{}_{}_{}.db", prefix, std::process::id(), id));
        let _ = std::fs::remove_file(&path);
        path
    }

    fn open_test_db_at(path: &PathBuf) -> Database {
        let db = Database::new(path).unwrap();
        run_migrations(&db).unwrap();
        db
    }

    fn test_db() -> Database {
        let path = test_db_path("cozmio_memory_consolidation");
        open_test_db_at(&path)
    }

    fn packet() -> ConsolidationPacket {
        prepare_memory_consolidation_impl(vec![
            ConsolidationSourceInput {
                source_kind: "model_output".to_string(),
                source_text: "Model said: use the existing command style.".to_string(),
                source_ref: Some("model:1".to_string()),
                timestamp: Some("2026-05-07T00:00:00Z".to_string()),
            },
            ConsolidationSourceInput {
                source_kind: "executor_result".to_string(),
                source_text: "Executor result: tests passed.".to_string(),
                source_ref: Some("executor:1".to_string()),
                timestamp: Some("2026-05-07T00:00:01Z".to_string()),
            },
        ])
        .unwrap()
    }

    #[test]
    fn production_hot_context_loader_reads_known_human_context_file() {
        let path = test_db_path("cozmio_human_context").with_file_name(format!(
            "cozmio_human_context_{}_{}.md",
            std::process::id(),
            TEST_DB_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, "\nStable hot context from disk.\n").unwrap();

        let loaded = load_current_hot_context_from_path(&path);

        assert_eq!(loaded, "Stable hot context from disk.");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn production_hot_context_loader_marks_missing_source_unavailable() {
        let path = test_db_path("cozmio_missing_human_context").with_file_name(format!(
            "cozmio_missing_human_context_{}_{}.md",
            std::process::id(),
            TEST_DB_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_file(&path);

        let loaded = load_current_hot_context_from_path(&path);

        assert!(loaded.starts_with("[hot_context_unavailable: human_context.md not found at "));
        assert!(loaded.contains(&path.display().to_string()));
    }

    #[test]
    fn memory_operation_validation_rejects_invalid_operation_type() {
        let packet = packet();
        let input = MemoryOperationInput {
            operation_type: "infer_intent".to_string(),
            layer: Some("episode".to_string()),
            body: Some("Preserve this text.".to_string()),
            source_refs: vec!["model:1".to_string()],
            target_memory_id: None,
            producer: None,
        };

        let err = validate_memory_operation(&input, &packet).unwrap_err();
        assert!(err.starts_with("Unsupported operation_type:"));
    }

    #[test]
    fn memory_operation_validation_rejects_mismatched_required_layers() {
        let packet = packet();
        let cases = [
            (REMEMBER_EPISODE, "reflection", "episode"),
            (REMEMBER_REFLECTION, "episode", "reflection"),
            (REMEMBER_SKILL, "episode", "procedure"),
            (UPDATE_HOT_CONTEXT, "episode", "hot_context_proposal"),
        ];

        for (operation_type, provided_layer, required_layer) in cases {
            let input = MemoryOperationInput {
                operation_type: operation_type.to_string(),
                layer: Some(provided_layer.to_string()),
                body: Some("Body text.".to_string()),
                source_refs: vec!["model:1".to_string()],
                target_memory_id: None,
                producer: None,
            };

            let err = validate_memory_operation(&input, &packet).unwrap_err();
            assert!(
                err.contains(&format!(
                    "operation_type {operation_type} requires layer {required_layer}, got {provided_layer}"
                )),
                "unexpected error for {operation_type}: {err}"
            );
        }
    }

    #[test]
    fn memory_operation_validation_defaults_required_layers_when_omitted() {
        let packet = packet();
        let cases = [
            (REMEMBER_EPISODE, "episode"),
            (REMEMBER_REFLECTION, "reflection"),
            (REMEMBER_SKILL, "procedure"),
            (UPDATE_HOT_CONTEXT, "hot_context_proposal"),
        ];

        for (operation_type, required_layer) in cases {
            let input = MemoryOperationInput {
                operation_type: operation_type.to_string(),
                layer: None,
                body: Some("Body text.".to_string()),
                source_refs: vec!["model:1".to_string()],
                target_memory_id: None,
                producer: None,
            };

            let (layer, _, _, _) = validate_memory_operation(&input, &packet).unwrap();
            assert_eq!(layer.as_deref(), Some(required_layer));
        }
    }

    #[test]
    fn apply_episode_creates_active_memory_preserving_body() {
        let db = test_db();
        let packet = packet();
        let input = MemoryOperationInput {
            operation_type: REMEMBER_EPISODE.to_string(),
            layer: Some("episode".to_string()),
            body: Some("Model said: use the existing command style.".to_string()),
            source_refs: vec!["model:1".to_string()],
            target_memory_id: None,
            producer: None,
        };

        let result = apply_memory_operation_impl(&db, &packet, input).unwrap();
        let memory = result.memory.unwrap();
        assert_eq!(memory.lifecycle, "active");
        assert_eq!(memory.layer, "episode");
        assert_eq!(memory.body, "Model said: use the existing command style.");
        assert_eq!(memory.source_refs, vec!["model:1".to_string()]);
        assert_eq!(memory.producer, DEFAULT_PRODUCER);
        assert_eq!(list_agent_memories_impl(&db, 10).unwrap().len(), 1);
    }

    #[test]
    fn apply_abstain_creates_operation_without_memory() {
        let db = test_db();
        let packet = packet();
        let input = MemoryOperationInput {
            operation_type: ABSTAIN.to_string(),
            layer: None,
            body: Some("Agent output: no memory operation is warranted.".to_string()),
            source_refs: vec![],
            target_memory_id: None,
            producer: None,
        };

        let result = apply_memory_operation_impl(&db, &packet, input).unwrap();
        assert!(result.memory.is_none());
        assert_eq!(result.operation.status, "accepted");
        assert!(list_agent_memories_impl(&db, 10).unwrap().is_empty());
    }

    #[test]
    fn public_apply_memory_operation_supersede_records_experience_source() {
        let db_path = test_db_path("cozmio_memory_command_supersede");
        let db = open_test_db_at(&db_path);
        let store = AgentMemoryStore::new(&db);
        let original_id = store
            .remember_memory(MemoryWrite {
                body: "Original command-facing memory.".to_string(),
                layer: "episode".to_string(),
                lifecycle: "active".to_string(),
                source_refs: vec!["model:1".to_string()],
                supersedes_id: None,
                producer: DEFAULT_PRODUCER.to_string(),
                expires_at: None,
            })
            .unwrap();
        drop(store);
        drop(db);
        let packet = packet();
        let open_path = db_path.clone();
        let result = apply_memory_operation_command_path(
            || Ok(open_test_db_at(&open_path)),
            packet,
            MemoryOperationInput {
                operation_type: REMOVE_OR_SUPERSEDE.to_string(),
                layer: Some("episode".to_string()),
                body: Some("Corrected command-facing memory.".to_string()),
                source_refs: vec!["model:1".to_string(), "executor:1".to_string()],
                target_memory_id: Some(original_id.to_string()),
                producer: Some("command-test".to_string()),
            },
        )
        .unwrap();

        let memory = result.memory.expect("expected successor memory");
        assert_eq!(memory.body, "Corrected command-facing memory.");
        assert_eq!(memory.supersedes, Some(original_id.to_string()));

        let verify_db = open_test_db_at(&db_path);
        let verify_store = AgentMemoryStore::new(&verify_db);
        assert_eq!(
            verify_store.get(original_id).unwrap().unwrap().lifecycle,
            "superseded"
        );
        let feedback = verify_store
            .recent_experience_sources(10)
            .unwrap()
            .into_iter()
            .find(|source| source.event_kind == "memory_superseded")
            .expect("expected command path memory_superseded source");
        assert_eq!(
            feedback.raw_ref.as_deref(),
            Some(format!("agent_memory:{original_id}").as_str())
        );
        assert!(feedback
            .factual_text
            .contains("remove_or_supersede operation superseded"));
        assert!(feedback.artifact_refs_json.contains("model:1"));
        assert!(feedback.artifact_refs_json.contains("executor:1"));
    }

    #[test]
    fn public_apply_memory_operation_reject_only_records_experience_source() {
        let db_path = test_db_path("cozmio_memory_command_reject");
        let db = open_test_db_at(&db_path);
        let store = AgentMemoryStore::new(&db);
        let original_id = store
            .remember_memory(MemoryWrite {
                body: "Command-facing memory to reject.".to_string(),
                layer: "episode".to_string(),
                lifecycle: "active".to_string(),
                source_refs: vec!["model:1".to_string()],
                supersedes_id: None,
                producer: DEFAULT_PRODUCER.to_string(),
                expires_at: None,
            })
            .unwrap();
        drop(store);
        drop(db);
        let packet = packet();
        let open_path = db_path.clone();
        let result = apply_memory_operation_command_path(
            || Ok(open_test_db_at(&open_path)),
            packet,
            MemoryOperationInput {
                operation_type: REMOVE_OR_SUPERSEDE.to_string(),
                layer: None,
                body: None,
                source_refs: vec!["model:1".to_string()],
                target_memory_id: Some(original_id.to_string()),
                producer: Some("command-test".to_string()),
            },
        )
        .unwrap();

        assert!(result.memory.is_none());
        let verify_db = open_test_db_at(&db_path);
        let verify_store = AgentMemoryStore::new(&verify_db);
        let original = verify_store.get(original_id).unwrap().unwrap();
        assert_eq!(original.lifecycle, "rejected");
        let feedback = verify_store
            .recent_experience_sources(10)
            .unwrap()
            .into_iter()
            .find(|source| source.event_kind == "memory_rejected")
            .expect("expected command path memory_rejected source");
        assert_eq!(
            feedback.raw_ref.as_deref(),
            Some(format!("agent_memory:{original_id}").as_str())
        );
        assert!(feedback
            .factual_text
            .contains("remove_or_supersede operation rejected"));
        assert!(feedback
            .factual_text
            .contains(&format!("memory_id={original_id}")));
    }
    #[test]
    fn apply_remove_or_supersede_records_feedback_and_creates_successor() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);
        let original_id = store
            .remember_memory(MemoryWrite {
                body: "Original memory text.".to_string(),
                layer: "episode".to_string(),
                lifecycle: "active".to_string(),
                source_refs: vec!["model:1".to_string()],
                supersedes_id: None,
                producer: DEFAULT_PRODUCER.to_string(),
                expires_at: None,
            })
            .unwrap();
        let packet = packet();
        let input = MemoryOperationInput {
            operation_type: REMOVE_OR_SUPERSEDE.to_string(),
            layer: Some("episode".to_string()),
            body: Some("Corrected memory text.".to_string()),
            source_refs: vec!["model:1".to_string(), "executor:1".to_string()],
            target_memory_id: Some(original_id.to_string()),
            producer: Some("test-consolidator".to_string()),
        };

        let result = apply_memory_operation_impl(&db, &packet, input).unwrap();
        let successor = result.memory.unwrap();
        assert_eq!(successor.body, "Corrected memory text.");
        assert_eq!(successor.layer, "episode");
        assert_eq!(successor.supersedes, Some(original_id.to_string()));
        assert_eq!(
            successor.source_refs,
            vec!["executor:1".to_string(), "model:1".to_string()]
        );
        assert_eq!(
            store.get(original_id).unwrap().unwrap().lifecycle,
            "superseded"
        );

        let recent = store.recent_experience_sources(10).unwrap();
        let feedback = recent
            .iter()
            .find(|source| source.event_kind == "memory_superseded")
            .expect("expected memory_superseded experience source");
        assert_eq!(
            feedback.raw_ref.as_deref(),
            Some(format!("agent_memory:{original_id}").as_str())
        );
        assert!(feedback
            .factual_text
            .contains("remove_or_supersede operation superseded"));
        assert!(feedback
            .factual_text
            .contains(&format!("memory_id={original_id}")));
        assert!(feedback.artifact_refs_json.contains("model:1"));
        assert!(feedback.artifact_refs_json.contains("executor:1"));
    }

    #[test]
    fn apply_remove_or_supersede_without_body_records_rejection_feedback() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);
        let original_id = store
            .remember_memory(MemoryWrite {
                body: "Memory to reject.".to_string(),
                layer: "episode".to_string(),
                lifecycle: "active".to_string(),
                source_refs: vec!["model:1".to_string()],
                supersedes_id: None,
                producer: DEFAULT_PRODUCER.to_string(),
                expires_at: None,
            })
            .unwrap();
        let packet = packet();
        let input = MemoryOperationInput {
            operation_type: REMOVE_OR_SUPERSEDE.to_string(),
            layer: None,
            body: None,
            source_refs: vec!["model:1".to_string()],
            target_memory_id: Some(original_id.to_string()),
            producer: Some("test-consolidator".to_string()),
        };

        let result = apply_memory_operation_impl(&db, &packet, input).unwrap();
        assert!(result.memory.is_none());
        let original = store.get(original_id).unwrap().unwrap();
        assert_eq!(original.lifecycle, "rejected");
        assert_eq!(
            original.rejected_reason.as_deref(),
            Some("remove_or_supersede operation without successor body")
        );

        let recent = store.recent_experience_sources(10).unwrap();
        let feedback = recent
            .iter()
            .find(|source| source.event_kind == "memory_rejected")
            .expect("expected memory_rejected experience source");
        assert_eq!(
            feedback.raw_ref.as_deref(),
            Some(format!("agent_memory:{original_id}").as_str())
        );
        assert!(feedback
            .factual_text
            .contains("remove_or_supersede operation rejected"));
        assert!(feedback
            .factual_text
            .contains(&format!("memory_id={original_id}")));
    }

    #[test]
    fn packet_builder_includes_recent_sources_related_memories_and_hot_context() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);
        store
            .insert_experience_source(&ExperienceSource {
                source_ref: "experience:trace-1:popup_confirmed:abc".to_string(),
                timestamp: Some("2026-05-07T00:00:00Z".to_string()),
                trace_id: Some("trace-1".to_string()),
                session_id: None,
                event_kind: "popup_confirmed".to_string(),
                window_title: Some("Cozmio memory".to_string()),
                process_name: Some("Code.exe".to_string()),
                factual_text: "Popup was confirmed for Cozmio memory task.".to_string(),
                raw_ref: None,
                artifact_refs_json: "[]".to_string(),
            })
            .unwrap();
        store
            .remember_memory(MemoryWrite {
                body: "Cozmio memory task used targeted tests.".to_string(),
                layer: "episode".to_string(),
                lifecycle: "active".to_string(),
                source_refs: vec!["experience:trace-1:popup_confirmed:abc".to_string()],
                supersedes_id: None,
                producer: DEFAULT_PRODUCER.to_string(),
                expires_at: None,
            })
            .unwrap();

        let packet =
            build_consolidation_packet_from_store(&store, 10, "stable hot context".to_string())
                .unwrap();
        assert_eq!(packet.sources.len(), 1);
        assert_eq!(
            packet.sources[0].source_ref,
            "experience:trace-1:popup_confirmed:abc"
        );
        assert!(packet.sources[0]
            .source_text
            .contains("factual_text=Popup was confirmed"));
        assert_eq!(packet.current_hot_context, "stable hot context");
        assert_eq!(packet.related_memories.len(), 1);
        assert_eq!(packet.related_memories[0].layer, "episode");
    }

    #[test]
    fn consolidation_prompt_contains_required_boundaries_and_tool_contract() {
        let prompt = build_consolidation_prompt(&packet()).unwrap().prompt;
        assert!(prompt.contains("Do not invent motives"));
        assert!(prompt.contains("Preserve uncertainty"));
        assert!(prompt.contains("Write only if future Cozmio behavior should change"));
        assert!(prompt.contains("Cite source_refs"));
        assert!(prompt.contains("Prefer feedback-bearing experience"));
        assert!(prompt.contains("Avoid stable conclusions from screenshot-only evidence"));
        assert!(prompt.contains("update_hot_context"));
        assert!(prompt.contains("hot_context_proposal only"));
        assert!(prompt.contains("FACTUAL_PACKET"));
    }

    #[test]
    fn manual_consolidation_run_applies_multiple_operations_and_records_run() {
        let db = test_db();
        let packet = packet();
        let run = run_manual_consolidation_with_packet(
            &db,
            "manual_dev",
            packet,
            vec![
                MemoryOperationInput {
                    operation_type: REMEMBER_EPISODE.to_string(),
                    layer: Some("episode".to_string()),
                    body: Some("Model suggested existing command style.".to_string()),
                    source_refs: vec!["model:1".to_string()],
                    target_memory_id: None,
                    producer: Some("test-consolidator".to_string()),
                },
                MemoryOperationInput {
                    operation_type: REMEMBER_SKILL.to_string(),
                    layer: None,
                    body: Some(
                        "When command style matters, inspect existing command-facing tests first."
                            .to_string(),
                    ),
                    source_refs: vec!["model:1".to_string(), "executor:1".to_string()],
                    target_memory_id: None,
                    producer: Some("test-consolidator".to_string()),
                },
            ],
        )
        .unwrap();

        assert_eq!(run.status, "completed");
        assert_eq!(run.operations.len(), 2);
        let memories = list_agent_memories_impl(&db, 10).unwrap();
        assert_eq!(memories.len(), 2);
        assert!(memories.iter().any(|memory| memory.layer == "episode"));
        assert!(memories.iter().any(|memory| memory.layer == "procedure"));
    }

    #[test]
    fn manual_consolidation_run_records_abstain_without_memory() {
        let db = test_db();
        let run = run_manual_consolidation_with_packet(
            &db,
            "manual_dev",
            packet(),
            vec![MemoryOperationInput {
                operation_type: ABSTAIN.to_string(),
                layer: None,
                body: Some("No durable behavior change should be written.".to_string()),
                source_refs: vec![],
                target_memory_id: None,
                producer: Some("test-consolidator".to_string()),
            }],
        )
        .unwrap();

        assert_eq!(run.status, "completed");
        assert_eq!(run.operations[0].operation.operation_type, ABSTAIN);
        assert!(run.operations[0].memory.is_none());
        assert!(list_agent_memories_impl(&db, 10).unwrap().is_empty());
    }

    #[test]
    fn inspector_snapshot_exposes_sources_runs_operations_and_memory_rows() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);
        store
            .insert_experience_source(&ExperienceSource {
                source_ref: "experience:test:popup_confirmed".to_string(),
                timestamp: Some("2026-05-07 00:00:00".to_string()),
                trace_id: Some("trace-inspector".to_string()),
                session_id: None,
                event_kind: "popup_confirmed".to_string(),
                window_title: Some("Cozmio".to_string()),
                process_name: Some("Code.exe".to_string()),
                factual_text: "popup_confirmed trace_id=trace-inspector".to_string(),
                raw_ref: None,
                artifact_refs_json: "[]".to_string(),
            })
            .unwrap();

        let run = run_manual_consolidation_with_packet(
            &db,
            "manual_dev",
            packet(),
            vec![MemoryOperationInput {
                operation_type: REMEMBER_REFLECTION.to_string(),
                layer: None,
                body: Some("Agent-written memory body with cited facts.".to_string()),
                source_refs: vec!["model:1".to_string()],
                target_memory_id: None,
                producer: Some("test-consolidator".to_string()),
            }],
        )
        .unwrap();

        let memory_id = run.operations[0].memory.as_ref().unwrap().memory_id.clone();
        reject_agent_memory_impl(&db, &memory_id, Some("test rejection")).unwrap();

        let snapshot = get_memory_inspector_snapshot_impl(&db, 20).unwrap();

        assert!(snapshot
            .recent_experience_sources
            .iter()
            .any(|source| source.source_ref == "experience:test:popup_confirmed"));
        assert!(snapshot
            .recent_consolidation_runs
            .iter()
            .any(|candidate| candidate.run_id == run.run_id && candidate.packet_source_count == 2));
        assert!(snapshot
            .recent_operations
            .iter()
            .any(|operation| operation.operation_type == "rejected"));
        assert!(snapshot
            .memories
            .iter()
            .any(|memory| memory.memory_id == memory_id && memory.lifecycle == "rejected"));
    }

    #[test]
    fn duplicate_exact_active_memory_is_not_appended_repeatedly() {
        let db = test_db();
        let packet = packet();
        let input = MemoryOperationInput {
            operation_type: REMEMBER_REFLECTION.to_string(),
            layer: Some("reflection".to_string()),
            body: Some("Use the same command-facing test path next time.".to_string()),
            source_refs: vec!["model:1".to_string(), "executor:1".to_string()],
            target_memory_id: None,
            producer: Some("test-consolidator".to_string()),
        };

        let first = apply_memory_operation_impl(&db, &packet, input.clone()).unwrap();
        let second = apply_memory_operation_impl(&db, &packet, input).unwrap();
        assert_eq!(
            first.memory.unwrap().memory_id,
            second.memory.unwrap().memory_id
        );
        assert_eq!(list_agent_memories_impl(&db, 10).unwrap().len(), 1);
    }

    #[test]
    fn hot_context_update_creates_draft_proposal_memory() {
        let db = test_db();
        let result = apply_memory_operation_impl(
            &db,
            &packet(),
            MemoryOperationInput {
                operation_type: UPDATE_HOT_CONTEXT.to_string(),
                layer: None,
                body: Some("Potential stable context update; requires acceptance.".to_string()),
                source_refs: vec!["executor:1".to_string()],
                target_memory_id: None,
                producer: Some("test-consolidator".to_string()),
            },
        )
        .unwrap();

        let memory = result.memory.unwrap();
        assert_eq!(memory.layer, "hot_context_proposal");
        assert_eq!(memory.lifecycle, "active");
        assert!(memory.body.contains("requires acceptance"));
    }

    #[test]
    fn automatic_flywheel_gate_disables_experience_and_model_memory_writes() {
        let db = test_db();
        let store = AgentMemoryStore::new(&db);

        let source = crate::experience_recorder::record_experience_with_store_if_enabled(
            &store,
            crate::experience_recorder::ExperienceEventInput::new(
                "popup_model_output",
                "Popup model output captured for gate test.",
            ),
            false,
        )
        .unwrap();
        assert_eq!(source, None);

        let memory_id = auto_remember_model_output_with_db_if_enabled(
            &db,
            false,
            "Cozmio - Claude Code",
            "Code.exe",
            Some("trace-disabled"),
            "This model output should not be persisted automatically.",
            1002,
        )
        .unwrap();
        assert_eq!(memory_id, None);

        assert!(store.recent_experience_sources(10).unwrap().is_empty());
        assert!(list_agent_memories_impl(&db, 10).unwrap().is_empty());
    }

    #[test]
    fn auto_remember_model_output_no_longer_writes_direct_episode_memory() {
        let db = test_db();
        let empty = auto_remember_model_output_with_db(
            &db,
            "Cozmio - Claude Code",
            "Code.exe",
            Some("trace-empty"),
            "   ",
            1000,
        )
        .unwrap();
        assert_eq!(empty, None);

        let non_empty = auto_remember_model_output_with_db(
            &db,
            "Cozmio - Claude Code",
            "Code.exe",
            Some("trace-1"),
            "继续推进自动记忆飞轮。",
            1001,
        )
        .unwrap();
        assert_eq!(non_empty, None);
        assert!(list_agent_memories_impl(&db, 10).unwrap().is_empty());
    }
}
