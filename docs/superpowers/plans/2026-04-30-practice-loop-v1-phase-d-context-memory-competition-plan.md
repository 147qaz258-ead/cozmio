# Practice Loop V1 — Phase D: Context Memory Competition 实施方案

> **智能执行体须知**：本方案以运行效果、验证资产和飞轮写回为中心。默认不含代码示例。

## 1. Flywheel Context

- active task: `PRACTICE-LOOP-V1`
- current phase: Phase D — Context Memory Competition
- latest verification: Phase C completed (b8bc772)
- blocker (if any): None
- next expected step: Phase E — Practice Dashboard

## 2. Goal

从 Phase C 已沉淀的 `active` 状态 `MemoryCandidate` 中，通过向量相似性 + 事实信号 + token budget 做竞争，选择少量材料进入 context pack 预览（`CompetitionResult`）。

Phase D 不做人工审批闸门，不做固定的 Top-N 机械排名，而是让向量相似性和事实信号在 token budget 约束下竞争，产出少量 Provenance 可追踪的记忆材料给 context pack 使用。

## 3. Product Type

- type: `deterministic_software` + `model_output_validated`
- core risk:
  - 向量模型未就绪导致竞争退化
  - token budget 约束与向量分数权重关系不明确
  - 竞争结果无 provenance，context pack 无法溯源
  - Phase D 变成静态排名而非动态竞争
- verification style: cargo build + unit tests + 向量检索链路验证
- 链路验证核心原则：测试必须验证"功能真正执行"，不只是"代码能编译"
- 向量检索链路验证必须包含：
  1. embed_provider.embed() 被调用（通过 mock call count 验证）
  2. vector_score 是真实 cosine similarity（0.0-1.0 范围），不是 1.0 placeholder
  3. vector_score = None 当 vector_provider = None（graceful fallback，不是 error）

## 4. Global Roadmap

| Phase | 目标 | 依赖 | 验收意图 |
|-------|------|------|---------|
| H1: Phase A | Ledger Foundation (JSONL + SQLite + Content Store) | — | PASS |
| H2: Phase B | Execution Return Stabilization | Phase A | PASS |
| H3: Phase C | Memory Distillation | Phase A+B | PASS |
| H4: Phase D | Context Memory Competition | Phase C | 向量+事实信号+token竞争，选择少量材料进入 context pack 预览 |
| H5: Phase E | Practice Dashboard | Phase A+B+C+D | Loop 时间线、记忆收件箱、效果信号可见于 UI |
| H6: Phase F | Evaluation Loop | Phase C | 样本集评估；模型/agent 输出评估材料带 provenance |

## 5. Scope

### In（本次包含）

- 新增 `CompetitionResult` struct：包含选中的 memory_id、memory_text、source_event_ids、provenance、vector_score、signal_score、final_score
- 新增 `MemoryCompetition::compete_candidates()` 方法：对 `active` 状态 candidates 做向量检索 + fact 信号评分 + token budget 约束
- 新增 `compete_for_context(ActivityNote, token_budget) -> CompetitionResult` IPC 命令：外部调用入口
- 向量检索：当 `embedding_ref` 存在时，使用向量相似性评分；不存在时跳过向量评分（不降级）
- `signal_facts` 事实信号评分：事实信号（execution_status、source_event_count、has_error_text、user_confirmed_count）参与竞争权重
- token budget 约束：在 final_score 排序后，按 token budget 截取
- Phase D 复用 `MemoryCompetition` 架构，扩展 `ReminderContext` 增加 `competition_result` 字段
- 竞争结果写入 ledger（可选的 observable record，不做强制）
- `get_competition_preview()` IPC 命令：返回当前 competition 结果供调试/UI 使用

### Out（本次不包含）

- 修改 `MemoryCandidate` schema（已在 Phase C 定义完整）
- 强制 approval gate 或人工审批作为竞争入口
- 修改 Phase C 的 `distillation backend` 或生成流程
- 将所有 candidates 都放入 context pack
- Phase D 负责长期记忆删除/过期逻辑
- Dashboard UI（Phase E）
- Phase F 样本集评估

## 6. Current Truth

