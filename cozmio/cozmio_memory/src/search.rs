use std::sync::Arc;

use crate::context_slices::ContextSlicesStore;
use crate::db::Database;
use crate::embed_provider::EmbeddingProvider;
use crate::error::MemoryError;
use crate::memory_events::MemoryEventsStore;
use crate::vec_store::InMemoryVecStore;
use rusqlite::params;

/// Expand query with aliases for better recall.
/// Handles Chinese keyword mapping and English synonym expansion.
fn expand_query(text: &str) -> Vec<String> {
    let mut queries = vec![text.to_string()];

    // Chinese alias map (simplified for H1)
    let aliases = vec![
        // "提醒没有帮助" → related queries
        (
            "提醒",
            vec!["toast", "通知", "弹窗", "confirm", "continue", "judgment"],
        ),
        ("帮助", vec!["help", "useful", "abstain", "skip"]),
        // "不要牺牲体验" → related concepts
        (
            "体验",
            vec!["experience", "UX", "终局", "quality", "end state"],
        ),
        ("方便", vec!["convenient", "easy", "simple", "quick"]),
        (
            "牺牲",
            vec!["sacrifice", "compromise", "tradeoff", "prefer"],
        ),
        // "硬件盒执行链路" → related terms
        ("硬件", vec!["hardware", "box", "device", "pi", "raspberry"]),
        ("盒", vec!["box", "device", "unit"]),
        ("执行", vec!["execute", "run", "dispatch", "relay"]),
        ("链路", vec!["chain", "link", "path", "flow"]),
        // "Toast" variants
        ("toast", vec!["通知", "弹窗", "提醒", "notification"]),
        // "CONTINUE/ABSTAIN" content context
        ("continue", vec!["继续", "执行", "dispatch", "confirm"]),
        ("abstain", vec!["跳过", "skip", "neutral", "无信号"]),
        ("error", vec!["错误", "失败", "fail", "model_error"]),
    ];

    // Check if query contains any alias trigger
    let text_lower = text.to_lowercase();
    for (trigger, expansions) in &aliases {
        if text_lower.contains(&trigger.to_lowercase()) {
            for exp in expansions {
                queries.push(exp.to_string());
            }
        }
    }

    queries
}

/// Sanitize a single FTS5 query term by escaping special characters
fn sanitize_term(term: &str) -> String {
    // FTS5 special characters: " - ( ) * : ^
    let special_chars = ['"', '-', '(', ')', '*', ':', '^', '{', '}', '[', ']', '~'];
    let mut result = term.to_string();
    for c in special_chars {
        result = result.replace(c, " ");
    }
    result.trim().to_string()
}

/// Search query with optional filters
pub struct SearchQuery {
    /// Keyword or semantic query text
    pub text: Option<String>,
    /// ISO8601 time range filter (start, end)
    pub time_range: Option<(String, String)>,
    /// Thread ID filter (references task_threads.id)
    pub thread_id: Option<i64>,
    /// Maximum results to return
    pub limit: usize,
}

/// A single search result entry
pub struct SearchResult {
    pub event_id: i64,
    pub score: f32,
    pub source: String,
    pub content: String,
    pub window_title: Option<String>,
    pub timestamp: String,
    pub evidence_source: String,
    pub thread_id: Option<i64>,
}

/// Combined search results
pub struct SearchResults {
    pub events: Vec<SearchResult>,
    pub total_fts: usize,
    pub total_vec: usize,
}

/// Hybrid search engine: FTS5 + vector semantic + metadata filtering
pub struct SearchEngine<'a> {
    db: &'a Database,
    embed_provider: Option<Arc<dyn EmbeddingProvider>>,
    dimension: usize,
}

impl<'a> SearchEngine<'a> {
    pub fn new(db: &'a Database, embed_provider: Option<Arc<dyn EmbeddingProvider>>) -> Self {
        let dimension = embed_provider
            .as_ref()
            .map(|p| p.dimension())
            .unwrap_or(384);
        Self {
            db,
            embed_provider,
            dimension,
        }
    }

