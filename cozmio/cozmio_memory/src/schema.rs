use crate::db::Database;
use crate::error::MemoryError;
use rusqlite::Connection;

pub fn run_migrations(db: &Database) -> Result<(), MemoryError> {
    let conn = db.conn.lock().unwrap();

    // memory_events table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memory_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            source TEXT NOT NULL,
            window_title TEXT,
            content TEXT NOT NULL,
            raw_ref TEXT,
            embedding BLOB,
            thread_id INTEGER REFERENCES task_threads(id),
            evidence_source TEXT DEFAULT 'imported'
        )",
        [],
    )?;

    // context_slices table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS context_slices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            summary TEXT NOT NULL,
            entities TEXT,
            topics TEXT,
            raw_refs TEXT,
            evidence_source TEXT DEFAULT 'generated'
        )",
        [],
    )?;

    // task_threads table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_threads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            current_state TEXT,
            open_questions TEXT,
            decisions TEXT,
            recent_slice_ids TEXT,
            created_at TEXT,
            updated_at TEXT
        )",
        [],
    )?;

    // decision_memory table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS decision_memory (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            memory_type TEXT NOT NULL,
            content TEXT NOT NULL,
            evidence TEXT,
            related_thread_id INTEGER REFERENCES task_threads(id),
            evidence_source TEXT DEFAULT 'seed'
        )",
        [],
    )?;

    // skill_memory table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS skill_memory (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            procedure TEXT NOT NULL,
            success_context TEXT,
            usage_count INTEGER DEFAULT 0,
            last_used_at TEXT,
            evidence_source TEXT DEFAULT 'seed'
        )",
        [],
    )?;

    preserve_legacy_agent_memory_tables(&conn)?;

    // agent_memories table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agent_memories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            body TEXT NOT NULL,
            layer TEXT NOT NULL,
            lifecycle TEXT NOT NULL DEFAULT 'active',
            source_refs_json TEXT NOT NULL DEFAULT '[]',
            supersedes_id INTEGER REFERENCES agent_memories(id),
            producer TEXT NOT NULL DEFAULT 'legacy',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            last_used_at TEXT,
            used_count INTEGER NOT NULL DEFAULT 0,
            rejected_reason TEXT,
            expires_at TEXT
        )",
        [],
    )?;
    add_column_if_missing(
        &conn,
        "agent_memories",
        "source_refs_json",
        "TEXT NOT NULL DEFAULT '[]'",
    )?;
    add_column_if_missing(
        &conn,
        "agent_memories",
        "supersedes_id",
        "INTEGER REFERENCES agent_memories(id)",
    )?;
    add_column_if_missing(
        &conn,
        "agent_memories",
        "producer",
        "TEXT NOT NULL DEFAULT 'legacy'",
    )?;
    add_column_if_missing(&conn, "agent_memories", "rejected_reason", "TEXT")?;
    add_column_if_missing(&conn, "agent_memories", "expires_at", "TEXT")?;

    // memory_operations table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memory_operations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            memory_id INTEGER NOT NULL DEFAULT 0,
            operation TEXT NOT NULL DEFAULT '',
            layer TEXT NOT NULL DEFAULT '',
            lifecycle TEXT NOT NULL DEFAULT 'active',
            body TEXT NOT NULL DEFAULT '',
            operation_type TEXT NOT NULL DEFAULT '',
            target_memory_id INTEGER REFERENCES agent_memories(id),
            resulting_memory_id INTEGER REFERENCES agent_memories(id),
            source_refs_json TEXT NOT NULL DEFAULT '[]',
            status TEXT NOT NULL DEFAULT 'applied',
            error_text TEXT,
            producer TEXT NOT NULL DEFAULT 'legacy',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;
    add_column_if_missing(
        &conn,
        "memory_operations",
        "operation_type",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(
        &conn,
        "memory_operations",
        "target_memory_id",
        "INTEGER REFERENCES agent_memories(id)",
    )?;
    add_column_if_missing(
        &conn,
        "memory_operations",
        "resulting_memory_id",
        "INTEGER REFERENCES agent_memories(id)",
    )?;
    add_column_if_missing(
        &conn,
        "memory_operations",
        "source_refs_json",
        "TEXT NOT NULL DEFAULT '[]'",
    )?;
    add_column_if_missing(
        &conn,
        "memory_operations",
        "status",
        "TEXT NOT NULL DEFAULT 'applied'",
    )?;
    add_column_if_missing(&conn, "memory_operations", "error_text", "TEXT")?;
    add_column_if_missing(
        &conn,
        "memory_operations",
        "producer",
        "TEXT NOT NULL DEFAULT 'legacy'",
    )?;
    conn.execute(
        "UPDATE memory_operations SET operation_type = operation WHERE operation_type = ''",
        [],
    )?;
    conn.execute(
        "UPDATE memory_operations SET target_memory_id = memory_id WHERE target_memory_id IS NULL AND memory_id > 0",
        [],
    )?;
    conn.execute(
        "UPDATE memory_operations SET resulting_memory_id = memory_id WHERE resulting_memory_id IS NULL AND memory_id > 0",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS consolidation_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            trigger_kind TEXT NOT NULL,
            route TEXT NOT NULL,
            packet_json TEXT NOT NULL,
            model_name TEXT,
            output_text TEXT,
            status TEXT NOT NULL,
            error_text TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            completed_at TEXT
        )",
        [],
    )?;
    add_column_if_missing(
        &conn,
        "consolidation_runs",
        "trigger_kind",
        "TEXT NOT NULL DEFAULT 'legacy'",
    )?;
    add_column_if_missing(
        &conn,
        "consolidation_runs",
        "route",
        "TEXT NOT NULL DEFAULT 'local'",
    )?;
    add_column_if_missing(
        &conn,
        "consolidation_runs",
        "packet_json",
        "TEXT NOT NULL DEFAULT '{}'",
    )?;
    add_column_if_missing(&conn, "consolidation_runs", "model_name", "TEXT")?;
    add_column_if_missing(&conn, "consolidation_runs", "output_text", "TEXT")?;
    add_column_if_missing(
        &conn,
        "consolidation_runs",
        "status",
        "TEXT NOT NULL DEFAULT 'running'",
    )?;
    add_column_if_missing(&conn, "consolidation_runs", "error_text", "TEXT")?;
    add_column_if_missing(
        &conn,
        "consolidation_runs",
        "created_at",
        "TEXT NOT NULL DEFAULT '1970-01-01 00:00:00'",
    )?;
    add_column_if_missing(&conn, "consolidation_runs", "completed_at", "TEXT")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS experience_sources (
            source_ref TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            trace_id TEXT,
            session_id TEXT,
            event_kind TEXT NOT NULL,
            window_title TEXT,
            process_name TEXT,
            factual_text TEXT NOT NULL,
            raw_ref TEXT,
            artifact_refs_json TEXT NOT NULL DEFAULT '[]'
        )",
        [],
    )?;
    add_column_if_missing(
        &conn,
        "experience_sources",
        "timestamp",
        "TEXT NOT NULL DEFAULT '1970-01-01 00:00:00'",
    )?;
    add_column_if_missing(&conn, "experience_sources", "trace_id", "TEXT")?;
    add_column_if_missing(&conn, "experience_sources", "session_id", "TEXT")?;
    add_column_if_missing(
        &conn,
        "experience_sources",
        "event_kind",
        "TEXT NOT NULL DEFAULT 'legacy'",
    )?;
    add_column_if_missing(&conn, "experience_sources", "window_title", "TEXT")?;
    add_column_if_missing(&conn, "experience_sources", "process_name", "TEXT")?;
    add_column_if_missing(
        &conn,
        "experience_sources",
        "factual_text",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(&conn, "experience_sources", "raw_ref", "TEXT")?;
    add_column_if_missing(
        &conn,
        "experience_sources",
        "artifact_refs_json",
        "TEXT NOT NULL DEFAULT '[]'",
    )?;

    // FTS5 virtual table on memory_events
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS memory_events_fts USING fts5(
            content,
            window_title,
            content='memory_events',
            content_rowid='id'
        )",
        [],
    )?;

    // sqlite-vec virtual table on embedding[384]
    // NOTE: sqlite-vec requires native compilation which may fail on some build environments.
    // The table schema reference is kept for documentation; vec0 extension must be loaded
    // at runtime via sqlite-vec's loadable extension mechanism.
    // conn.execute(
    //     "CREATE VIRTUAL TABLE IF NOT EXISTS memory_events_vec USING vec0(
    //         embedding[384]
    //     )",
    //     [],
    // )?;

    // Conditionally create sqlite-vec virtual table when vec feature is enabled
    #[cfg(feature = "vec")]
    {
        use rusqlite::params;
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS memory_events_vec USING vec0(embedding[384])",
            params![],
        )?;
    }

    // AFTER INSERT trigger to sync FTS5
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS memory_events_ai AFTER INSERT ON memory_events BEGIN
            INSERT INTO memory_events_fts(rowid, content, window_title)
            VALUES (new.id, new.content, new.window_title);
        END",
        [],
    )?;

    Ok(())
}

