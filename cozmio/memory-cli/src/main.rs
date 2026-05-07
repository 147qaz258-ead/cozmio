use anyhow::Result;
use clap::Parser;
use cozmio_memory::search::SearchQuery;
use cozmio_memory::{
    build_slices_from_events, create_provider, default_action_log_path, import_from_file,
    link_events_to_threads, run_migrations, seed_demo_data, ActivityNote, ContextSlicesStore,
    Database, DecisionMemoryStore, MemoryCore, MemoryEventsStore, ProviderType, SkillMemoryStore,
    TaskThreadsStore,
};

#[derive(Parser)]
#[command(name = "memory-cli")]
#[command(about = "Debug/verification interface for the cozmio memory system")]
enum Cli {
    /// Seed demo data into all memory stores (threads, decisions, skills)
    Seed,
    /// Show record counts, time range, and source distribution
    Stats,
    /// Import events from action_log.jsonl
    Import,
    /// Search memory with a keyword query
    Search { query: String },
    /// Generate candidate suggestions (offline, no toast)
    Replay { since: Option<String> },
    /// View context slice details by ID
    Inspect { slice_id: i64 },
    /// Rebuild FTS and vector indexes
    RebuildIndex,
    /// Build context slices from imported events
    BuildSlices,
    /// Link imported events to task threads based on keywords
    LinkThreads,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize database
    let db_path = Database::memory_dir().join("memory.db");
    std::fs::create_dir_all(db_path.parent().unwrap())?;
    let db = Database::new(&db_path)?;
    run_migrations(&db)?;

    match cli {
        Cli::Seed => cmd_seed(&db),
        Cli::Stats => cmd_stats(&db),
        Cli::Import => cmd_import(&db),
        Cli::Search { query } => cmd_search(&db, &query),
        Cli::Replay { since: _ } => cmd_replay(&db),
        Cli::Inspect { slice_id } => cmd_inspect(&db, slice_id),
        Cli::RebuildIndex => cmd_rebuild_index(&db),
        Cli::BuildSlices => cmd_build_slices(&db),
        Cli::LinkThreads => cmd_link_threads(&db),
    }
}

fn cmd_seed(db: &Database) -> Result<()> {
    println!("Seeding demo data (threads, decisions, skills)...");
    seed_demo_data(db)?;
    println!("Seed data inserted successfully.");
    println!("\nNote: build_reminder_context requires 'imported' evidence.");
    println!("Run 'memory-cli import' first to import action log data.");
    Ok(())
}

fn cmd_stats(db: &Database) -> Result<()> {
    println!("=== Memory Statistics ===\n");

    let events_store = MemoryEventsStore::new(db);
    let slices_store = ContextSlicesStore::new(db);
    let threads_store = TaskThreadsStore::new(db);
    let decisions_store = DecisionMemoryStore::new(db);
    let skills_store = SkillMemoryStore::new(db);

    // Event counts
    let total_events = events_store.count()?;
    println!("memory_events:");
    println!("  total: {}", total_events);

    // Time range
    if let Ok((min_ts, max_ts)) = events_store.get_time_range() {
        if !min_ts.is_empty() && !max_ts.is_empty() {
            println!("  time_range: {} -> {}", min_ts, max_ts);
        } else {
            println!("  time_range: (empty)");
        }
    }

    // Evidence source distribution for events
    let imported_count = events_store.count_by_evidence_source("imported")?;
    let generated_count = events_store.count_by_evidence_source("generated")?;
    let seed_count = events_store.count_by_evidence_source("seed")?;
    println!("  by_evidence_source:");
    println!("    imported: {}", imported_count);
    println!("    generated: {}", generated_count);
    println!("    seed: {}", seed_count);

    // Context slices
    let slices = slices_store.get_recent(1000)?;
    println!("\ncontext_slices: {}", slices.len());

    // Task threads
    let threads = threads_store.get_all()?;
    println!("\ntask_threads: {}", threads.len());
    for thread in threads.iter().take(5) {
        println!("  - {} (state: {:?})", thread.name, thread.current_state);
    }
    if threads.len() > 5 {
        println!("  ... and {} more", threads.len() - 5);
    }

    // Decisions
    let decisions = decisions_store.get_all()?;
    println!("\ndecision_memory: {}", decisions.len());

    // Skills
    let skills = skills_store.get_all()?;
    println!("\nskill_memory: {}", skills.len());
    for skill in skills.iter().take(5) {
        println!("  - {} (usage: {})", skill.name, skill.usage_count);
    }
    if skills.len() > 5 {
        println!("  ... and {} more", skills.len() - 5);
    }

    println!("\n=== Database Location ===");
    println!("{}", db_path_display());

    Ok(())
}

fn cmd_import(db: &Database) -> Result<()> {
    let log_path = default_action_log_path();
    println!("Importing from: {:?}", log_path);

    if !log_path.exists() {
        println!(
            "No action_log.jsonl found at {:?}. Nothing to import.",
            log_path
        );
        return Ok(());
    }

    let result = import_from_file(db, &log_path)?;
    println!("\nImport complete:");
    println!("  imported: {}", result.imported);
    println!("  skipped:  {}", result.skipped);
    println!("  errors:   {}", result.errors);

    Ok(())
}