    /// Perform hybrid search: FTS5 keyword + vector semantic + metadata filters
    pub fn search(&self, query: &SearchQuery) -> Result<SearchResults, MemoryError> {
        let mut all_events: Vec<SearchResult> = Vec::new();
        let mut total_fts = 0;
        let mut total_vec = 0;

        // 1. FTS5 keyword search
        if let Some(ref text) = query.text {
            let expanded = expand_query(text);
            // Use OR expansion if we have aliases, otherwise use original text
            let fts_query = if expanded.len() > 1 {
                // Build OR query from expanded terms
                let sanitized: Vec<String> = expanded
                    .iter()
                    .map(|s| sanitize_term(s))
                    .filter(|s| !s.is_empty())
                    .collect();
                sanitized.join(" OR ")
            } else {
                text.clone()
            };
            let fts_results = self.search_fts(&fts_query, query.thread_id, query.limit * 2)?;
            total_fts = fts_results.len();
            all_events.extend(fts_results);
        }

        // 2. Vector semantic search (if provider available and we have text)
        if let (Some(ref text), Some(ref provider)) = (&query.text, &self.embed_provider) {
            if provider.is_available() {
                if let Ok(query_vec) = provider.embed(text) {
                    let vec_results =
                        self.search_vector(&query_vec, query.thread_id, query.limit * 2)?;
                    total_vec = vec_results.len();
                    // Merge with FTS results: avoid duplicates, keep highest score
                    for vr in vec_results {
                        if !all_events.iter().any(|e| e.event_id == vr.event_id) {
                            all_events.push(vr);
                        }
                    }
                }
            }
        }

        // 3. Apply time_range filter
        if let Some((ref start, ref end)) = query.time_range {
            all_events.retain(|e| e.timestamp >= *start && e.timestamp <= *end);
        }

        // 4. Sort by score descending and limit
        all_events.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all_events.truncate(query.limit);

        Ok(SearchResults {
            events: all_events,
            total_fts,
            total_vec,
        })
    }

