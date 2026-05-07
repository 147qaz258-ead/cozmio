# Cozmio Agent Self-Maintained Memory Flywheel — Wide Stage Plan

**日期**: 2026-05-07  
**状态**: 进行中  
**承接文档**:

- `docs/superpowers/specs/2026-05-07-agent-self-maintained-memory-flywheel-design.md`
- `docs/superpowers/plans/2026-05-07-agent-self-maintained-memory-flywheel-stage1-plan.md`

## 0. Planning Position

当前已经完成的只是最小自动闭环:

```text
local popup model non-empty output
  -> auto write episode memory
  -> future popup context recalls active memory
```

这证明了飞轮可以转起来，但还没有证明 Cozmio 会在真实使用中越来越懂用户。后续不再拆成 `Stage 1.5` 这种小修补，而是直接进入一个大范围阶段性目标:

```text
真实使用经历
  -> 自动事实记录
  -> 自动反馈/结果归档
  -> consolidation agent 自我维护 memory
  -> popup / handoff / silence 行为改变
  -> debug UI 和 eval replay 证明使用效果变好
```

本计划的目标是一次覆盖足够多的内容，让下一次实施后可以直接看使用效果，而不是只看存储表是否写入成功。

## 1. Product Outcome

本阶段完成后，用户应该能明显感到:

1. Cozmio 记得最近真实发生过什么，而不是只复述一段滚动摘要。
2. Cozmio 会因为用户确认、取消、忽略、执行成功、执行失败而改变下次出现方式。
3. Cozmio 能把反复有效的帮助方式沉淀成 procedure，而不是每次重新猜。
4. Cozmio 的弹窗不只是“更频繁”或“更少”，而是更贴合上下文、更少重复打扰、更能接上用户正在推进的工作。
5. 开发者能看到每条记忆为什么存在、来自哪些事实、什么时候被召回、有没有造成行为变化。

验收不以“新增了多少 API”为准，而以以下可见效果为准:

- Replay 同一批真实 traces 时，带 memory 的 popup 文本比无 memory 更具体。
- 对重复 dismiss 的场景，模型输入中能看到相关反思，后续输出减少同类打扰或改变措辞。
- 对确认并成功执行的场景，后续 handoff 提案更短、更直接、更接近成功模式。
- Memory inspector 能展示 active/reflection/procedure/hot-context proposal 及 provenance。
- 用户可以 reject/supersede 错误记忆，下一轮 recall 不再使用它。

## 2. Scope

本阶段一次性覆盖六条主线:

1. Experience Ledger: 统一事实经历记录。
2. Feedback And Outcome Learning: 把用户反馈和 executor 结果变成学习信号。
3. Consolidation Agent: 自动维护 episode/reflection/procedure/hot context。
4. Recall Admission: 把 memory 作为上下文预算进入 popup 和 handoff。
5. Memory Inspector: 让记忆、来源、召回和操作可检查、可纠正。
6. Eval Replay: 用真实 trace 对比使用效果。

这些主线可以并行推进，但最终必须打通一条完整使用路径:

```text
observe -> popup -> user action -> executor outcome
  -> factual experience packet
  -> consolidation run
  -> memory operation
  -> recall admission
  -> changed next popup/handoff
  -> replay/eval/report
```

## 3. Non-Goals

- 不把代码写成“用户正在卡住”“项目迭代机会”这类语义判断器。
- 不用 popup cooldown、硬规则 gate 或 vector score 替代模型判断。
- 不上传 raw screenshot 或完整屏幕日志到云端。
- 不删除现有 action log、ledger、decision memory、skill memory、vector 相关代码。
- 不让 memory body 变成 rigid semantic fields。结构化字段只服务存储、生命周期、来源和工具协议。
- 不追求一次做完所有美观 UI，但必须有可用的 debug/inspector。

## 4. Semantic Boundary

这是本阶段的硬规格，不是设计建议。

代码只能产生和传递以下 packet:

- `factual packet`: timestamp、window_title、process_name、trace_id、session_id、raw model output、notification lifecycle、user action、executor result/error、file refs、raw refs、artifact refs、source refs。
- `candidate packet`: memory id、layer、lifecycle、body excerpt、source refs、retrieval score、last_used_at、use_count、exact duplicate key、legacy store name。
- `budget packet`: token limit、estimated token count、admitted item ids、excluded item ids、exclusion reason as mechanical fact such as over_budget / inactive_lifecycle / missing_source_ref。
- `source/provenance packet`: source_ref、event_kind、timestamp、trace/session link、producer、route、artifact refs、content refs。
- `permission/routing packet`: local/cloud/executor route, material ids allowed for the route, redaction status, user/developer approval event refs, feature flags, technical denial reason.

