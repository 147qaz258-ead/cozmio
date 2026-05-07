use crate::context_slices::{ContextSlice, ContextSlicesStore};
use crate::db::Database;
use crate::error::MemoryError;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::HashMap;

/// Build context slices from all imported events in the database.
/// Clears existing generated slices first, then creates slices from all events
/// with evidence_source = 'imported'.
pub fn build_slices_from_events(db: &Database) -> Result<usize, MemoryError> {
    // Fetch all imported events ordered by timestamp
    let events: Vec<_> = {
        let conn = db.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, source, window_title, content, raw_ref, embedding, thread_id, evidence_source
             FROM memory_events WHERE evidence_source = 'imported' ORDER BY timestamp ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::memory_events::MemoryEvent {
                id: Some(row.get(0)?),
                timestamp: row.get(1)?,
                source: row.get(2)?,
                window_title: row.get(3)?,
                content: row.get(4)?,
                raw_ref: row.get(5)?,
                embedding: row.get(6)?,
                thread_id: row.get(7)?,
                evidence_source: row.get(8)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(MemoryError::from)?
    };

    if events.is_empty() {
        return Ok(0);
    }

    // Clear existing generated slices
    let slices_store = ContextSlicesStore::new(db);
    {
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM context_slices WHERE evidence_source = 'generated'",
            [],
        )?;
    }

    // Group events into 15-minute windows
    let windows = group_into_windows(&events);

    let mut slice_count = 0;
    for (_window_key, window_events) in windows {
        let slice = build_slice_from_window(&window_events);
        if let Err(e) = slices_store.insert(&slice) {
            eprintln!("Failed to insert slice: {}", e);
        } else {
            slice_count += 1;
        }
    }

    Ok(slice_count)
}

/// Group events into 15-minute time windows.
/// Returns a map from window key (timestamp of window start) to events in that window.
fn group_into_windows(
    events: &[crate::memory_events::MemoryEvent],
) -> HashMap<i64, Vec<&crate::memory_events::MemoryEvent>> {
    let mut windows: HashMap<i64, Vec<&crate::memory_events::MemoryEvent>> = HashMap::new();

    for event in events {
        // Parse timestamp - try RFC3339 first, then fall back to parsing
        let window_key = parse_timestamp_to_window_key(&event.timestamp);
        windows.entry(window_key).or_default().push(event);
    }

    windows
}

/// Parse a timestamp string and return the 15-minute window key (epoch seconds rounded down to 15 min).
fn parse_timestamp_to_window_key(timestamp: &str) -> i64 {
    // Try RFC3339 format first
    if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
        return floor_to_15_min(dt.timestamp());
    }

    // Try naive datetime formats
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M",
    ];

    for fmt in &formats {
        if let Ok(ndt) = NaiveDateTime::parse_from_str(timestamp, fmt) {
            let dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(ndt, Utc);
            return floor_to_15_min(dt.timestamp());
        }
    }

    // Fallback: try parsing as integer timestamp
    if let Ok(ts) = timestamp.parse::<i64>() {
        return floor_to_15_min(ts);
    }

    // Default to current time floored to 15 min
    floor_to_15_min(Utc::now().timestamp())
}

/// Floor a timestamp to the nearest 15-minute boundary.
fn floor_to_15_min(timestamp: i64) -> i64 {
    (timestamp / 900) * 900
}

/// Build a ContextSlice from a group of events in the same time window.
fn build_slice_from_window(events: &[&crate::memory_events::MemoryEvent]) -> ContextSlice {
    // Calculate window boundaries
    let timestamps: Vec<&str> = events.iter().map(|e| e.timestamp.as_str()).collect();
    let (start_time, end_time) = compute_window_bounds(&timestamps);

    // Collect window titles with counts
    let mut title_counts: HashMap<String, usize> = HashMap::new();
    for event in events {
        if let Some(ref title) = event.window_title {
            if !title.is_empty() {
                *title_counts.entry(title.clone()).or_default() += 1;
            }
        }
    }

    // Extract mechanical tokens from stored factual event content.
    let keywords = extract_keywords(events);

    // Build summary
    let summary = build_summary(events.len(), &title_counts, &keywords);

    // Get top 5 window titles as entities
    let entities: Vec<String> = top_n_by_count(&title_counts, 5)
        .into_iter()
        .map(|(title, _)| title)
        .collect();

    // Get top 10 keywords as topics
    let topics: Vec<String> = keywords.into_iter().take(10).collect();

    // Collect event IDs as raw_refs
    let raw_refs: Vec<i64> = events.iter().filter_map(|e| e.id).collect();

    ContextSlice {
        id: None,
        start_time,
        end_time,
        summary,
        entities,
        topics,
        raw_refs,
        evidence_source: "generated".to_string(),
    }
}