fn cmd_search(db: &Database, query: &str) -> Result<()> {
    println!("Searching for: {}\n", query);

    // Try to create a disabled provider since we may not have embeddings
    let provider_result = create_provider(ProviderType::Disabled);
    let embed_provider = match provider_result {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("Warning: Could not create embedding provider: {}", e);
            None
        }
    };

    let core = MemoryCore::new(db, embed_provider);
    let search_engine = core.search_engine();

    let results = search_engine.search(&SearchQuery {
        text: Some(query.to_string()),
        time_range: None,
        thread_id: None,
        limit: 20,
    })?;

    println!(
        "Found {} results (FTS: {}, Vec: {})\n",
        results.events.len(),
        results.total_fts,
        results.total_vec
    );

    for result in results.events.iter().take(20) {
        println!(
            "[{}] {}",
            result.timestamp,
            result.content.chars().take(100).collect::<String>()
        );
        println!("  window: {:?}", result.window_title);
        println!(
            "  source: {}, evidence: {}",
            result.source, result.evidence_source
        );
        println!("  score: {:.4}\n", result.score);
    }

    Ok(())
}

fn cmd_replay(db: &Database) -> Result<()> {
    println!("Generating candidate suggestions (replay mode)...\n");

    // Get recent events for replay
    let events_store = MemoryEventsStore::new(db);
    let recent_events = events_store.get_recent(10)?;

    if recent_events.is_empty() {
        println!("No recent events found. Import some data first.");
        return Ok(());
    }

    // Create an ActivityNote from the most recent event
    // Use the event's own timestamp minus 2h so get_since_ts() finds slices
    // from the same time period (slices cover the window containing the event)
    let latest = &recent_events[0];
    let event_time = chrono::DateTime::parse_from_rfc3339(&latest.timestamp)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());
    let note_timestamp = event_time - chrono::Duration::hours(2);
    let note = ActivityNote {
        window_title: latest.window_title.clone().unwrap_or_default(),
        content_text: latest.content.clone(),
        timestamp: note_timestamp,
        current_thread_id: latest.thread_id,
    };

    // Try to build reminder context
    let provider_result = create_provider(ProviderType::Disabled);
    let embed_provider = match provider_result {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("Warning: Could not create embedding provider: {}", e);
            None
        }
    };

    let core = MemoryCore::new(db, embed_provider);
    let competition = core.competition();

    match competition.build_reminder_context(&note) {
        Ok(ctx) => {
            println!("=== Reminder Context ===\n");
            println!("current_activity: {}", ctx.current_activity);
            println!("recent_context: {}", ctx.recent_context);
            println!("related_decisions: {}", ctx.related_decisions);
            println!("relevant_skills: {}", ctx.relevant_skills);
            if let Some(ref state) = ctx.task_state {
                println!("task_state: {}", state);
            }
            println!("\neevidence_refs ({} total):", ctx.evidence_refs.len());
            for (i, r#ref) in ctx.evidence_refs.iter().take(10).enumerate() {
                println!(
                    "  {}. [{}] {} - {}",
                    i + 1,
                    r#ref.memory_type,
                    r#ref.source,
                    r#ref.content_snippet.chars().take(60).collect::<String>()
                );
            }
        }
        Err(e) => {
            println!("Failed to build reminder context: {}", e);
            println!(
                "\nNote: build_reminder_context requires at least one 'imported' evidence source."
            );
            println!("Import some action_log.jsonl data first.");
        }
    }

    Ok(())
}

fn cmd_inspect(db: &Database, slice_id: i64) -> Result<()> {
    let slices_store = ContextSlicesStore::new(db);

    match slices_store.get_by_id(slice_id)? {
        Some(slice) => {
            println!("=== Context Slice #{} ===\n", slice_id);
            println!("start_time: {}", slice.start_time);
            println!("end_time:   {}", slice.end_time);
            println!("summary:    {}", slice.summary);
            println!("entities:   {:?}", slice.entities);
            println!("topics:     {:?}", slice.topics);
            println!("raw_refs:   {:?}", slice.raw_refs);
            println!("source:     {}", slice.evidence_source);
        }
        None => {
            println!("Context slice #{} not found.", slice_id);
        }
    }

    Ok(())
}

fn cmd_rebuild_index(db: &Database) -> Result<()> {
    println!("Rebuilding indexes...\n");

    let conn = db.conn.lock().unwrap();

    // Rebuild FTS5 index
    println!("Rebuilding FTS5 index...");
    match conn.execute(
        "INSERT INTO memory_events_fts(memory_events_fts) VALUES('rebuild')",
        [],
    ) {
        Ok(_) => println!("  FTS5 index rebuilt successfully."),
        Err(e) => println!("  FTS5 rebuild failed (may be unsupported): {}", e),
    }

    // Note: Vector index rebuild would require sqlite-vec extension
    // which is not directly accessible via rusqlite
    println!("\nNote: Vector index (vec0) rebuild requires sqlite-vec extension support.");
    println!("If vector search is not working, ensure sqlite-vec is properly loaded.");

    println!("\n=== Indexes ===");
    println!("FTS5: memory_events_fts (full-text search on content, window_title)");
    println!("vec0: memory_events_vec (vector embeddings, 384 dimensions)");
    println!("\nDatabase location: {}", db_path_display());

    Ok(())
}

fn db_path_display() -> String {
    Database::memory_dir()
        .join("memory.db")
        .to_string_lossy()
        .to_string()
}

fn cmd_build_slices(db: &Database) -> Result<()> {
    println!("Building context slices from imported events...");
    let count = build_slices_from_events(db)?;
    println!("Built {} context slices.", count);
    println!("\nSlices will now provide recent_context in ReminderContext.");
    Ok(())
}

fn cmd_link_threads(db: &Database) -> Result<()> {
    println!("Linking imported events to task threads...");
    let count = link_events_to_threads(db)?;
    println!("Linked {} events to task threads.", count);
    println!("\nEvents will now have thread_id for task_state in ReminderContext.");
    Ok(())
}
