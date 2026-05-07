# Long-Term Memory Module Design

## Agent Design Reading

```
1. User signal:
用户说"我们需要一个长期记忆模块，你帮我设计一下"。
The request is about adding a long-term memory module to the existing cozmio cognitive desktop agent.

2. Target agent behavior:
The agent should remember cross-session facts, patterns, and decisions.
When a situation recurs or a related context is detected, relevant long-term memory
should be surfaced to the model as part of the reminder context — without requiring
re-import of old data.

3. Evidence available:
- Existing cozmio_memory crate: MemoryEventsStore, ContextSlicesStore, DecisionMemoryStore,
  SkillMemoryStore, TaskThreadsStore
- importer: imports action_log.jsonl into memory_events with evidence_source="imported"
- slice_builder: groups imported events into 15-minute ContextSlices
- competition: builds ReminderContext from recent slices + search + decisions + skills
- FTS5 virtual table on memory_events(content, window_title)
- Optional sqlite-vec for embedding vector search

4. Context needed:
Current system provides short-to-medium term memory (events within hours, generated slices).
What is missing is long-term memory that:
  a) persists across sessions (months, not hours)
  b) summarizes recurring patterns into reusable knowledge
  c) can be retrieved by semantic similarity or temporal proximity
  d) feeds into model judgment without noise

5. Model decision:
Where in the existing system does the model (or program) make a memory decision?
- MemoryCompetition::build_reminder_context assembles the reminder context.
  It relies on recent slices (last 2 hours) and search.
  Model judgment: which evidence to surface and how to format it for the prompt.
  The model does not yet have a concept of "this is a long-term pattern worth remembering."

6. Tools/actions:
- Write path: import events → build slices (already exists, time-windowed)
- Missing: an extraction step that takes slices or patterns and writes to a
  long-term store with semantic labels, not just raw events.
- Missing: a retrieval path that queries long-term memory by topical similarity
  and recency weight, surfacing it into ReminderContext.

7. Feedback loop:
Current: decision made → action logged → imported into events → sliced into context.
The feedback loop stops at the slice layer. There is no loop that says
"this situation has occurred before, here is what happened last time."

8. Verification gate:
- Import events and verify slices are created (existing test coverage)
- Verify that ReminderContext includes relevant past evidence when a recurring
  window_title is encountered
- Verify that old events (weeks old) can be retrieved and do not simply
  fall off the edge of the system

9. Current weakest layer:
The retrieval and synthesis layer. The system writes a lot but does not
summarize, distill, or prioritize across sessions. The 2-hour window for
context slices is too short for cross-session memory.
```

---

## Core Problem Statement

"长期记忆" is not one feature. It contains at least four design questions:

| # | Question | Current State | Gap |
|---|----------|---------------|-----|
| 1 | What should be remembered? | Events + slices + decisions + skills | No cross-session distillation |
| 2 | When should it be written? | On import, on slice building | No semantic compression step |
| 3 | When should it be retrieved? | 2-hour rolling window in competition | No semantic similarity retrieval |
| 4 | How does retrieved memory affect model judgment? | Via ReminderContext (already built) | But it only sees 2 hours of context |

The first design artifact should be the **memory lifecycle**, not the database schema.

---

## Proposed Memory Lifecycle

```
observe → candidate memory → confidence scoring → long-term write
                                                              ↓
prompt ← retrieval trigger ← semantic search ← relevance scoring
              ↑                                         ↑
         model judgment ← context assembly ← recent evidence + long-term evidence
```

### Lifecycle Detail

**Step 1: Candidate Memory Generation**
- After ContextSlices are built, a new process (`memory_distiller`) examines
  each slice and checks for cross-session patterns:
  - Same window_title appears in multiple sessions (>24h apart)
  - Same judgment appears repeatedly in similar contexts
  - A decision was made that contradicts a past decision in similar context
- Candidate memory is a structured note: "In situation X, the pattern was Y, and the outcome was Z."

**Step 2: Confidence Scoring**
- Programmatic scoring based on:
  - Frequency: how many times has this pattern appeared?
  - Recency: when was the most recent occurrence?
  - Distinctiveness: does this differentiate this agent's behavior from generic?
  - Conflict: does this contradict a higher-confidence memory?
- Threshold: memories with confidence >= T are written to `episodic_memory` table.
  Below threshold: retained as candidate, not surfaced.

**Step 3: Long-Term Write**
- Written to `episodic_memory` table:
  - `id`, `pattern_type`, `summary`, `confidence`, `first_seen`, `last_seen`,
    `occurrence_count`, `related_slice_ids`, `evidence_source`
- `pattern_type`: one of `recurring_situation`, `decision_pattern`, `skill_acquisition`, `error_signal`

**Step 4: Retrieval Trigger**
- At each `MemoryCompetition::build_reminder_context` call, after assembling
  short-term evidence, also run a semantic search against `episodic_memory`.
