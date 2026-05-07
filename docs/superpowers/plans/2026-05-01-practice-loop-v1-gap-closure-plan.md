# Practice Loop V1 剩余设计闭环一次性完成实施方案

> **智能执行体须知**：本方案以运行效果、验证资产和飞轮写回为中心。默认不含代码示例。本方案用于把 `docs/superpowers/specs/2026-04-29-practice-loop-v1-design.md` 中尚未落地的设计项一次性收敛到可验证状态。

## 1. Flywheel Context

- active task: Practice Loop V1 设计文档剩余开发闭环
- current phase: Phase G Competition→Judgment Pipeline 已接入；主循环 Step 6 阻塞修复已验证通过；当前产品剩余现象是外部 Ollama/model endpoint 返回 502 导致无法验证真实弹窗质量
- latest verification:
  - `verification/last_result.json`: `MAIN-LOOP-STEP6-BLOCKING` = `pass_with_external_model_service_failure`
  - `verification/last_verification.json`: `cargo fmt --check` pass；`CARGO_TARGET_DIR=target/verify-main-loop cargo build -p cozmio` pass；`CARGO_TARGET_DIR=target/verify-main-loop cargo test -p cozmio -- --test-threads=1` pass；桌面运行日志证明主循环越过 Step 6 并到达 model call
- blocker:
  - 真实 model-output / popup 质量验证被当前配置 `http://test:11434` + `test_model` 的 502 阻塞
  - 默认 `target/debug/cozmio.exe` 在 Windows 上可能被运行中的桌面进程锁住，构建验证需要使用独立 `CARGO_TARGET_DIR`
  - 仓库当前有大量既有修改和 target 删除记录；实施时只能改动本方案列出的源文件与飞轮文件，不能清理或覆盖无关改动
- next expected step: 执行 H1-H7 一次性闭环，实现 content ref 可解析、transcript import、daily distillation、memory edit/merge/stale、evaluation feedback、display queue，并完成验证与飞轮写回

## 2. Goal

把 Practice Loop V1 从“主要模块存在”推进到“设计文档剩余关键链路全部可运行、可观察、可验证”：内容引用能取回，Claude Code 工作历史能进入蒸馏，记忆候选可编辑合并并有 stale 事实信号，evaluation 结果能进入改进建议池，模型输出有显示队列保护，最终用自动测试、固定样本和桌面运行证据证明链路闭合。

## 3. Product Type

- type: `deterministic_software + execution_trace + desktop_ui_runtime + model_output_validated`
- core risk:
  - deterministic_software: Rust/JS 多文件集成、SQLite schema 迁移、IPC 命令注册、现有测试不回退
  - execution_trace: transcript/content/evaluation/memory 之间的 trace_id、content_ref、source_event_ids 不能断链
  - desktop_ui_runtime: Practice Dashboard 的 Inbox / Evaluation / pending queue 可见状态必须和后端一致
  - model_output_validated: evaluation feedback 与 prompt/context 改进建议必须来自模型或用户输出，系统只存储事实和 provenance
- verification style:
  - `cargo fmt --check`
  - `CARGO_TARGET_DIR=target/verify-practice-loop-v1 cargo build -p cozmio`
  - `CARGO_TARGET_DIR=target/verify-practice-loop-v1 cargo test -p cozmio -- --test-threads=1`
  - `CARGO_TARGET_DIR=target/verify-practice-loop-v1 cargo test -p cozmio_memory -- --test-threads=1`
  - IPC-level smoke tests using deterministic fixtures for content refs, transcript import, memory candidate edit/merge/stale, evaluation feedback records
  - desktop UI Layer 1-3 check through existing `verification/verify_ui.ps1` or `verification/desktop_behavior_test.ps1` when app build is available
  - model-output validation uses saved samples only when a reachable model endpoint is configured; if endpoint still returns 502, mark that part `blocked_by_external_model_service` and keep feature state incomplete for model quality

## 4. Global Roadmap

