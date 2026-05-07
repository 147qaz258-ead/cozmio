# Practice Loop V1 Phase E — Practice Dashboard 实施方案

> **智能执行体须知**：本方案以运行效果、验证资产和飞轮写回为中心。默认不含代码示例。

## 1. Flywheel Context

- active task: PRACTICE-LOOP-V1-PHASE-D (COMPLETE)
- current phase: Phase E — Practice Dashboard
- latest verification: Phase D complete, cargo build pass, 118 cozmio tests pass, 18 competition tests pass
- blocker: 7 pre-existing cozmio_memory lib test failures (Windows SQLite temp dir isolation) — not blocking
- next expected step: Phase E implementation

## 2. Goal

让用户能看到 Practice Loop 的完整运转：从观察到记忆重用的链条可视化、记忆候选收件箱管理、上下文包预览、效果信号统计。

## 3. Product Type

- type: `desktop_ui_runtime` + `deterministic_software`
- core risk: UI 正确渲染数据、IPC 数据链路完整、后端查询方法可用
- verification style: cargo build + cargo test + UI automation (Layer 1-3) + 截图证据

## 4. Global Roadmap

| Phase | 目标 | 依赖 | 验收意图 |
|-------|------|------|---------|
| H1 | Loop Timeline — 事件时间线面板 | Ledger IPC 注册 | 时间线渲染 ledger 事件，按 trace 分组可展开 |
| H2 | Memory Inbox — 记忆候选收件箱 | Memory Candidate IPC 已有 | 列出 active 候选，可 reject，显示 producer/source |
| H3 | Context Pack Preview — 上下文包预览面板 | Competition IPC 已有 | 调用 compete_for_context，展示选中记忆 + token 预算 + trace |
| H4 | Effect Signals — 效果信号统计 | 各 IPC 数据汇总 | 统计卡片：confirmed/cancelled/execution success/memory selection |

## 5. Scope

### In（本次包含）

- 注册 `get_timeline`、`get_trace_detail`、`rebuild_ledger_projection` 到 invoke_handler（代码已存在，只需注册）
- 新增 `get_distillation_jobs` IPC 命令（列出蒸馏作业历史）
- 新增 `get_effect_signals` IPC 命令（聚合效果信号统计）
- 新增 `PracticeDashboard.js` 前端面板，包含 4 个子视图（Timeline / Inbox / Preview / Signals）
- App.js sidebar 新增 Practice 导航项
- `distill_commands.rs` 新增 DistillationJobStore list 方法

### Out（本次不包含）

- 记忆候选的 edit / merge 功能（设计文档提到但 V1 不做）
- Claude Code transcript import UI
- 评估样本采集 UI（Phase F 内容）
- 定时自动蒸馏调度（仅 manual trigger）
- Dashboard 数据持久化/缓存策略优化
- 记忆候选 approve 流程（当前设计只有 active/rejected 状态，approve 意味着保持 active）

## 6. Current Truth

- files inspected:
  - `src-tauri/src/main.rs` — invoke_handler 注册 38 个命令
  - `src-tauri/src/commands.rs` — `get_timeline`/`get_trace_detail`/`rebuild_ledger_projection` 已定义但未注册
  - `src-tauri/src/ledger.rs` — LedgerManager 完整实现，query_timeline/query_trace/query_by_session 可用
  - `src-tauri/src/memory_commands.rs` — 9 个 memory IPC 命令已注册
  - `src-tauri/src/distill_commands.rs` — 4 个 distill IPC 命令已注册
  - `src-tauri/src/competition_commands.rs` — 3 个 competition IPC 命令已注册
  - `src-tauri/src/components/App.js` — sidebar 3 项（status/history/settings），showPanel 路由
  - `src-tauri/src/components/StatusPanel.js` — 复杂面板模式参考
  - `src-tauri/src/components/HistoryList.js` — 列表 + 详情模式参考
  - `cozmio_memory/src/schema.rs` — memory_candidates + distillation_jobs + candidate_embeddings 表
  - `cozmio_memory/src/competition.rs` — CompetitionResultEntry/Trace 完整字段