    /// FTS5 keyword search using bm25 ranking
    fn search_fts(
        &self,
        text: &str,
        thread_id: Option<i64>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, MemoryError> {
        let conn = self.db.conn.lock().unwrap();
        if let Some(tid) = thread_id {
            let mut stmt = conn.prepare(
                "SELECT m.id, m.timestamp, m.source, m.content, m.window_title,
                        bm25(memory_events_fts) as score, m.evidence_source, m.thread_id
                 FROM memory_events_fts fts
                 JOIN memory_events m ON fts.rowid = m.id
                 WHERE memory_events_fts MATCH ?1 AND m.thread_id = ?2
                 ORDER BY score DESC
                 LIMIT ?3",
            )?;
            let rows = stmt.query_map(params![text, tid, limit as i64], |row| {
                Ok(SearchResult {
                    event_id: row.get(0)?,
                    timestamp: row.get(1)?,
                    source: row.get(2)?,
                    content: row.get(3)?,
                    window_title: row.get(4)?,
                    score: row.get(5)?,
                    evidence_source: row.get(6)?,
                    thread_id: row.get(7)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(MemoryError::from)
        } else {
            let mut stmt = conn.prepare(
                "SELECT m.id, m.timestamp, m.source, m.content, m.window_title,
                        bm25(memory_events_fts) as score, m.evidence_source, m.thread_id
                 FROM memory_events_fts fts
                 JOIN memory_events m ON fts.rowid = m.id
                 WHERE memory_events_fts MATCH ?1
                 ORDER BY score DESC
                 LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![text, limit as i64], |row| {
                Ok(SearchResult {
                    event_id: row.get(0)?,
                    timestamp: row.get(1)?,
                    source: row.get(2)?,
                    content: row.get(3)?,
                    window_title: row.get(4)?,
                    score: row.get(5)?,
                    evidence_source: row.get(6)?,
                    thread_id: row.get(7)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(MemoryError::from)
        }
    }

    /// Vector semantic search using sqlite-vec (when feature="vec") or InMemoryVecStore fallback
    fn search_vector(
        &self,
        query_vec: &[f32],
        thread_id: Option<i64>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, MemoryError> {
        #[cfg(feature = "vec")]
        {
            // Use sqlite-vec for vector search
            let conn = self.db.conn.lock().unwrap();
            if let Some(tid) = thread_id {
                let mut stmt = conn.prepare(
                    "SELECT m.id, m.timestamp, m.source, m.content, m.window_title,
                            -vec_distance_cosine(embedding) as score, m.evidence_source, m.thread_id
                     FROM memory_events_vec
                     JOIN memory_events m ON memory_events_vec.rowid = m.id
                     WHERE embedding MATCH ?1 AND m.thread_id = ?2
                     ORDER BY score DESC
                     LIMIT ?3",
                )?;
                let rows = stmt.query_map(params![query_vec, tid, limit as i64], |row| {
                    Ok(SearchResult {
                        event_id: row.get(0)?,
                        timestamp: row.get(1)?,
                        source: row.get(2)?,
                        content: row.get(3)?,
                        window_title: row.get(4)?,
                        score: row.get(5)?,
                        evidence_source: row.get(6)?,
                        thread_id: row.get(7)?,
                    })
                })?;
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(MemoryError::from)
            } else {
                let mut stmt = conn.prepare(
                    "SELECT m.id, m.timestamp, m.source, m.content, m.window_title,
                            -vec_distance_cosine(embedding) as score, m.evidence_source, m.thread_id
                     FROM memory_events_vec
                     JOIN memory_events m ON memory_events_vec.rowid = m.id
                     WHERE embedding MATCH ?1
                     ORDER BY score DESC
                     LIMIT ?2",
                )?;
                let rows = stmt.query_map(params![query_vec, limit as i64], |row| {
                    Ok(SearchResult {
                        event_id: row.get(0)?,
                        timestamp: row.get(1)?,
                        source: row.get(2)?,
                        content: row.get(3)?,
                        window_title: row.get(4)?,
                        score: row.get(5)?,
                        evidence_source: row.get(6)?,
                        thread_id: row.get(7)?,
                    })
                })?;
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(MemoryError::from)
            }
        }
        #[cfg(not(feature = "vec"))]
        {
            // Fall back to InMemoryVecStore - load all vectors from DB and search in-memory
            // Collect stored embeddings from memory_events table (optionally filtered by thread_id)
            let embeddings: Vec<(i64, Vec<f32>)> = {
                let conn = self.db.conn.lock().unwrap();
                if let Some(tid) = thread_id {
                    let mut stmt = conn.prepare(
                        "SELECT id, embedding FROM memory_events WHERE embedding IS NOT NULL AND thread_id = ?1"
                    )?;
                    let rows = stmt.query_map(params![tid], |row| {
                        let id: i64 = row.get(0)?;
                        let embedding_blob: Option<Vec<u8>> = row.get(1)?;
                        Ok((id, embedding_blob))
                    })?;
                    rows.filter_map(|row| {
                        row.ok().and_then(|(id, blob)| {
                            blob.map(|b| {
                                let embedding: Vec<f32> = b
                                    .chunks_exact(4)
                                    .map(|chunk| {
                                        f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                                    })
                                    .collect();
                                (id, embedding)
                            })
                        })
                    })
                    .collect()
                } else {
                    let mut stmt = conn.prepare(
                        "SELECT id, embedding FROM memory_events WHERE embedding IS NOT NULL",
                    )?;
                    let rows = stmt.query_map([], |row| {
                        let id: i64 = row.get(0)?;
                        let embedding_blob: Option<Vec<u8>> = row.get(1)?;
                        Ok((id, embedding_blob))
                    })?;
                    rows.filter_map(|row| {
                        row.ok().and_then(|(id, blob)| {
                            blob.map(|b| {
                                let embedding: Vec<f32> = b
                                    .chunks_exact(4)
                                    .map(|chunk| {
                                        f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                                    })
                                    .collect();
                                (id, embedding)
                            })
                        })
                    })
                    .collect()
                }
            }; // conn and stmt dropped here

            if embeddings.is_empty() {
                return Ok(vec![]);
            }

            let mut in_mem_store = InMemoryVecStore::new(self.dimension);
            for (id, emb) in embeddings {
                in_mem_store.insert(id, emb);
            }

            // Search in-memory store
            let results = in_mem_store.search(query_vec, limit);
            if results.is_empty() {
                return Ok(vec![]);
            }

            // Fetch full event details for matched IDs
            let store = MemoryEventsStore::new(self.db);
            let mut all_results = Vec::new();
            for (event_id, score) in results {
                if let Ok(Some(event)) = store.get_by_id(event_id) {
                    all_results.push(SearchResult {
                        event_id,
                        score: 1.0 - score, // convert cosine distance to similarity
                        source: event.source,
                        content: event.content,
                        window_title: event.window_title,
                        timestamp: event.timestamp,
                        evidence_source: event.evidence_source,
                        thread_id: event.thread_id,
                    });
                }
            }
            Ok(all_results)
        }
    }

    /// Search context_slices by keyword match on summary
    pub fn search_slices(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::context_slices::ContextSlice>, MemoryError> {
        let store = ContextSlicesStore::new(self.db);
        let slices = store.get_recent(limit * 2)?;
        let query_lower = query.to_lowercase();
        let filtered: Vec<_> = slices
            .into_iter()
            .filter(|s| s.summary.to_lowercase().contains(&query_lower))
            .take(limit)
            .collect();
        Ok(filtered)
    }
}