代码绝不产生或传递以下结论:

- 这意味着什么。
- 是否值得记住。
- 这是不是 procedure。
- 用户是否不喜欢、是否被打扰、是否满意。
- 未来应该怎么做。
- 当前任务阶段、用户意图、是否卡住、是否适合弹窗。

这些语义只能由 consolidation agent 或 popup/handoff agent 用自然语言合成，并且必须带 source refs。代码可以存储 agent 写出的自然语言正文和 provenance，但不能把这些语义拆成代码拥有的规则字段。

代码只验证机械条件:

- source refs 存在且属于当前允许材料。
- memory body 非空。
- lifecycle 是存储状态，不是语义判断。
- token budget 合法。
- permission/routing packet 合法。
- 操作目标存在、重复键匹配、来源链可追溯。

## 5. Current Baseline

### Original Baseline

初始已完成:

- `cozmio_memory/src/agent_memory.rs`: active/rejected memory store、operation log、轻量 recall。
- `cozmio_memory/src/schema.rs`: agent memory tables 与旧 schema preservation。
- `src-tauri/src/memory_consolidation.rs`: apply/list/auto remember 的基础命令和自动写入。
- `src-tauri/src/main_loop.rs`: non-empty popup model output 自动写 episode。
- `src-tauri/src/prompt_context.rs`: popup context 自动召回 active memory。
- `src-tauri/src/tray.rs`、`src-tauri/src/types.rs`: 补齐缺失模块，恢复 app 编译。

验证基线:

- `cargo test -p cozmio_memory`: 22 passed。
- `cargo test -p cozmio memory_consolidation`: 4 passed。
- `cargo check -p cozmio_memory`: passed。
- `cargo check -p cozmio`: passed，仍有既有 warnings。

当前不足:

- 自动写入只是 raw popup output episode，不是真正 consolidation agent。
- feedback/outcome 还没进入 memory learning。
- reflection/procedure/hot context proposal 没有真实生产路径。
- provenance 还不是严格 source refs 链路。
- 没有 memory inspector。
- 没有 replay eval。

### Implementation Status — 2026-05-07

已推进:

- Workstream A/B: foreground observation、raw model output、popup displayed、confirm/cancel/dismiss/expire 等路径已开始进入 factual experience source；这些记录只包含时间、窗口、进程、trace、event_kind、raw text/source refs 等事实边界。
- Workstream C/D: 旧的 raw model output 直接写长期 episode 路径已禁用；hot context 不再作为每轮滚动摘要的主要写入目标，consolidation path 成为记忆语义合成入口。
- Workstream E: popup runtime context 已走 recall admission；hot stable context、recent feedback facts、admitted memory metadata 分 channel 进入 prompt，action log 只能吃剩余预算。
- Workstream G: 已新增 Memory inspector 初版，可查看 memory rows、recent experience sources、memory operations、consolidation runs、source refs、route/packet counts，并可 reject active memory。
- Semantic boundary: runtime 检查已覆盖 `model_client.rs`、`prompt_context.rs`、`main_loop.rs`、`window_monitor.rs`、`StatusPanel.js`、`MemoryInspector.js` 等关键路径，阻止低维语义字段重新进入 runtime prompt/UI。

已验证:

- `cargo check -p cozmio`
- `cargo test -p cozmio prompt_context`
- `cargo test -p cozmio window_monitor`
- `cargo test -p cozmio memory_consolidation`
- `cargo test -p cozmio --test semantic_boundary`
- `node --check cozmio/src-tauri/src/components/MemoryInspector.js`
- `node --check cozmio/src-tauri/src/components/App.js`

仍未完成:

- handoff/executor packet 的完整 recall admission 验收。
- Memory inspector 的 supersede memory、apply hot context proposal、run consolidation now、replay selected trace。
- Procedure candidate packet 和 procedure recall 的完整闭环。
- Privacy/routing inspector：显示 local/cloud/executor route、allowed material ids、redaction status、approval refs。
- Replay eval：还没有真实 trace 前后对比报告。

## 6. Target Architecture For This Stage

