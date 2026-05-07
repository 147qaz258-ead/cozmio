# Practice Loop V1 Phase F — Evaluation Loop 实施方案

> **智能执行体须知**：本方案以运行效果、验证资产和飞轮写回为中心。默认不含代码示例。

## 1. Flywheel Context

- active task: PRACTICE-LOOP-V1-PHASE-E (COMPLETE)
- current phase: Phase F — Evaluation Loop
- latest verification: Phase E complete, cargo build pass, 118 cozmio tests pass, Practice Dashboard 4 tabs working
- blocker: none
- next expected step: Phase F implementation

## 2. Goal

捕获真实工作样本，建立评估循环：用执行 agent 对样本做 groundedness 判断，结果存储并反馈到 prompt/context 设计。

## 3. Product Type

- type: `model_output_validated` + `deterministic_software`
- core risk: 样本捕获完整性、evaluator 输出可信度、存储 schema 可扩展
- verification style: 固定样本集评估 + cargo build + cargo test

## 4. Global Roadmap

| Phase | 目标 | 依赖 | 验收意图 |
|-------|------|------|---------|
| H1 | Sample Capture — 样本捕获 schema + 手动采集 UI | Ledger IPC 已注册 | 真实 trace 生成可评估的 sample 记录 |
| H2 | Evaluator IPC — 执行 agent 评估接口 | Phase H1 sample schema | 调用 evaluator，对样本打分（pass/partial/fail）+ groundedness notes |
| H3 | Evaluation Store — 评估结果持久化 | Phase H2 evaluator output | 将 evaluator output 存储，支持查询历史评估 |
| H4 | Evaluation Dashboard Tab — 评估结果查看 | Phase H2+H3 | Practice Dashboard 新增 Evaluation tab，历史样本 + 评分分布 |
| H5 | Loop Closing — 用评估结果反馈 prompt/context 设计 | Phase H4 可用数据 | 基于 groundedness 报告识别 hallucination/abstain 模式，生成调整建议 |

## 5. Scope

### In（本次包含）

- EvaluationSample schema（Rust）：screenshot_ref, window_facts, context_pack, model_raw_output, user_action, execution_result, captured_at, source_trace_id
- EvaluationResult schema（Rust）：sample_id, evaluator_output (pass/partial/fail), groundedness_notes, recommendation, evaluated_at
- `save_evaluation_sample(sample)` IPC 命令 — 手动触发或从 ledger event 自动生成
- `evaluate_sample(sample_id)` IPC 命令 — 调用执行 agent（relay bridge）做评估
- `get_evaluation_results(limit)` IPC 命令 — 查询历史评估
- Practice Dashboard 新增 Evaluation tab：展示样本列表 + 评估结果

### Out（本次不包含）

- 自动定时样本采集（manual trigger only）
- Claude Code transcript import UI
- Evaluator prompt 自动优化（Phase H5 需要先有数据）
- 多 evaluator 集成（Phase F 只用 relay bridge 作为 evaluator）

## 6. Current Truth

- files inspected:
  - `cozmio/src-tauri/src/commands.rs` — IPC 命令注册模式
  - `cozmio/src-tauri/src/relay_bridge.rs` — relay dispatch + execution trace
  - `cozmio/src-tauri/src/ledger.rs` — LedgerEvent schema
  - `docs/superpowers/specs/2026-04-29-practice-loop-v1-design.md` — Section 14 Evaluation Loop
- existing entry points:
  - `distill_commands.rs` — MemoryCandidate/DistillationJob schema，参考 IPC 注册模式
  - `commands.rs` — 已有 `get_timeline`, `get_trace_detail`, `rebuild_ledger_projection` 等 ledger IPC
- existing runtime path:
  - Ledger → Memory Candidate → Distillation → Competition → Dashboard
  - 新增路径：Ledger/Execution Trace → Evaluation Sample → Evaluator → Evaluation Result → Dashboard
- existing verification: `cargo test -p cozmio` pass, 118 tests

