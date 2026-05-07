# Context Harness H2 Practice Loop Design

> Status: draft for review
> Date: 2026-04-28

## 1. Background

Cozmio 的长期目标不是做一个简单的桌面弹窗工具，而是逐渐进化成一个会观察、会记忆、会接执行端、会帮助用户迭代项目本身的桌面智能体系统。

当前体验已经从完全不可读的弹窗，进化到能大概说出用户在做什么并给出一点建议。但它还没有稳定成为真正可用的帮助，因为它仍然存在这些问题：

- 弹窗内容有滞后感。
- 弹窗缺少对前后过程的连续理解。
- 弹窗不知道用户正在推进的阶段和长期目标。
- 建议有时像旁观者评论，而不是进入用户工作流。
- 执行端能启动和运行，但执行结果没有稳定回流到 Cozmio。
- Cozmio 缺少对一天工作过程的沉淀。
- Cozmio 还不能把 Claude Code、relay、subagent 等执行端痕迹转化成后续可用记忆。

用户真正关心的不是“多做一个 memory 功能”，而是 Cozmio 能否逐渐形成一个自我迭代循环：它观察桌面，理解事实材料，借助执行端完成任务，记录执行痕迹，再用这些痕迹改善下一次观察、弹窗和项目迭代。

## 2. Vision

最终形态中，Cozmio 应该有两条并行线：

1. 端侧观察线
   - 持续观察桌面窗口、截图、用户操作和当前上下文。
   - 使用本地视觉模型生成弹窗内容或工具调用意图。
   - 负责轻量、即时、低延迟的观察和交互。
   - 不承担大规模长期日志理解。

2. 执行端迭代线
   - 接入 Claude Code、relay、subagent、未来更多执行端。
   - 执行更重的任务、读取更长日志、生成更高质量总结。
   - 把执行结果、失败原因、阶段性进展沉淀成带 provenance 的记忆。
   - 反过来为端侧观察线提供少量高价值上下文。

长期目标是：当用户离开桌面时，Cozmio 仍然可以在用户授权范围内继续运行、观察、执行、沉淀和准备下一步。但 H2 不直接做完全自主运行。H2 的目标是先让实践闭环可信：弹窗是否更有用、日志是否能回流、记忆是否有来源、系统是否没有伪造语义。

## 3. Product Type

H2 是混合型阶段：

- 软件可靠性阶段：清理 runtime 假语义，增强测试 gate，修正评估命令，建立 trace/export 能力。
- 模型输出验证阶段：用真实样本评估本地模型弹窗质量，而不是只看测试是否通过。
- 记忆架构阶段：定义执行端如何把长日志变成带来源的模型生成记忆。
- 自我迭代准备阶段：让 Cozmio 能看见自己项目的执行痕迹，但不由系统硬编码“这是项目迭代机会”。

成功不是弹窗变少，而是弹窗更具体、更有依据、更能进入用户当前工作流。

## 4. Core Philosophy

### 4.1 Model Is The Semantic Layer

本项目的核心哲学是：模型是高维语义层，系统代码是低维事实和工具层。

系统代码可以提供：

- 时间戳
- 窗口标题
- 进程名
- 截图
- trace id
- session id
- source path
- record id
- duration/count
- raw model output
- raw execution output
- raw error text
- UI event name
- tool affordance
- provenance metadata

系统代码不能生成：

- 用户意图
- 任务阶段
- 项目阶段
- 用户是否卡住
- 当前页面是否重要
- 当前行为是否值得弹窗
- 当前是否是项目迭代机会
- 模型是否应该沉默
- 弹窗是否语义上有用

如果系统需要语义文本，这些语义必须来自：

- 本地模型输出
- 执行端 Agent 输出
- 用户明确写下的文本

并且必须保存 provenance：

- timestamp
- source path
- source range 或 record id
- producer
- raw summary text

### 4.2 System Must Not Put A Harness On The Model

Cozmio 不应该用低维程序给模型加枷锁。

禁止方向：

- 不添加 popup cooldown。
- 不添加 frequency cap。
- 不因为用户快速切换窗口就机械压制弹窗。
- 不要求模型输出 `decision: popup | silence`。
- 不添加 `should_popup` 或 `should_silence` 字段。
- 不用程序判断“信息不够所以不能弹”。
- 不把弹窗频繁本身视为失败。

问题不在于模型想说，而在于系统给它的材料、工具和回流不足。H2 应该改善材料和闭环，而不是限制模型。

### 4.3 Context Pack Is A Fact Harness

上下文包不是系统总结，不是系统判断，也不是系统给模型的语义暗示。

它应该是事实材料：

