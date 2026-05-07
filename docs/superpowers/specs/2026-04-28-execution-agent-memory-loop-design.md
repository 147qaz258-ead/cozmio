# Execution Agent Memory Loop Design

> Status: ready for implementation planning
> Date: 2026-04-28

## Goal

Use stronger execution-side agents to reduce large logs into model-generated summaries with provenance, then expose only small factual references or selected summary snippets to the local model.

## Boundary

System code does not summarize meaning. System code schedules, stores, indexes, clips, and records provenance.

Semantic summaries may come from:

- execution agent output
- local model output
- user-authored notes

Semantic summaries must include:

- timestamp
- source path
- source byte range or record id
- producer name
- raw summary text

## Inputs

- Cozmio action log JSONL
- relay session outputs
- Claude Code project conversation logs
- subagent logs

## Outputs

- daily_summary records
- project_summary records
- source_index records

## Local Model Exposure Rule

The local model may receive only small selected records. It must not receive raw full-day logs or complete Claude Code conversations.

## Non-Goals

- No system-authored user intent.
- No popup cooldown.
- No structured silence field.
- No direct injection of large execution logs into the 4k local model context.
