# Practice Loop V1 Phase G — Competition→Judgment Pipeline 接入

> **智能执行体须知**：本方案以运行效果、验证资产和飞轮写回为中心。默认不含代码示例。

## 1. Flywheel Context

- active task: PRACTICE-LOOP-V1-PHASE-G
- current phase: Phase G (Loop Closing)
- latest verification: Phase F complete, cargo build pass, 126+4 tests pass
- blocker (if any): ReminderContext 已实现但 judgment pipeline 未接入
- next expected step: 将 ReminderContext 接入 main_loop 的 judgment 流程

## 2. Goal

把 `ReminderContextDto`（来自 `build_activity_context`）接入 `main_loop.rs` 的 judgment 调用链，使模型在判断时能看见竞争胜出的记忆内容，而非仅靠 action_log。

一句话：让 competition 的输出真正进入模型的输入。

## 3. Product Type

- type: deterministic_software + model_output_validated
- core risk:
  - 代码正确性：Competition 调用不能阻塞或拖慢 judgment 流程
  - 模型输出质量：接入后模型判断质量是否真的提升（需要样本验证）
- verification style: cargo build + 单元测试 + 样本集评价

## 4. Global Roadmap

| Phase | 目标 | 依赖 | 验收意图 |
|-------|------|------|---------|
| H1 | Competition 结果接入 judgment prompt | Phase D 通过 | 模型输入中可见 recent_context 内容 |
| H2 | 自动触发 sample capture（Step 4 收尾） | H1 通过 | relay_execution_result_received 时自动捕获 |
| H3 | Evaluation Loop 自动闭合 | H1+H2 通过 | groundedness 报告可触发 prompt 调整 |

**本次执行：H1 完整 + H2 完整**

## 5. Scope

### In（本次包含）

- `main_loop.rs` judgment 流程调用 `build_activity_context`
- `build_popup_context` 增加 ReminderContext 参数
- `model_client.rs` prompt 增加 competition context block
- 竞争调用非阻塞：judgment 超时时间内完成
- IPC 命令注册（若需要新接口）
- 样本集验证：capture 真实样本，对比接入前后模型输出差异

### Out（本次不包含）

- Evaluation Loop 自动闭合（H3）
- prompt 动态调整逻辑（H3）
- 多模型 A/B testing（H3）

## 6. Current Truth

### 文件检查

- inspected files:
  - `src-tauri/src/main_loop.rs` — judgment 调用链（line 192-207）
  - `src-tauri/src/prompt_context.rs` — `build_popup_context` 当前只用 ActionLogger
  - `src-tauri/src/model_client.rs` — `build_prompt_with_context` 当前只用 `process_context`
  - `src-tauri/src/memory_commands.rs` — `build_activity_context` 返回 `ReminderContextDto`
  - `src-tauri/src/competition_commands.rs` — `compete_for_context` IPC 注册

### 现有入口点

- `build_activity_context(window_title, content_text, current_thread_id, token_budget)` → `ReminderContextDto`
- `ReminderContextDto` 字段：`current_activity`, `recent_context`, `related_decisions`, `relevant_skills`, `task_state`, `evidence_refs`
- IPC 命令：`compete_for_context` 已注册在 `generate_handler![]`

### 现有 runtime path

```
main_loop judgment 流程:
  build_popup_context(logger, title, process, process_context)
    → 只用 ActionLogger (action_log.jsonl)
    → popup_context block → call_raw_with_context(snapshot, process_context, Some(&popup_context))
      → build_prompt_with_context
        → 只用 process_context + popup_context
          → prompt block: window_info + process_context + popup_context (无记忆内容)
```

```
competition path (NOT in judgment flow):
  compete_for_context IPC
    → MemoryCore.build_reminder_context()
      → ReminderContextDto (recent_context 等字段有数据)
```

### 现有验证（若无）

- Phase D 测试：`cargo test -p cozmio_memory -- competition` — 18/18 pass
- Phase F 测试：`cargo test -p cozmio -- evaluation` — 7/7 pass
- `build_activity_context` 已有单元测试覆盖

## 7. Implementation Shape

### H1: Competition → Judgment Pipeline 接入（本次核心）

**Step 1: 修改 `build_popup_context` 函数签名**

在 `prompt_context.rs` 中：

- `build_popup_context` 增加参数：`reminder_context: Option<&ReminderContextDto>`
- 当 `reminder_context` 为 `Some` 时，在 popup_context 中追加一段 competition block
- competition block 格式（人类可读，不过度结构化）：

