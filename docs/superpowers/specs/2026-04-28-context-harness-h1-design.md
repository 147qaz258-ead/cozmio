# Context Harness H1 Design

> Status: draft for review
> Date: 2026-04-28

## Goal

Improve Cozmio popup usefulness by giving the local model better factual context and better tool affordances, without using system code to decide meaning for the model.

The popup remains model-led. The system does not mechanically limit popup frequency, does not force a structured decision field, and does not decide whether the model is allowed to appear.

## Core Boundary

System code may provide facts:

- timestamp
- window title
- process name
- trace id
- session id
- relay status
- user interface action id
- raw model output
- execution result text
- error text
- source path
- duration and count values

System code must not create semantic conclusions:

- user intent
- task stage
- stuck / not stuck
- project iteration opportunity
- whether a page is useful
- whether an action is important

If semantic text is needed, it must come from a model or execution agent and be stored with provenance.

## H1 Scope

### 1. Remove Hardcoded Semantic Hints

Remove system-generated context such as:

- workspace/project hints that expand retrieval based on `cozmio`, `claude`, or local path strings
- popup strategy text
- hardcoded "keep silent" instructions
- hardcoded "project iteration" suggestions
- fake confidence values for UI actions

### 2. Build a Fact Harness

The context harness should be a plain factual material block:

```text
current_window: title="...", process="..."
process_context: stay_duration_seconds=..., switches_last_minute=...
action_log_tail:
- timestamp=..., age_seconds=..., window="...", action=..., result="..."
```

This block is not a summary and not a judgment.

### 3. Keep Local Model Context Small

The local model should receive only a small tail of factual records. Large execution logs and Claude Code conversations should not be injected directly into the local model.

Long logs must first be reduced by an execution-side model into model-generated summaries with provenance.

### 4. Use Execution Agents for Higher-Dimensional Memory

Execution agents may periodically read richer sources:

- Cozmio action logs
- relay session outputs
- Claude Code conversation logs
- subagent logs

They may produce daily or project-level semantic summaries. Those summaries are model output, not system facts, and must include timestamp and source references.

### 5. Evaluation Loop

Testing should focus on usefulness, not lower popup count.

Metrics:

- popup entered the user's workflow
- suggestion was specific enough to act on
- output did not invent unsupported context
- output did not rely on system-created semantics
- local model stayed within context limits
- execution-side summaries improved future context

## Explicit Non-Goals

- No popup cooldown.
- No frequency cap.
- No rule that suppresses popups because the user switches windows quickly.
- No `decision: popup | silence` protocol.
- No system-authored labels like `stuck`, `working`, `project_phase`, or `iteration_opportunity`.

## Next Implementation Slice

1. Clean current prompt context so it only emits factual key-value material.
2. Remove prompt text that tells the model to keep silent or treat path matches as project intent.
3. Persist this semantic boundary in repo guidance.
4. Verify with unit tests and one independent local-model evaluation agent.
5. Design a separate execution-agent summarization loop for Claude Code and relay logs.
