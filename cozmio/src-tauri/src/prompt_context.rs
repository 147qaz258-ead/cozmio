use crate::logging::{ActionLogger, ActionRecord};
use crate::window_monitor::ProcessContext;
use cozmio_memory::{AgentMemory, AgentMemoryStore};
use std::collections::BTreeSet;

const MAX_RECENT_RECORDS: usize = 18;
const MAX_INCLUDED_RECORDS: usize = 6;
const MAX_CONTEXT_CHARS: usize = 2400;
const MAX_FIELD_CHARS: usize = 180;
const ACTION_LOG_TAIL_BYTES: u64 = 64 * 1024;
const MAX_RECALLED_MEMORIES: usize = 4;
const MAX_MEMORY_CHARS: usize = 260;
const MAX_RECENT_FEEDBACK_FACTS: usize = 4;
const MAX_HOT_CONTEXT_CHARS: usize = 360;
const MAX_FEEDBACK_ADMISSION_BLOCK_CHARS: usize = 520;
const MAX_MEMORY_ADMISSION_BLOCK_CHARS: usize = 720;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecallAdmissionResult {
    budget: RecallAdmissionBudget,
    hot_context: HotStableContextAdmission,
    feedback_facts: Vec<FeedbackFactAdmission>,
    memories: Vec<MemoryAdmission>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecallAdmissionBudget {
    max_memories: usize,
    max_recent_feedback_facts: usize,
    max_hot_context_chars: usize,
    max_memory_chars: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HotStableContextAdmission {
    source_type: &'static str,
    source_ref: &'static str,
    chars: usize,
    body: String,
    mechanical_reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FeedbackFactAdmission {
    source_type: &'static str,
    source_ref: String,
    event_kind: String,
    age_seconds: i64,
    factual_text: String,
    mechanical_score: usize,
    mechanical_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MemoryAdmission {
    source_type: &'static str,
    memory_id: i64,
    layer: String,
    last_used_at: Option<String>,
    source_refs: Vec<String>,
    mechanical_score: usize,
    mechanical_reason: String,
    body: String,
}

pub fn build_popup_context(
    logger: &ActionLogger,
    window_title: &str,
    process_name: &str,
    process_context: &ProcessContext,
) -> String {
    let recent = logger
        .get_recent_tail(MAX_RECENT_RECORDS, ACTION_LOG_TAIL_BYTES)
        .unwrap_or_else(|error| {
            log::warn!("Failed to load recent action history for popup context: {error}");
            Vec::new()
        });
    let hot_context = load_hot_stable_context();
    let db = crate::memory_consolidation::open_memory_db().ok();
    let store = db.as_ref().map(AgentMemoryStore::new);

    build_popup_context_with_recent_and_store(
        &recent,
        window_title,
        process_name,
        process_context,
        &hot_context,
        store.as_ref(),
    )
}

fn build_popup_context_with_recent_and_store(
    recent: &[ActionRecord],
    window_title: &str,
    process_name: &str,
    process_context: &ProcessContext,
    hot_context: &str,
    store: Option<&AgentMemoryStore<'_>>,
) -> String {
    let memory_query = build_memory_query(window_title, process_name, recent);
    let admission = build_recall_admission(&memory_query, recent, hot_context, store);

    let mut protected_lines = vec![
        format_process_context(process_context),
        format!(
            "current_window: title=\"{}\", process=\"{}\"",
            clip(window_title, 120),
            clip(process_name, 80)
        ),
    ];

    if let Some(channel) = format_hot_stable_context_channel(&admission.hot_context) {
        protected_lines.push(channel);
    }
    if let Some(block) = format_recall_admission_block(&admission) {
        protected_lines.push(block);
    }

    let mut context = clip(&protected_lines.join("\n"), MAX_CONTEXT_CHARS);
    let remaining_chars = MAX_CONTEXT_CHARS
        .saturating_sub(context.chars().count())
        .saturating_sub(1);

    if let Some(tail) = format_action_log_tail(recent, remaining_chars) {
        if !context.is_empty() {
            context.push('\n');
        }
        context.push_str(&tail);
    }

    clip(&context, MAX_CONTEXT_CHARS)
}

fn format_process_context(context: &ProcessContext) -> String {
    format!(
        "process_context_facts: stay_duration_seconds={}, switches_last_minute={}, rapid_switch_intervals_under_5s={}, foreground_changed_within_5s={}",
        context.stay_duration_seconds,
        context.switches_in_last_minute,
        context.rapid_switch_intervals_under_5s,
        context.foreground_changed_within_5s
    )
}

fn compact_record(record: &ActionRecord) -> String {
    let age_seconds = chrono::Utc::now()
        .timestamp()
        .saturating_sub(record.timestamp)
        .max(0);
    let mut parts = vec![
        format!("timestamp={}", record.timestamp),
        format!("age_seconds={}", age_seconds),
        format!("window=\"{}\"", clip(&record.window_title, 80)),
        format!("status_label=\"{}\"", clip(&record.status_label, 40)),
        format!("system_action={}", record.system_action),
    ];

    if let Some(feedback) = non_empty_opt(record.user_feedback.as_deref()) {
        parts.push(format!("feedback=\"{}\"", clip(feedback, MAX_FIELD_CHARS)));
    }
    if let Some(content) = non_empty_opt(record.content_text.as_deref()) {
        parts.push(format!("content=\"{}\"", clip(content, MAX_FIELD_CHARS)));
    } else if !record.model_text.trim().is_empty() {
        parts.push(format!(
            "raw_model_text=\"{}\"",
            clip(&record.model_text, MAX_FIELD_CHARS)
        ));
    }
    if let Some(result) = non_empty_opt(record.result_text.as_deref()) {
        parts.push(format!("result=\"{}\"", clip(result, MAX_FIELD_CHARS)));
    }
    if let Some(error) = non_empty_opt(record.error_text.as_deref()) {
        parts.push(format!("error=\"{}\"", clip(error, MAX_FIELD_CHARS)));
    }

    parts.join(", ")
}

fn format_action_log_tail(recent: &[ActionRecord], max_chars: usize) -> Option<String> {
    if recent.is_empty() || max_chars == 0 {
        return None;
    }

    let mut lines = vec![String::from("action_log_tail:")];
    for record in recent.iter().take(MAX_INCLUDED_RECORDS) {
        lines.push(format!("- {}", compact_record(record)));
    }

    let tail = lines.join("\n");
    if tail.chars().count() <= max_chars {
        return Some(tail);
    }

    let header = "action_log_tail:\n";
    if max_chars <= header.chars().count() {
        return None;
    }

    Some(format!(
        "{}{}",
        header,
        clip(&tail[header.len()..], max_chars - header.chars().count())
    ))
}

fn non_empty_opt(value: Option<&str>) -> Option<&str> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn build_memory_query(window_title: &str, process_name: &str, recent: &[ActionRecord]) -> String {
    let mut parts = vec![window_title.to_string(), process_name.to_string()];
    for record in recent.iter().take(3) {
        parts.push(record.window_title.clone());
        if let Some(content) = non_empty_opt(record.content_text.as_deref()) {
            parts.push(content.to_string());
        }
        if let Some(feedback) = non_empty_opt(record.user_feedback.as_deref()) {
            parts.push(feedback.to_string());
        }
    }
    parts.join(" ")
}

fn build_recall_admission(
    query: &str,
    recent: &[ActionRecord],
    hot_context: &str,
    store: Option<&AgentMemoryStore<'_>>,
) -> RecallAdmissionResult {
    let budget = RecallAdmissionBudget {
        max_memories: MAX_RECALLED_MEMORIES,
        max_recent_feedback_facts: MAX_RECENT_FEEDBACK_FACTS,
        max_hot_context_chars: MAX_HOT_CONTEXT_CHARS,
        max_memory_chars: MAX_MEMORY_CHARS,
    };
    let hot_context = HotStableContextAdmission {
        source_type: "hot_stable_context",
        source_ref: "human_context.md",
        chars: hot_context.chars().count(),
        body: clip(hot_context.trim(), budget.max_hot_context_chars),
        mechanical_reason: "always_admitted_as_separate_channel",
    };
    let feedback_facts = recent_feedback_facts(recent, budget.max_recent_feedback_facts);
    let memories = store
        .map(|store| admitted_memories(store, query, budget.max_memories))
        .unwrap_or_default();

    RecallAdmissionResult {
        budget,
        hot_context,
        feedback_facts,
        memories,
    }
}

fn admitted_memories(
    store: &AgentMemoryStore<'_>,
    query: &str,
    limit: usize,
) -> Vec<MemoryAdmission> {
    let memories = match store.recall_active_by_text(query, limit) {
        Ok(memories) => memories,
        Err(error) => {
            log::warn!("Agent memory recall admission failed: {}", error);
            return Vec::new();
        }
    };

    memories
        .into_iter()
        .filter_map(|memory| memory_admission(store, query, memory))
        .collect()
}

fn memory_admission(
    store: &AgentMemoryStore<'_>,
    query: &str,
    memory: AgentMemory,
) -> Option<MemoryAdmission> {
    let memory_id = memory.id.unwrap_or_default();
    let refreshed_last_used_at = if memory_id > 0 {
        if let Err(error) = store.mark_used(memory_id) {
            log::debug!("Failed to mark admitted memory as used: {}", error);
        }
        store
            .get(memory_id)
            .ok()
            .flatten()
            .and_then(|updated| updated.last_used_at)
    } else {
        memory.last_used_at.clone()
    };
    let matched_terms = matched_terms(&memory.body, query);
    let mechanical_score = matched_terms.len();
    let source_refs = serde_json::from_str::<Vec<String>>(&memory.source_refs_json)
        .unwrap_or_else(|_| Vec::new());
    Some(MemoryAdmission {
        source_type: "agent_memory",
        memory_id,
        layer: memory.layer,
        last_used_at: refreshed_last_used_at,
        source_refs,
        mechanical_score,
        mechanical_reason: format!(
            "matched_query_terms=[{}]",
            matched_terms.into_iter().collect::<Vec<_>>().join("|")
        ),
        body: clip(&memory.body, MAX_MEMORY_CHARS),
    })
}

fn recent_feedback_facts(recent: &[ActionRecord], limit: usize) -> Vec<FeedbackFactAdmission> {
    recent
        .iter()
        .filter_map(feedback_fact)
        .take(limit)
        .collect()
}

fn feedback_fact(record: &ActionRecord) -> Option<FeedbackFactAdmission> {
    let mut factual_parts = Vec::new();
    let mut reasons = Vec::new();

    if let Some(feedback) = non_empty_opt(record.user_feedback.as_deref()) {
        factual_parts.push(format!("feedback=\"{}\"", clip(feedback, MAX_FIELD_CHARS)));
        reasons.push("record_has_user_feedback");
    }
    if let Some(result) = non_empty_opt(record.result_text.as_deref()) {
        factual_parts.push(format!("result=\"{}\"", clip(result, MAX_FIELD_CHARS)));
        reasons.push("record_has_executor_result");
    }
    if let Some(error) = non_empty_opt(record.error_text.as_deref()) {
        factual_parts.push(format!("error=\"{}\"", clip(error, MAX_FIELD_CHARS)));
        reasons.push("record_has_error");
    }
    if is_feedback_action(&record.system_action) {
        factual_parts.push(format!("action={}", record.system_action));
        reasons.push("record_has_feedback_lifecycle_action");
    }

    if factual_parts.is_empty() {
        return None;
    }

    let age_seconds = chrono::Utc::now()
        .timestamp()
        .saturating_sub(record.timestamp)
        .max(0);
    Some(FeedbackFactAdmission {
        source_type: "recent_feedback_fact",
        source_ref: action_record_source_ref(record),
        event_kind: record.system_action.clone(),
        age_seconds,
        factual_text: factual_parts.join(", "),
        mechanical_score: reasons.len(),
        mechanical_reason: reasons.join("+"),
    })
}

fn is_feedback_action(action: &str) -> bool {
    matches!(
        action,
        "confirmed"
            | "cancelled"
            | "canceled"
            | "dismissed"
            | "expired"
            | "awaiting-confirmation"
            | "completed"
            | "failed"
            | "interrupted"
    )
}

fn action_record_source_ref(record: &ActionRecord) -> String {
    format!(
        "action_log:{}:{}",
        record.timestamp,
        record.trace_id.as_deref().unwrap_or("no-trace")
    )
}

fn format_recall_admission_block(admission: &RecallAdmissionResult) -> Option<String> {
    let has_feedback = !admission.feedback_facts.is_empty();
    let has_memory = !admission.memories.is_empty();
    if !has_feedback && !has_memory {
        return None;
    }

    let mut lines = vec![
        String::from("recall_admission:"),
        format!(
            "- budget: max_memories={}, max_recent_feedback_facts={}, max_memory_chars={}, max_feedback_admission_chars={}, max_memory_admission_chars={}",
            admission.budget.max_memories,
            admission.budget.max_recent_feedback_facts,
            admission.budget.max_memory_chars,
            MAX_FEEDBACK_ADMISSION_BLOCK_CHARS,
            MAX_MEMORY_ADMISSION_BLOCK_CHARS
        ),
    ];

    if !admission.feedback_facts.is_empty() {
        lines.push(format_feedback_admission_section(
            &admission.feedback_facts,
            MAX_FEEDBACK_ADMISSION_BLOCK_CHARS,
        ));
    }

    if !admission.memories.is_empty() {
        lines.push(format_memory_admission_section(
            &admission.memories,
            MAX_MEMORY_ADMISSION_BLOCK_CHARS,
        ));
    }

    Some(lines.join("\n"))
}

fn format_feedback_admission_section(facts: &[FeedbackFactAdmission], max_chars: usize) -> String {
    let mut lines = vec![String::from("recent_feedback_facts:")];
    for fact in facts {
        lines.push(format!(
            "- source_type={}, source_ref={}, event_kind={}, age_seconds={}, mechanical_score={}, mechanical_reason={}, factual_text=\"{}\"",
            fact.source_type,
            fact.source_ref,
            fact.event_kind,
            fact.age_seconds,
            fact.mechanical_score,
            fact.mechanical_reason,
            clip(&fact.factual_text, MAX_FIELD_CHARS)
        ));
    }
    clip(&lines.join("\n"), max_chars)
}

fn format_memory_admission_section(memories: &[MemoryAdmission], max_chars: usize) -> String {
    let mut lines = vec![String::from("admitted_memories:")];
    for memory in memories {
        lines.push(format!(
            "- source_type={}, memory_id={}, layer={}, last_used_at={}, source_refs={}, mechanical_score={}, mechanical_reason={}, body=\"{}\"",
            memory.source_type,
            memory.memory_id,
            memory.layer,
            memory.last_used_at.as_deref().unwrap_or("none"),
            format_source_refs(&memory.source_refs),
            memory.mechanical_score,
            memory.mechanical_reason,
            clip(&memory.body, MAX_MEMORY_CHARS)
        ));
    }
    clip(&lines.join("\n"), max_chars)
}

fn format_hot_stable_context_channel(admission: &HotStableContextAdmission) -> Option<String> {
    if admission.body.trim().is_empty() {
        return None;
    }

    Some(format!(
        "hot_stable_context:\n- source_type={}, source_ref={}, budget_chars={}, chars={}, mechanical_reason={}, body=\"{}\"",
        admission.source_type,
        admission.source_ref,
        MAX_HOT_CONTEXT_CHARS,
        admission.chars,
        admission.mechanical_reason,
        clip(&admission.body, MAX_HOT_CONTEXT_CHARS)
    ))
}

fn format_source_refs(source_refs: &[String]) -> String {
    if source_refs.is_empty() {
        return "[]".to_string();
    }
    format!("[{}]", source_refs.join("|"))
}

fn load_hot_stable_context() -> String {
    let Some(base) = dirs::data_local_dir() else {
        return String::new();
    };
    let path = base.join("cozmio").join("human_context.md");
    std::fs::read_to_string(path).unwrap_or_default()
}

fn matched_terms(body: &str, query: &str) -> BTreeSet<String> {
    let body = body.to_lowercase();
    recall_terms(query)
        .into_iter()
        .filter(|term| body.contains(term.as_str()))
        .collect()
}

fn recall_terms(text: &str) -> BTreeSet<String> {
    text.split(|ch: char| !(ch.is_alphanumeric() || ch == '_' || ch as u32 > 0x7f))
        .map(str::trim)
        .filter(|term| term.chars().count() >= 2)
        .map(str::to_lowercase)
        .collect()
}

fn clip(value: &str, max_chars: usize) -> String {
    let mut clipped: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        clipped.push_str("...");
    }
    clipped.replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::ActionRecord;
    use cozmio_memory::{run_migrations, Database, MemoryWrite};
    use std::fs;

    fn test_logger(name: &str) -> ActionLogger {
        let path = std::env::temp_dir()
            .join(format!("cozmio_prompt_context_{name}"))
            .join("action_log.jsonl");
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
            fs::create_dir_all(parent).unwrap();
        }
        ActionLogger::with_path(path)
    }

    fn record(window_title: &str, action: &str, feedback: Option<&str>) -> ActionRecord {
        ActionRecord {
            timestamp: 1000,
            trace_id: Some(String::from("trace")),
            session_id: None,
            window_title: window_title.to_string(),
            judgment: String::from("CONTINUE"),
            model_text: String::from("帮你继续推进 Cozmio 弹窗质量"),
            status_label: String::from("CONTINUE"),
            confidence_score: 1.0,
            grounds: String::from("test"),
            system_action: action.to_string(),
            content_text: Some(String::from("项目迭代建议")),
            result_text: None,
            error_text: None,
            user_feedback: feedback.map(str::to_string),
            model_name: None,
            captured_at: None,
            call_started_at: None,
            call_duration_ms: None,
        }
    }

    fn long_record(index: usize) -> ActionRecord {
        let mut record = record(
            &format!("Very long window title for prompt clipping regression {index}"),
            "completed",
            Some("confirmed"),
        );
        record.timestamp = 1000 + index as i64;
        record.trace_id = Some(format!("trace-{index}"));
        record.content_text = Some("x".repeat(900));
        record.result_text = Some("y".repeat(900));
        record
    }

    fn test_memory_db(name: &str) -> Database {
        let path = std::env::temp_dir().join(format!(
            "cozmio_prompt_context_memory_{name}_{}.db",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        let db = Database::new(&path).unwrap();
        run_migrations(&db).unwrap();
        db
    }

    #[test]
    fn includes_process_and_recent_feedback() {
        let logger = test_logger("feedback");
        logger
            .log(record(
                "Cozmio - Claude Code",
                "dismissed",
                Some("ui_closed"),
            ))
            .unwrap();

        let context = build_popup_context(
            &logger,
            "Cozmio - Claude Code",
            "Code.exe",
            &ProcessContext {
                stay_duration_seconds: 42,
                switches_in_last_minute: 2,
                rapid_switch_intervals_under_5s: 0,
                foreground_changed_within_5s: false,
            },
        );

        assert!(context.contains("stay_duration_seconds=42"));
        assert!(context.contains("feedback=\"ui_closed\""));
        assert!(context.contains("timestamp=1000"));
        assert!(!context.contains("检索线索"));
        assert!(!context.contains("弹窗策略"));
    }

    #[test]
    fn recall_admission_includes_hot_context_feedback_and_memory_debug_metadata() {
        let logger = test_logger("admission");
        logger
            .log(record(
                "Cozmio - Claude Code",
                "dismissed",
                Some("ui_closed"),
            ))
            .unwrap();
        let recent = logger.get_recent_tail(10, ACTION_LOG_TAIL_BYTES).unwrap();
        let db = test_memory_db("admission");
        let store = AgentMemoryStore::new(&db);
        store
            .remember_memory(MemoryWrite {
                body: String::from(
                    "Cozmio popup memory flywheel work should preserve source refs and debug metadata.",
                ),
                layer: String::from("reflection"),
                lifecycle: String::from("active"),
                source_refs: vec![String::from("experience:trace-1:popup_dismissed")],
                supersedes_id: None,
                producer: String::from("test-consolidator"),
                expires_at: None,
            })
            .unwrap();

        let context = build_popup_context_with_recent_and_store(
            &recent,
            "Cozmio - Claude Code",
            "Code.exe",
            &ProcessContext {
                stay_duration_seconds: 42,
                switches_in_last_minute: 2,
                rapid_switch_intervals_under_5s: 0,
                foreground_changed_within_5s: false,
            },
            "用户长期偏好：不要让代码硬编码语义，记忆必须可追溯。",
            Some(&store),
        );

        assert!(context.contains("recall_admission:"));
        assert!(context.contains("budget: max_memories="));
        assert!(context.contains("source_type=hot_stable_context"));
        assert!(context.contains("human_context.md"));
        assert!(context.contains("用户长期偏好"));
        assert!(context.contains("source_type=recent_feedback_fact"));
        assert!(context.contains(
            "mechanical_reason=record_has_user_feedback+record_has_feedback_lifecycle_action"
        ));
        assert!(context.contains("source_type=agent_memory"));
        assert!(context.contains("layer=reflection"));
        assert!(context.contains("source_refs=[experience:trace-1:popup_dismissed]"));
        assert!(context.contains("mechanical_score="));
        assert!(context.contains("matched_query_terms="));
        assert!(context.contains("last_used_at="));
    }

    #[test]
    fn recall_admission_excludes_rejected_memory_and_marks_admitted_memory_used() {
        let db = test_memory_db("used");
        let store = AgentMemoryStore::new(&db);
        let active = store
            .remember_memory(MemoryWrite {
                body: String::from("Cozmio memory flywheel active recall metadata."),
                layer: String::from("episode"),
                lifecycle: String::from("active"),
                source_refs: vec![String::from("experience:trace-2:popup_confirmed")],
                supersedes_id: None,
                producer: String::from("test-consolidator"),
                expires_at: None,
            })
            .unwrap();
        let rejected = store
            .remember_memory(MemoryWrite {
                body: String::from("Cozmio memory flywheel rejected recall metadata."),
                layer: String::from("episode"),
                lifecycle: String::from("active"),
                source_refs: vec![String::from("experience:trace-3:popup_cancelled")],
                supersedes_id: None,
                producer: String::from("test-consolidator"),
                expires_at: None,
            })
            .unwrap();
        store
            .reject_with_reason(rejected, Some("test rejection"), "test")
            .unwrap();

        let context = build_popup_context_with_recent_and_store(
            &[],
            "Cozmio memory flywheel",
            "Code.exe",
            &ProcessContext {
                stay_duration_seconds: 1,
                switches_in_last_minute: 0,
                rapid_switch_intervals_under_5s: 0,
                foreground_changed_within_5s: false,
            },
            "",
            Some(&store),
        );

        assert!(context.contains(&format!("memory_id={active}")));
        assert!(!context.contains(&format!("memory_id={rejected}")));
        assert!(!context.contains(&format!(
            "memory_id={active}, layer=episode, last_used_at=none"
        )));
        assert_eq!(store.get(active).unwrap().unwrap().used_count, 1);
    }

    #[test]
    fn recall_admission_survives_action_log_budget_pressure() {
        let recent: Vec<ActionRecord> = (0..MAX_INCLUDED_RECORDS).map(long_record).collect();
        let db = test_memory_db("budget_pressure");
        let store = AgentMemoryStore::new(&db);
        store
            .remember_memory(MemoryWrite {
                body: String::from(
                    "Cozmio memory flywheel budget pressure should keep runtime recall admission visible.",
                ),
                layer: String::from("reflection"),
                lifecycle: String::from("active"),
                source_refs: vec![String::from("experience:trace-budget:popup_confirmed")],
                supersedes_id: None,
                producer: String::from("test-consolidator"),
                expires_at: None,
            })
            .unwrap();

        let context = build_popup_context_with_recent_and_store(
            &recent,
            "Cozmio memory flywheel budget pressure",
            "Code.exe",
            &ProcessContext {
                stay_duration_seconds: 99,
                switches_in_last_minute: 1,
                rapid_switch_intervals_under_5s: 0,
                foreground_changed_within_5s: true,
            },
            &"hot stable context must remain independently visible. ".repeat(40),
            Some(&store),
        );

        assert!(context.chars().count() <= MAX_CONTEXT_CHARS + 3);
        assert!(context.contains("hot_stable_context:"));
        assert!(context.contains("source_type=hot_stable_context"));
        assert!(context.contains("recall_admission:"));
        assert!(context.contains("source_type=recent_feedback_fact"));
        assert!(context.contains("source_type=agent_memory"));
        assert!(context.contains("source_refs=[experience:trace-budget:popup_confirmed]"));
        assert!(context.contains("action_log_tail:"));
    }

    #[test]
    fn recent_feedback_admission_survives_memory_budget_pressure() {
        let mut feedback_record = record(
            "Cozmio memory flywheel feedback priority",
            "dismissed",
            Some("critical_dismiss_feedback_should_remain_visible"),
        );
        feedback_record.trace_id = Some(String::from("trace-critical-feedback"));
        let recent = vec![feedback_record];
        let db = test_memory_db("feedback_priority");
        let store = AgentMemoryStore::new(&db);
        for index in 0..MAX_RECALLED_MEMORIES {
            store
                .remember_memory(MemoryWrite {
                    body: format!(
                        "Cozmio memory flywheel feedback priority recalled memory {index}. {}",
                        "long recalled memory body ".repeat(40)
                    ),
                    layer: String::from("reflection"),
                    lifecycle: String::from("active"),
                    source_refs: vec![format!("experience:trace-memory-{index}")],
                    supersedes_id: None,
                    producer: String::from("test-consolidator"),
                    expires_at: None,
                })
                .unwrap();
        }

        let context = build_popup_context_with_recent_and_store(
            &recent,
            "Cozmio memory flywheel feedback priority",
            "Code.exe",
            &ProcessContext {
                stay_duration_seconds: 12,
                switches_in_last_minute: 0,
                rapid_switch_intervals_under_5s: 0,
                foreground_changed_within_5s: false,
            },
            "",
            Some(&store),
        );

        assert!(context.contains("recent_feedback_facts:"));
        assert!(context.contains("trace-critical-feedback"));
        assert!(context.contains("critical_dismiss_feedback_should_remain_visible"));
        assert!(context.contains("admitted_memories:"));
        assert!(context.contains("source_type=agent_memory"));
    }
}
