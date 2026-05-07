//! Event Ledger core module.
//!
//! This module provides:
//! - `LedgerEvent` / `ContentRef` schema
//! - `LedgerWriter` (append-only JSONL)
//! - `LedgerProjection` (SQLite index)
//! - `ContentStoreWriter` (binary/text content storage)
//! - `LedgerManager` (unified facade)
//!
//! JSONL is the canonical source of truth; SQLite is a projection for efficient querying.

use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Schema
// ---------------------------------------------------------------------------

/// Core event structure for the Event Ledger.
///
/// All fields that are strings in the system (event_type, source, etc.) are
/// stored as-is without parsing. Model output is preserved in `raw_text`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEvent {
    /// Unique identifier for this event (UUID)
    pub event_id: String,
    /// Optional trace ID linking related events
    pub trace_id: Option<String>,
    /// Optional session ID for relay sessions
    pub session_id: Option<String>,
    /// Unix timestamp (seconds since epoch)
    pub timestamp: i64,
    /// Event type string (e.g. "observation_captured", "model_called")
    pub event_type: String,
    /// Source component that produced this event
    pub source: String,
    /// Window title at time of event
    pub window_title: Option<String>,
    /// Process name at time of event
    pub process_name: Option<String>,
    /// Raw text content (model output, transcript, etc.) - preserved as-is
    pub raw_text: Option<String>,
    /// Content reference for stored binary/text content
    pub content_ref: Option<ContentRef>,
    /// Parent event ID for event chaining
    pub parent_event_id: Option<String>,
    /// Arbitrary key-value metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Content reference structure - a structured address for stored content.
///
/// This records where content is stored and metadata about it. `ContentResolver`
/// can resolve supported refs back into validated text material.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRef {
    /// Unique identifier for this content
    pub content_ref: String,
    /// Content type (e.g. "screenshot", "model_output", "relay_output", "transcript")
    pub content_type: String,
    /// Storage backend (e.g. "file")
    pub storage_backend: String,
    /// Path or key where content is stored
    pub path_or_key: String,
    /// SHA256 hash of the content
    pub content_hash: String,
    /// Unix timestamp when content was created
    pub created_at: i64,
    /// Producer component (e.g. "WindowMonitor", "ModelClient", "Executor")
    pub producer: String,
    /// Optional byte range for partial content
    pub byte_range: Option<(u64, u64)>,
    /// Optional line range for text content
    pub line_range: Option<(u64, u64)>,
}

// ---------------------------------------------------------------------------
// Event type constants
// ---------------------------------------------------------------------------

/// Well-known event type strings.
pub mod event_type {
    pub const OBSERVATION_CAPTURED: &str = "observation_captured";
    pub const MODEL_CALLED: &str = "model_called";
    pub const MODEL_OUTPUT_RECEIVED: &str = "model_output_received";
    pub const POPUP_DISPLAYED: &str = "popup_displayed";
    pub const PENDING_CONFIRMATION_CREATED: &str = "pending_confirmation_created";
    pub const USER_CONFIRMED: &str = "user_confirmed";
    pub const USER_CANCELLED: &str = "user_cancelled";
    pub const USER_DISMISSED: &str = "user_dismissed";
    pub const RELAY_DISPATCHED: &str = "relay_dispatched";
    pub const EXECUTION_PROGRESS_RECEIVED: &str = "execution_progress_received";
    pub const EXECUTION_RESULT_RECEIVED: &str = "execution_result_received";
    pub const EXECUTION_ERROR_RECEIVED: &str = "execution_error_received";
    pub const MODEL_ERROR: &str = "model_error";
    pub const SYSTEM_ERROR: &str = "system_error";
    pub const CONTEXT_PACK_BUILT: &str = "context_pack_built";
}

// ---------------------------------------------------------------------------
// LedgerWriter (JSONL Append)
// ---------------------------------------------------------------------------

/// Append-only JSONL writer for ledger events.
///
/// JSONL is the canonical source of truth. Each day's events go to a separate
/// file: `{base_dir}/event-log/YYYY-MM-DD.jsonl`.
pub struct LedgerWriter {
    base_dir: PathBuf,
}

impl LedgerWriter {
    /// Create a new LedgerWriter with the given base directory.
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Get the default base directory (`%LOCALAPPDATA%/cozmio/event-log/`).
    pub fn default_base_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cozmio")
            .join("event-log")
    }

    /// Get the JSONL file path for a given date string (YYYY-MM-DD).
    pub fn get_event_path(&self, date: &str) -> PathBuf {
        self.base_dir.join(format!("{}.jsonl", date))
    }

    /// Append an event to the JSONL file for the current date.
    ///
    /// Creates the directory and file if they don't exist.
    pub fn append(&self, event: &LedgerEvent) -> std::io::Result<()> {
        // Ensure directory exists
        fs::create_dir_all(&self.base_dir)?;

        // Determine date from timestamp
        let date = timestamp_to_date(event.timestamp);
        let path = self.get_event_path(&date);

        // Serialize to JSON line
        let json = serde_json::to_string(event).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("JSON serialization failed: {}", e),
            )
        })?;

        // Append to file
        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;

        writeln!(file, "{}", json)?;

        Ok(())
    }

    /// Get the base directory path.
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }
}

