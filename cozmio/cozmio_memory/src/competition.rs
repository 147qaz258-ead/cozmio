use std::sync::Arc;

use crate::context_slices::ContextSlicesStore;
use crate::db::Database;
use crate::decision_memory::DecisionMemoryStore;
use crate::embed_provider::EmbeddingProvider;
use crate::error::MemoryError;
use crate::search::{SearchEngine, SearchQuery};
use crate::skill_memory::SkillMemoryStore;
use crate::task_threads::TaskThreadsStore;
use chrono::{DateTime, Utc};

pub struct ActivityNote {
    pub window_title: String,
    pub content_text: String,
    pub timestamp: DateTime<Utc>,
    pub current_thread_id: Option<i64>,
}

pub struct EvidenceRef {
    pub source: String,
    pub memory_type: String,
    pub id: i64,
    pub content_snippet: String,
    pub timestamp: Option<String>,
}

pub struct ReminderContext {
    pub current_activity: String,
    pub recent_context: String,
    pub related_decisions: String,
    pub relevant_skills: String,
    pub task_state: Option<String>,
    pub evidence_refs: Vec<EvidenceRef>,
}

pub struct MemoryCompetition<'a> {
    db: &'a Database,
    search_engine: SearchEngine<'a>,
}

impl<'a> MemoryCompetition<'a> {
    pub fn new(db: &'a Database, search_engine: SearchEngine<'a>) -> Self {
        Self { db, search_engine }
    }

    pub fn build_reminder_context(
        &self,
        note: &ActivityNote,
    ) -> Result<ReminderContext, MemoryError> {
        let mut evidence_refs = Vec::new();
        let mut recent_context_parts = Vec::new();
        let mut related_decisions_parts = Vec::new();
        let mut relevant_skills_parts = Vec::new();
        let mut task_state_val = None;

        // 1. recent_context from context_slices (last 2 hours, relative to ActivityNote.timestamp)
        let slices_store = ContextSlicesStore::new(self.db);
        if let Ok(slices) = slices_store.get_since_ts(note.timestamp) {
            for slice in slices {
                if !slice.summary.is_empty() {
                    recent_context_parts.push(slice.summary.clone());
                }
                evidence_refs.push(EvidenceRef {
                    source: slice.evidence_source,
                    memory_type: "context_slice".to_string(),
                    id: slice.id.unwrap_or(0),
                    content_snippet: slice.summary,
                    timestamp: Some(slice.start_time),
                });
            }
        }

        // 2. Search for related events using the window_title + content_text
        // Sanitize FTS5 special characters (especially dots from file paths like cmd.exe)
        let search_text = sanitize_fts5_query(&note.window_title);
        if !search_text.is_empty() {
            if let Ok(search_results) = self.search_engine.search(&SearchQuery {
                text: Some(search_text.clone()),
                time_range: None,
                thread_id: note.current_thread_id,
                limit: 10,
            }) {
                for result in search_results.events {
                    evidence_refs.push(EvidenceRef {
                        source: result.evidence_source,
                        memory_type: "memory_event".to_string(),
                        id: result.event_id,
                        content_snippet: result.content.chars().take(200).collect(),
                        timestamp: Some(result.timestamp),
                    });
                }
            }
        }

        // 3. related decisions from decision_memory
        // Try thread-specific decisions first; if none found (or thread_id not set), fall back to all decisions.
        // Seed decisions are only auxiliary — they must be clearly labeled in evidence_refs.
        let decisions_store = DecisionMemoryStore::new(self.db);
        let decisions_source;
        let decisions: Vec<_> = if let Some(tid) = note.current_thread_id {
            decisions_store.get_by_thread(tid).unwrap_or_default()
        } else {
            Vec::new()
        };

        if !decisions.is_empty() {
            decisions_source = "thread";
            for decision in decisions {
                related_decisions_parts.push(decision.content.clone());
                evidence_refs.push(EvidenceRef {
                    source: decision.evidence_source,
                    memory_type: "decision".to_string(),
                    id: decision.id.unwrap_or(0),
                    content_snippet: decision.content,
                    timestamp: None,
                });
            }
        } else {
            // Fall back to all decisions (mainly seed decisions as auxiliary)
            decisions_source = "seed_auxiliary";
            if let Ok(all_decisions) = decisions_store.get_all() {
                for decision in all_decisions.into_iter().take(5) {
                    let is_seed = decision.evidence_source == "seed";
                    let label = if is_seed { "[seed辅助] " } else { "" };
                    related_decisions_parts.push(format!("{}{}", label, decision.content));
                    evidence_refs.push(EvidenceRef {
                        source: decision.evidence_source.clone(),
                        memory_type: "decision".to_string(),
                        id: decision.id.unwrap_or(0),
                        content_snippet: decision.content,
                        timestamp: None,
                    });
                }
            }
        }
        if decisions_source == "seed_auxiliary" {
            eprintln!("[ReminderContext] thread_id={:?} but no decisions found; showing seed decisions as AUXILIARY only", note.current_thread_id);
        }

        // 4. relevant_skills from skill_memory
        let skills_store = SkillMemoryStore::new(self.db);
        if let Ok(skills) = skills_store.get_all() {
            for skill in skills.into_iter().take(3) {
                let skill_text = format!("{}: {}", skill.name, skill.procedure);
                relevant_skills_parts.push(skill_text);
                evidence_refs.push(EvidenceRef {
                    source: skill.evidence_source,
                    memory_type: "skill".to_string(),
                    id: skill.id.unwrap_or(0),
                    content_snippet: skill.procedure.chars().take(200).collect(),
                    timestamp: skill.last_used_at,
                });
            }
        }

        // 5. task_state from task_threads
        if let Some(tid) = note.current_thread_id {
            let threads_store = TaskThreadsStore::new(self.db);
            if let Ok(Some(thread)) = threads_store.get_by_id(tid) {
                let state_text = format!(
                    "Task: {} - {}",
                    thread.name,
                    thread.current_state.unwrap_or_default()
                );
                task_state_val = Some(state_text);
            }
        }

        // CRITICAL CHECK: must have at least one imported evidence
        let has_imported = evidence_refs.iter().any(|e| e.source == "imported");
        if !has_imported {
            return Err(MemoryError::InsufficientImportedData);
        }

        Ok(ReminderContext {
            current_activity: format!("{} - {}", note.window_title, note.content_text),
            recent_context: recent_context_parts.join(" | "),
            related_decisions: related_decisions_parts.join(" | "),
            relevant_skills: relevant_skills_parts.join(" | "),
            task_state: task_state_val,
            evidence_refs,
        })
    }
}