```text
Foreground loop
  -> ExperienceRecorder records observation/model output/popup lifecycle
  -> UserActionRecorder records confirm/cancel/dismiss/expire/edit
  -> ExecutorOutcomeRecorder records dispatch/result/error/artifacts
  -> ExperiencePacketBuilder builds factual packets
  -> ConsolidationScheduler runs after feedback/outcome or idle window
  -> ConsolidationAgent reads packet + related memories
  -> MemoryTool validates and applies operations
  -> RecallAdmission selects active memories under budget
  -> Popup/Handoff model receives current facts + recalled agent memory
  -> EvalReplay compares behavior with and without memory
```

This is not one small feature. It is the first complete memory-maintenance loop.

## 7. Workstream A — Unified Experience Ledger

### Goal

Every meaningful event in a Cozmio use cycle becomes factual source material for memory maintenance.

### Required Implementation

Add or consolidate a factual experience layer that records:

- foreground observation metadata;
- raw popup model output;
- popup displayed;
- user confirm;
- user cancel;
- user dismiss;
- confirmation expired;
- user edited handoff text if available;
- relay dispatch requested;
- executor session started;
- executor progress summary;
- executor completed;
- executor failed;
- user correction/reject memory;
- app paused/stopped shortly after intervention.

The record shape must be factual:

```text
source_ref
timestamp
trace_id
session_id?
event_kind
window_title?
process_name?
factual_text
raw_ref?
artifact_refs?
```

`event_kind` is a mechanical event type, not semantic user state.

### Files Likely Touched

- `cozmio/src-tauri/src/logging.rs`
- `cozmio/src-tauri/src/ledger.rs`
- `cozmio/src-tauri/src/main_loop.rs`
- `cozmio/src-tauri/src/commands.rs`
- `cozmio/src-tauri/src/protocol_handler.rs`
- `cozmio/src-tauri/src/relay_bridge.rs`
- `cozmio/cozmio_memory/src/memory_events.rs`

### Acceptance

- A single trace can show complete lifecycle: model output -> popup -> user action -> executor outcome.
- Every memory operation can cite source refs from this layer.
- No source text contains code-generated semantic conclusions.

## 8. Workstream B — Feedback And Outcome Learning

### Goal

Cozmio learns from what the user did with its intervention, not only from what it observed.

### Required Learning Signals

Record as facts:

- confirmed popup;
- cancelled popup;
- dismissed popup;
- ignored/expired popup;
- user modified handoff text;
- executor completed;
- executor failed;
- user interrupted execution;
- user corrected memory;
- user rejected memory;
- app paused/stopped soon after intervention.

### Consolidation Boundary

The code records these feedback/outcome events only as a `factual packet` with source refs. The packet may say a popup was confirmed, cancelled, dismissed, expired, edited, dispatched, completed, failed, interrupted, corrected, rejected, paused, or stopped.

The packet must not say:

- the user disliked the popup;
- the popup was poorly timed;
- an executor result proves a procedure;
- a failure means a reflection is needed;
- a repeated event is a stable pattern;
- future behavior should change.

Only the consolidation agent can write those meanings in natural language, cite the packet source refs, and choose remember/update/supersede/abstain operations.

### Files Likely Touched

- `src-tauri/src/notification_manager.rs`
- `src-tauri/src/protocol_handler.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/main_loop.rs`
- `src-tauri/src/relay_bridge.rs`
- `src-tauri/src/memory_consolidation.rs`

### Acceptance

- Confirm/cancel/expire actions appear in factual packet.
- Executor success/failure appears in factual packet.
- Consolidation can write reflection from agent-interpreted feedback facts.
- Consolidation can write procedure only when the agent cites factual packet refs and writes the reusable pattern in natural language.

## 9. Workstream C — Real Consolidation Agent

### Goal

Replace “model output directly becomes episode” as the main learning path with a self-maintaining consolidation agent.

The old automatic episode write must not be the learning path. The runtime path should be:

```text
factual packet + related memories
  -> consolidation prompt
  -> agent decides operations
  -> code validates operations
  -> memory store updates
```

### Required Operations

Support full operation set:

- `remember_episode`
- `remember_reflection`
- `remember_skill`
- `update_hot_context`
- `remove_or_supersede`
- `abstain`

### Required Layers

- `episode`: what happened.
- `reflection`: what should change next time.
- `procedure`: reusable way to help or handoff.
- `hot_context_proposal`: small stable context update candidate.