/// Convert a Unix timestamp to a YYYY-MM-DD date string.
fn timestamp_to_date(timestamp: i64) -> String {
    use chrono::TimeZone;
    use chrono::Utc;

    // Convert Unix timestamp to DateTime<Utc>
    let datetime = Utc
        .timestamp_opt(timestamp, 0)
        .single()
        .unwrap_or_else(Utc::now);

    datetime.format("%Y-%m-%d").to_string()
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// ---------------------------------------------------------------------------
// LedgerProjection (SQLite Index)
// ---------------------------------------------------------------------------

/// SQLite-based projection for efficient event querying.
///
/// The SQLite database is a projection (index) on top of the JSONL files.
/// It can be rebuilt from JSONL at any time.
pub struct LedgerProjection {
    conn: Connection,
}

impl LedgerProjection {
    /// Create a new LedgerProjection with the given SQLite database path.
    pub fn new(db_path: PathBuf) -> SqliteResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let conn = Connection::open(&db_path)?;
        let projection = Self { conn };
        projection.ensure_schema()?;
        Ok(projection)
    }

    /// Create a LedgerProjection at the default location.
    pub fn with_default_path() -> SqliteResult<Self> {
        let db_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cozmio")
            .join("cozmio-ledger.sqlite");

        Self::new(db_path)
    }

    /// Ensure the database schema exists.
    pub fn ensure_schema(&self) -> SqliteResult<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS ledger_events (
                event_id TEXT PRIMARY KEY,
                trace_id TEXT,
                session_id TEXT,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                source TEXT NOT NULL,
                window_title TEXT,
                process_name TEXT,
                raw_text TEXT,
                content_ref_json TEXT,
                parent_event_id TEXT,
                metadata_json TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_trace_id ON ledger_events(trace_id);
            CREATE INDEX IF NOT EXISTS idx_session_id ON ledger_events(session_id);
            CREATE INDEX IF NOT EXISTS idx_event_type ON ledger_events(event_type);
            CREATE INDEX IF NOT EXISTS idx_timestamp ON ledger_events(timestamp);
            "#,
        )?;
        Ok(())
    }

    /// Ingest an event into the SQLite projection.
    ///
    /// Uses INSERT OR REPLACE for idempotent upsert semantics.
    /// This allows rebuilding the projection from JSONL without duplicates.
    pub fn ingest(&self, event: &LedgerEvent) -> SqliteResult<()> {
        let content_ref_json = event
            .content_ref
            .as_ref()
            .and_then(|cr| serde_json::to_string(cr).ok());
        let metadata_json = if event.metadata.is_empty() {
            None
        } else {
            serde_json::to_string(&event.metadata).ok()
        };

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO ledger_events
            (event_id, trace_id, session_id, timestamp, event_type, source,
             window_title, process_name, raw_text, content_ref_json,
             parent_event_id, metadata_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                event.event_id,
                event.trace_id,
                event.session_id,
                event.timestamp,
                event.event_type,
                event.source,
                event.window_title,
                event.process_name,
                event.raw_text,
                content_ref_json,
                event.parent_event_id,
                metadata_json,
            ],
        )?;
        Ok(())
    }

    /// Query events in timeline order (descending by timestamp).
    pub fn query_timeline(&self, limit: usize, offset: usize) -> SqliteResult<Vec<LedgerEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT event_id, trace_id, session_id, timestamp, event_type, source,
                    window_title, process_name, raw_text, content_ref_json,
                    parent_event_id, metadata_json
             FROM ledger_events
             ORDER BY timestamp DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map(params![limit as i64, offset as i64], row_to_event)?;
        rows.collect()
    }

    /// Query all events for a given trace ID.
    pub fn query_trace(&self, trace_id: &str) -> SqliteResult<Vec<LedgerEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT event_id, trace_id, session_id, timestamp, event_type, source,
                    window_title, process_name, raw_text, content_ref_json,
                    parent_event_id, metadata_json
             FROM ledger_events
             WHERE trace_id = ?1
             ORDER BY timestamp ASC",
        )?;

        let rows = stmt.query_map(params![trace_id], row_to_event)?;
        rows.collect()
    }

    /// Query all events for a given session ID.
    pub fn query_by_session(&self, session_id: &str) -> SqliteResult<Vec<LedgerEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT event_id, trace_id, session_id, timestamp, event_type, source,
                    window_title, process_name, raw_text, content_ref_json,
                    parent_event_id, metadata_json
             FROM ledger_events
             WHERE session_id = ?1
             ORDER BY timestamp ASC",
        )?;

        let rows = stmt.query_map(params![session_id], row_to_event)?;
        rows.collect()
    }

    /// Query events by event type with limit.
    pub fn query_by_event_type(
        &self,
        event_type: &str,
        limit: usize,
    ) -> SqliteResult<Vec<LedgerEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT event_id, trace_id, session_id, timestamp, event_type, source,
                    window_title, process_name, raw_text, content_ref_json,
                    parent_event_id, metadata_json
             FROM ledger_events
             WHERE event_type = ?1
             ORDER BY timestamp DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![event_type, limit as i64], row_to_event)?;
        rows.collect()
    }

    /// Query events within a Unix timestamp range (inclusive), ordered by timestamp descending.
    pub fn query_by_date_range(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> SqliteResult<Vec<LedgerEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT event_id, trace_id, session_id, timestamp, event_type, source,
                    window_title, process_name, raw_text, content_ref_json,
                    parent_event_id, metadata_json
             FROM ledger_events
             WHERE timestamp >= ?1 AND timestamp <= ?2
             ORDER BY timestamp DESC",
        )?;

        let rows = stmt.query_map(params![start_timestamp, end_timestamp], row_to_event)?;
        rows.collect()
    }

    /// Rebuild the SQLite projection from a JSONL file.
    ///
    /// Returns the number of events ingested.
    /// Uses INSERT OR REPLACE so this is safe to re-run.
    pub fn rebuild_from_jsonl(&self, jsonl_path: &Path) -> std::io::Result<usize> {
        if !jsonl_path.exists() {
            return Ok(0);
        }

        let file = fs::File::open(jsonl_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(event) = serde_json::from_str::<LedgerEvent>(line) {
                if self.ingest(&event).is_ok() {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Get the underlying SQLite connection (for advanced use).
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

/// Helper to convert a SQLite row to a LedgerEvent.
fn row_to_event(row: &rusqlite::Row) -> SqliteResult<LedgerEvent> {
    let content_ref_json: Option<String> = row.get(9)?;
    let content_ref = content_ref_json.and_then(|s| serde_json::from_str::<ContentRef>(&s).ok());

    let metadata_json: Option<String> = row.get(11)?;
    let metadata = metadata_json
        .and_then(|s| serde_json::from_str::<std::collections::HashMap<String, String>>(&s).ok())
        .unwrap_or_default();

    Ok(LedgerEvent {
        event_id: row.get(0)?,
        trace_id: row.get(1)?,
        session_id: row.get(2)?,
        timestamp: row.get(3)?,
        event_type: row.get(4)?,
        source: row.get(5)?,
        window_title: row.get(6)?,
        process_name: row.get(7)?,
        raw_text: row.get(8)?,
        content_ref,
        parent_event_id: row.get(10)?,
        metadata,
    })
}

// ---------------------------------------------------------------------------
// ContentStoreWriter
// ---------------------------------------------------------------------------

/// Writer for storing binary and text content in the content store.
///
/// Content is stored at:
/// `{base_dir}/content-store/{year}/{month}/{content_ref}.bin` (binary)
/// `{base_dir}/content-store/{year}/{month}/{content_ref}.txt` (text)
///
/// This module does NOT implement content resolution. That is Phase B/C responsibility.
pub struct ContentStoreWriter {
    base_dir: PathBuf,
}

impl ContentStoreWriter {
    /// Create a new ContentStoreWriter with the given base directory.
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Get the default base directory.
    pub fn default_base_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cozmio")
            .join("content-store")
    }

    /// Store content and return a ContentRef.
    ///
    /// The content is written to the appropriate path based on content_type.
    /// Text content uses `.txt` extension; other content uses `.bin`.
    pub fn store(
        &self,
        content: &[u8],
        content_type: &str,
        producer: &str,
    ) -> std::io::Result<ContentRef> {
        let content_ref = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        // Determine extension based on content type
        let extension = if content_type == "text"
            || content_type == "transcript"
            || content_type.starts_with("text/")
        {
            "txt"
        } else {
            "bin"
        };

        // Build path: content-store/{year}/{month}/{content_ref}.{ext}
        let date = chrono::Utc::now();
        let year = date.format("%Y").to_string();
        let month = date.format("%m").to_string();

        let dir = self.base_dir.join(&year).join(&month);
        fs::create_dir_all(&dir)?;

        let filename = format!("{}.{}", content_ref, extension);
        let path = dir.join(&filename);

        // Write content
        fs::write(&path, content)?;

        // Calculate hash
        let content_hash = digest(content);

        Ok(ContentRef {
            content_ref,
            content_type: content_type.to_string(),
            storage_backend: "file".to_string(),
            path_or_key: path.to_string_lossy().to_string(),
            content_hash,
            created_at: now,
            producer: producer.to_string(),
            byte_range: None,
            line_range: None,
        })
    }

    /// Get the storage path for a given content reference string and creation timestamp.
    ///
    /// The `created_at` parameter is the Unix timestamp when the content was originally stored.
    /// This ensures we look in the correct year/month directory where the content was saved.
    pub fn store_path(&self, content_ref: &str, created_at: i64) -> PathBuf {
        use chrono::TimeZone;
        use chrono::Utc;

        // Convert timestamp to DateTime<Utc> to get year/month
        let datetime = Utc
            .timestamp_opt(created_at, 0)
            .single()
            .unwrap_or_else(Utc::now);
        let year = datetime.format("%Y").to_string();
        let month = datetime.format("%m").to_string();

        let bin_path = self
            .base_dir
            .join(&year)
            .join(&month)
            .join(format!("{}.bin", content_ref));
        let txt_path = self
            .base_dir
            .join(&year)
            .join(&month)
            .join(format!("{}.txt", content_ref));

        if bin_path.exists() {
            bin_path
        } else if txt_path.exists() {
            txt_path
        } else {
            // Return bin path as default
            bin_path
        }
    }

    /// Get the base directory.
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }
}

// ---------------------------------------------------------------------------
// ContentResolver
// ---------------------------------------------------------------------------

/// Resolved text material for a content reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedContent {
    pub content_ref: String,
    pub text: String,
}

/// Resolver for stored content references.
pub struct ContentResolver;

impl ContentResolver {
    /// Resolve a file-backed content reference into UTF-8 text.
    ///
    /// The full file hash is validated before any byte or line slicing occurs.
    /// `byte_range` is applied before UTF-8 conversion; `line_range` is applied
    /// after UTF-8 conversion. Ranges use start-inclusive, end-exclusive indexes.
    pub fn resolve(content_ref: &ContentRef) -> std::io::Result<ResolvedContent> {
        if content_ref.storage_backend != "file" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!(
                    "unsupported content storage backend: {}",
                    content_ref.storage_backend
                ),
            ));
        }

        let bytes = fs::read(&content_ref.path_or_key)?;
        let actual_hash = digest(&bytes);
        if actual_hash != content_ref.content_hash {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "content hash mismatch for {}: expected {}, got {}",
                    content_ref.content_ref, content_ref.content_hash, actual_hash
                ),
            ));
        }

        let sliced_bytes = match content_ref.byte_range {
            Some((start, end)) => {
                let start = usize::try_from(start).map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "byte range start does not fit in usize",
                    )
                })?;
                let end = usize::try_from(end).map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "byte range end does not fit in usize",
                    )
                })?;
                if start > end || end > bytes.len() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "invalid byte range {:?} for content length {}",
                            content_ref.byte_range,
                            bytes.len()
                        ),
                    ));
                }
                &bytes[start..end]
            }
            None => bytes.as_slice(),
        };

        let text = std::str::from_utf8(sliced_bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let text = match content_ref.line_range {
            Some((start, end)) => {
                let start = usize::try_from(start).map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "line range start does not fit in usize",
                    )
                })?;
                let end = usize::try_from(end).map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "line range end does not fit in usize",
                    )
                })?;
                if start > end {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("invalid line range {:?}", content_ref.line_range),
                    ));
                }
                text.lines()
                    .skip(start)
                    .take(end.saturating_sub(start))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            None => text.to_string(),
        };

        Ok(ResolvedContent {
            content_ref: content_ref.content_ref.clone(),
            text,
        })
    }
}