- files inspected:
  - `cozmio_memory/src/memory_candidate.rs` — MemoryCandidate struct, MemoryCandidateStore::list()
  - `cozmio_memory/src/competition.rs` — MemoryCompetition, ReminderContext, build_reminder_context()
  - `cozmio_memory/src/search.rs` — SearchEngine, expand_query
  - `cozmio_memory/src/lib.rs` — re-exports, module structure
  - `cozmio_memory/src/embed_provider.rs` — EmbeddingProvider trait, ProviderType
  - `src-tauri/src/memory_commands.rs` — existing IPC boundary types
  - `src-tauri/src/distill_commands.rs` — Phase C IPC patterns
- 现有 `MemoryCandidate` 字段：`memory_id`, `created_at`, `producer`, `source_event_ids`, `source_paths`, `source_ranges`, `memory_text`, `memory_kind`, `signal_facts`, `supersedes`, `expires_at`, `status`, `embedding_ref`
- Phase C 已实现 `get_memory_candidates(limit, status)` IPC 命令，可获取 `active` candidates
- `embedding_ref` 字段：在 Phase C 可选存储，当 distillation backend 配置了 embedding 时填充
- 向量模型已集成：`EmbeddingProvider` trait，`create_provider()` 返回 FastEmbed/InMemoryVecStore 或 mock
- `signal_facts` 结构已知：包含 `execution_status`、`source_event_count`、`has_error_text`、`user_confirmed_count`
- `MemoryCompetition::build_reminder_context()` 已存在，产出 `ReminderContext`
- Phase D 新竞争维度：向量检索（基于 `embedding_ref`）+ fact signal score

## 7. Implementation Shape（Phase D）

### Step 1: Define CompetitionResult Schema

在 `cozmio_memory/src/competition.rs` 新增 `CompetitionResultEntry` struct：

- `memory_id: String`
- `memory_text: String`
- `memory_kind: String`
- `vector_score: Option<f32>`（向量相似性，当 embedding_ref 存在时）
- `signal_score: f32`（fact signal 评分）
- `final_score: f32`（综合排名分）
- `source_event_ids: Vec<String>`（provenance）
- `provenance: String`（producer + memory_kind）

同时扩展 `ReminderContext` 新增可选字段 `competition_entries: Vec<CompetitionResultEntry>`。

### Step 2: Implement Fact Signal Score Function

在 `cozmio_memory/src/competition.rs` 新增 `compute_signal_score(signal_facts: &serde_json::Value) -> f32`：

- 读取 `signal_facts` 中的事实字段
- 评分规则（可配置，不硬编码）：
  - `execution_status == "completed"` →加分
  - `has_error_text == true` →减分或标记
  - `user_confirmed_count > 0` →加分（用户确认过的执行更有价值）
  - `source_event_count` →基础分
- 返回 0.0-1.0 之间的 normalized score

### Step 3: Implement Vector Similarity Score

在 `cozmio_memory/src/competition.rs` 新增 `compute_vector_score(memory: &MemoryCandidate, query_embedding: &[f32]) -> Option<f32>`：

- 当 `memory.embedding_ref` 为 `None` 时，返回 `None`（不降级，不给默认分）
- 当 `memory.embedding_ref` 存在时，从 embedding store 加载向量，计算 cosine similarity
- 返回 `Some(f32)` 范围 0.0-1.0

### Step 4: Implement compete_candidates Method

在 `MemoryCompetition` 新增 `compete_candidates` 方法：

签名：`compete_candidates(&self, note: &ActivityNote, candidates: &[MemoryCandidate], token_budget: usize) -> CompetitionResult`

竞争流程：
1. 获取 query embedding（从 `ActivityNote.content_text` + `window_title`）
2. 对每个 candidate：
   - 计算 `vector_score`（可选，存在时计算）
   - 计算 `signal_score`（必须，基于 signal_facts）
   - 综合：`final_score = alpha * vector_score + (1-alpha) * signal_score`（alpha 可配置，默认 0.6）
3. 按 `final_score` 降序排列
4. 从 top 开始，选择能放入 `token_budget` 的 candidates
5. 每个选中的 candidate 构建 `CompetitionResultEntry`

### Step 5: Implement compete_for_context IPC Command

在 `src-tauri/src/` 新增 `competition_commands.rs`：

`#[tauri::command] compete_for_context(app, note_text: String, window_title: String, token_budget: usize) -> Result<CompetitionResult, String>`