fn preserve_legacy_agent_memory_tables(conn: &Connection) -> Result<(), MemoryError> {
    preserve_legacy_table_without_column(conn, "agent_memories", "id")?;
    preserve_legacy_table_without_column(conn, "memory_operations", "id")?;
    Ok(())
}

fn preserve_legacy_table_without_column(
    conn: &Connection,
    table_name: &str,
    required_column: &str,
) -> Result<(), MemoryError> {
    if !table_exists(conn, table_name)? {
        return Ok(());
    }

    if table_columns(conn, table_name)?
        .iter()
        .any(|column| column == required_column)
    {
        return Ok(());
    }

    let preserved_name = format!("{}_legacy_{}", table_name, chrono::Utc::now().timestamp());
    conn.execute(
        &format!("ALTER TABLE {table_name} RENAME TO {preserved_name}"),
        [],
    )?;
    Ok(())
}

fn add_column_if_missing(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
    column_definition: &str,
) -> Result<(), MemoryError> {
    if table_columns(conn, table_name)?
        .iter()
        .any(|column| column == column_name)
    {
        return Ok(());
    }

    conn.execute(
        &format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {column_definition}"),
        [],
    )?;
    Ok(())
}

fn table_exists(conn: &Connection, table_name: &str) -> Result<bool, MemoryError> {
    let mut stmt =
        conn.prepare("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1")?;
    let mut rows = stmt.query([table_name])?;
    Ok(rows.next()?.is_some())
}

fn table_columns(conn: &Connection, table_name: &str) -> Result<Vec<String>, MemoryError> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let rows = stmt.query_map([], |row| row.get(1))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(MemoryError::from)
}