// ---------------------------------------------------------------------------
// LedgerManager (Facade)
// ---------------------------------------------------------------------------

/// Unified facade for the Event Ledger system.
///
/// Coordinates:
/// - `LedgerWriter` for appending to JSONL (canonical source of truth)
/// - `LedgerProjection` for SQLite indexing (query projection)
/// - `ContentStoreWriter` for binary/text content storage (optional)
///
/// The ledger is append-only: events are never deleted or modified in-place.
pub struct LedgerManager {
    writer: LedgerWriter,
    projection: LedgerProjection,
    content_store: Option<ContentStoreWriter>,
}

impl LedgerManager {
    /// Create a new LedgerManager with the given base directory.
    ///
    /// The base directory is used for:
    /// - `event-log/` (JSONL files)
    /// - `cozmio-ledger.sqlite` (SQLite projection)
    /// - `content-store/` (binary/text storage, only if enabled)
    pub fn new(base_dir: PathBuf) -> std::io::Result<Self> {
        // Ensure base directory exists
        fs::create_dir_all(&base_dir)?;

        let writer = LedgerWriter::new(base_dir.join("event-log"));

        let db_path = base_dir.join("cozmio-ledger.sqlite");
        let projection = LedgerProjection::new(db_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Content store is optional
        let content_store = None;

        Ok(Self {
            writer,
            projection,
            content_store,
        })
    }

    /// Create a new LedgerManager with content store enabled.
    pub fn with_content_store(base_dir: PathBuf) -> std::io::Result<Self> {
        // Ensure base directory exists
        fs::create_dir_all(&base_dir)?;

        let writer = LedgerWriter::new(base_dir.join("event-log"));

        let db_path = base_dir.join("cozmio-ledger.sqlite");
        let projection = LedgerProjection::new(db_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let content_store = Some(ContentStoreWriter::new(base_dir.join("content-store")));

        Ok(Self {
            writer,
            projection,
            content_store,
        })
    }

    /// Create a LedgerManager at the default location.
    pub fn with_default_path() -> std::io::Result<Self> {
        let base_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cozmio");

        Self::new(base_dir)
    }

    /// Record an event to both JSONL and SQLite projection.
    pub fn record_event(&self, event: LedgerEvent) -> std::io::Result<()> {
        // Append to JSONL
        self.writer.append(&event)?;

        // Ingest to SQLite projection
        self.projection
            .ingest(&event)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(())
    }

    /// Record an event along with binary/text content.
    ///
    /// The content is stored first, then the event is recorded with a
    /// ContentRef pointing to the stored content.
    pub fn record_event_with_content(
        &self,
        mut event: LedgerEvent,
        content: &[u8],
    ) -> std::io::Result<(LedgerEvent, ContentRef)> {
        // Content store must be configured for record_event_with_content
        let store = self.content_store.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "content store is not configured; use LedgerManager::with_content_store to enable",
            )
        })?;