/// Compute window start and end times from a list of timestamp strings.
fn compute_window_bounds(timestamps: &[&str]) -> (String, String) {
    let mut min_ts: Option<i64> = None;
    let mut max_ts: i64 = 0;

    for ts_str in timestamps {
        let key = parse_timestamp_to_window_key(ts_str);
        if let Some(min) = min_ts {
            if key < min {
                min_ts = Some(key);
            }
        } else {
            min_ts = Some(key);
        }
        if key > max_ts {
            max_ts = key;
        }
    }

    let start = min_ts.unwrap_or(Utc::now().timestamp());
    let end = start + 900 - 1; // 15 minutes - 1 second

    let start_dt = DateTime::from_timestamp(start, 0).unwrap_or_else(|| Utc::now());
    let end_dt = DateTime::from_timestamp(end, 0).unwrap_or_else(|| Utc::now());

    (start_dt.to_rfc3339(), end_dt.to_rfc3339())
}

/// Extract keywords from stored factual fields of events.
fn extract_keywords(events: &[&crate::memory_events::MemoryEvent]) -> Vec<String> {
    let mut word_counts: HashMap<String, usize> = HashMap::new();

    for event in events {
        let text = extract_factual_text(&event.content);
        if text.is_empty() {
            continue;
        }

        // Tokenize - split on whitespace, punctuation, and common delimiters
        for word in tokenize(&text) {
            let word = word.to_lowercase();
            if word.len() >= 2 && !is_stop_word(&word) {
                *word_counts.entry(word).or_default() += 1;
            }
        }
    }

    // Sort by count descending, then alphabetically
    let mut sorted: Vec<(String, usize)> = word_counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    sorted.into_iter().map(|(word, _)| word).collect()
}

/// Extract stored factual text fields from imported event content.
fn extract_factual_text(content: &str) -> String {
    let mut parts = Vec::new();
    for part in content.split('|') {
        let part = part.trim();
        if part.starts_with("content:")
            || part.starts_with("result:")
            || part.starts_with("error:")
            || part.starts_with("feedback:")
            || part.starts_with("system_action:")
        {
            parts.push(part.split(':').nth(1).unwrap_or("").trim().to_string());
        }
    }
    parts.join(" ")
}

/// Tokenize text into words.
fn tokenize(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for c in text.chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            current.push(c);
        } else if !current.is_empty() {
            words.push(current.clone());
            current.clear();
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

/// Check if a word is a common stop word that should be filtered out.
fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "the"
            | "a"
            | "an"
            | "and"
            | "or"
            | "but"
            | "in"
            | "on"
            | "at"
            | "to"
            | "for"
            | "of"
            | "with"
            | "by"
            | "from"
            | "as"
            | "is"
            | "was"
            | "are"
            | "were"
            | "been"
            | "be"
            | "have"
            | "has"
            | "had"
            | "do"
            | "does"
            | "did"
            | "will"
            | "would"
            | "should"
            | "could"
            | "may"
            | "might"
            | "must"
            | "can"
            | "this"
            | "that"
            | "these"
            | "those"
            | "it"
            | "its"
            | "they"
            | "them"
            | "their"
            | "what"
            | "which"
            | "who"
            | "when"
            | "where"
            | "why"
            | "how"
            | "all"
            | "each"
            | "every"
            | "both"
            | "few"
            | "more"
            | "most"
            | "other"
            | "some"
            | "such"
            | "no"
            | "not"
            | "only"
            | "same"
            | "so"
            | "than"
            | "too"
            | "very"
            | "just"
            | "also"
            | "now"
            | "here"
            | "there"
            | "then"
    )
}

/// Get top N items by count.
fn top_n_by_count(counts: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    let mut sorted: Vec<(String, usize)> = counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sorted.into_iter().take(n).collect()
}