- Search query: `window_title + content_text` (same as event search)
- Additional filter: `last_seen > now - 30 days` (old memories decay out unless reinforced)

**Step 5: Relevance Scoring**
- Results from episodic search are scored by:
  - Temporal proximity: recent occurrences weighted more
  - Situational overlap: same window_title or similar keywords
  - Conflict with current context: if current judgment contradicts a stored pattern,
    flag it as a "learned caution"
- Top K results (e.g., K=3) are added to `ReminderContext.evidence_refs`
  with `memory_type = "episodic_memory"`.

**Step 6: Memory Reinforcement**
- When an episodic memory is retrieved and the situation resolves,
  update `occurrence_count += 1` and `last_seen = now`.
- When a pattern has not been retrieved in 60 days, confidence decreases.
- When confidence drops below threshold, the memory enters an "archived" state
  (still queryable but not surfaced unless explicitly retrieved).

---

## Agent Shape: Shape C (Single Agent with Tools)

```
Use a single tool-using agent with a long-term memory store.

Agent goal: Surface relevant cross-session memory to improve model judgment quality.

Available tools:
  - write_episodic_memory(confidence, pattern_type, summary, related_slice_ids)
  - search_episodic_memory(query_text, max_age_days) → Vec<EpisodicMemory>
  - reinforce_episodic_memory(id)
  - decay_episodic_memory(id)
  - archive_episodic_memory(id)

Observation loop:
  ActivityNote → build_reminder_context (short-term) →
  search_episodic_memory → merge into ReminderContext → model judgment →
  outcome → reinforce or archive

Memory/state:
  - episodic_memory table (SQLite, same db as existing memory tables)
  - confidence score per memory (float, 0.0–1.0)
  - occurrence_count per memory
  - first_seen / last_seen timestamps

Stop condition:
  Confidence < threshold for N consecutive retrieval cycles without reinforcement
  OR user explicitly clears memory
```

---

## Architecture: New Module `episodic_memory`

### Database Schema

```sql
-- Add to schema.rs run_migrations()

CREATE TABLE IF NOT EXISTS episodic_memory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_type TEXT NOT NULL,       -- 'recurring_situation' | 'decision_pattern' | 'skill_acquisition' | 'error_signal'
    summary TEXT NOT NULL,            -- Human-readable distilled pattern
    confidence REAL DEFAULT 0.0,      -- 0.0–1.0, threshold = 0.4
    occurrence_count INTEGER DEFAULT 1,
    first_seen TEXT,                  -- RFC3339 timestamp
    last_seen TEXT,                   -- RFC3339 timestamp
    related_slice_ids TEXT,           -- JSON array of i64
    evidence_source TEXT DEFAULT 'distilled',
    status TEXT DEFAULT 'active'      -- 'active' | 'archived'
);

CREATE INDEX IF NOT EXISTS idx_episodic_status ON episodic_memory(status);
CREATE INDEX IF NOT EXISTS idx_episodic_last_seen ON episodic_memory(last_seen);
CREATE INDEX IF NOT EXISTS idx_episodic_pattern_type ON episodic_memory(pattern_type);
```

### Core Struct

```rust
// New file: cozmio_memory/src/episodic_memory.rs

#[derive(Debug, Clone)]
pub struct EpisodicMemory {
    pub id: Option<i64>,
    pub pattern_type: String,         // recurring_situation | decision_pattern | skill_acquisition | error_signal
    pub summary: String,
    pub confidence: f32,
    pub occurrence_count: i32,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
    pub related_slice_ids: Vec<i64>,
    pub evidence_source: String,
    pub status: String,              // active | archived
}

pub struct EpisodicMemoryStore<'a>(&'a Database);

impl<'a> EpisodicMemoryStore<'a> {
    pub fn insert(&self, mem: &EpisodicMemory) -> Result<i64, MemoryError> { ... }
    pub fn get_active(&self) -> Result<Vec<EpisodicMemory>, MemoryError> { ... }
    pub fn get_by_status(&self, status: &str) -> Result<Vec<EpisodicMemory>, MemoryError> { ... }
    pub fn search_by_summary(&self, query: &str, max_age_days: u32) -> Result<Vec<EpisodicMemory>, MemoryError> { ... }
    pub fn update_confidence(&self, id: i64, delta: f32) -> Result<(), MemoryError> { ... }
    pub fn increment_occurrence(&self, id: i64) -> Result<(), MemoryError> { ... }
    pub fn archive(&self, id: i64) -> Result<(), MemoryError> { ... }
}
```

### Distiller: Building Episodic Memories from Context Slices

