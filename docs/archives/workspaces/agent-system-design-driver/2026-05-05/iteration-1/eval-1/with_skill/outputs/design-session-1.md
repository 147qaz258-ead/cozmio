# Agent System Design Driver — Output

## Agent Design Reading

```
1. User signal:
   The user feels the assistant reacts too late — popup always feels like "after the fact"
   commentary, not timely proactive help.

2. Target agent behavior:
   The agent should offer suggestions at the moment they become relevant, before the user
   has already moved past the decision point or completed the action.

3. Evidence available:
   - Screenshot (current frame)
   - Window title + process name (current)
   - ProcessContext: stay_duration, switches_in_last_minute, is_oscillating,
     last_switch_direction, just_arrived (from window_monitor buffer)
   - Action log tail (last 18 records, last ~64KB)
   - Model call timing metadata

4. Context needed:
   - What the user is trying to do RIGHT NOW (intent signal, not just window title)
   - Whether the user appears stuck or in a decision moment
   - Whether a relevant suggestion would interrupt a flow vs. land at a natural break point
   - Whether recent dismissals suggest the suggestion type is unwanted

5. Model decision:
   Given current screen + process context + recent history, should I intervene now?
   If yes: what is the single most relevant thing to say, and is this a good moment?

6. Tools/actions:
   - Show Windows toast notification (current — fires AFTER the model call completes)
   - Suppress / defer notification
   - Log to action history
   - Relay task execution (for confirmed actions)

7. Feedback loop:
   User clicks Confirm / Cancel on the toast
   → logged in action_log_tail as user_feedback
   → next model call sees feedback: "ui_closed", "confirmed", "dismissed", etc.
   Loop closes but only retroactively.

8. Verification gate:
   Real interaction replay: screenshot + context → model output → whether the output
   references evidence from the current screen AND was delivered before the user
   had moved on.

9. Current weakest layer:
   LAYER 2 — Context Assembly. The agent sees what window is open but has no
   mechanism to detect "this is a good moment to intervene" vs. "the user is in
   the middle of something". It reacts to window title changes after-the-fact
   (poll on title change), rather than reasoning about the user's flow state.
```

---

## Root Cause Analysis

The "always late" feeling comes from three compounding issues:

### Problem A: Trigger is Passive (Layer 0)

The loop only fires when `window_change_detection` sees a title change. This means:

- The user is already on the new window before the agent even captures it
- By the time the model processes and the toast renders, 3–6+ seconds have passed
- The agent is always one "window state" behind the user's actual intent

**Evidence from code:**

```rust
// main_loop.rs, Step 4
if active_config.window_change_detection && !monitor.has_changed(&snapshot) {
    log::debug!("No window change detected, skipping");
    continue;  // ← nothing happens until the title changes
}
```

### Problem B: Context Lacks Intent Signal (Layer 2)

The popup context includes:
- ProcessContext (stay_duration, switches, oscillation, just_arrived) — structural
- Recent action log — historical
- Window title — positional

What it does NOT include:
- What the user is actively trying to accomplish
- Whether they are in a decision, editing, reading, or waiting state
- Whether the current window/activity has been stable for a while (good moment to suggest)
  vs. just arrived (bad moment — user is in the middle of something)

The `just_arrived` flag exists in `ProcessContext` but it's not used in the intervention
decision logic. The model only gets it as a number in a text block.

### Problem C: Delivery is One-Way, No Deferral (Layer 4)

Once the model returns content, the notification fires immediately. There is no:
- "This is a good moment" vs. "defer until user is less busy" judgment
- Priority / urgency tier in the model output (only Display / NoDisplay binary)
- Ability to queue a suggestion for later when context becomes more favorable

---

## Agent Shape Selection

**Shape B — Workflow with a judgment gate before action.**

The current shape is effectively a single model call that always converts its output
into a notification. The problem is there's no decision about **whether this is a
good moment to notify**, only whether to display content.

The fix is not to add more agents. The fix is to add a **timing/intervention fitness
judgment** as a structured gate inside the existing loop, before calling the notification.

```
Current loop:
  capture → build_context → model_call → notify (always)

Proposed loop:
  capture → build_context → [intervention_fitness_gate] → model_call → notify (only if FIT)

Intervention fitness gate:
  Given current process_context + action_log, should I interrupt now?
  - yes: this is a natural break / user has been stable / decision point visible
  - defer: just_arrived=true, is_oscillating=true, user is actively switching
  - abstain: recent feedback shows similar suggestions were dismissed
```

This is still a **Shape B workflow** — the model fills the judgment point inside a
fixed program flow. No multi-agent needed.

---

## Design Brief: Proactive Intervention System

### Target behavior

The agent should surface suggestions at moments that feel relevant, not moments that
feel like an audit. Suggestions should land at natural pause points in the user's
workflow, not during transitions or active work.

### Current user problem

All suggestions feel late because the system reacts to window state, not user state.

### Agent shape

Shape B workflow with an intervention fitness gate (structured program decision)
before the model call.

### Observation source

```
- WindowMonitor: title, process, stay_duration, switches, oscillation, just_arrived
- ActionLogger: last 18 records, last 64KB of history
- Current screenshot (for model)
```

### Evidence