```text
current_window: title="...", process="..."
process_context: stay_duration_seconds=..., switches_last_minute=..., is_oscillating=...
action_log_tail:
- timestamp=..., age_seconds=..., window="...", action=..., result="...", error="..."
```

上下文包不能包含：

```text
user_intent=...
project_phase=...
task_stage=...
stuck=true
iteration_opportunity=true
popup_strategy=...
should_silence=...
```

## 5. Current State

H1 已经完成或部分完成以下方向：

- `prompt_context.rs` 已开始生成 compact factual context。
- `model_client.rs` 已开始告诉模型系统材料只是事实，不是结论。
- `semantic_boundary.rs` 已存在，用于防止 runtime prompt code 引入硬编码语义或弹窗控制词。
- `AGENTS.md` 已记录语义边界。
- `execution-agent-memory-loop-design.md` 已初步定义执行端记忆循环。
- `cargo test -p cozmio` 当前可通过。
- `cargo test -p cozmio --test semantic_boundary -- --nocapture` 当前可真实执行 integration test。

但 H1 仍留下几个关键风险：

- runtime 仍然有 legacy 字段名：`judgment`、`level`、`confidence`、`CONTINUE`、`ABSTAIN`。
- 一些系统路由行为仍然可能被记录得像模型判断。
- 某些 `confidence=1.0` 并不是模型真实产出，而是系统填值。
- 本地模型配置可能指向 placeholder endpoint 或 placeholder model，导致评估结果不可信。
- 还没有真实样本证明 H1 factual context 对弹窗质量有帮助。
- `cozmio_memory` 中可能有 seed/search/slice/thread linker 生成或保留了语义内容，但 provenance 边界不够清晰。
- Claude Code 会话、subagent 日志、relay 日志还没有形成可复用的执行端记忆沉淀方式。

## 6. H2 Goal

H2 的目标是把 H1 的事实底座推进成一个真实可验证的实践闭环。

H2 要回答四个问题：

1. 当前 runtime 里还有哪些假语义、假 confidence、假闭环字段？
2. 本地模型在真实桌面样本上，拿到 factual context 后是否真的更有用？
3. 执行端日志和 Claude Code 会话应该怎样沉淀成长期记忆，而不塞爆端侧模型？
4. Cozmio 如何开始具备自我迭代能力，但不由系统硬编码“现在应该迭代项目”？

## 7. H2 Scope

### 7.1 Runtime Fake Semantics Audit

审计以下 runtime 字段：

- `judgment`
- `level`
- `confidence`
- `grounds`
- `CONTINUE`
- `ABSTAIN`
- `system_action`
- `next_step`
- `content_text`
- `result_text`
- `error_text`
- `user_feedback`

需要把当前混在一起的东西拆开理解：

```text
model_output: 模型原文
system_route: 系统事实路由，例如 awaiting-confirmation / error / relay-dispatched
ui_event: 用户界面动作，例如 confirm / cancel / dismiss
execution_result: 执行端返回结果
error_text: 系统或执行端错误
```

H2 不要求一次性删除 legacy schema，因为这可能影响历史记录和 UI。但必须建立边界：

- 模型没有输出 confidence 时，系统不得写 `confidence=1.0` 伪装模型确定性。
- UI 事件不得记录成模型 judgment。
- 系统路由不得记录成模型语义判断。
- compatibility field 可以存在，但必须有更清楚的 factual alias 或注释。

### 7.2 Real Local Model Evaluation

必须做真实样本评估，而不是只跑单元测试。

样本桶至少包括：

1. active work
   - 用户正在写代码、调试、规划、对话或操作工具。
2. ambiguous screen
   - 屏幕证据弱，模型应该自然表达不确定或输出很少。
3. execution trace visible
   - 屏幕上可见 Claude Code、relay、终端或 subagent 输出。
4. high-risk action
   - 删除、覆盖、发布、提交等风险动作。
5. context-tail useful
   - 最近 action log tail 有 UI 反馈或执行结果，理论上可帮助当前输出。

每个样本保存：

- screenshot
- window title
- process name
- factual context block
- prompt text 或 prompt hash
- raw model output
- model name
- call duration
- config snapshot
- timestamp
- human review verdict

评估结果：

- PASS：具体、有依据、能进入工作流。
- PARTIAL：有依据，但泛、薄、迟滞或帮助有限。
- FAIL：幻觉、泛泛建议、依赖系统语义、忽略事实材料。
- ENV_FAIL：Ollama endpoint/model/config 错误，不能作为模型质量样本。

### 7.3 Configuration Truth Check

在任何模型质量评估前，必须确认环境是真的。

需要记录：