- existing entry points:
  - `get_timeline(limit, offset)` — 在 commands.rs:97，返回 `Vec<serde_json::Value>`
  - `get_trace_detail(trace_id)` — 在 commands.rs:118，返回 `Vec<serde_json::Value>`
  - `rebuild_ledger_projection()` — 在 commands.rs:296
  - `get_memory_candidates(limit, status)` — 已注册 IPC，返回 `Vec<serde_json::Value>`
  - `reject_memory_candidate(memory_id)` — 已注册 IPC
  - `compete_for_context(note_text, window_title, token_budget)` — 已注册 IPC
  - `get_competition_preview(limit)` — 已注册 IPC
  - `get_memory_stats()` — 已注册 IPC

- existing runtime path: 前端 invoke → Tauri command → Rust backend → SQLite/JSONL → 返回 DTO
- existing verification: cargo build + cargo test，Phase D 全部通过

- key gap: `get_timeline`/`get_trace_detail` 未注册 invoke_handler，前端无法调用
- key gap: 无 `get_distillation_jobs` IPC 命令（DistillationJobStore 缺少 list 方法）
- key gap: 无 `get_effect_signals` IPC 命令（需要聚合多条查询）

## 7. Implementation Shape（仅 H1-H4）

### Step 1: 注册缺失的 Ledger IPC 命令

在 `main.rs` 的 `tauri::generate_handler![]` 中注册 `get_timeline`、`get_trace_detail`、`rebuild_ledger_projection`。

这三个命令的函数体已在 `commands.rs` 中完整实现，只需加到 invoke_handler 数组。

验证：cargo build 通过。前端可调用 `invoke('get_timeline', { limit: 50 })`。

### Step 2: 新增 `get_distillation_jobs` IPC 命令

在 `cozmio_memory/src` 的 distillation store 中添加 `list` 方法（按 created_at DESC，limit 参数）。

在 `distill_commands.rs` 中新增 `get_distillation_jobs(limit: Option<usize>) -> Vec<serde_json::Value>` IPC 命令，注册到 invoke_handler。

返回字段：`job_id, created_at, trigger, trace_id, date, producer, status, error_text`。

验证：cargo build + cargo test -p cozmio_memory。

### Step 3: 新增 `get_effect_signals` IPC 命令

在 `src-tauri/src/` 新增 `dashboard_commands.rs`，实现 `get_effect_signals(app) -> serde_json::Value`。

该命令聚合以下数据：
- 从 ledger 查询 `user_confirmed` / `user_cancelled` / `user_dismissed` 事件计数
- 从 ledger 查询 `execution_result_received` / `execution_error_received` 事件计数
- 从 memory DB 查询 active memory_candidates 总数
- 从 memory DB 查询 rejected memory_candidates 总数
- 从 memory DB 查询 embedding_ref IS NOT NULL 的候选数（embedding 覆盖率）
- 从 competition_preview JSONL 读取最近竞争记录中 memory selection_count 分布

注册到 invoke_handler。

验证：cargo build 通过。

### Step 4: 创建 `PracticeDashboard.js` 面板

创建 `src-tauri/src/components/PracticeDashboard.js`，导出 `createPracticeDashboard()` 函数。

面板结构：

**顶部 Tab 栏**：4 个标签（Timeline / Inbox / Preview / Signals），切换子视图。

**Timeline 子视图**：
- 调用 `invoke('get_timeline', { limit: 100 })`
- 按事件类型显示图标 + 时间 + window_title + raw_text 摘要
- 点击事件调用 `invoke('get_trace_detail', { traceId })` 展开完整 trace
- 事件类型分组：observation → model → confirmation → execution → result → memory

**Inbox 子视图**：
- 调用 `invoke('get_memory_candidates', { limit: 50, status: 'active' })`
- 每条候选显示：memory_text（截断）、producer、memory_kind、source_event_ids 数量、created_at
- 操作按钮：Reject（调用 `reject_memory_candidate`）
- 底部：显示 rejected 候选数量（可展开查看 rejected 列表）
- 调用 `invoke('get_distillation_jobs', { limit: 20 })` 显示蒸馏作业状态

**Preview 子视图**：
- 输入区：window_title（可选填）、content_text（可选填）、token_budget（滑块，默认 500）
- 调用 `invoke('compete_for_context', { noteText, windowTitle, tokenBudget })`
- 结果区：选中的记忆列表（memory_text + vector_score + selection_reason_facts + token_estimate）
- Trace 信息：candidate_pool_size、skipped_reasons、vector_available
- 历史预览：调用 `invoke('get_competition_preview', { limit: 10 })`