| Phase | 目标 | 依赖 | 验收意图 |
|-------|------|------|---------|
| H1 | 实现 ContentResolver，让 content_ref 能解析、校验 hash、按 byte/line range 返回材料 | 现有 `ContentRef` / `ContentStoreWriter` | 大内容不再只是可写不可读；distillation/evaluation/importer 可通过统一接口取内容 |
| H2 | 实现 Claude Code transcript import，把项目 transcript 文件作为 ledger content + distillation source | H1 | 至少一个 fixture transcript 能导入 ledger、生成 content_ref、形成可蒸馏 material |
| H3 | 实现 daily/range distillation runner，支持按日期范围收集 ledger + content refs 并生成 memory candidates | H1 + H2 | 从真实 ledger range 或 fixture range 触发蒸馏，输出带 provenance 的 MemoryCandidate |
| H4 | 完成 Memory Inbox 操作：approve/edit/reject/merge + stale factual indicators | H3 | 用户能在 UI/IPC 层修正候选记忆；系统能计算事实型 stale 信号但不写语义结论 |
| H5 | 实现 Evaluation feedback pool，把 EvaluationResult 的 recommendation 作为带 provenance 的改进建议记录，不直接改 runtime 行为 | H1 | evaluation 不再是死数据；prompt/context/competition 改进建议可被查看和导出 |
| H6 | 实现 display-layer pending queue/inbox 保护，记录 displayed_at/queued_at/user_seen_at，不做语义 suppression | 现有 main_loop / UI state / ledger | 所有模型输出入 ledger；UI 控制展示队列，模型仍控制是否输出内容 |
| H7 | 完成综合验证与飞轮写回 | H1-H6 | 测试、运行证据、feature_list、last_result、claude-progress 一致反映真实状态 |

## 5. Scope

### In（本次包含）

- `src-tauri/src/ledger.rs`
  - 增加 `ContentResolver`
  - 增加 content hash 校验
  - 支持 `ContentRef.byte_range` 和 `ContentRef.line_range` 的读取切片
  - 在 `LedgerManager` 上暴露只读解析入口
- `src-tauri/src/distill_commands.rs`
  - 将现有 `distill_trace` 扩展为可复用 material builder
  - 增加按日期范围/ledger range 触发的 distillation runner
  - 使用 ContentResolver 读取 content_ref 指向的大内容，而不是只保存 path
- 新增 `src-tauri/src/transcript_import.rs`
  - 支持导入 Claude Code JSONL transcript fixture 和真实项目 transcript 文件
  - 为每个导入片段写 ledger event，content_type 使用 `transcript`
  - 输出导入摘要：文件数、事件数、content_ref 数、失败文件列表
- `src-tauri/src/memory_commands.rs` 与 `cozmio_memory/src/memory_candidate.rs`
  - 增加 approve/edit/merge 命令或补齐已有状态流
  - 增加 stale factual indicators：`not_selected_for_30_days`、`superseded_by_memory_id`、`source_file_missing`
  - 保持 `memory_text` 只能来自用户编辑、模型/agent 输出或导入文本，不由系统编写语义结论
- `src-tauri/src/evaluation.rs` 与 `src-tauri/src/eval_commands.rs`
  - 增加 EvaluationFeedback 记录表和查询命令
  - `evaluate_sample` 保存结果后同步保存改进建议记录，字段保留 producer、sample_id、source_trace_id、recommendation、groundedness_notes
  - 不自动修改 prompt、competition 权重或 runtime 行为
- `src-tauri/src/main_loop.rs`、`src-tauri/src/ui_state.rs`、`src-tauri/src/components/StatusPanel.js`、`src-tauri/src/components/PracticeDashboard.js`
  - 增加 display-layer queue/inbox 数据结构与 UI 可见入口
  - 记录 `queued_at`、`displayed_at`、`user_seen_at`
  - 不新增 cooldown、frequency cap、should_popup、should_silence
- `src-tauri/src/main.rs` 与 capabilities
  - 注册新增 IPC 命令
  - 更新 Tauri capability schema 允许新命令
- `src-tauri/tests/semantic_boundary.rs`
  - 保持禁用语义字段扫描
  - 增加对 queue 保护的正向断言：允许 `queued_at/displayed_at/user_seen_at`，禁止 `cooldown/frequency_cap/should_silence`
- `verification/last_result.json`、`verification/last_verification.json`、`feature_list.json`、`claude-progress.txt`
  - 实施完成后写入真实验证结果

### Out（本次不包含）