流程：
1. 通过 `get_memory_candidates` 获取所有 `active` candidates
2. 调用 `compete_candidates()` 获取竞争结果
3. 可选：记录竞争结果到 ledger（observable record，不做强制）
4. 返回 `CompetitionResult`

### Step 6: Implement get_competition_preview IPC Command

`#[tauri::command] get_competition_preview(limit: usize) -> Result<Vec<CompetitionResultEntry>, String>`

- 返回最近的竞争结果（调试用）
- 从内存或 ledger 中读取

### Step 7: Add Unit Tests

- `test_signal_score_computed_from_facts`
- `test_vector_score_none_when_no_embedding_ref`
- `test_vector_score_computed_when_embedding_ref_exists`
- `test_compete_candidates_respects_token_budget`
- `test_compete_candidates_ranks_by_final_score`
- `test_compete_candidates_skips_candidates_without_embedding_ref_for_vector_score`
- `test_competition_result_entry_has_full_provenance`
- `test_compete_for_context_returns_valid_competition_result`

## 8. Verification Asset

- verification type: `deterministic_software` + `链路验证`
- command:
  - `cargo build -p cozmio`
  - `cargo build -p cozmio_memory`
  - `cargo test -p cozmio_memory -- competition`
- 链路验证测试（必须全部通过，不只是"编译通过"）：
  - `test_embed_provider_is_called_when_provider_specified` — 验证 embed provider 被调用
  - `test_vector_retrieval_uses_real_cosine_similarity` — 验证 vector_score 不是 1.0 placeholder
  - `test_vector_score_none_when_no_embedding_ref` — 验证无 embedding 时 graceful
- expected evidence:
  - build passes without errors
  - all unit tests pass (包括链路验证测试)
  - embed_provider.embed() was called (verified via mock call count)
  - vector_score is real cosine similarity (0.0-1.0 range), not always 1.0
  - `compete_for_context` IPC command registered
  - `get_competition_preview` IPC command registered
  - active candidates can be retrieved and scored
  - vector_score computed when embedding_ref exists
  - signal_score computed from signal_facts
  - final_score = alpha * vector + (1-alpha) * signal
  - token budget respected (top candidates selected within budget)
  - competition result entries contain full provenance
  - no embedding_ref → vector_score = None (graceful, not error)
- failure condition:
  - build fails
  - IPC commands not registered
  - vector_score is always 1.0 (placeholder behavior, not real cosine similarity)
  - embed_provider.embed() was not called (vector retrieval not executed)
  - vector_score error when embedding unavailable
  - token budget not respected
  - provenance missing in competition result
- writeback targets:
  - `verification/last_result.json`
  - `feature_list.json`
  - `claude-progress.txt`

## 9. Phase Gate

Phase D 只有满足以下条件才能标记为完成：

- [ ] `cargo build -p cozmio && cargo build -p cozmio_memory` 通过，无编译错误
- [ ] `cargo test -p cozmio_memory -- memory_candidate` 所有测试通过
- [ ] `cargo test -p cozmio -- competition` 所有 competition 相关测试通过
- [ ] `compete_for_context` IPC 命令存在且可调用
- [ ] `get_competition_preview` IPC 命令存在且可调用
- [ ] `CompetitionResultEntry` 包含完整 provenance（memory_id、source_event_ids、producer、memory_kind）
- [ ] `vector_score` 为 `None` 当 embedding_ref 不存在时（不报错、不降级）
- [ ] `signal_score` 基于 `signal_facts` 计算（不硬编码、不用语义）
- [ ] `final_score` 综合 vector_score + signal_score（alpha 可配置）
- [ ] token budget 约束被尊重（按 final_score 排序后截取）
- [ ] Phase D 不实现人工审批闸门
- [ ] Phase D 不修改 MemoryCandidate schema
- [ ] Phase D 不把所有 candidates 都放入 context pack
- [ ] `verification/last_result.json` 已更新
- [ ] `feature_list.json` 相关条目已更新 `PRACTICE-LOOP-V1-PHASE-D` 状态
- [ ] `claude-progress.txt` 已有下一轮交接内容

## 10. Next Execution Step

- next phase: Phase E — Practice Dashboard
- goal: Loop 时间线、记忆收件箱、效果信号可见于 UI
- entry skill: `superpowers:subagent-driven-development`（推荐，用于 Phase 粒度执行）
- stop condition: Phase E 验证资产通过；Dashboard 显示 memory inbox、loop timeline、effect signals