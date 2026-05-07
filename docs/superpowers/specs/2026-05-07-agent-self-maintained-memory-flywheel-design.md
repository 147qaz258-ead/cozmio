# Cozmio Agent Self-Maintained Memory Flywheel Design

> Status: draft for review
> Date: 2026-05-07

## 1. Purpose

Cozmio should become more useful as it observes the user's real computer work. The target experience is not "a memory feature"; it is an agent that treats each observation as experience, can later reflect on those experiences, and changes its future intervention behavior because of what it learned.

The current memory path does not yet create that effect. `human_context.md` is maintained as a rewritten snapshot, while the richer `action_log.jsonl`, event ledger, execution results, user confirmation, dismissal, and distillation paths are not yet connected into a stable learning loop.

This design defines a memory refactor based on current agent practice:

- Hermes: small always-loaded memory, session search, and procedural skills.
- Letta / MemGPT: self-editing memory managed by the agent through tools.
- LangGraph / Deep Agents: background consolidation rather than constant hot-path memory rewriting.
- Reflexion: feedback-driven natural-language reflection.
- Voyager: repeated confirmed behavior with executor outcome facts can be consolidated into reusable procedural memory by an agent.

## 2. Product Experience

Primary user object: the user's lived work session on their desktop, not a database row, vector score, or popup record.

User-visible flow:

1. Cozmio quietly observes foreground work and records factual experience.
2. Cozmio occasionally appears only when the current agent thinks it has a useful action or suggestion.
3. The user confirms, dismisses, or lets the suggestion expire.
4. Cozmio records what happened, including executor success or failure when work is handed off.
5. Later, Cozmio's suggestions reflect what it has learned from prior experience.

Required states:

- Empty: no meaningful experience has been consolidated yet; Cozmio should rely on current observation.
- Pending: a memory consolidation job has recent experience to review.
- Running: the consolidation agent is reading factual records and deciding whether to update memory.
- Completed: memory was updated, abstained, or left unchanged with a recorded reason.
- Failed: consolidation failed with a technical error; prior memory remains valid.

Experience acceptance:

- After repeated user dismissal facts in a similar situation, the popup agent should have enough agent-written memory to avoid repeating the same style of intervention unless new evidence appears.
- After repeated confirmation and executor outcome facts in a similar situation, the popup/handoff agent should have enough agent-written memory to make that kind of help more direct and better scoped.
- A user or developer should be able to inspect why a memory exists by following provenance back to source events or logs.

## 3. Agent Boundary

### Model Owns

- Deciding what an observed episode means.
- Deciding whether recent experience is worth remembering.
- Writing natural-language memory entries, reflections, and skill-like procedures.
- Replacing or removing older memory when new experience contradicts it.
- Deciding how retrieved memories should affect the current popup text or handoff proposal.

### Code Owns

- Capturing factual events: timestamp, window title, process name, model raw output, popup display, user confirmation, dismissal, relay dispatch, executor result, errors, and file references.
- Constructing `factual packet`, `candidate packet`, `budget packet`, `source/provenance packet`, and `permission/routing packet` only.
- Preserving append-only source material.
- Cutting factual records into readable windows by time, trace, session, or event adjacency.
- Running retrieval, vector search, token budgeting, deduplication by exact identity, provenance validation, and storage.
- Enforcing privacy, local/remote routing, technical permission gates, and failure handling.

Code never owns conclusions such as what an event means, whether it is worth remembering, whether it is a procedure, whether the user liked or disliked it, or what should happen next. Those conclusions must be written by an agent or user/developer as natural language with source refs.

### User Owns

- Confirming or dismissing interventions.
- Choosing whether cloud/execution-side consolidation is allowed for selected material.
- Correcting memory when it feels wrong.
- Disabling or pausing observation.

### Executor Owns

- Complex task execution after user confirmation.
- Returning concrete progress, result, failure, and artifact references.
- Producing high-quality reflections when asked to consolidate execution traces.

### UI Owns

- Showing current state, pending suggestions, memory review items, and provenance.
- Never inventing execution status or semantic interpretation.
- Making memory updates inspectable and reversible.

## 4. Design Principles