/// Build a deterministic summary string for a window.
fn build_summary(
    event_count: usize,
    title_counts: &HashMap<String, usize>,
    keywords: &[String],
) -> String {
    let mut parts = Vec::new();

    // Event count
    parts.push(format!("{}个事件", event_count));

    // Top window titles
    let top_titles = top_n_by_count(title_counts, 5);
    if !top_titles.is_empty() {
        let title_strs: Vec<String> = top_titles
            .iter()
            .map(|(title, count)| format!("{}({}次)", title, count))
            .collect();
        parts.push(format!("窗口标题：{}", title_strs.join(", ")));
    }

    // Top keywords
    if !keywords.is_empty() {
        let keyword_sample: Vec<String> = keywords.iter().take(10).cloned().collect();
        parts.push(format!("关键词：{}", keyword_sample.join(", ")));
    }

    parts.join("。")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_events::{MemoryEvent, MemoryEventsStore};
    use tempfile::tempdir;

    #[test]
    fn test_parse_timestamp_to_window_key() {
        // Test RFC3339 format
        let ts = "2024-01-15T10:23:45Z";
        let key = parse_timestamp_to_window_key(ts);
        // Should floor to 10:15:00 (10:23:45 -> 10:15:00)
        assert_eq!(key % 900, 0);

        // Test integer timestamp
        let key2 = parse_timestamp_to_window_key("1705315000");
        assert_eq!(key2 % 900, 0);
    }

    #[test]
    fn test_is_stop_word() {
        assert!(is_stop_word("the"));
        assert!(is_stop_word("is"));
        assert!(is_stop_word("and"));
        assert!(!is_stop_word("error"));
        assert!(!is_stop_word("continue"));
    }

    #[test]
    fn test_tokenize() {
        let words = tokenize("content: raw model text | error: error occurred");
        assert!(words.contains(&"content".to_string()));
        assert!(words.contains(&"error".to_string()));
    }

    #[test]
    fn test_build_slices_from_events() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_slices.db");

        let db = Database::new(&db_path).unwrap();
        crate::run_migrations(&db).unwrap();

        // Insert test events
        let store = MemoryEventsStore::new(&db);
        let base_time = DateTime::parse_from_rfc3339("2024-01-15T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        for i in 0..5 {
            let event = MemoryEvent {
                id: None,
                timestamp: (base_time + chrono::Duration::minutes(i * 2)).to_rfc3339(),
                source: "test".to_string(),
                window_title: Some(format!("Test Window {}", i % 2)),
                content: format!("content: test{} | feedback: test feedback", i),
                raw_ref: None,
                embedding: None,
                thread_id: None,
                evidence_source: "imported".to_string(),
            };
            store.insert(&event).unwrap();
        }

        // Build slices
        let count = build_slices_from_events(&db).unwrap();
        assert_eq!(count, 1); // All 5 events should fit in one 15-min window

        // Verify slice was created
        let slices_store = ContextSlicesStore::new(&db);
        let slices = slices_store.get_recent(10).unwrap();
        assert_eq!(slices.len(), 1);
        assert_eq!(slices[0].evidence_source, "generated");
        assert!(!slices[0].summary.is_empty());
        assert_eq!(slices[0].raw_refs.len(), 5);
    }

    #[test]
    fn test_multiple_windows() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_windows.db");

        let db = Database::new(&db_path).unwrap();
        crate::run_migrations(&db).unwrap();

        let store = MemoryEventsStore::new(&db);
        let base_time = DateTime::parse_from_rfc3339("2024-01-15T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        // Insert events spread across ~45 minutes (should create 3 windows)
        for i in 0..9 {
            let event = MemoryEvent {
                id: None,
                timestamp: (base_time + chrono::Duration::minutes(i * 5)).to_rfc3339(),
                source: "test".to_string(),
                window_title: Some("Test".to_string()),
                content: format!(
                    "content: step{} | feedback: ground{}",
                    i, i
                ),
                raw_ref: None,
                embedding: None,
                thread_id: None,
                evidence_source: "imported".to_string(),
            };
            store.insert(&event).unwrap();
        }

        let count = build_slices_from_events(&db).unwrap();
        assert_eq!(count, 3); // 3 windows of 15 minutes each

        // Verify all slices have evidence_source = "generated"
        let slices_store = ContextSlicesStore::new(&db);
        let slices = slices_store.get_recent(10).unwrap();
        for slice in slices {
            assert_eq!(slice.evidence_source, "generated");
        }
    }

    #[test]
    fn test_build_summary() {
        let mut titles = HashMap::new();
        titles.insert("Notepad".to_string(), 5);
        titles.insert("Chrome".to_string(), 3);

        let keywords = vec![
            "error".to_string(),
            "continue".to_string(),
            "confirm".to_string(),
        ];

        let summary = build_summary(5, &titles, &keywords);
        assert!(summary.contains("5个事件"));
        assert!(summary.contains("Notepad"));
        assert!(summary.contains("error"));
    }
}