### Consolidation Prompt Contract

The agent receives:

- recent factual packet;
- related existing memories;
- current hot context;
- operation tool descriptions;
- privacy/routing limits.

The agent must decide:

- add memory;
- update/supersede memory;
- propose hot context update;
- remember procedure;
- abstain.

The prompt must say:

- do not invent motives;
- preserve uncertainty;
- write only if future behavior should change;
- cite source refs;
- feedback/outcome events are facts, not conclusions;
- do not promote screenshot-derived text into durable memory unless cited later facts support the agent's natural-language conclusion.

### Scheduler

Run consolidation automatically in these cases:

- after user confirms/cancels/dismisses popup;
- after pending confirmation expires;
- after executor completes/fails;
- during idle time when unconsolidated factual events exist;
- optional manual dev command for replay/debug.

Do not run consolidation every observation.

### Files Likely Touched

- `src-tauri/src/memory_consolidation.rs`
- `src-tauri/src/model_client.rs`
- `src-tauri/src/main_loop.rs`
- `src-tauri/src/commands.rs`
- `cozmio_memory/src/agent_memory.rs`
- `cozmio_memory/src/schema.rs`

### Acceptance

- A consolidation run can produce multiple operations.
- `abstain` is recorded as a run result.
- Duplicate exact memories are not appended repeatedly.
- Superseded memory remains inspectable but not live-recalled.
- Hot context proposal is draft until accepted.

## 10. Workstream D — Hot Stable Context Refactor

### Goal

Stop treating `human_context.md` as rolling observation summary. It should become tiny stable memory.

### Required Behavior

- Read `human_context.md` as hot stable context.
- Stop every-observation overwrite path or put it behind disabled feature flag.
- Create hot context proposals from consolidation.
- Apply hot context update only when:
  - source refs exist;
  - text fits budget;
  - the update text was written by the consolidation agent or accepted by the user/developer;
  - permission/routing packet allows the storage route.

Code must not decide that a proposed line is "stable enough", "safe", or "local-only stable fact" in the semantic sense. It can only validate provenance, budget, route, lifecycle, and an explicit acceptance event if acceptance is required.

### Candidate Split

Consider splitting into:

- `USER.md`: durable user/project preferences.
- `AGENT.md`: Cozmio behavior/self-maintenance principles.

This mirrors Hermes-style small memory without turning it into a rolling diary.

### Files Likely Touched

- `src-tauri/src/human_memory.rs`
- `src-tauri/src/memory_consolidation.rs`
- `src-tauri/src/prompt_context.rs`
- `src-tauri/src/commands.rs`

### Acceptance

- Observation loop no longer rewrites hot context by default.
- Hot context remains small and stable.
- Hot context proposals are inspectable.
- Popup prompt includes hot stable context plus recalled memories separately.

## 11. Workstream E — Recall Admission Refactor

### Goal

Memory recall becomes budgeted evidence admission, not semantic authority.

### Required Behavior

Recall candidates from:

- active episode memories;
- reflection memories;
- procedure memories;
- recent feedback facts;
- hot stable context;
- legacy decision/skill/context stores as candidates only.

Admission rules:

- hard token budget;
- candidate ordering may use mechanical facts only: lifecycle, source kind, recency timestamp, retrieval score, exact text/query match, last_used_at, use_count, and token cost;
- user corrections are source facts, not proof of what future behavior means;
- procedure memories are admitted as natural-language candidates by mechanical match or explicit agent request, not because code concludes the current task "needs a procedure";
- mark recalled memories as used;
- include memory id, layer, last_used_at, source refs in debug output;
- do not let vector score decide popup permission.

### Vector Role

Existing vector/competition code should be reframed:

- old: memory competition;
- new: recall admission.

Vector score can rank candidates, but not interpret them.

### Files Likely Touched

- `cozmio_memory/src/competition.rs`
- `cozmio_memory/src/search.rs`
- `cozmio_memory/src/agent_memory.rs`
- `src-tauri/src/prompt_context.rs`
- `src-tauri/src/memory_commands.rs`

### Acceptance

- Popup context can show recalled episode/reflection/procedure under budget.
- Rejected/superseded/expired memory is excluded.
- Recall debug output shows candidate packet and budget packet facts only.
- Semantic boundary tests reject forbidden hard-coded labels.

## 12. Workstream F — Agent-Written Procedure / Skill Memory