1. Code must not hard-code semantic labels such as "user is stuck", "project iteration", "student discount intent", or "workflow stage".
2. Vector search is recall infrastructure, not judgment. It can find candidates; it cannot decide what they mean.
3. Memory entries are natural-language agent artifacts with provenance, not program-authored facts.
4. `human_context.md` is hot memory for stable context, not a rolling observation summary.
5. Full logs remain append-only. Consolidated memory is replaceable and reviewable.
6. Model output should not be forced into low-dimensional decision fields for memory writing. If code needs indexes, it stores storage metadata around natural-language memory, not inside the agent's reasoning contract.
7. Background consolidation runs outside the foreground observation hot path; the foreground loop records packets.

## 5. Existing System Assessment

Current useful assets:

- `src-tauri/src/main_loop.rs` records raw model output and popup decisions into action history.
- `src-tauri/src/human_memory.rs` already asks the local model to maintain a human memory file.
- `src-tauri/src/ledger.rs` defines append-only ledger events and content references.
- `src-tauri/src/distill_commands.rs` already models factual input material, a distillation backend, memory candidates, source event ids, source quotes, and abstention.
- `cozmio_memory/src/competition.rs` can build a reminder context from memory stores and evidence refs.
- `cozmio_memory/src/search.rs`, embedding providers, FTS, and vector components can remain useful for retrieval.

Current design problems:

- The hot path rewrites `human_context.md` too often and treats the file as a current rolling summary.
- Memory competition risks turning into code-owned semantic ranking when scores or labels become the reason something is shown.
- Some context strings expose program-shaped labels to the model rather than human-readable experience.
- User feedback and executor outcomes are not yet the central signal for learning.
- Skills exist as a store, but repeated confirmation, executor outcome, edit, and artifact facts are not yet consolidated by an agent into procedural memory.

## 6. Considered Approaches

### Approach A: Keep rewriting `human_context.md`

This is the smallest change, but it preserves the current weakness. The memory remains a single mutable paragraph, and the agent cannot inspect a durable history of why it learned something.

Decision: reject.

### Approach B: Make vector competition the memory brain

This uses embeddings, score formulas, and signal facts to rank memories into context. It is useful for retrieval, but it becomes dangerous if the score is treated as semantic truth.

Decision: keep only as recall and admission infrastructure.

### Approach C: Agent-owned consolidation over factual experience

Code records and retrieves factual material. A consolidation agent reads recent experience, decides what it learned, writes natural-language memory with provenance, and updates stable context or skills only when justified.

Decision: choose this as the target architecture.

## 7. Target Architecture

```text
Foreground observation
  -> factual event ledger
  -> local popup agent reads current observation + selected memory
  -> popup / silence / handoff proposal
  -> user feedback and executor result
  -> append-only experience record

Background consolidation
  -> fetch recent unconsolidated experience
  -> retrieve related memories
  -> agent reflects and writes memory operations
  -> provenance validation by code
  -> memory store update
  -> selected stable memory appears in future context
```

The core shape is an agentic workflow, not a pure deterministic pipeline:

- Shape B workflow for scheduling, storage, provenance, and retrieval.
- Shape C agent loop for consolidation, because deciding what changed in memory is semantic work.

## 8. Memory Layers

### Layer 0: Raw Experience

Append-only factual records. This includes window observations, model raw outputs, popup lifecycle events, user confirmation/dismissal, relay dispatch, executor logs, errors, and content refs.

Code can cut these into readable windows, but cannot label their semantic meaning.

### Layer 1: Episodic Memory

Natural-language descriptions of what happened, written by an agent from source experience. These are not always loaded. They are searched when current context resembles prior experience.

Example shape:

```text
On 2026-05-07, Cozmio proposed help while the user was discussing memory flywheel design in the Cozmio repository. The user emphasized that code must not hard-identify semantics and that the project should become genuinely useful through agent-owned memory maintenance. This episode is relevant when future work touches memory architecture, popup behavior, or vector competition.
Sources: source event references retained by the memory operation.
```

### Layer 2: Reflective Memory

Agent-written lessons from feedback. This is where Reflexion-style learning lives.

Example shape:

```text
When the user rejects memory designs, the rejection is often about code taking semantic ownership away from the agent. Future memory proposals should preserve agent interpretation rights and use code only for factual boundaries, provenance, retrieval, and safety.
Sources: source event references retained by the memory operation.
```

### Layer 3: Procedural Memory

Reusable behavior strategies. This is where Hermes/Voyager-style skill memory belongs.

Example shape:

```text
For Cozmio memory design work, first inspect current memory-writing paths, then separate factual logging, agent consolidation, retrieval admission, and popup behavior. Avoid proposing field-based semantic schemas unless they are storage-only and hidden from model reasoning.
Sources: source event references retained by the memory operation.
```

### Layer 4: Hot Stable Context

Small always-loaded memory, replacing the current rolling use of `human_context.md`.

This layer should contain only durable context that helps almost every observation:

- The user is building Cozmio, a desktop cognitive agent.
- The user strongly values agent semantic freedom and dislikes code-owned semantic labeling.
- The project boundary says model output is the high-dimensional semantic layer.

This file should be updated by consolidation, not every observation.

## 9. Memory Operations

The consolidation agent should be given explicit memory tools, not a prompt that asks it to output a full rewritten file.

Allowed operations:

- `remember_episode`: add an episodic memory from recent experience.
- `remember_reflection`: add or update a lesson learned from feedback.
- `remember_skill`: add or update a reusable procedure.
- `update_hot_context`: propose a concise replacement for stable hot memory.
- `remove_or_supersede`: retire outdated memory with a provenance-backed reason.
- `abstain`: record that recent experience did not justify memory changes.

These operations can have structured tool parameters for storage, but the semantic body remains natural language. The model must not be prompted to fill rigid semantic fields such as intent, workflow stage, or user state.

## 10. Vector And Competition Refactor

The previous vector memory competition should be reinterpreted as candidate retrieval and budget admission.

Allowed:

- Find memories textually or semantically similar to the current observation.
- Order candidates by mechanical recency and reference-count heuristics.
- Keep token budgets small.
- Return provenance and score breakdown for debugging.

Not allowed:

- Using code-owned score formulas to conclude user intent.
- Using memory kind as a forced semantic label in model reasoning.
- Treating vector similarity as permission to popup.
- Hiding semantic summaries behind program field names that the model must obey.

New name recommendation: replace "memory competition" in user-facing and agent-facing concepts with "memory recall admission". The word competition can remain internal if useful for implementation, but the concept should be retrieval under budget, not a semantic contest.

## 11. Popup Agent Relationship

The local 4B model can continue to decide whether to appear, because popup timing is a low-latency local task.

However, the local popup agent should receive:

- Current observation facts.
- A small number of recalled memory excerpts written by prior agents.
- Recent user feedback facts.
- Available action descriptions.

It should not receive:

- Program-authored claims about the user's intent.
- Hard stages or semantic labels.
- A forced JSON contract for its natural suggestion text.

The popup decision remains model-owned. Code may treat empty output as silence if that remains the product contract, but code must not add semantic rule gates that decide when the model is allowed to appear.

## 12. Privacy And Local/Cloud Boundary

Because screen observation is sensitive, the memory design must make privacy understandable.

Local-only by default:

- Raw screenshots.
- Raw foreground event logs.
- Full action history.
- Full window titles unless explicitly allowed for a remote execution task.

Eligible for cloud/executor consolidation only after an explicit permission/routing packet allows it:

- User-confirmed tasks.
- Explicit execution sessions.
- Redacted or selected factual excerpts.
- Developer-triggered memory maintenance for project work.

The user should be able to inspect what would be sent before enabling any cloud-backed consolidation mode.

## 13. Consolidation Prompt Contract

The consolidation agent should receive natural-language factual material, not low-dimensional semantic fields.

Prompt intent:

```text
You are maintaining Cozmio's memory from factual experience records.

Read the recent experience and existing memories. Decide whether this experience should change future behavior.

Only write memory that would help Cozmio act better in a similar future situation.
Preserve uncertainty. Do not invent motives. Keep provenance.
If nothing should be remembered, abstain.
```

The tool layer, not the prompt, enforces:

- source event ids exist;
- memory body is non-empty;
- hot context stays within budget;
- proposed removals reference existing memory;
- dangerous prompt-injection or secret-looking content is blocked.

## 14. Minimum Verifiable Slice

The first implementation slice should prove the flywheel without rebuilding everything.

Input:

- A small set of recent action log or ledger events.
- At least one popup confirmation or dismissal.
- Existing `human_context.md`.

Process:

- Build a readable factual packet from source events.
- Run a consolidation agent once, manually or via IPC.
- Write memory operations to a reviewable store.
- Keep `human_context.md` unchanged unless the agent explicitly proposes a stable hot-context update.

Expected output:

- At least one episodic memory or an explicit abstention.
- If user feedback exists, one reflection about what should change next time.
- Provenance references back to source events.
- No code-authored semantic labels.

Success criteria:

- The next popup context can include a recalled memory that was produced by consolidation.
- A reviewer can explain why the memory exists by reading its sources.
- Re-running consolidation over the same source range does not duplicate exact memory.
- If the agent abstains, the abstention reason is recorded as a consolidation result, not as a memory.

## 15. Implementation Boundary For Future Plan

Likely files to inspect or change during planning:

- `cozmio/src-tauri/src/human_memory.rs`
- `cozmio/src-tauri/src/main_loop.rs`
- `cozmio/src-tauri/src/distill_commands.rs`
- `cozmio/src-tauri/src/ledger.rs`
- `cozmio/src-tauri/src/memory_commands.rs`
- `cozmio/cozmio_memory/src/competition.rs`
- `cozmio/cozmio_memory/src/search.rs`
- `cozmio/cozmio_memory/src/skill_memory.rs`

Likely changes:

- Stop treating hot memory maintenance as an every-observation rewrite.
- Add a consolidation job or command that reads recent factual experience.
- Add memory operation storage with provenance.
- Reframe competition as recall admission.
- Feed recalled agent-written memory excerpts into popup context.
- Add memory review/debug UI later, after the core loop works.

## 16. Non-Goals

- No full autonomous cloud upload of screen logs.
- No code-authored user intent.
- No popup cooldown or hard rule gate as a replacement for model judgment.
- No prompt format that turns the model into a field filler for memory semantics.
- No attempt to solve all UI for memory review in the first slice.
- No deletion of existing logs or vector infrastructure.

## 17. Missing Design Modules

The sections above define the direction, but they are not yet enough for a safe implementation. A useful Cozmio memory flywheel still needs the following design modules before the refactor should be treated as complete.

### 17.1 Memory Lifecycle

Missing question: what happens to a memory after it is written?

Required design:

- Draft: agent proposed the memory, but it has not yet entered popup context.
- Active: memory can be recalled into popup or consolidation context.
- Superseded: a newer memory replaced it.
- Rejected: user or reviewer marked it wrong.
- Expired: memory is too old or too situation-specific to affect future behavior.
- Archived: retained for audit/search, but not used for live suggestions.

This lifecycle is important because bad memory is worse than no memory. The agent must be able to revise itself without pretending old conclusions never existed.

### 17.2 Feedback Taxonomy

Missing question: what counts as learning signal?

Required packet design:

- User confirmed popup.
- User dismissed popup.
- User ignored popup until expiry.
- User edited or rewrote handoff text.
- Executor completed successfully.
- Executor failed.
- User corrected memory.
- User disabled or paused Cozmio shortly after an intervention.

These are facts, not semantic judgments. Code records them in a `factual packet` with source refs. Code must not label them as negative feedback, user preference, annoyance, approval, or proof of a workflow. The consolidation agent decides what they mean and cites the packet refs when writing memory.

### 17.3 Memory Write Agent Contract

Missing question: when is experience worth remembering?

Required agent contract:

- The consolidation agent writes memory only when its natural-language reasoning says future popup, handoff, or abstention behavior should change.
- The agent may treat feedback/outcome facts as stronger evidence, but code only passes them as facts.
- The agent may describe repeated patterns, but code only passes counts, timestamps, source refs, and retrieved candidate text.
- The agent preserves uncertainty when evidence is thin.
- The agent abstains when it cannot justify a memory from cited source refs.
- The agent must not promote screenshot-derived text into durable memory unless later cited facts support the natural-language conclusion.

This contract lives in consolidation instructions and evaluation cases. Code enforces only source existence, body non-emptiness, lifecycle validity, route permission, and budget limits.

### 17.4 Recall Admission Packet Contract

Missing question: when should a memory enter the current model context?

Required design:

- Hot stable context is always loaded but extremely small.
- Recalled memories enter only under a strict token budget.
- Retrieval should include provenance and last-used metadata.
- Candidate ordering may use mechanical facts only: lifecycle, source kind, recency timestamp, retrieval score, exact text/query match, last-used timestamp, use count, and token cost.
- User corrections are source facts, not code-owned proof of what future behavior means.
- Procedural memory enters as a natural-language candidate by mechanical match or explicit agent request, not because code concludes the current task needs a procedure.
- Raw logs should never enter popup context unless clipped into a readable factual packet.