- 不修复外部模型服务本身；如果 Ollama endpoint 仍返回 502，记录为外部服务阻塞，不把 model-output quality 标记为 pass
- 不实现完全自动无人值守运行；设计文档 Section 22 明确 V1 不做 fully autonomous operation
- 不把 evaluation recommendation 自动应用到 prompt 或 competition 权重；本次只建立可审查 feedback pool
- 不导入所有历史 Claude Code transcript；本次实现 importer 和 fixture/指定路径导入，不批量扫描全盘
- 不清理仓库中已有的 `target/` 删除记录和无关 skill/CLAUDE.md 修改
- 不新增 popup cooldown、frequency cap、system-authored usefulness label

## 6. Current Truth

- files inspected:
  - `docs/superpowers/specs/2026-04-29-practice-loop-v1-design.md`
  - `claude-progress.txt`
  - `feature_list.json`
  - `verification/last_result.json`
  - `verification/last_verification.json`
  - `src-tauri/src/ledger.rs`
  - `src-tauri/src/evaluation.rs`
  - `src-tauri/src/eval_commands.rs`
  - `src-tauri/src/components/PracticeDashboard.js`
- existing entry points:
  - Event Ledger: `LedgerEvent`, `ContentRef`, `LedgerWriter`, `LedgerProjection`, `ContentStoreWriter`, `LedgerManager`
  - Execution return: `relay_bridge.rs` writes execution result/error and supports sample auto-capture
  - Memory distillation: `distill_trace` and `MemoryCandidateStore`
  - Memory competition: `build_activity_context` / `ReminderContextDto` enters `build_popup_context`
  - Dashboard: `PracticeDashboard.js` tabs Timeline / Inbox / Preview / Signals / Evaluation
  - Evaluation: `save_evaluation_sample`, `evaluate_sample`, `get_evaluation_samples`, `get_evaluation_results`
- existing runtime path:
  - foreground capture → main loop → `build_activity_context_sync` with 400ms graceful degradation → `build_popup_context` → model call → executor / UI / ledger
  - relay result → `relay_bridge.rs` → ledger/action log → optional evaluation sample capture
  - distill trace → ledger trace query → distillation backend → memory candidate store → embedding/competition → context pack
- existing verification:
  - last verified with independent target dir: build pass, 126 unit tests pass, 4 semantic boundary tests pass
  - runtime log proves main loop reaches model call; external model endpoint returns 502
- known missing pieces from design doc:
  - content_ref resolver absent; `ledger.rs` explicitly says resolver is not implemented
  - Claude Code transcript import absent; only legacy `action_log.jsonl` importer exists
  - daily/range distillation runner absent; current distillation is trace-level manual path
  - Memory Inbox lacks edit/merge and stale factual indicators
  - EvaluationResult is stored but not converted into a reviewable feedback pool
  - display-layer queue/inbox protection is not implemented as specified by Section 3.2

## 7. Implementation Shape（当前一次性执行包）

1. **H1 ContentResolver**
   - Add `ContentResolutionRequest` and `ResolvedContent` in `ledger.rs`.
   - Implement `ContentResolver::resolve(&ContentRef)` for `storage_backend == "file"` only.
   - Validate that resolved bytes match `content_hash` before slicing.
   - Apply `byte_range` before UTF-8 conversion when present; apply `line_range` after UTF-8 conversion when present.
   - Expose `LedgerManager::resolve_content_ref(&self, content_ref: &ContentRef)`.
   - Add unit tests for full text, byte range, line range, missing file, hash mismatch, unsupported backend.

2. **H2 Transcript Import**
   - Create `transcript_import.rs` with importer functions that accept explicit file paths or a directory path under the user-provided project transcript root.
   - Parse JSONL lines conservatively: preserve each valid line as source material, extract timestamp/session-like fields only when present, and never infer user intent.
   - Store each imported chunk through `LedgerManager::with_content_store(...).record_event_with_content(...)` using `event_type = "transcript_imported"`, `source = "claude_code_transcript_import"`, `content_type = "transcript"`.
   - Return `TranscriptImportSummary { files_scanned, events_imported, content_refs_created, skipped_lines, failed_files }`.
   - Register IPC commands `import_claude_transcripts` and `get_transcript_import_summary`.
   - Add tests using fixture JSONL containing one valid Claude-like message line, one tool-result-like line, and one invalid line; expected result imports valid lines and reports skipped invalid line.