```
- ProcessContext fields (structural signal — are they switching, oscillating, stable?)
- just_arrived flag (temporal signal — did they just get here?)
- is_oscillating flag (temporal signal — are they bouncing between windows?)
- Recent feedback (dismissal rate signal — are similar suggestions unwanted?)
- stay_duration_seconds (engagement signal — have they been here long enough to need help?)
```

### Context entering model

Same as today, PLUS a constructed `intervention_fitness_summary` text block:

```
intervention_fitness:
  just_arrived={bool}       ← model should de-prioritize if true
  is_oscillating={bool}    ← model should de-prioritize if true
  switches_last_minute={n} ← high number = scattered attention
  stay_duration_seconds={n}← high number = user is settled (good moment)
  recent_dismissal_rate={float}  ← if high, reduce suggestion frequency
  last_{n}_suggestions=[list of recent suggestion themes]
```

### Model decision

The model makes a **semantic judgment**: given all the above, is this a good moment
to show a suggestion? What should the suggestion be? Should it be surfaced now or deferred?

The model's natural-language output carries the intervention timing judgment. The
system parses:
- `DISPLAY_NOW` / `DISPLAY_DEFERRED` / `NO_DISPLAY` (replacing the current binary)

### Tools/actions

```
- notify_now()        → fires Windows toast immediately
- notify_deferred()  → queues suggestion, fires on next favorable moment
- log_suppressed()   → records that suggestion was suppressed (feedback for future calls)
- update_fitness_score() → refines the fitness gate based on user response
```

### Feedback loop

```
User response (confirm/cancel/dismiss) → logged with suggestion content
→ affects next intervention_fitness_summary (higher dismissal rate = more conservative)
→ next model call sees the updated fitness context
```

### Memory/state

```
intervention_fitness_state:
  recent_dismissal_rate (float, EMA over last N suggestions)
  suppression_count_last_hour (int)
  last_suggestion_theme (string, for deduplication)
  defer_queue: [DeferredSuggestion]  ← suggestions waiting for better moment
```

### Traceability

Every notification carries:
- `trace_id` (existing)
- `intervention_fitness_summary` at time of decision (new — stored in PendingConfirmationInfo.process_context or a new field)
- Whether it was shown now / deferred / suppressed

### Verification gate

```
1. Structural: build passes, tests pass
2. Behavioral: run a probe with oscillating window switches,
   verify that suggestions are NOT fired during oscillation
   but ARE fired after 30s of stability
3. Regression: existing action_log tail evaluation still works
4. Sample: run 5 representative scenarios, verify the intervention timing
   feels right (human judgment required for Layer 4)
```

### First buildable slice

**Add `intervention_fitness_summary` to the prompt context and a `DISPLAY_DEFERRED`
mode.** This is the smallest change that makes the model aware of timing fitness
without changing the notification flow.

Changes required:
1. `prompt_context.rs` — add `intervention_fitness_summary` to `build_popup_context`
2. `model_client.rs` — extend `InterventionMode` with `DisplayDeferred`
3. `main_loop.rs` — handle `DisplayDeferred` by storing in a defer queue instead of notifying
4. A new `defer_queue` field in `AppState` (in `commands.rs`)
5. A deferred-notification flush trigger: when `just_arrived=false && is_oscillating=false &&
   stay_duration_seconds > 15` and defer queue is non-empty, flush the best deferred suggestion

---

## Architecture Correction

**Current direction:** Pure reactive model call on window title change.

**Problem with this direction:** The agent is always one context-switch behind the user's
intent. "Proactive" in name only — it's a delayed reactor.

**Better framing:** The system should have a **fitness gate** between "something changed"
and "call the model". The fitness gate is a lightweight program-level check (not a model call)
that decides whether the current process context is favorable for intervention. If not,
skip the model call entirely (save latency + avoid irrelevant suggestions).

```
Lightweight gate (program, not model):
  if just_arrived:     skip model call
  if is_oscillating:   skip model call
  if switches_last_minute > 8: skip model call
  else: call model normally
```

Then the model gets richer fitness context and decides `DISPLAY_NOW / DISPLAY_DEFERRED / NO_DISPLAY`.

**Agent layer affected:** Layer 0 (trigger) and Layer 2 (context assembly).

**What should be model judgment:** Whether to surface a suggestion now, defer it, or abstain.

**What should be program plumbing:** Whether to call the model at all, based on structural
process signals (not semantic ones).

**Concrete next artifact:** A `InterventionFitness` struct in `window_monitor.rs` or a new
`fitness_gate.rs` that produces a fitness decision from `ProcessContext` + recent history,
used in `main_loop.rs` before `build_popup_context` is called.

---

## Summary of Changes Needed

| Priority | File | Change |
|----------|------|--------|
| 1 | `window_monitor.rs` | Add `InterventionFitness::from_context()` — program-level gate |
| 2 | `main_loop.rs` | Call fitness gate before `build_popup_context`; skip if unfit |
| 3 | `prompt_context.rs` | Add `intervention_fitness_summary` to popup context block |
| 4 | `model_client.rs` | Extend `InterventionMode` with `DisplayDeferred` |
| 5 | `commands.rs` / `runtime_state.rs` | Add `defer_queue` to `AppState` |
| 6 | `main_loop.rs` | Handle deferred queue: flush when fitness becomes favorable |
| 7 | `notification_manager.rs` | (no change — stays as notification presenter) |

**Verification:**
```bash
cd cozmio && cargo build
```
Must pass. Then run real interaction probes on the scenarios in the design brief.