This is where vector search belongs: it proposes candidates, code creates a budget packet, and the model decides how to use the admitted natural-language material.

### 17.5 Consolidation Cadence

Missing question: when does the memory-maintaining agent run?

Required design options:

- Manual developer command: safest first slice.
- Local idle-time consolidation: runs when the app is idle and no popup is pending.
- Nightly local consolidation: cheaper and less disruptive.
- Post-executor consolidation: run after a confirmed task completes or fails.
- Cloud/executor consolidation: only for explicitly allowed material.

Initial recommendation: start with manual command plus post-executor consolidation. Do not run consolidation every observation.

### 17.6 Memory Tool Protocol

Missing question: what tools does the consolidation agent actually have?

Required tool definitions:

```text
Tool: search_experience
Purpose: retrieve factual event windows by time, trace, or text.
Input: query text or source ids.
Output: clipped factual records with provenance.

Tool: search_memory
Purpose: retrieve existing active, superseded, or archived memories.
Input: natural-language query and memory layer filter.
Output: memory text, lifecycle state, provenance, last used time.

Tool: propose_memory_operation
Purpose: ask code to store add/update/supersede/abstain operations.
Input: operation type, natural-language body, source ids.
Output: accepted/rejected with validation error if provenance is invalid.

Tool: propose_hot_context_update
Purpose: update the small always-loaded context only when stable.
Input: replacement text and source ids.
Output: accepted/rejected with budget and provenance validation.
```

The operation type is a storage action. The memory body remains natural language.

The tool protocol must not include fields for intent, disliked_by_user, procedure_source, success_means, task_stage, future_policy, or similar semantic shortcuts. If such meaning is needed, it belongs inside the agent-written natural-language body with source refs.

### 17.7 Conflict And Contradiction Handling

Missing question: what if memories disagree?

Required design:

- New memory can supersede old memory only with source evidence.
- Contradictory memories should both remain inspectable.
- Recall admission can expose lifecycle, source kind, and recency facts; the popup agent decides how newer corrections affect the current response.
- User-authored corrections outrank agent-generated memories.
- Executor results outrank popup predictions.
- The agent should say uncertainty remains when evidence conflicts.

Without this, the flywheel will accumulate confident but stale beliefs.

### 17.8 Forgetting, Decay, And Compression

Missing question: how does the system avoid becoming a second giant context window?

Required design:

- Passive observations can receive short mechanical retention windows unless later cited by agent-written memory.
- Reflections can receive longer mechanical retention windows than raw episodes.
- Procedural memory retention can use mechanical use_count, last_used_at, lifecycle, and explicit user/developer actions.
- Hot stable context must have a hard character budget.
- Recalled memory should update `last_used_at` for later pruning decisions.
- Consolidation should merge near-duplicate memories instead of appending forever.

Forgetting must be visible and reversible. A memory should expire for a recorded mechanical reason or agent/user-authored natural-language reason with source refs, not disappear silently.

### 17.9 Privacy Review Path

Missing question: how does the user trust a screen-observing agent?

Required design:

- A local-only mode where raw events never leave the machine.
- A review-before-send mode for cloud/executor consolidation.
- Redaction preview for window titles, screenshots, file paths, and raw text.
- A memory inspector showing what Cozmio thinks it learned.
- A "forget this" action at the memory and source-range level.
- A clear indicator when memory maintenance is running.

Routing code emits only route, allowed material ids, redaction status, approval refs, feature flags, and technical denial reasons. Content risk explanations must be written by an agent or user/developer, not inferred by code.

### 17.10 Popup Learning Boundary

Missing question: how does memory change popup behavior without code-owned gates?

Required design:

- Popup agent reads recalled memories and feedback facts.
- The model decides whether to appear and what to say.
- Code records popup lifecycle facts.
- Consolidation writes reflections such as "this kind of interruption was usually dismissed".
- Future popup prompts include those reflections as prior experience.

The system learns popup taste through memory, not through mechanical cooldowns or hard-coded semantic suppression.

### 17.11 Executor Handoff Learning

Missing question: how does Cozmio learn what work should be delegated?

Required design:

- Record user confirmation text and any edits before dispatch.
- Record executor session id, result, error, and artifacts.
- Consolidation compares proposed task vs completed task.
- Executor outcome facts can be cited by the consolidation agent when it writes procedural memory.
- Failed or overbroad handoffs become reflective memory.