3. **H3 Daily / Range Distillation**
   - Refactor `distill_commands.rs` material assembly so trace-level and date-range distillation share one builder.
   - Add `distill_event_range(start_timestamp, end_timestamp)` command.
   - Collect ledger events in timestamp range, include execution result refs, transcript refs, and raw_text fields.
   - Use `ContentResolver` to expand transcript/relay/model output refs into bounded text material for the execution agent; cap per-content material by a fixed byte limit recorded as factual metadata.
   - Store `DistillationJob` with `trigger = "event_range"`, input event ids and content refs.
   - Add tests that insert fixture ledger events with content refs, run range material builder, and assert source_event_ids/source_paths/source_ranges survive into MemoryCandidate.

4. **H4 Memory Inbox completion**
   - Extend memory candidate storage with factual status transitions: `active`, `approved`, `rejected`, `merged`, `superseded`.
   - Add `edit_memory_candidate(memory_id, new_memory_text, editor_producer)` command. The `editor_producer` must be `user` or model/agent producer string supplied by caller; system does not generate replacement semantics.
   - Add `merge_memory_candidates(primary_memory_id, merged_memory_ids, merged_text, producer)` command. Store merged candidate with source ids from all merged candidates and mark merged inputs as `superseded` with `supersedes` or `superseded_by` relationship.
   - Add stale factual computation command `get_memory_factual_indicators(memory_id)` returning booleans and source facts only: selected count, days since last selected, source files missing, superseded target.
   - Update `PracticeDashboard.js` Inbox cards with Approve, Edit, Reject, Merge entry points and visible source ids.
   - Add tests for edit provenance, merge provenance, stale selected-30-days calculation, source-file-missing detection.

5. **H5 Evaluation feedback pool**
   - Add `evaluation_feedback` SQLite table with fields: `id`, `sample_id`, `source_trace_id`, `producer`, `groundedness_notes`, `recommendation`, `created_at`, `status`.
   - On `evaluate_sample_impl`, after saving `EvaluationResult`, save one `EvaluationFeedback` row.
   - Add commands `get_evaluation_feedback(limit, status)` and `mark_evaluation_feedback_reviewed(id)`.
   - Show feedback in Practice Dashboard Evaluation tab as reviewable recommendations.
   - Keep this as evidence for human review; do not modify prompt files, runtime prompt assembly, competition weights, or model config automatically.
   - Add tests for feedback row creation and retrieval after evaluation result save.

6. **H6 Display-layer queue/inbox protection**
   - Add factual pending-output queue structures in UI state: output id, trace_id, raw model text reference, queued_at, displayed_at, user_seen_at, status.
   - Route every non-empty model output to ledger first, then enqueue it if another pending item is active.
   - UI displays one active pending item and exposes queued items in Practice Dashboard or StatusPanel details.
   - User actions update factual status: seen, dismissed, confirmed, cancelled.
   - Add semantic boundary assertions that queue fields are allowed factual timestamps while suppression words remain forbidden.
   - Add UI tests or DOM/state checks showing two fixture pending outputs result in one active and one queued item.

7. **H7 Verification and flywheel writeback**
   - Run formatting, build, unit tests, semantic boundary tests, and UI/runtime smoke checks using an isolated target directory.
   - If external model endpoint still returns 502, record deterministic/runtime parts as pass and model-output quality as blocked, not pass.
   - Update `verification/last_result.json` and `verification/last_verification.json` with command list, evidence, uncovered areas, and failure conditions.
   - Update `feature_list.json` with a new `PRACTICE-LOOP-V1-GAP-CLOSURE` feature entry and per-H phase statuses.
   - Update `claude-progress.txt` with exact next handoff: what passed, what remains blocked by external model service, how to run the next validation.

## 8. Verification Asset

- verification type: `deterministic_software + execution_trace + desktop_ui_runtime + model_output_validated`
- command / script:
  - `cargo fmt --check`
  - `CARGO_TARGET_DIR=target/verify-practice-loop-v1 cargo build -p cozmio`
  - `CARGO_TARGET_DIR=target/verify-practice-loop-v1 cargo test -p cozmio -- --test-threads=1`
  - `CARGO_TARGET_DIR=target/verify-practice-loop-v1 cargo test -p cozmio_memory -- --test-threads=1`
  - `powershell -ExecutionPolicy Bypass -File verification/verify_ui.ps1` when the desktop executable can be launched without lock conflicts
  - fixture-driven IPC smoke path for transcript import, content resolution, range distillation, memory edit/merge, evaluation feedback, pending queue state