```
=== selected memories ===
recent_context: (reminder_context.recent_context 内容，截取到 MAX 字符)
related_decisions: (reminder_context.related_decisions 内容)
relevant_skills: (reminder_context.relevant_skills 内容)
current_activity: (reminder_context.current_activity 内容)
```

**Step 2: 在 `main_loop.rs` judgment 流程中调用 `build_activity_context`**

在 `main_loop.rs` line 192 之前（约 line 185-190 区间）：

- 在 judgment 触发时，调用 `build_activity_context`（异步，非阻塞）
- 传入当前 `snapshot.window_info.title` 和 `process_context` 相关内容
- 获取 `ReminderContextDto`，传给 `build_popup_context`
- 设置合理的 token_budget（如 600 tokens），避免 context 过长

**关键约束：judgment 必须在 poll_interval 内完成（默认 3-10 秒）**
competition 调用需要 < 500ms，否则需要加超时和 fallback

**Step 3: 修改 `model_client.rs` prompt 增加 competition context**

在 `build_prompt_with_context` 中，当传入 `popup_context` 包含 competition block 时，模型能看见这些内容。

实际工作已在 Step 1 的 `build_popup_context` 中处理，model_client 不需要改。

**Step 4: 处理竞争调用失败（graceful degradation）**

- 若 `build_activity_context` 调用失败（如数据库锁定），judgment 流程继续，只用 action_log popup_context
- 记录 warning log，不阻塞主流程

### H2: Sample 自动触发（Phase F Step 4 收尾）

**Step 5: 在 `main_loop.rs` 的 `relay_execution_result_received` 事件处理处**

- 当收到 `relay_execution_result_received` ledger event 时
- 自动调用 `save_evaluation_sample`，传入当前 trace_id
- 添加 feature flag 控制，默认关闭（`save_sample_auto_enabled` config 字段）

## 8. Verification Asset

- verification type: deterministic_software + 链路验证
- command / script:
  - `cargo build -p cozmio` — 编译通过
  - `cargo test -p cozmio` — 全部测试通过
  - `cargo test -p cozmio_memory -- competition` — 竞争逻辑（含向量检索链路验证）通过
- 链路验证必须确认：
  - `compete_for_context` 返回的 `vector_score` 是真实 cosine similarity（0.0-1.0），不是 1.0 placeholder
  - embed_provider.embed() 被调用（不是 stub 返回）
- expected evidence:
  - judgment prompt 中包含 `=== selected memories ===` block
  - `recent_context` 内容非空时有实际记忆数据
  - vector_score 是 0.0-1.0 之间，不是 uniform 1.0
  - competition 调用失败时 judgment 仍正常完成（graceful degradation）
- evidence location:
  - `cozmio.log` 中的 competition 调用日志
  - Evaluation tab 中 capture 的样本的 `context_pack_summary` 字段包含 recent_context
- failure condition:
  - vector_score is always 1.0 (vector retrieval stub not fixed)
  - embed_provider.embed() was not called
  - judgment 超时（> poll_interval）→ 回滚 competition 调用
  - compilation error → 阻塞
  - test failure → 修复后继续
- writeback targets:
  - `verification/last_result.json`
  - `feature_list.json`
  - `claude-progress.txt`

## 9. Phase Gate

本 Phase 只有满足以下条件才能标记为完成：

- [ ] `cargo build -p cozmio` 通过
- [ ] `cargo test -p cozmio` 全部通过
- [ ] `cargo test -p cozmio_memory -- competition` 通过（含链路验证测试）
- [ ] **vector_score 是真实 cosine similarity（0.0-1.0），不是 1.0 placeholder**（新增）
- [ ] **embed_provider.embed() 被调用**（新增）
- [ ] Evaluation tab 中 capture 的样本，`context_pack_summary` 可见 recent_context 内容
- [ ] competition 调用失败时 judgment 流程继续（graceful degradation 验证）
- [ ] `verification/last_result.json` 已更新
- [ ] `feature_list.json` `PRACTICE-LOOP-V1-PHASE-G` 条目已添加
- [ ] `claude-progress.txt` 已有下一轮交接内容

## 10. Next Execution Step

- next phase: H3 — Evaluation Loop 自动闭合（基于 groundedness 报告触发 prompt 调整）
- goal: Evaluation 结果（groundedness_notes）可反馈到 prompt 内容或 competition 策略调整
- entry skill: `superpowers:subagent-driven-development`（推荐，用于 Phase 粒度执行）
- stop condition: H1 + H2 验证通过，H3 可选（若时间允许）