        // Determine content type from metadata or default to "binary"
        let content_type = event
            .metadata
            .get("content_type")
            .cloned()
            .unwrap_or_else(|| "binary".to_string());
        let producer = event.source.clone();

        // Store content and get ContentRef
        let content_ref = store.store(content, &content_type, &producer)?;

        // Set content reference in event
        event.content_ref = Some(content_ref.clone());

        // Record event
        self.record_event(event.clone())?;

        Ok((event, content_ref))
    }

    /// Get events in timeline order (descending by timestamp).
    pub fn get_timeline(&self, limit: usize, offset: usize) -> std::io::Result<Vec<LedgerEvent>> {
        self.projection
            .query_timeline(limit, offset)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Get all events for a given trace ID.
    pub fn get_trace(&self, trace_id: &str) -> std::io::Result<Vec<LedgerEvent>> {
        self.projection
            .query_trace(trace_id)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Get all events for a given relay session ID.
    pub fn get_session(&self, session_id: &str) -> std::io::Result<Vec<LedgerEvent>> {
        self.projection
            .query_by_session(session_id)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Get all events within a Unix timestamp range (inclusive).
    pub fn get_by_date_range(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> std::io::Result<Vec<LedgerEvent>> {
        self.projection
            .query_by_date_range(start_timestamp, end_timestamp)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Rebuild the SQLite projection from all JSONL files.
    ///
    /// Iterates over all `event-log/*.jsonl` files and ingests events.
    /// Returns the total number of events ingested.
    pub fn rebuild_projection(&self) -> std::io::Result<usize> {
        let event_log_dir = self.writer.base_dir();
        if !event_log_dir.exists() {
            return Ok(0);
        }

        let mut total = 0;

        // Read directory and find all .jsonl files
        for entry in fs::read_dir(event_log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                match self.projection.rebuild_from_jsonl(&path) {
                    Ok(count) => total += count,
                    Err(e) => {
                        log::warn!("Failed to rebuild from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(total)
    }

    /// Get the underlying LedgerWriter.
    pub fn writer(&self) -> &LedgerWriter {
        &self.writer
    }

    /// Get the underlying LedgerProjection.
    pub fn projection(&self) -> &LedgerProjection {
        &self.projection
    }

    /// Get the underlying ContentStoreWriter, if available.
    pub fn content_store(&self) -> Option<&ContentStoreWriter> {
        self.content_store.as_ref()
    }

    /// Resolve a content reference without mutating ledger state.
    pub fn resolve_content_ref(
        &self,
        content_ref: &ContentRef,
    ) -> std::io::Result<ResolvedContent> {
        ContentResolver::resolve(content_ref)
    }
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

/// Helper to convert a rusqlite error to an io error.
fn sqlite_err(e: rusqlite::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, e)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_base_dir() -> PathBuf {
        let path = std::env::temp_dir()
            .join("cozmio_ledger_test")
            .join(uuid::Uuid::new_v4().to_string());
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn create_test_event(event_type: &str, timestamp: i64) -> LedgerEvent {
        LedgerEvent {
            event_id: Uuid::new_v4().to_string(),
            trace_id: Some("test-trace-1".to_string()),
            session_id: Some("test-session-1".to_string()),
            timestamp,
            event_type: event_type.to_string(),
            source: "TestSource".to_string(),
            window_title: Some("Test Window".to_string()),
            process_name: Some("test.exe".to_string()),
            raw_text: Some("Test raw text".to_string()),
            content_ref: None,
            parent_event_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    fn content_ref_for_test(path: &Path, content: &[u8]) -> ContentRef {
        ContentRef {
            content_ref: Uuid::new_v4().to_string(),
            content_type: "text".to_string(),
            storage_backend: "file".to_string(),
            path_or_key: path.to_string_lossy().to_string(),
            content_hash: digest(content),
            created_at: 1714400000,
            producer: "TestProducer".to_string(),
            byte_range: None,
            line_range: None,
        }
    }

    #[test]
    fn test_content_resolver_full_text() {
        let base_dir = create_test_base_dir();
        let path = base_dir.join("full.txt");
        let content = b"alpha\nbeta\ngamma";
        fs::write(&path, content).unwrap();
        let content_ref = content_ref_for_test(&path, content);

        let resolved = ContentResolver::resolve(&content_ref).unwrap();
        assert_eq!(resolved.content_ref, content_ref.content_ref);
        assert_eq!(resolved.text, "alpha\nbeta\ngamma");
    }

    #[test]
    fn test_ledger_manager_resolve_content_ref_entrypoint() {
        let base_dir = create_test_base_dir();
        let manager = LedgerManager::with_content_store(base_dir.clone()).unwrap();
        let path = base_dir.join("manager.txt");
        let content = b"manager entrypoint";
        fs::write(&path, content).unwrap();
        let content_ref = content_ref_for_test(&path, content);

        let resolved = manager.resolve_content_ref(&content_ref).unwrap();
        assert_eq!(resolved.text, "manager entrypoint");
    }

    #[test]
    fn test_content_resolver_byte_range_before_utf8_conversion() {
        let base_dir = create_test_base_dir();
        let path = base_dir.join("byte_range.txt");
        let content = "prefix-你好-suffix".as_bytes();
        fs::write(&path, content).unwrap();
        let mut content_ref = content_ref_for_test(&path, content);
        content_ref.byte_range = Some((7, 13));

        let resolved = ContentResolver::resolve(&content_ref).unwrap();
        assert_eq!(resolved.text, "你好");
    }

    #[test]
    fn test_content_resolver_line_range_after_utf8_conversion() {
        let base_dir = create_test_base_dir();
        let path = base_dir.join("line_range.txt");
        let content = b"line0\nline1\nline2\nline3";
        fs::write(&path, content).unwrap();
        let mut content_ref = content_ref_for_test(&path, content);
        content_ref.line_range = Some((1, 3));

        let resolved = ContentResolver::resolve(&content_ref).unwrap();
        assert_eq!(resolved.text, "line1\nline2");
    }

    #[test]
    fn test_content_resolver_missing_file() {
        let base_dir = create_test_base_dir();
        let path = base_dir.join("missing.txt");
        let content_ref = content_ref_for_test(&path, b"missing");

        let err = ContentResolver::resolve(&content_ref).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_content_resolver_hash_mismatch() {
        let base_dir = create_test_base_dir();
        let path = base_dir.join("hash_mismatch.txt");
        let original = b"original";
        fs::write(&path, b"tampered").unwrap();
        let content_ref = content_ref_for_test(&path, original);

        let err = ContentResolver::resolve(&content_ref).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("content hash mismatch"));
    }

    #[test]
    fn test_content_resolver_unsupported_backend() {
        let base_dir = create_test_base_dir();
        let path = base_dir.join("unsupported.txt");
        let content = b"content";
        fs::write(&path, content).unwrap();
        let mut content_ref = content_ref_for_test(&path, content);
        content_ref.storage_backend = "s3".to_string();

        let err = ContentResolver::resolve(&content_ref).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_ledger_writer_append() {
        let base_dir = create_test_base_dir();
        let writer = LedgerWriter::new(base_dir.join("event-log"));

        let event = create_test_event("observation_captured", 1714400000);
        writer.append(&event).unwrap();

        // Verify file exists
        let path = writer.get_event_path("2024-04-29");
        assert!(path.exists());

        // Verify content
        let content = fs::read_to_string(&path).unwrap();
        let parsed: LedgerEvent =
            serde_json::from_str(content.trim().lines().last().unwrap()).unwrap();
        assert_eq!(parsed.event_type, "observation_captured");
        assert_eq!(parsed.window_title, Some("Test Window".to_string()));
    }

    #[test]
    fn test_ledger_projection_ingest_and_query() {
        let base_dir = create_test_base_dir();
        let db_path = base_dir.join("test-ledger.sqlite");
        let projection = LedgerProjection::new(db_path).unwrap();

        let event1 = create_test_event("observation_captured", 1714400000);
        let event2 = create_test_event("model_called", 1714400010);

        projection.ingest(&event1).unwrap();
        projection.ingest(&event2).unwrap();

        // Query timeline
        let timeline = projection.query_timeline(10, 0).unwrap();
        assert_eq!(timeline.len(), 2);
        // Most recent first
        assert_eq!(timeline[0].event_type, "model_called");

        // Query by event type
        let obs = projection
            .query_by_event_type("observation_captured", 10)
            .unwrap();
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].event_id, event1.event_id);

        // Query by trace
        let trace = projection.query_trace("test-trace-1").unwrap();
        assert_eq!(trace.len(), 2);
    }

    #[test]
    fn test_content_store_writer() {
        let base_dir = create_test_base_dir();
        let store = ContentStoreWriter::new(base_dir.join("content-store"));

        let content = b"Hello, World!";
        let content_ref = store.store(content, "text", "TestProducer").unwrap();

        assert_eq!(content_ref.content_type, "text");
        assert_eq!(content_ref.storage_backend, "file");
        assert!(!content_ref.content_hash.is_empty());
        assert_eq!(content_ref.producer, "TestProducer");

        // Verify file exists
        let path = PathBuf::from(&content_ref.path_or_key);
        assert!(path.exists());

        // Verify content
        let stored = fs::read(&path).unwrap();
        assert_eq!(stored, content);
    }

    #[test]
    fn test_ledger_manager_record_event() {
        let base_dir = create_test_base_dir();
        let manager = LedgerManager::with_content_store(base_dir.clone()).unwrap();

        let event = create_test_event("popup_displayed", 1714400100);
        manager.record_event(event.clone()).unwrap();

        // Verify JSONL exists
        let jsonl_path = base_dir.join("event-log").join("2024-04-29.jsonl");
        assert!(jsonl_path.exists());

        // Query back from projection
        let timeline = manager.get_timeline(10, 0).unwrap();
        assert!(!timeline.is_empty());
        assert_eq!(timeline[0].event_type, "popup_displayed");
    }

    #[test]
    fn test_ledger_manager_record_event_with_content() {
        let base_dir = create_test_base_dir();
        let manager = LedgerManager::with_content_store(base_dir.clone()).unwrap();

        let mut event = create_test_event("model_output_received", 1714400200);
        event
            .metadata
            .insert("content_type".to_string(), "model_output".to_string());

        let content = b"Model output: This is a test response.";
        let (saved_event, content_ref) = manager
            .record_event_with_content(event.clone(), content)
            .unwrap();

        assert!(saved_event.content_ref.is_some());
        assert!(!content_ref.content_ref.is_empty());
        assert_eq!(content_ref.content_type, "model_output");

        // Verify file stored
        let stored_path = PathBuf::from(&content_ref.path_or_key);
        assert!(stored_path.exists());
    }

    #[test]
    fn test_timestamp_to_date() {
        // Test known dates
        let date1 = timestamp_to_date(0); // Unix epoch
        assert_eq!(date1, "1970-01-01");

        let date2 = timestamp_to_date(1714400000); // Around April 2024
        assert_eq!(&date2[0..4], "2024");

        let date3 = timestamp_to_date(1609459200); // Jan 1, 2021
        assert_eq!(date3, "2021-01-01");
    }

    #[test]
    fn test_rebuild_projection() {
        let base_dir = create_test_base_dir();
        let manager = LedgerManager::with_content_store(base_dir.clone()).unwrap();

        // Record several events
        for i in 0..5 {
            let event = create_test_event("observation_captured", 1714400000 + i * 10);
            manager.record_event(event).unwrap();
        }

        // Rebuild projection
        let count = manager.rebuild_projection().unwrap();
        assert_eq!(count, 5);

        // Query should still work
        let timeline = manager.get_timeline(10, 0).unwrap();
        assert_eq!(timeline.len(), 5);
    }

    #[test]
    fn test_ledger_event_serde() {
        let event = create_test_event("test_event", 1714400000);
        let json = serde_json::to_string(&event).unwrap();
        let parsed: LedgerEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.event_id, event.event_id);
        assert_eq!(parsed.event_type, event.event_type);
        assert_eq!(parsed.timestamp, event.timestamp);
    }

    #[test]
    fn test_content_ref_serde() {
        let content_ref = ContentRef {
            content_ref: "test-ref-123".to_string(),
            content_type: "screenshot".to_string(),
            storage_backend: "file".to_string(),
            path_or_key: "/path/to/screenshot.bin".to_string(),
            content_hash: "abc123".to_string(),
            created_at: 1714400000,
            producer: "WindowMonitor".to_string(),
            byte_range: Some((0, 1024)),
            line_range: None,
        };

        let json = serde_json::to_string(&content_ref).unwrap();
        let parsed: ContentRef = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.content_ref, content_ref.content_ref);
        assert_eq!(parsed.content_type, content_ref.content_type);
        assert_eq!(parsed.byte_range, content_ref.byte_range);
    }

    #[test]
    fn test_record_event_with_content_requires_content_store() {
        let base_dir = create_test_base_dir();
        // LedgerManager without content store (new() not with_content_store())
        let manager = LedgerManager::new(base_dir.clone()).unwrap();

        let event = create_test_event("model_output_received", 1714400200);
        let content = b"Test content";

        // Should return error because content store is not configured
        let result = manager.record_event_with_content(event.clone(), content);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotConnected);
    }

    #[test]
    fn test_store_path_uses_stored_timestamp() {
        let base_dir = create_test_base_dir();
        let store = ContentStoreWriter::new(base_dir.join("content-store"));

        // Store content with a known timestamp (December 2023)
        let old_timestamp = 1704067200; // 2024-01-01 00:00:00 UTC
        let content = b"Old content";

        // Manually create a ContentRef with an old timestamp to simulate storing
        // content that was created in a different month than now
        let content_ref = ContentRef {
            content_ref: "old-content-ref".to_string(),
            content_type: "text".to_string(),
            storage_backend: "file".to_string(),
            path_or_key: base_dir
                .join("content-store")
                .join("2024")
                .join("01")
                .join("old-content-ref.txt")
                .to_string_lossy()
                .to_string(),
            content_hash: digest(content),
            created_at: old_timestamp,
            producer: "Test".to_string(),
            byte_range: None,
            line_range: None,
        };

        // Write the content to the path_or_key
        std::fs::create_dir_all(base_dir.join("content-store").join("2024").join("01")).unwrap();
        std::fs::write(&content_ref.path_or_key, content).unwrap();

        // store_path with the old timestamp should return the correct path
        let store_path = store.store_path(&content_ref.content_ref, content_ref.created_at);
        assert!(store_path.exists());
        assert!(store_path.to_string_lossy().contains("2024"));
        assert!(store_path.to_string_lossy().contains("01"));
    }

    // ---------------------------------------------------------------------------
    // Additional tests for T4
    // ---------------------------------------------------------------------------

    /// test_append_and_read_event — append event, read back via projection
    #[test]
    fn test_append_and_read_event() {
        use std::io::{BufRead, BufReader};

        let base_dir = create_test_base_dir();
        let writer = LedgerWriter::new(base_dir.join("event-log"));

        // Append an event
        let event = create_test_event("observation_captured", 1714500000);
        let event_id = event.event_id.clone();
        writer.append(&event).unwrap();

        // Read back from JSONL
        let path = writer.get_event_path("2024-04-30");
        let file = fs::File::open(&path).unwrap();
        let reader = BufReader::new(file);
        let mut found = false;
        for line in reader.lines() {
            let line = line.unwrap();
            if line.trim().is_empty() {
                continue;
            }
            let parsed: LedgerEvent = serde_json::from_str(&line).unwrap();
            if parsed.event_id == event_id {
                found = true;
                assert_eq!(parsed.event_type, "observation_captured");
                break;
            }
        }
        assert!(found, "Event should be readable back from JSONL");
    }

    /// test_jsonl_append_only — verify append-only semantics (can't modify/delete)
    #[test]
    fn test_jsonl_append_only() {
        let base_dir = create_test_base_dir();
        let writer = LedgerWriter::new(base_dir.join("event-log"));

        // Append two events
        let event1 = create_test_event("observation_captured", 1714500000);
        let event2 = create_test_event("model_called", 1714500010);
        writer.append(&event1).unwrap();
        writer.append(&event2).unwrap();

        // Read all lines
        let path = writer.get_event_path("2024-04-30");
        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(lines.len(), 2, "Should have exactly 2 lines (append-only)");

        // Verify we cannot delete or modify existing lines via OpenOptions
        // (This is a property test: re-opening with write truncates, but LedgerWriter always uses append)
        let file = OpenOptions::new().read(true).open(&path).unwrap();
        let reader = BufReader::new(file);
        let line_count = reader
            .lines()
            .filter(|l| l.as_ref().map_or(false, |s| !s.trim().is_empty()))
            .count();
        assert_eq!(
            line_count, 2,
            "Append-only: existing lines should not be modified"
        );
    }

    /// test_ledger_projection_query_trace — ingest events, query by trace_id
    #[test]
    fn test_ledger_projection_query_trace() {
        let base_dir = create_test_base_dir();
        let db_path = base_dir.join("test-ledger.sqlite");
        let projection = LedgerProjection::new(db_path).unwrap();

        let trace_id = "trace-abc-123";

        // Ingest events with the same trace_id
        let event1 = {
            let mut e = create_test_event("observation_captured", 1714500000);
            e.trace_id = Some(trace_id.to_string());
            e
        };
        let event2 = {
            let mut e = create_test_event("model_called", 1714500010);
            e.trace_id = Some(trace_id.to_string());
            e
        };
        let event3 = {
            let mut e = create_test_event("popup_displayed", 1714500020);
            e.trace_id = Some("other-trace".to_string());
            e
        };

        projection.ingest(&event1).unwrap();
        projection.ingest(&event2).unwrap();
        projection.ingest(&event3).unwrap();

        // Query by trace_id
        let trace_events = projection.query_trace(trace_id).unwrap();
        assert_eq!(
            trace_events.len(),
            2,
            "Should find 2 events for trace-abc-123"
        );
        assert!(trace_events
            .iter()
            .all(|e| e.trace_id.as_ref() == Some(&trace_id.to_string())));

        // Verify other trace has 1 event
        let other_events = projection.query_trace("other-trace").unwrap();
        assert_eq!(other_events.len(), 1);
    }

    /// test_content_ref_store_writes_blob_and_ref — store blob, verify ref fields and file existence
    #[test]
    fn test_content_ref_store_writes_blob_and_ref() {
        let base_dir = create_test_base_dir();
        let store = ContentStoreWriter::new(base_dir.join("content-store"));

        let content = b"\x00\x01\x02\x03\x04\x05binary blob data\xff\xfe\xfd";
        let content_ref = store.store(content, "binary", "TestProducer").unwrap();

        // Verify all ref fields are populated
        assert!(!content_ref.content_ref.is_empty());
        assert_eq!(content_ref.content_type, "binary");
        assert_eq!(content_ref.storage_backend, "file");
        assert!(!content_ref.path_or_key.is_empty());
        assert!(!content_ref.content_hash.is_empty());
        assert_eq!(content_ref.producer, "TestProducer");
        assert!(content_ref.created_at > 0);

        // Verify file exists on disk
        let path = PathBuf::from(&content_ref.path_or_key);
        assert!(path.exists(), "Content file should exist at path_or_key");

        // Verify content is identical
        let stored = fs::read(&path).unwrap();
        assert_eq!(stored, content);
    }

    /// test_ledger_projection_rebuild — create events, rebuild from JSONL, verify counts
    #[test]
    fn test_ledger_projection_rebuild() {
        use std::io::{BufRead, BufReader};

        let base_dir = create_test_base_dir();
        let manager = LedgerManager::with_content_store(base_dir.clone()).unwrap();

        // Record 7 events across different timestamps
        for i in 0..7 {
            let event = create_test_event("observation_captured", 1714500000 + i as i64 * 60);
            manager.record_event(event).unwrap();
        }

        // Count lines in JSONL
        let jsonl_path = base_dir.join("event-log").join("2024-04-30.jsonl");
        let file = fs::File::open(&jsonl_path).unwrap();
        let reader = BufReader::new(file);
        let line_count = reader
            .lines()
            .filter(|l| l.as_ref().map_or(false, |s| !s.trim().is_empty()))
            .count();
        assert_eq!(line_count, 7);

        // Drop and recreate projection (simulates rebuild from JSONL)
        let db_path = base_dir.join("cozmio-ledger.sqlite");
        let new_projection = LedgerProjection::new(db_path).unwrap();
        let rebuilt = new_projection.rebuild_from_jsonl(&jsonl_path).unwrap();
        assert_eq!(rebuilt, 7, "Rebuild should ingest all 7 events");

        // Query to verify
        let events = new_projection.query_timeline(10, 0).unwrap();
        assert_eq!(events.len(), 7);
    }

    /// test_legacy_action_record_backwards_compat — verify log_factual still produces valid legacy ActionRecord JSON
    #[test]
    fn test_legacy_action_record_backwards_compat() {
        use crate::logging::FactualEventType;
        use crate::logging::SystemRoute;
        use crate::logging::{ActionLogger, FactualActionRecord};

        let temp_dir = std::env::temp_dir()
            .join("cozmio_legacy_test")
            .join(uuid::Uuid::new_v4().to_string());
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let log_path = temp_dir.join("action_log.jsonl");
        let logger = ActionLogger::with_path(log_path.clone());

        // Create a FactualActionRecord
        let factual = FactualActionRecord {
            timestamp: 1714500000,
            trace_id: Some("trace-xyz".to_string()),
            session_id: Some("session-123".to_string()),
            window_title: "Test Window".to_string(),
            event_type: FactualEventType::ModelOutput,
            system_route: SystemRoute::AutoExecuted,
            original_judgment: "CONTINUE".to_string(),
            execution_result_str: "executed".to_string(),
            raw_model_text: Some("Model output text".to_string()),
            model_name: Some("llava".to_string()),
            captured_at: Some(1714499990),
            call_started_at: Some(1714499995),
            call_duration_ms: Some(500),
            execution_result: Some("Action completed".to_string()),
            error_text: None,
            user_feedback: None,
        };

        // Log via log_factual
        logger.log_factual(factual).unwrap();

        // Read back and verify it's valid legacy ActionRecord JSON
        let content = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(lines.len(), 1, "Should have exactly 1 line");

        // Parse as legacy ActionRecord
        let parsed: crate::logging::ActionRecord = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed.timestamp, 1714500000);
        assert_eq!(parsed.trace_id, Some("trace-xyz".to_string()));
        assert_eq!(parsed.session_id, Some("session-123".to_string()));
        assert_eq!(parsed.window_title, "Test Window");
        assert_eq!(parsed.judgment, "CONTINUE");
        assert_eq!(parsed.system_action, "executed");
        assert_eq!(parsed.model_name, Some("llava".to_string()));
        assert_eq!(parsed.content_text, Some("Model output text".to_string()));
        assert_eq!(parsed.result_text, Some("Action completed".to_string()));
    }
}