**Signals 子视图**：
- 调用 `invoke('get_effect_signals')`
- 4 张统计卡片：
  - User Feedback：confirmed / cancelled / dismissed 计数
  - Execution：success / failure 计数
  - Memory Pool：active / rejected / embedded 计数
  - Competition：最近竞争的平均选中记忆数、embedding 覆盖率
- 所有数字为事实计数，不做语义标注

CSS class 遵循现有模式：`.practice-dashboard`、`.tab-bar`、`.tab-item`、`.timeline-list`、`.inbox-list`、`.preview-panel`、`.signal-card`。

验证：cargo build + 手动启动应用查看面板。

### Step 5: 集成到 App.js

在 App.js sidebar 中添加第 4 个导航项：Practice（图标用循环/仪表盘含义的 Unicode 字符）。

在 `initApp` 中 import `createPracticeDashboard`，调用并 append 到 panelContainer。

在 `showPanel` 中为 `'practice'` 面板添加刷新逻辑（切换到该面板时重新加载数据）。

验证：应用启动后 sidebar 显示 4 项，点击 Practice 显示 Dashboard 面板。

### Step 6: 测试与验证

- cargo build 通过
- cargo test -p cozmio --lib 通过
- cargo test -p cozmio_memory 通过（competition 相关）
- 新增 `get_distillation_jobs` 的 unit test
- 新增 `get_effect_signals` 的 unit test（或集成测试，mock AppState）
- UI Layer 1-3 验证：应用启动 → 点击 Practice → 4 个 tab 均可切换 → 数据加载无报错

### Step 7: 飞轮写回

- 更新 `verification/last_result.json`
- 更新 `feature_list.json`（PRACTICE-LOOP-V1-PHASE-E 条目）
- 更新 `claude-progress.txt`

## 8. Verification Asset

- verification type: `desktop_ui_runtime` + `deterministic_software`
- command / script:
  - `cargo build -p cozmio`
  - `cargo test -p cozmio --lib`
  - `cargo test -p cozmio_memory -- competition`
  - `cargo test -p cozmio -- dashboard`（新增测试）
- expected evidence:
  - Build 通过
  - 所有测试通过
  - UI 启动后 Practice 面板可访问，4 个 tab 均能加载和切换
  - Timeline 显示 ledger 事件
  - Inbox 显示 memory candidates，Reject 按钮可点击
  - Preview 可输入参数并返回竞争结果
  - Signals 显示事实计数卡片
- evidence location: `verification/last_result.json` + 截图（Layer 4 需人工）
- failure condition: build 失败 / 测试失败 / 面板加载报错 / IPC 调用返回错误
- writeback targets:
  - `verification/last_result.json`
  - `feature_list.json`
  - `claude-progress.txt`

## 9. Phase Gate

本 Phase 只有满足以下条件才能标记为完成：

- [ ] cargo build 通过
- [ ] cargo test 通过
- [ ] `get_timeline`/`get_trace_detail`/`rebuild_ledger_projection` 已注册到 invoke_handler
- [ ] `get_distillation_jobs` IPC 命令存在并可调用
- [ ] `get_effect_signals` IPC 命令存在并可调用
- [ ] PracticeDashboard.js 面板存在，4 个子视图可切换
- [ ] App.js sidebar 包含 Practice 导航项
- [ ] Timeline 子视图调用 `get_timeline` 并渲染
- [ ] Inbox 子视图调用 `get_memory_candidates` 并渲染，Reject 按钮可点击
- [ ] Preview 子视图调用 `compete_for_context` 并渲染结果
- [ ] Signals 子视图调用 `get_effect_signals` 并渲染统计卡片
- [ ] `verification/last_result.json` 已更新
- [ ] `feature_list.json` PRACTICE-LOOP-V1-PHASE-E 条目已更新
- [ ] `claude-progress.txt` 已有下一轮交接内容

## 10. Next Execution Step

- next phase: Phase E H1-H4 实施
- goal: 完成 Practice Dashboard 的 4 个子视图 + 后端 IPC 补全
- entry skill: `superpowers:subagent-driven-development`
- stop condition: Phase Gate 全部通过