- config 中的 `ollama_url`
- config 中的 `model_name`
- `/api/tags` 返回的模型列表
- 实际调用使用的模型
- 是否发生 fallback
- call duration
- error text

如果配置是 `http://test:11434` 或 `test_model` 之类 placeholder，评估必须停止。不能把环境错误当成模型能力差。

### 7.4 Execution-Side Memory Loop

端侧模型上下文小，可能只有几千 token，不适合读取一天日志、Claude Code 会话或 subagent 全量记录。

长日志应该交给执行端 Agent 处理：

```text
raw logs -> execution agent/model summary -> provenance-backed memory -> small selected context for local model
```

执行端可以读取：

- Cozmio action logs
- relay session outputs
- Claude Code conversation logs
- subagent logs
- local project files
- verification reports

执行端可以生成：

- daily_summary
- project_summary
- task_thread_summary
- execution_failure_summary
- user_preference_summary
- memory_candidate

但这些都是模型或执行端产物，不是系统语义。

每条 summary 必须包含：

- timestamp
- producer
- source path
- source record id 或 byte range
- source time range
- raw summary text
- optional confidence only if model explicitly produced it

### 7.5 Claude Code And Subagent Logs As Work Memory

Claude Code 会话和 subagent 日志是用户长期工作的一部分。它们可以成为 Cozmio 长期记忆的重要来源，但不能直接塞给端侧模型。

H2 应该定义一个最小可行流程：

1. 找到本地 Claude Code / subagent logs。
2. 用执行端 Agent 读取指定时间范围或指定项目路径。
3. 让执行端 Agent 生成摘要。
4. 保存摘要和来源。
5. 端侧模型只收到少量被选中的摘要片段或事实引用。

重要边界：

- 系统可以索引路径、时间、文件大小、记录 id。
- 系统不能读到路径里有 `cozmio` 就推断“用户正在迭代 Cozmio”。
- “这段会话表示用户想迭代项目”必须来自模型/执行端总结或用户文本。

### 7.6 Screen Use Memory

Cozmio 还需要一种轻量的屏幕使用记忆，但这也不能变成系统语义判断。

允许记录：

- 窗口标题
- 进程名
- 停留时间
- 切换次数
- 截图 hash 或 sample id
- 用户是否确认/取消/关闭弹窗
- 执行是否完成

不允许系统记录：

- 用户正在学习
- 用户正在写代码
- 用户卡住了
- 用户在做项目规划
- 用户需要帮助

如果需要这些语义，必须由模型对事实材料输出，并带 provenance。

### 7.7 Memory Competition Direction

长期记忆多了以后，不能把所有东西塞给端侧模型。需要 memory competition。

H2 只定义方向，不实现完整系统。

可用机械信号：

- recency
- source type
- explicit user feedback
- record count
- embedding similarity
- exact path match
- exact trace/session link
- execution result status
- model-produced summary score, if model explicitly outputs it

不可用系统伪语义：

- 这条更重要
- 这是项目阶段
- 这是用户意图
- 这是卡点
- 这是迭代机会

竞争结果应该输出少量候选，例如 3 到 6 条，而不是长上下文。

### 7.8 Self-Iteration Loop

Cozmio 的自我迭代不是写死“看到 Claude Code 就改项目”。正确方式是提供事实和工具，让模型或执行端判断。

事实链可以是：

```text
current_window: Claude Code
visible_text: project files / task discussion / test failure
recent_action_log: prior popup / execution result
execution_trace: relay completed or failed
available_tool: can ask execution agent to inspect project
```

模型或执行端可以据此产生语义：

```text
这可能与 Cozmio 项目迭代有关，因为...
建议让执行端检查...
```

但系统不能直接写：

```text
iteration_opportunity=true
user_is_working_on_cozmio=true
```

H2 的自我迭代目标是准备闭环：

1. Cozmio 观察到事实。
2. 模型提出可进入工作流的建议。
3. 用户确认或未来策略允许执行。
4. 执行端执行。
5. 执行结果回流。
6. 执行端生成 provenance-backed summary。
7. 后续弹窗可以引用少量相关记忆。

## 8. Data Flow

### 8.1 Immediate Popup Flow

```text
foreground window + screenshot
  -> factual process context
  -> action log tail
  -> local model prompt
  -> raw model output
  -> desktop popup / confirmation
  -> UI event / execution route
  -> factual action log
```

### 8.2 Execution Memory Flow

```text
Cozmio logs + relay logs + Claude Code logs + subagent logs
  -> execution-side reader
  -> execution agent/model summary
  -> provenance-backed memory record
  -> memory competition
  -> small context candidate
  -> local model context pack
```

### 8.3 Practice Evaluation Flow