```rust
// New file: cozmio_memory/src/distiller.rs

/// Examine context slices and existing episodic memories to produce
/// new episodic memories or reinforce existing ones.
pub fn run_distillation(db: &Database) -> Result<usize, MemoryError> {
    // 1. Load all active episodic memories for conflict detection
    // 2. Load recent context slices (last 7 days, not already linked to episodic)
    // 3. For each slice, check:
    //    a. Does a similar episodic memory already exist?
    //       → reinforce existing (confidence += 0.1, occurrence += 1)
    //    b. Is this a new recurring pattern (same window_title across sessions)?
    //       → create new episodic memory with initial confidence = 0.5
    //    c. Does this contradict an existing episodic memory?
    //       → create new episodic with flag "conflict_detected", both stay active
    // 4. Run decay: confidence -= 0.05 for every 30 days without occurrence
    // 5. Archive any episodic with confidence < 0.2
}
```

Key distillation triggers:
- Same `window_title` appearing in slices more than 3 times across sessions
- Same `judgment` appearing in similar content patterns
- `error_text` present → auto-create `error_signal` episodic with high initial confidence
- Decision contradicts prior decision in same `thread_id` → create `decision_pattern` with both referenced

### Retrieval: Merging into ReminderContext

```rust
// Modifies: competition.rs — MemoryCompetition::build_reminder_context

// After building short-term ReminderContext:
// 1. Run episodic search using sanitized window_title + content_text
// 2. Filter to status='active' and last_seen within 30 days
// 3. Score by: (occurrence_count * 0.3) + (recency_factor * 0.7)
// 4. Take top 3, add to evidence_refs with memory_type = "episodic_memory"
// 5. If any episodic has confidence > 0.7 and relates to current situation,
//    add a special flag: ReminderContext.has_strong_episodic_match = true
```

### Confidence Decay / Reinforcement Schedule

```
Reinforcement rules:
  - Episodic retrieved + situation confirms pattern → confidence += 0.1 (cap at 1.0)
  - Episodic retrieved + situation contradicts pattern → confidence -= 0.15

Decay rules (run on startup and every 24h):
  - Each active episodic with last_seen > 30 days → confidence -= 0.05 per 30-day period
  - confidence < 0.2 → archive (status = 'archived')

Archive rules:
  - Archived episodics are not surfaced unless explicit search with `include_archived=true`
  - User can view/manage archived memories via UI
```

---

## Implementation Sequence (First Buildable Slice)

### Phase 1: Schema + Store (1 day)
- Add `episodic_memory` table to migrations
- Implement `EpisodicMemoryStore` with basic CRUD
- Add `episodic_memory` re-export to `lib.rs`
- Write unit tests for store operations

### Phase 2: Distiller (2 days)
- Implement `run_distillation` function
- Trigger: called at startup and after `build_slices_from_events` completes
- Write integration test: insert slices → run distillation → verify episodic created

### Phase 3: Retrieval Integration (1 day)
- Modify `MemoryCompetition::build_reminder_context` to search episodic
- Verify episodic results appear in `ReminderContext.evidence_refs`
- Add `has_strong_episodic_match` to `ReminderContext` struct

### Phase 4: Confidence Lifecycle (1 day)
- Implement reinforcement on retrieval (update confidence + occurrence)
- Implement decay on startup
- Implement archive when confidence < 0.2

### Phase 5: UI (optional, later)
- Show "Episodes found: N" in reminder context display
- Allow user to view, reinforce, or clear episodic memories
- Persist user corrections back to episodic store

---

## Verification Gate

```
1. Run distiller on existing data → episodic memories created
2. Search episodic with a known window_title → relevant episodic retrieved
3. Reinforce an episodic → occurrence_count increments, confidence increases
4. Let memory decay (mock time passage) → confidence decreases, eventually archives
5. Verify archived episodic does not appear in normal ReminderContext build
6. Verify ReminderContext.evidence_refs contains episodic entries when pattern matches
7. Verify build_reminder_context still returns InsufficientImportedData error
   when no imported evidence exists (existing guard is preserved)
```

---

## Key Design Decisions

1. **No embeddings for episodic search yet.** Use FTS5 on `summary` field.
   sqlite-vec is conditional and adds build complexity.
   FTS5 on summary text is sufficient for Phase 1.

2. **Confidence threshold 0.4.** Memories below this are candidates but not surfaced.
   This prevents low-confidence speculation from polluting the prompt.

3. **30-day retrieval window.** Memories older than 30 days without reinforcement
   are not surfaced. This matches the "medium-term" gap between short-term slices
   (2h) and truly persistent skills/decisions.

4. **Pattern types are enumerable.** Not free-form. This keeps distillation deterministic
   and debuggable. New pattern types require code change (not user-provided).

5. **Distillation is not model-driven.** The distiller runs programmatic rules on
   existing data. Model judgment is only used when surfacing memory to the prompt,
   not when deciding what to remember. This avoids the circular problem of
   "model decides what to remember, then model uses that memory to decide."

---

## Open Questions for User

1. What is the target memory horizon? 30 days? 90 days? Indefinite?
2. Should user be able to manually add/edit/clear episodic memories?
3. Is there a specific pattern type that is most important (e.g., error signals)?
4. Should episodic memory feed into the model prompt, or only be visible in UI?
5. What is the acceptable latency for the distillation step (runs at startup)?