- expected evidence:
  - ContentResolver tests pass for hash validation and range slicing
  - transcript import fixture produces ledger events with `content_type=transcript` and resolvable content refs
  - range distillation fixture produces MemoryCandidate with source_event_ids and content refs preserved
  - memory edit/merge commands preserve producer/source provenance and do not invent semantic labels
  - evaluation feedback rows appear after evaluation result save
  - pending queue stores all model outputs in ledger and exposes factual queue timestamps in UI state
  - semantic boundary tests continue to pass and do not allow `should_popup`, `should_silence`, `cooldown`, `frequency cap`
- evidence location:
  - `verification/last_result.json`
  - `verification/last_verification.json`
  - cargo test output in session transcript
  - UI verification output or screenshot path if UI check is run
  - ledger fixture files under test temp directories only
- failure condition:
  - any cargo build/test failure
  - any semantic boundary test failure
  - any resolver hash mismatch not reported as error
  - any transcript import path that stores bare path without content_ref
  - any memory candidate edit/merge that drops source_event_ids/source_paths/source_ranges
  - any evaluation recommendation that directly mutates runtime prompt/context behavior without review
  - any display queue implementation that suppresses model outputs or adds semantic popup policy
  - external model endpoint 502 blocks only model-output quality validation; it does not fail deterministic H1-H6 unless runtime code regresses
- writeback targets:
  - `verification/last_result.json`
  - `verification/last_verification.json`
  - `feature_list.json`
  - `claude-progress.txt`

## 9. Phase Gate

本一次性执行包只有满足以下条件才能标记为完成：

- [ ] H1-H6 源码修改已完成，新增 IPC 已注册，capability 允许调用
- [ ] `cargo fmt --check` 已运行并通过
- [ ] isolated target build 已运行并通过
- [ ] isolated target tests 已运行并通过，包含 semantic boundary tests
- [ ] transcript/content/evaluation/memory/queue fixture 证据能支撑链路结论
- [ ] UI Layer 1-3 自动化已运行；如果 Windows exe lock 或外部服务阻塞，证据中明确标注
- [ ] `verification/last_result.json` 已更新，含时间戳、验证类型、命令、结果摘要、失败/阻塞项
- [ ] `verification/last_verification.json` 已更新，记录每条验证命令与证据
- [ ] `feature_list.json` 已新增或更新 `PRACTICE-LOOP-V1-GAP-CLOSURE`，不得把被 502 阻塞的 model-output quality 写成 pass
- [ ] `claude-progress.txt` 已写入下一轮交接内容

## 10. Next Execution Step

- next phase: H1-H7 一次性执行包
- goal: 按本方案完成 Practice Loop V1 设计文档剩余关键链路，并以 deterministic/runtime/model evidence 写回飞轮
- entry skill: `superpowers:subagent-driven-development`（推荐，用于 Phase 粒度执行）
- stop condition:
  - 所有 H1-H6 功能修改完成且 H7 验证与飞轮写回完成；或
  - 出现无法绕过的外部服务阻塞，且代码验证已完成、阻塞项已写入 `verification/last_result.json` 和 `claude-progress.txt`

## 自我审查

- Flywheel Context 检查：已读取 `claude-progress.txt`、`feature_list.json`、`verification/last_result.json`、`verification/last_verification.json`。
- 产品类型检测：本方案标注 `deterministic_software + execution_trace + desktop_ui_runtime + model_output_validated`，每类均有对应验证资产。
- 规格覆盖度：覆盖设计文档剩余缺口 ContentResolver、transcript import、daily/range distillation、Memory Inbox edit/merge/stale、Evaluation feedback、display queue、semantic boundary、飞轮写回。
- 占位符排查：正文不使用待定占位词；所有 Phase 均有明确目标、入口和验收证据。
- 类型一致性：复用现有 `LedgerEvent`、`ContentRef`、`LedgerManager`、`EvaluationResult`、`MemoryCandidate`、`PracticeDashboard.js`、`build_activity_context` 等实际入口。
- Phase Gate 检查：H7 明确完成条件，外部模型 502 不被美化为 pass。
- 全局路线图检查：H1-H7 目标、依赖、验收意图明确；当前执行包给出 7 个可实施步骤。
- 计划保存检查：本文件保存到 `docs/superpowers/plans/2026-05-01-practice-loop-v1-gap-closure-plan.md`。本次未创建 Git 提交，因为当前会话没有收到明确提交授权，且仓库存在大量既有未提交改动。