### Goal

Let the consolidation agent decide whether repeated confirmed handoffs, executor outcomes, edits, artifacts, timestamps, and source refs justify reusable procedural memory.

### Procedure Candidate Packet

Code may build a candidate packet containing only:

- source refs for user confirmation events;
- handoff text or edited handoff text refs;
- executor result/error refs;
- artifact refs;
- mechanical counts of similar text/query matches;
- timestamps and session ids;
- existing procedure memory ids retrieved as candidates.

Code must not label this packet as "a reusable procedure", "successful pattern", "user-approved workflow", or "safe future behavior". The consolidation agent decides whether the cited facts justify a procedure memory and writes the body in natural language.

### Procedure Shape

Natural language body, for example:

```text
When helping with Cozmio memory architecture, first inspect current memory-writing paths,
then separate factual logging, consolidation, recall admission, popup behavior, UI inspection,
and eval replay. Keep code-owned fields factual and let the model own semantic interpretation.
```

Storage metadata may include:

- layer = `procedure`;
- lifecycle;
- source refs;
- use_count;
- last_used_at.

### Files Likely Touched

- `cozmio_memory/src/skill_memory.rs`
- `cozmio_memory/src/agent_memory.rs`
- `src-tauri/src/memory_consolidation.rs`
- `src-tauri/src/prompt_context.rs`

### Acceptance

- Executor outcome facts can be cited by the consolidation agent when it writes procedure memory.
- Procedure memory can be recalled into future handoff/popup context.
- Procedure usage increments `use_count`.

## 13. Workstream G — Memory Inspector / Debug UI

**状态**: 部分完成。

已实现:

- Tauri command `get_memory_inspector_snapshot` returns memory rows, recent experience sources, memory operations, and consolidation run summaries.
- UI panel `MEMORY` displays agent-written memory bodies, source refs, lifecycle, producer, recent factual text, operation rows, route, packet id, source counts, and related memory counts.
- Active memory can be rejected from the inspector.
- Inspector UI is included in semantic boundary scanning.

未完成:

- Supersede memory from inspector.
- Apply hot context proposal from inspector.
- Run consolidation now from inspector with agent-produced operations.
- Replay selected trace from inspector.
- Show latest popup recall admission preview from recorded runtime debug metadata.

### Goal

The flywheel must be inspectable. A memory system that cannot be inspected cannot be trusted.

### Minimum UI

Add a memory/debug page or panel showing:

- recent experiences;
- consolidation runs;
- memory operations;
- active memories by layer;
- draft hot context proposals;
- rejected/superseded memories;
- provenance/source refs;
- recalled memory preview for current context;
- buttons:
  - reject memory;
  - supersede memory;
  - apply hot context proposal;
  - run consolidation now;
  - replay selected trace.

### UI Boundary

UI displays facts and agent-written text. UI must not invent why a memory is true.

### Files Likely Touched

- `src-tauri/src/components/*`
- `src-tauri/src/main.js`
- `src-tauri/src/styles.css`
- `src-tauri/src/commands.rs`
- `src-tauri/src/memory_consolidation.rs`
- `src-tauri/src/memory_commands.rs`

### Acceptance

- Developer can inspect a memory and follow its source refs. **Implemented.**
- Developer can reject a bad memory and confirm it disappears from recall. **Partially implemented:** reject exists; full recall disappearance should be confirmed through replay/latest recall preview.
- Developer can see which memories entered the latest popup context. **Not complete:** runtime admission metadata exists in prompt context, but inspector does not yet show latest popup recall preview.

## 14. Workstream H — Evaluation Replay

### Goal

Prove the agent is getting better, not just writing more memory.

### Eval Set

Start with 10-20 real Cozmio traces:

- helpful popup should appear;
- correct behavior is silence;
- user dismissed repeated interruption;
- user confirmed and executor succeeded;
- executor failed;
- user corrected memory/design direction;
- route-denied or unapproved-material trace should abstain or remain local-only.

### Replay Modes

For each trace, compare:

- no memory;
- current memory;
- memory without reflection;
- memory with reflection/procedure.

### Metrics

Use human-reviewable scoring produced outside code-owned runtime rules:

- usefulness;
- specificity;
- interruption quality;
- alignment with user-authored instructions or human-review labels;
- handoff scope quality;
- privacy behavior;
- no hard-coded semantic violation.

### Files Likely Touched