This is the main path from "desktop observer" to "digital worker".

### 17.12 Evaluation Protocol

Missing question: how do we prove the agent is getting better?

Required design:

- Replay historical event windows with and without memory.
- Compare popup usefulness, specificity, and interruption quality.
- Include negative cases where the correct behavior is silence.
- Include privacy cases where consolidation should abstain.
- Human reviewers label cases where silence is expected; runtime code does not infer that label.
- Track repeated suggestion reduction after dismissal facts through eval annotations.
- Track handoff quality after executor outcome facts through eval annotations.

The first eval set can be small: 10-20 real Cozmio traces. It should be enough to catch whether memory helps or merely adds noise.

### 17.13 Debug And Review UI

Missing question: how does the developer see the flywheel working?

Required design:

- Recent experiences list.
- Consolidation runs list.
- Memory operations list.
- Active memories by layer.
- Provenance viewer.
- Recalled memory preview for the current popup context.
- User correction controls.

This does not have to be pretty in the first slice, but it must exist somewhere. A memory system that cannot be inspected cannot be trusted.

### 17.14 Migration From Current System

Missing question: how do we move without losing existing work?

Required design:

- Keep current `human_context.md` as initial hot stable context.
- Stop hot-path overwrites behind a feature flag.
- Import existing `action_log.jsonl` and event logs as raw experience.
- Treat existing decision/skill/context stores as legacy memory candidates until reconsolidated.
- Preserve vector indexes, but relabel their role as recall admission.
- Add a one-time consolidation pass over recent project-related logs.

No existing memory infrastructure should be deleted in the first refactor.

### 17.15 Rollout Stages

Missing question: what order avoids overbuilding?

Recommended stages:

1. Manual consolidation over selected logs, write reviewable memory operations.
2. Feed active recalled memories into popup context under budget.
3. Add feedback taxonomy recording for popup and executor outcomes.
4. Add post-executor consolidation.
5. Add memory inspector/debug UI.
6. Add local idle-time consolidation.
7. Add optional cloud/executor consolidation with review-before-send.
8. Add evaluation replay and regression set.

The first stage should prove "logs -> agent memory -> changed future context" before adding automation.

### 17.16 Open Questions

- Should the first consolidation agent be local 4B, execution-side large model, or selectable?
- Should hot stable context remain a single `human_context.md`, or split into `USER.md` and `AGENT.md` like Hermes?
- Should user corrections be stored as immutable high-priority memories?
- What exact source material is allowed to leave the machine in cloud-backed modes?
- How should screenshot-derived facts be represented without storing or uploading the image?
- Should popup memory and execution memory share one store or separate stores?
- What is the minimum UI needed for trust before background consolidation is enabled?

## 18. Design Completeness Assessment

Current document status after adding the missing modules:

- Direction is clear enough: agent-owned memory over factual experience.
- Semantic boundary is clear enough: code records facts and retrieves; model interprets.
- Minimum slice is clear enough: manual consolidation first.
- Implementation is not yet fully specified: schema, IPC commands, exact prompts, feature flags, and tests still need a writing-plan phase.
- Evaluation is now identified but not yet authored as a runnable eval set.
- UI trust surface is identified but not yet designed visually.

The next design artifact should be either:

- an implementation plan for Stage 1 manual consolidation, or
- an eval design for proving memory improves popup behavior before broad refactor.

## 19. Design Review

Product experience:

- Primary user object is the lived desktop work session.
- User-visible flow is five steps.
- Empty, pending, running, completed, and failed states are defined.
- Acceptance criteria focus on changed future behavior, not storage success alone.

Agent boundary:

- Model, code, user, executor, and UI responsibilities are separated.
- Code does not own semantic interpretation.
- UI does not invent memory, execution, or completion states.

Technical consistency:

- Existing ledger, distillation, search, embedding, and skill stores are reused conceptually.
- Vector search is kept as retrieval infrastructure, not semantic authority.
- The design can be handed to implementation planning as a bounded refactor.

## 20. References

- Hermes memory docs: `MEMORY.md`, `USER.md`, session search, external providers, and agent-managed memory tools.
- Letta / MemGPT: self-editing memory and memory hierarchy.
- LangGraph / Deep Agents: long-term memory, skills as procedural memory, and background consolidation.
- Reflexion: verbal reflection from feedback into episodic memory.
- Voyager: lifelong learning through an expanding skill library.
