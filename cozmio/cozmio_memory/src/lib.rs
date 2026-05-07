pub mod agent_memory;
pub mod competition;
pub mod context_slices;
pub mod db;
pub mod decision_memory;
pub mod embed_disabled;
pub mod embed_fastreembed;
pub mod embed_mock;
pub mod embed_provider;
pub mod error;
pub mod importer;
pub mod memory_events;
pub mod schema;
pub mod search;
pub mod seed;
pub mod skill_memory;
pub mod slice_builder;
pub mod task_threads;
pub mod thread_linker;
pub mod vec_store;

// re-exports
pub use importer::{default_action_log_path, import_from_file, ActionRecord, ImportResult};
pub use schema::run_migrations;
pub use seed::seed_demo_data;
pub use slice_builder::build_slices_from_events;
pub use thread_linker::{link_events_to_threads, relink_all_events};

// re-export types
pub use agent_memory::{
    AgentMemory, AgentMemoryStore, ConsolidationRun, ExperienceSource, MemoryOperation,
    MemoryOperationRecord, MemoryWrite,
};
pub use competition::{ActivityNote, EvidenceRef, MemoryCompetition, MemoryCore, ReminderContext};
pub use context_slices::{ContextSlice, ContextSlicesStore};
pub use db::Database;
pub use decision_memory::{Decision, DecisionMemoryStore};
pub use embed_provider::{create_provider, EmbeddingProvider, ProviderType};
pub use error::MemoryError;
pub use memory_events::{MemoryEvent, MemoryEventsStore};
pub use search::SearchQuery;
pub use skill_memory::{Skill, SkillMemoryStore};
pub use task_threads::{TaskThread, TaskThreadUpdate, TaskThreadsStore};
pub use vec_store::InMemoryVecStore;