- `src-tauri/src/evaluation.rs`
- `src-tauri/src/eval_commands.rs`
- `src-tauri/src/memory_consolidation.rs`
- `cozmio/verification/*`
- possibly `cozmio/tests/*`

### Acceptance

- Replay report shows at least one before/after behavior change caused by memory.
- Human-labeled cases where silence is expected remain silent or less intrusive.
- Bad memory can be rejected and replay changes accordingly.

## 15. Workstream I — Privacy And Routing

### Goal

Keep screen-derived memory trustworthy.

### Required Modes

- local-only default;
- review-before-send for cloud/executor consolidation;
- redaction preview for window title, file paths, raw text, screenshots;
- no raw screenshot upload in this stage;
- explicit provenance for any material leaving local process.

Routing code may only construct a `permission/routing packet`: route, allowed material ids, redaction status, approval source refs, feature flags, and denial reason. It must not classify content as "sensitive" or infer whether material is safe to send. Any explanation of why material should or should not leave the machine must be agent-authored or user/developer-authored text with source refs.

### Files Likely Touched

- `src-tauri/src/config.rs`
- `src-tauri/src/memory_consolidation.rs`
- `src-tauri/src/relay_bridge.rs`
- `src-tauri/src/components/*`

### Acceptance

- Config shows current memory maintenance mode.
- Cloud/executor consolidation cannot run without explicit allowed material.
- Inspector shows local/cloud route for each consolidation run.

## 16. Data Model Target

The current simple integer-id memory store should expand or be migrated to support:

```text
agent_memories:
  id
  body
  layer
  lifecycle
  source_refs_json
  supersedes_id?
  producer
  created_at
  updated_at
  last_used_at
  used_count
  rejected_reason?
  expires_at?

memory_operations:
  id
  operation_type
  target_memory_id?
  resulting_memory_id?
  body?
  layer?
  source_refs_json
  status
  error_text?
  producer
  created_at

consolidation_runs:
  id
  trigger_kind
  route
  packet_json
  model_name?
  output_text?
  status
  error_text?
  created_at
  completed_at?

experience_sources:
  source_ref
  timestamp
  trace_id?
  session_id?
  event_kind
  window_title?
  process_name?
  factual_text
  raw_ref?
  artifact_refs_json
```

This schema is allowed because fields are factual, operational, or storage metadata. The semantic memory remains natural-language body written by the agent.

## 17. Implementation Strategy

This should be executed as one broad stage with parallel workstreams, not a chain of tiny stages.

### Worker Split

Worker A: Storage And Migration

- Owns `cozmio_memory/src/agent_memory.rs`
- Owns `cozmio_memory/src/schema.rs`
- Owns tests for lifecycle, source refs, operations, runs.

Worker B: Experience And Feedback Recording

- Owns factual event recording in `logging.rs`, `ledger.rs`, `main_loop.rs`, `commands.rs`, `protocol_handler.rs`.
- Ensures confirm/cancel/dismiss/expire/executor outcome all create source refs.

Worker C: Consolidation Agent

- Owns `memory_consolidation.rs`
- Builds packet, prompt, scheduler, operation validation, abstain recording.

Worker D: Recall Admission

- Owns `prompt_context.rs`, recall admission helpers, legacy memory candidate integration.
- Ensures no code-owned semantic gates.

Worker E: Inspector UI

- Owns frontend components and IPC commands for memory/runs/provenance/replay.

Worker F: Eval Replay

- Owns eval fixtures, replay commands, report generation, semantic boundary checks.

Integrator:

- Owns main path wiring, final verification, documentation update, feature list, rollout toggle.

### Merge Rule

Each worker must leave behavior behind a feature flag or local config if the path is risky:

- `memory_flywheel_enabled`
- `auto_consolidation_enabled`
- `local_only_memory_consolidation`
- `memory_inspector_enabled`

Default should enable local automatic packet recording and local agent consolidation once tests pass:

- auto factual recording: on;
- local consolidation after feedback/outcome: on;
- cloud/executor consolidation: off unless explicitly allowed;
- hot context auto-apply: off initially, proposals only.

## 18. Rollout Plan

### Wide Stage Milestone 1: Automatic Experience Capture

Visible outcome:

- Inspector shows complete trace lifecycle.
- Popup confirm/cancel/expire and executor results are source material.

Exit criteria:

- One real Cozmio session produces complete factual packet with no semantic labels.