pub struct MemoryCore<'a> {
    pub db: &'a Database,
    pub embed_provider: Option<Arc<dyn EmbeddingProvider>>,
}

impl<'a> MemoryCore<'a> {
    pub fn new(db: &'a Database, embed_provider: Option<Arc<dyn EmbeddingProvider>>) -> Self {
        Self { db, embed_provider }
    }

    pub fn search_engine(&self) -> SearchEngine<'_> {
        SearchEngine::new(self.db, self.embed_provider.clone())
    }

    pub fn competition(&self) -> MemoryCompetition<'_> {
        MemoryCompetition::new(self.db, self.search_engine())
    }
}

/// Sanitize a search query for FTS5 by escaping problematic characters.
/// FTS5 special characters: ^ - + : * ( ) " ' { } [ ] \ /
/// The dot (.) from file paths like "cmd.exe" causes syntax errors.
fn sanitize_fts5_query(input: &str) -> String {
    // Replace common FTS5 special characters with spaces
    // This is a simple approach that preserves word boundaries for tokenization
    let mut result = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '.' | '^' | '-' | '+' | ':' | '*' | '(' | ')' | '[' | ']' | '{' | '}' | '\\' | '/' => {
                result.push(' ');
            }
            _ => {
                result.push(ch);
            }
        }
    }
    // Collapse multiple spaces into one
    let mut collapsed = String::new();
    let mut last_was_space = false;
    for ch in result.chars() {
        if ch == ' ' {
            if !last_was_space {
                collapsed.push(ch);
            }
            last_was_space = true;
        } else {
            collapsed.push(ch);
            last_was_space = false;
        }
    }
    collapsed.trim().to_string()
}