```text
sample capture
  -> screenshot + factual context + raw output
  -> human review
  -> PASS/PARTIAL/FAIL/ENV_FAIL
  -> prompt/context/memory design adjustment
```

## 9. Storage Concepts

H2 can introduce or prepare these conceptual records.

### 9.1 Factual Event

```text
record_type: factual_event
timestamp: ...
trace_id: ...
window_title: ...
process_name: ...
event_name: ui_cancelled / ui_confirmed / relay_completed / model_error
raw_text: ...
source: cozmio_runtime
```

### 9.2 Model Output Record

```text
record_type: model_output
timestamp: ...
trace_id: ...
model_name: ...
prompt_hash: ...
context_hash: ...
raw_output: ...
call_duration_ms: ...
source_window: ...
```

### 9.3 Execution Summary

```text
record_type: execution_summary
timestamp: ...
producer: claude_code / relay_agent / subagent / local_model
source_paths: [...]
source_record_ids: [...]
source_time_range: ...
summary_text: ...
```

### 9.4 Memory Candidate

```text
record_type: memory_candidate
created_at: ...
source_summary_id: ...
selection_facts: recency / embedding_similarity / explicit_user_feedback
text: ...
```

These are conceptual shapes, not final schema. Implementation planning should decide whether to use existing `cozmio_memory` tables, JSONL, or a migration path.

## 10. Evaluation Criteria

H2 succeeds if:

- `semantic_boundary` integration test cannot false-pass with 0 tests.
- Runtime code does not reintroduce hardcoded semantic prompt terms.
- UI-only events are not recorded as model judgments.
- Fake system confidence is removed, downgraded, or clearly marked compatibility-only.
- At least 5 real samples are reviewed.
- Evaluation separates model failure from environment failure.
- At least one result says whether H1 factual context helped, harmed, or made no difference.
- Execution-side memory summaries have a provenance design ready for implementation.

H2 fails if:

- Popup count reduction becomes the main metric.
- System code adds silence/cooldown/frequency rules.
- System code creates task-stage or user-intent labels.
- Long logs are directly injected into local model prompt.
- `[ERROR]` output is counted as model behavior.
- Claude Code logs are treated as semantic memory without model/user provenance.

## 11. Non-Goals

H2 will not:

- Build a full autonomous overnight self-runner.
- Implement complete vector memory competition.
- Migrate all historical memory data.
- Force a structured model output protocol.
- Solve all popup quality issues at once.
- Replace the local model.
- Add programmatic popup suppression.

## 12. Proposed H2 Implementation Slices

### Slice A: Test And Boundary Correctness

- Ensure `semantic_boundary.rs` scans runtime code and ignores test-only assertions.
- Ensure documented test command uses `--test semantic_boundary`.
- Add or update tests that prevent reintroduction of hardcoded semantic prompt terms.

### Slice B: Runtime Log Semantics Audit

- Find all `confidence: 1.0` values that are system-created.
- Separate model raw output from system route and UI event.
- Downgrade UI events to factual records.
- Preserve compatibility where needed, but document it.

### Slice C: Real Model Evaluation Runbook

- Verify Ollama config.
- Capture real samples.
- Save screenshot, context, raw output, metadata.
- Review samples with PASS/PARTIAL/FAIL/ENV_FAIL.

### Slice D: Execution-Side Memory Provenance

- Define where execution summaries are stored.
- Define how source references are represented.
- Define how small selected memories enter local context.
- Avoid direct long-log injection.

### Slice E: First Self-Iteration Trial

- Use a real Cozmio-related work session as sample.
- Let model or execution agent identify whether it is useful to continue project iteration.
- If user confirms, route to execution endpoint.
- Store execution result and summary.
- Evaluate whether later popup improved.

## 13. Open Questions

1. Should legacy fields be renamed in H2, or should H2 add factual aliases first and postpone schema migration?
2. Should real sample capture be manual first, or should Cozmio add a debug export command?
3. Should execution summaries initially live in `cozmio_memory`, or in a simpler JSONL store until provenance stabilizes?
4. Should Claude Code logs be summarized daily, per project session, or only when Cozmio detects a relevant execution trace?
5. Should memory competition initially use only mechanical ranking, or include model-generated summary quality when available?

## 14. Review Notes

This document is intentionally broader than the previous H2 summary. It preserves the discussions about:

- model-led popup behavior
- no system-authored semantics
- no mechanical silence
-端侧模型 context limits
- execution-side stronger agents
- Claude Code and subagent logs as long-term work memory
- vector memory and memory competition as future direction
- self-iteration as a loop, not a hardcoded rule

The next step after review is not to implement every section at once. The next step is to turn the first narrow slice into a plan: test/boundary correctness plus runtime fake semantic audit.