### Wide Stage Milestone 2: Automatic Consolidation Agent

Visible outcome:

- After feedback or executor result, consolidation run appears automatically.
- It writes episode/reflection/procedure or abstains.

Exit criteria:

- User dismissal can lead to reflection memory.
- Executor success can lead to procedure memory.

### Wide Stage Milestone 3: Behavior-Changing Recall

Visible outcome:

- Next popup prompt includes relevant recalled memory.
- Popup/handoff text changes because of prior memory.

Exit criteria:

- Replay shows changed output caused by memory.
- Rejected memory no longer affects replay.

### Wide Stage Milestone 4: Trust And Review

Visible outcome:

- User/developer can inspect, reject, supersede, and apply hot context proposals.

Exit criteria:

- Every active memory has visible provenance.
- Bad memory can be corrected without DB surgery.

### Wide Stage Milestone 5: Evaluation Report

Visible outcome:

- A report compares no-memory vs memory behavior over real traces.

Exit criteria:

- At least 10 traces replay.
- Report identifies wins, regressions, privacy abstentions, and next fixes.

## 19. Verification Matrix

| Capability | Required Verification |
|---|---|
| Experience capture | Unit tests for each event kind; real trace smoke test |
| Feedback learning | Confirm/cancel/expire/executor result packet tests |
| Consolidation operations | Tests for remember/reflection/procedure/hot proposal/supersede/abstain |
| Provenance | Unknown source refs rejected; source refs visible in inspector |
| Recall admission | Active recalled; rejected/superseded/expired excluded; budget respected |
| Semantic boundary | Tests scan prompts/context for forbidden hard-coded labels |
| Hot context | Proposal created; no hot file overwrite without acceptance |
| UI inspector | Manual smoke test plus IPC tests |
| Eval replay | Report generated from fixed trace set |
| App compile | `cargo check -p cozmio_memory`, `cargo check -p cozmio` |

## 20. Commands To Run

From `D:/C_Projects/Agent/cozmio/cozmio`:

```bash
cargo test -p cozmio_memory
cargo test -p cozmio memory_consolidation
cargo test -p cozmio prompt_context
cargo test -p cozmio semantic_boundary
cargo check -p cozmio_memory
cargo check -p cozmio
```

After UI work:

```bash
cd src-tauri
npm run tauri dev
```

Manual verification:

1. Run Cozmio.
2. Trigger a popup.
3. Confirm or cancel it.
4. If confirmed, let executor complete or fail.
5. Open memory inspector.
6. Confirm factual packet, consolidation run, memory operation, and recalled context exist.
7. Replay the same trace with and without memory.

## 21. Feature List And Verification Artifacts

Update `cozmio/feature_list.json` with a broad feature entry:

```json
{
  "id": "MEMORY-FLYWHEEL-WIDE-STAGE",
  "category": "agent-memory",
  "title": "Agent self-maintained automatic memory flywheel",
  "type": "agent_system",
  "description": "Records factual desktop experience, learns from feedback and executor outcomes, consolidates agent-written memory, recalls it into popup/handoff context, exposes inspector UI, and verifies behavior changes by replay.",
  "status": "pending",
  "passes": false
}
```

After implementation, write `cozmio/verification/last_result.json` with:

- commands run;
- trace replay count;
- memory operations created;
- rejected/superseded memory behavior;
- observed popup/handoff changes;
- known regressions.

## 22. Done Definition

This stage is not done when tables exist. It is done when:

1. Real user interactions create factual experience packets.
2. Consolidation runs automatically after feedback/outcome.
3. Agent writes episode/reflection/procedure/hot-context proposal or abstains.
4. Memory operations have provenance and lifecycle.
5. Recalled memory changes future popup/handoff context.
6. User/developer can inspect and reject memory.
7. Replay eval shows behavior difference with memory enabled.
8. Semantic boundary tests pass.
9. App compiles and targeted tests pass.

## 23. Execution Note

The previously implemented small automatic episode path should be treated as bootstrap/fallback, not as the final architecture.

The next implementation pass should not stop after adding one missing module. It should execute the wide stage through at least:

- automatic feedback/outcome source recording;
- automatic consolidation run;
- reflection/procedure creation;
- recall into popup/handoff;
- inspector/debug surface;
- replay report.

Only then can the project evaluate the real user-facing effect: whether Cozmio is becoming more useful through lived experience.