## 7. Implementation Steps

### Step 1: Schema 定义（Rust）

在 `src-tauri/src/` 新建 `evaluation.rs`：

- `EvaluationSample` struct：字段包括 id, source_trace_id, screenshot_path, window_facts_json, context_pack_summary, model_raw_output, user_action_description, execution_result_summary, captured_at
- `EvaluationResult` struct：字段包括 id, sample_id, judgment (pass/partial/fail), groundedness_notes, recommendation, evaluated_by (relay_agent), evaluated_at
- `EvaluationStore`（参考 `DistillationJobStore`）：SQLite 持久化，methods: `save_sample`, `get_sample`, `save_result`, `get_results`

### Step 2: IPC 命令注册

在 `commands.rs` 或新建 `eval_commands.rs`：

- `save_evaluation_sample(app, trace_id)` — 从指定 trace_id 提取事件，生成 EvaluationSample，写入 store
- `evaluate_sample(app, sample_id)` — 调用 relay_bridge dispatch，传入 sample 上下文，收到 execution result 后存储为 EvaluationResult
- `get_evaluation_samples(limit)` — 返回样本列表
- `get_evaluation_results(limit)` — 返回评估结果列表
- 将以上命令注册到 `main.rs` 的 `generate_handler![]`

### Step 3: Practice Dashboard — Evaluation Tab

在 `PracticeDashboard.js` 新增第四个 tab（与 Timeline/Inbox/Preview/Signals 并列）：

- Evaluation tab 内容：样本列表（每条显示 source_trace_id + captured_at + judgment 标签）
- 点击样本展开：显示 window_facts、model_raw_output、groundedness_notes、recommendation
- "Capture Sample" 按钮：调用 `save_evaluation_sample`，从当前 ledger trace 生成样本
- "Evaluate" 按钮：对选中样本调用 `evaluate_sample`

### Step 4: 样本自动触发（可选，manual trigger）

在 `main_loop.rs` 的 ledger 事件处理中，当 `relay_execution_result_received` 事件触发时，可选调用 `save_evaluation_sample` 自动捕获本次 execution 作为样本（当前为 manual trigger，UI 按钮触发）。

### Step 5: Build + Test

- `cargo build -p cozmio` — 验证编译通过
- `cargo test -p cozmio` — 验证全部测试通过
- 手动测试：启动应用 → Practice Dashboard → Evaluation tab → Capture Sample → Evaluate → 查看结果

## 8. Verification Asset

- verification type: deterministic_software + model_output_validated
- command:
  ```bash
  cargo build -p cozmio
  cargo test -p cozmio
  ```
- expected evidence:
  - Build: 0 errors, warnings only
  - Tests: all pass（sample/evaluation 相关的 relay_bridge 测试如果存在则覆盖）
- failure condition:
  - `cargo build` 有 error
  - `cargo test` 有 test failure（非 pre-existing 的）
- writeback targets:
  - `verification/last_result.json`
  - `feature_list.json`
  - `claude-progress.txt`

## 9. Phase Gate

本 Phase 只有满足以下条件才能标记为完成：

- [ ] `cargo build -p cozmio` 通过
- [ ] `cargo test -p cozmio` 通过（全部测试）
- [ ] Evaluation tab 可访问（Practice Dashboard sidebar → Evaluation tab）
- [ ] Capture Sample 成功生成 EvaluationSample 记录
- [ ] Evaluate 成功调用 relay_bridge 并存储 EvaluationResult
- [ ] `verification/last_result.json` 已更新
- [ ] `feature_list.json` 已添加 Phase F 条目
- [ ] `claude-progress.txt` 已更新下一轮交接

## 10. Next Execution Step

- next phase: H1 — Sample Capture + Evaluator IPC
- goal: 建立样本捕获 schema、手动采集 UI、evaluator 接口
- entry skill: `superpowers:subagent-driven-development`
- stop condition: Phase F H1-H3 完成，Evaluation tab 可用，cargo build + test 通过