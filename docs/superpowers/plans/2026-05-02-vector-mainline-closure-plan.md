# Vector Mainline Closure 实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:executing-plans` 按任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。本方案取代 `2026-05-02-vector-competition-callchain-fix-plan.md` 中“feature flag 非阻塞”和“build_activity_context_sync 不做 backfill 非阻塞”的判断，并取代 `2026-05-01-practice-loop-v1-phase-g-competition-integration-plan.md` 中把自动 sample capture 混入 Phase G 的范围。

**目标**：让向量模型真实进入 Cozmio 弹窗判断主线：候选记忆生成后有 embedding，运行时用同一个 embedding provider 生成 query embedding，竞争胜出的 `competition_entries` 作为单次调用的运行时上下文包进入模型输入，并被 ledger/evaluation 证据捕获。

**架构思路**：本轮只闭合向量竞争到 judgment runtime context 的主线，不做自动 sample capture，不做 prompt 自动调参。固定提示词只说明“如何对待材料”，不承载记忆内容；记忆内容只能出现在本次调用的 runtime context packet 中。系统代码只处理事实、存储、provider 调用、数值分数、content refs 和 trace；任何语义总结仍来自模型/执行端产物本身。运行证据必须能回答四个问题：provider 是否可用、candidate 是否有 embedding、query embedding 是否生成、selected memory 是否进入 runtime context packet。

**技术栈**：Rust / Tauri / cozmio_memory / FastEmbed / SQLite + JSONL ledger / vanilla JavaScript Practice Dashboard

---

## 产品类型

本轮是 `deterministic_software + model_output_validated`：

- 软件实现部分：Cargo feature、embedding provider 调用链、candidate embedding 写入、prompt context 构造、ledger/evaluation 捕获。
- 模型输出验证部分：接入后弹窗模型是否能看见真实 selected memories，需要通过真实 context pack 样本验证，不能只用 build/test 通过代表完成。

## 范围

### 本轮包含

- 修复默认运行时未启用 FastEmbed 的问题。
- 移除 competition 主线中的硬编码 provider 字符串二次创建。
- 让 distillation 生成的 active memory candidate 自动尝试写入 embedding。
- 让 `build_activity_context_sync` 在进入 competition 前对少量缺失 embedding 的 active candidates 做事实性 backfill。
- 让 runtime context packet 携带 `competition_entries` 的真实 selected memory 内容、数值分数和 provenance facts。
- 让 ledger 保存模型实际收到的 popup/context pack 文本，使 Evaluation 可以捕获真实证据。

### 本轮不包含

- 不实现自动 sample capture。
- 不实现 Evaluation 自动闭环或 prompt 自动调参。
- 不加入机械 popup 限制、冷却时间、静默策略。
- 不加入系统生成的用户意图、阶段、重要性、任务判断。

## 当前问题

1. `cozmio/src-tauri/Cargo.toml` 依赖 `cozmio_memory` 时没有开启 `fastembed` feature，普通 app 运行时 provider 会不可用。
2. `build_activity_context_sync` 会跑 competition，但没有保证 active candidates 已有 embedding。
3. `run_competition_and_build_context` 硬编码 `Some("fastembed".to_string())`，没有使用 `MemoryCore` 持有的 provider 实例。
4. `build_popup_context` 注入的是老的 ReminderContext 字段，不是竞争胜出的 `competition_entries`。
5. `save_evaluation_sample_impl` 的 `context_pack_summary` 不是模型实际收到的 context pack。

---

## 文件结构

**修改：**

- `cozmio/src-tauri/Cargo.toml`：启用 `cozmio_memory/fastembed`。
- `cozmio/cozmio_memory/src/embed_provider.rs`：给 provider 增加事实性名称。
- `cozmio/cozmio_memory/src/embed_disabled.rs`：实现 provider 名称。
- `cozmio/cozmio_memory/src/embed_mock.rs`：实现 provider 名称。
- `cozmio/cozmio_memory/src/embed_fastreembed.rs`：实现 provider 名称。
- `cozmio/cozmio_memory/src/competition.rs`：让 `MemoryCompetition` 持有 provider，新增 provider 实例路径的 competition 函数，保留旧函数给测试兼容。
- `cozmio/src-tauri/src/memory_commands.rs`：主线创建 provider 后先做小批量 backfill，再 build context。
- `cozmio/src-tauri/src/competition_commands.rs`：IPC 路径使用同一个 provider 实例，不再依赖硬编码字符串路径。
- `cozmio/src-tauri/src/distill_commands.rs`：distillation 写入 candidate 后立即尝试 embedding。
- `cozmio/src-tauri/src/prompt_context.rs`：临时保留现有文件名，但职责改成构造 runtime context packet；不得把记忆写入固定提示词语义层。
- `cozmio/src-tauri/src/ledger.rs`：增加 `context_pack_built` 事件类型。
- `cozmio/src-tauri/src/main_loop.rs`：模型调用前记录实际 popup context。
- `cozmio/src-tauri/src/eval_commands.rs`：Evaluation sample 捕获真实 context pack event。
- `cozmio/src-tauri/tests/semantic_boundary.rs`：允许事实性 context pack event，继续禁止伪语义。

**不修改：**

- 不修改 popup 频率策略。
- 不修改模型输出格式约束。
- 不修改用户确认/取消流程。

---

## 任务 1：启用运行时 FastEmbed feature

**涉及文件**：

- 修改：`cozmio/src-tauri/Cargo.toml:19`

- [ ] **步骤 1：修改依赖 feature**

把：

```toml
cozmio_memory = { path = "../cozmio_memory", default-features = false }
```

改成：

```toml
cozmio_memory = { path = "../cozmio_memory", default-features = false, features = ["fastembed"] }
```

- [ ] **步骤 2：验证 feature 被接入**

执行命令：

```bash
cd cozmio
cargo tree -p cozmio -i fastembed
```

预期结果：

```text
fastembed v5...
└── cozmio_memory v0.1.0
    └── cozmio v0.1.0
```

- [ ] **步骤 3：构建主 app**

执行命令：

```bash
cd cozmio
cargo build -p cozmio
```

预期结果：编译通过。若 FastEmbed 下载/初始化依赖导致构建失败，本任务停止，记录完整错误，不进入任务 2。

---

## 任务 2：让 competition 使用同一个 provider 实例

**涉及文件**：

- 修改：`cozmio/cozmio_memory/src/embed_provider.rs`
- 修改：`cozmio/cozmio_memory/src/embed_disabled.rs`
- 修改：`cozmio/cozmio_memory/src/embed_mock.rs`
- 修改：`cozmio/cozmio_memory/src/embed_fastreembed.rs`
- 修改：`cozmio/cozmio_memory/src/competition.rs`

- [ ] **步骤 1：给 EmbeddingProvider 增加事实性名称**

在 `cozmio/cozmio_memory/src/embed_provider.rs` 中，把 trait 改成：

```rust
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, MemoryError>;
    fn dimension(&self) -> usize;
    fn is_available(&self) -> bool;
    fn provider_name(&self) -> &'static str;
}
```

- [ ] **步骤 2：实现 provider_name**

在 `embed_disabled.rs` 中添加：

```rust
fn provider_name(&self) -> &'static str {
    "disabled"
}
```

在 `embed_mock.rs` 中添加：

```rust
fn provider_name(&self) -> &'static str {
    "mock"
}
```

在 `embed_fastreembed.rs` 中添加：

```rust
fn provider_name(&self) -> &'static str {
    "fastembed"
}
```

- [ ] **步骤 3：修改 MemoryCompetition 持有 provider**

在 `cozmio/cozmio_memory/src/competition.rs` 中，把结构体改成：

```rust
pub struct MemoryCompetition<'a> {
    db: &'a Database,
    search_engine: SearchEngine<'a>,
    embed_provider: Option<Arc<dyn EmbeddingProvider>>,
}
```

把构造函数改成：

```rust
pub fn new(
    db: &'a Database,
    search_engine: SearchEngine<'a>,
    embed_provider: Option<Arc<dyn EmbeddingProvider>>,
) -> Self {
    Self {
        db,
        search_engine,
        embed_provider,
    }
}
```

把 `MemoryCore::competition()` 改成：

```rust
pub fn competition(&self) -> MemoryCompetition<'_> {
    MemoryCompetition::new(self.db, self.search_engine(), self.embed_provider.clone())
}
```

- [ ] **步骤 4：新增 provider 实例路径的竞争函数**

在 `compete_candidates` 旁边新增函数：

```rust
pub fn compete_candidates_with_query_embedding(
    db: &Database,
    note: &ActivityNote,
    candidates: &[MemoryCandidate],
    token_budget: usize,
    alpha: f32,
    query_embedding: Option<&[f32]>,
    vector_provider: Option<String>,
) -> CompetitionResult {
    let query_text = build_query_from_activity(note);
    let mut skipped_reasons: Vec<String> = Vec::new();

    let eligible_candidates: Vec<_> = candidates
        .iter()
        .filter_map(|c| {
            if c.status != "active" && c.status != "completed" {
                skipped_reasons.push("not_active_status".to_string());
                return None;
            }
            Some(c)
        })
        .collect();

    let scored_candidates: Vec<_> = eligible_candidates
        .iter()
        .map(|c| {
            let vector_score = query_embedding.and_then(|qe| compute_vector_score(c, qe, db));
            let signal_score = compute_signal_score(&c.signal_facts);
            let final_score = compute_final_score(vector_score, signal_score, alpha);
            let token_estimate = estimate_tokens(&c.memory_text);
            let has_valid_embedding = c.embedding_ref.is_some() && vector_score.is_some();
            (c, vector_score, signal_score, final_score, token_estimate, !has_valid_embedding)
        })
        .collect();

    let mut sorted = scored_candidates;
    sorted.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    let mut selected: Vec<CompetitionResultEntry> = Vec::new();
    let mut current_token_count: usize = 0;

    for (candidate, vector_score, _signal_score, final_score, token_estimate, missing_emb) in sorted {
        if missing_emb {
            skipped_reasons.push("missing_embedding".to_string());
        }

        if current_token_count + token_estimate <= token_budget {
            selected.push(CompetitionResultEntry {
                memory_id: candidate.memory_id.clone(),
                memory_text: candidate.memory_text.clone(),
                memory_kind: candidate.memory_kind.clone(),
                vector_score,
                fact_trace: candidate.signal_facts.clone(),
                selection_reason_facts: vec![format!("final_score_{:.3}", final_score)],
                token_estimate,
                source_event_ids: candidate.source_event_ids.clone(),
                source_paths: candidate.source_paths.clone(),
                source_ranges: candidate.source_ranges.clone(),
                producer: candidate.producer.clone(),
            });
            current_token_count += token_estimate;
        } else {
            skipped_reasons.push("token_budget_exceeded".to_string());
        }
    }

    CompetitionResult {
        entries: selected,
        trace: CompetitionTrace {
            query_facts: serde_json::json!({
                "window_title": note.window_title,
                "content_text": note.content_text,
                "query_text": query_text,
                "timestamp": note.timestamp.to_rfc3339(),
            }),
            candidate_pool_size: candidates.len(),
            skipped_reasons,
            vector_available: query_embedding.is_some()
                && candidates.iter().any(|c| c.embedding_ref.is_some()),
            vector_provider,
        },
    }
}
```

- [ ] **步骤 5：让旧函数委托新函数**

把旧 `compete_candidates` 中从 query embedding 生成之后的重复逻辑删除，改成：

```rust
pub fn compete_candidates(
    db: &Database,
    note: &ActivityNote,
    candidates: &[MemoryCandidate],
    token_budget: usize,
    alpha: f32,
    vector_provider: Option<String>,
) -> CompetitionResult {
    let query_text = build_query_from_activity(note);
    let query_embedding: Option<Vec<f32>> = vector_provider.as_ref().and_then(|provider_name| {
        let provider_type = match provider_name.as_str() {
            "fastembed" => ProviderType::FastEmbed,
            "mock" => ProviderType::Mock,
            _ => ProviderType::Disabled,
        };
        create_provider(provider_type)
            .ok()
            .and_then(|p| if p.is_available() { Some(p) } else { None })
            .and_then(|p| p.embed(&query_text).ok())
    });

    compete_candidates_with_query_embedding(
        db,
        note,
        candidates,
        token_budget,
        alpha,
        query_embedding.as_deref(),
        vector_provider,
    )
}
```

- [ ] **步骤 6：run_competition_and_build_context 使用 self.embed_provider**

把 `run_competition_and_build_context` 中的 `compete_candidates(... Some("fastembed".to_string()))` 改成：

```rust
let query_text = build_query_from_activity(note);
let (query_embedding, provider_name) = self
    .embed_provider
    .as_ref()
    .and_then(|provider| {
        if !provider.is_available() {
            return None;
        }
        provider
            .embed(&query_text)
            .ok()
            .map(|embedding| (embedding, provider.provider_name().to_string()))
    })
    .map(|(embedding, name)| (Some(embedding), Some(name)))
    .unwrap_or((None, None));

let result = compete_candidates_with_query_embedding(
    self.db,
    note,
    &candidates,
    token_budget,
    0.3,
    query_embedding.as_deref(),
    provider_name,
);
```

- [ ] **步骤 7：运行 competition 测试**

执行命令：

```bash
cd cozmio
cargo test -p cozmio_memory -- competition
```

预期结果：全部通过。

---

## 任务 3：让候选记忆进入 embedding 主线

**涉及文件**：

- 修改：`cozmio/src-tauri/src/distill_commands.rs:546-576`
- 修改：`cozmio/src-tauri/src/memory_commands.rs:381-399`
- 修改：`cozmio/src-tauri/src/competition_commands.rs:146-160`

- [ ] **步骤 1：在 distillation 写入 candidate 后立即尝试 embedding**

在 `distill_commands.rs` 顶部 import：

```rust
use cozmio_memory::competition::backfill_candidate_embeddings;
use cozmio_memory::embed_provider::{create_provider, ProviderType};
```

在 `run_distillation_job` 中 `store.insert(&candidate)` 后，删除 placeholder 注释：

```rust
// Embed placeholder: if artifact.embed is true and we had an embedding backend,
// we would call it here. For now, embedding_ref remains None.
```

替换为：

```rust
if let Ok(provider) = create_provider(ProviderType::FastEmbed) {
    if provider.is_available() {
        let _ = backfill_candidate_embeddings(db, provider.as_ref(), 1)
            .map_err(|e| log::warn!("Candidate embedding backfill failed: {}", e));
    }
}
```

事实边界：这里不判断 candidate 是否“重要”，只对新写入的 active candidate 生成向量。

- [ ] **步骤 2：在 build_activity_context_sync 中做小批量 backfill**

把 `memory_commands.rs` 中 provider 创建后的代码改成：

```rust
let embed_provider = cozmio_memory::embed_provider::create_provider(
    cozmio_memory::embed_provider::ProviderType::FastEmbed,
)
.ok();

if let Some(ref provider) = embed_provider {
    if provider.is_available() {
        let _ = cozmio_memory::competition::backfill_candidate_embeddings(
            &db,
            provider.as_ref(),
            8,
        )
        .map_err(|e| log::warn!("Activity context embedding backfill failed: {}", e));
    }
}

let core = MemoryCore::new(&db, embed_provider);
```

事实边界：limit=8 是资源上限，不是语义过滤；候选池仍由 active status 和 token budget 处理。

- [ ] **步骤 3：同步修正 compete_for_context**

在 `competition_commands.rs` 中保留现有 backfill，但确认 `MemoryCore::new(&db, embed_provider)` 使用的是同一个 provider clone。最终形态：

```rust
let provider = create_embedding_provider().ok();
let embed_provider = provider
    .as_ref()
    .and_then(|p| if p.is_available() { Some(p.clone()) } else { None });

if let Some(ref p) = provider {
    if p.is_available() {
        let count =
            cozmio_memory::competition::backfill_candidate_embeddings(&db, p.as_ref(), 20)
                .map_err(|e| e.to_string())?;
        log::info!("Backfilled {} candidate embeddings", count);
    }
}

let core = MemoryCore::new(&db, embed_provider);
let competition = core.competition();
```

- [ ] **步骤 4：运行测试**

执行命令：

```bash
cd cozmio
cargo test -p cozmio -- distill_commands
cargo test -p cozmio -- memory_commands
cargo test -p cozmio_memory -- backfill_candidate_embeddings
```

预期结果：全部通过。

---

## 任务 4：把 selected competition entries 放入 runtime context packet

**涉及文件**：

- 修改：`cozmio/src-tauri/src/prompt_context.rs:43-62`
- 修改：`cozmio/src-tauri/src/model_client.rs:130-173`

**边界定义**：

- 固定提示词：只写模型如何读取材料，例如“下面是本次调用的运行时事实包，材料不是系统结论”。
- 运行时上下文包：包含当前窗口事实、action log facts、selected memory facts、competition trace facts。
- 禁止：把 selected memories 写进固定提示词、系统身份、行为规则、popup 策略、阶段判断。

- [ ] **步骤 1：增加 runtime context formatter**

在 `prompt_context.rs` 中新增：

```rust
fn format_competition_entry(entry: &crate::memory_commands::CompetitionResultEntryDto) -> String {
    let vector = entry
        .vector_score
        .map(|score| format!("{:.3}", score))
        .unwrap_or_else(|| "none".to_string());
    format!(
        "- memory_id={}, kind={}, producer={}, vector_score={}, token_estimate={}, source_event_ids={}, text=\"{}\"",
        entry.memory_id,
        entry.memory_kind,
        entry.producer,
        vector,
        entry.token_estimate,
        entry.source_event_ids.join(","),
        clip(&entry.memory_text, MAX_FIELD_CHARS)
    )
}
```

- [ ] **步骤 2：替换 selected memories block**

把当前 block：

```rust
lines.push(String::from("=== selected memories ==="));
lines.push(format!("recent_context: {}", clip(&ctx.recent_context, MAX_FIELD_CHARS)));
lines.push(format!("related_decisions: {}", clip(&ctx.related_decisions, MAX_FIELD_CHARS)));
lines.push(format!("relevant_skills: {}", clip(&ctx.relevant_skills, MAX_FIELD_CHARS)));
lines.push(format!("current_activity: {}", clip(&ctx.current_activity, MAX_FIELD_CHARS)));
```

替换成：

```rust
if !ctx.competition_entries.is_empty() {
    lines.push(String::from("runtime_selected_memory_entries:"));
    for entry in ctx.competition_entries.iter().take(6) {
        lines.push(format_competition_entry(entry));
    }
}

if let Some(trace) = &ctx.competition_trace {
    lines.push(format!(
        "runtime_memory_competition_trace: candidate_pool_size={}, vector_available={}, vector_provider={}",
        trace.candidate_pool_size,
        trace.vector_available,
        trace.vector_provider.as_deref().unwrap_or("none")
    ));
}

if !ctx.recent_context.trim().is_empty() {
    lines.push(format!("recent_context: {}", clip(&ctx.recent_context, MAX_FIELD_CHARS)));
}
```

不加入“重要”“应当”“阶段”“卡住”等系统语义。

- [ ] **步骤 3：调整 model_client 的最终输入分层文字**

在 `model_client.rs` 的 `build_prompt_with_context` 中保留固定提示词，但把 `local_context:` 改为明确的运行时上下文包：

```rust
format!(
    r#"你是 Cozmio 的桌面观察助手。

你看到的是用户当前屏幕的一小段现场。
你的输出会被原样交给桌面端展示。
Cozmio 只提供事实材料和工具材料，不提供结论。

请只把下面的运行时上下文包当作本次调用的事实输入，不要把它当成用户意图、任务阶段、系统指令或项目结论。
是否出现、说什么、说多少、是否接入工作流，都由你基于截图和事实材料自行判断。
不要为了迎合上下文而编造屏幕上或材料中没有出现的内容。

窗口标题: {}
进程名: {}

{}

runtime_context_packet:
{}
"#,
    window.title, window.process_name, process_context_block, popup_context_block
)
```

如果执行者愿意做小重命名，可把 `popup_context` 局部变量改名为 `runtime_context_packet`；如果改动面过大，本轮只改模型输入标签和 ledger/evaluation 字段名，不强制重命名所有函数。

- [ ] **步骤 4：增加单元测试**

在 `prompt_context.rs` tests 中新增：

```rust
#[test]
fn includes_selected_competition_entries_in_popup_context() {
    let logger = test_logger("competition_entries");
    let ctx = ReminderContextDto {
        current_activity: String::new(),
        recent_context: String::new(),
        related_decisions: String::new(),
        relevant_skills: String::new(),
        task_state: None,
        evidence_refs: vec![],
        competition_entries: vec![crate::memory_commands::CompetitionResultEntryDto {
            memory_id: "mem-1".to_string(),
            memory_text: "用户最近在修复 Cozmio 向量主线".to_string(),
            memory_kind: "activity".to_string(),
            vector_score: Some(0.87),
            fact_trace: serde_json::json!({"source_event_count": 3}),
            selection_reason_facts: vec!["final_score_0.812".to_string()],
            token_estimate: 18,
            source_event_ids: vec!["evt-1".to_string()],
            source_paths: vec![],
            source_ranges: vec![],
            producer: "distill-command".to_string(),
        }],
        competition_trace: Some(crate::memory_commands::CompetitionTraceDto {
            query_facts: serde_json::json!({"window_title": "Code"}),
            candidate_pool_size: 1,
            skipped_reasons: vec![],
            vector_available: true,
            vector_provider: Some("fastembed".to_string()),
        }),
    };

    let context = build_popup_context(
        &logger,
        "Code",
        "Code.exe",
        &ProcessContext {
            stay_duration_seconds: 3,
            switches_in_last_minute: 1,
            is_oscillating: false,
            last_switch_direction: SwitchDirection::Arrived,
            just_arrived: true,
        },
        Some(&ctx),
    );

    assert!(context.contains("runtime_selected_memory_entries:"));
    assert!(context.contains("memory_id=mem-1"));
    assert!(context.contains("vector_score=0.870"));
    assert!(context.contains("用户最近在修复 Cozmio 向量主线"));
    assert!(!context.contains("弹窗策略"));
    assert!(!context.contains("保持沉默"));
}
```

- [ ] **步骤 5：运行 runtime context 测试**

执行命令：

```bash
cd cozmio
cargo test -p cozmio -- prompt_context
```

预期结果：全部通过。

---

## 任务 5：记录模型实际收到的 context pack

**涉及文件**：

- 修改：`cozmio/src-tauri/src/ledger.rs:88-103`
- 修改：`cozmio/src-tauri/src/main_loop.rs:227-245`
- 修改：`cozmio/src-tauri/src/eval_commands.rs:80-93`

- [ ] **步骤 1：新增 ledger event type**

在 `ledger.rs` 的 `event_type` 中添加：

```rust
pub const CONTEXT_PACK_BUILT: &str = "context_pack_built";
```

- [ ] **步骤 2：在 main_loop 记录 context pack**

在 `main_loop.rs` 中 `let popup_context = build_popup_context(...)` 后、`call_raw_with_context` 前添加：

```rust
{
    let state = app_handle.state::<crate::commands::AppState>();
    let mut metadata = std::collections::HashMap::new();
    metadata.insert(
        "context_char_count".to_string(),
        popup_context.chars().count().to_string(),
    );
    metadata.insert(
            "has_runtime_selected_memory_entries".to_string(),
            popup_context.contains("runtime_selected_memory_entries:").to_string(),
    );

    let context_event = crate::ledger::LedgerEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        trace_id: Some(raw_trace_id_for_context.clone()),
        session_id: None,
        timestamp: chrono::Utc::now().timestamp(),
        event_type: crate::ledger::event_type::CONTEXT_PACK_BUILT.to_string(),
        source: "cozmio-desktop".to_string(),
        window_title: Some(snapshot.window_info.title.clone()),
        process_name: Some(snapshot.window_info.process_name.clone()),
        raw_text: Some(popup_context.clone()),
        content_ref: None,
        parent_event_id: None,
        metadata,
    };

    if let Err(e) = state.ledger_manager.record_event(context_event) {
        log::warn!("Failed to record context_pack_built event: {}", e);
    }
}
```

执行者必须在这段代码前创建 trace id。若当前 `raw_output.trace_id` 只能在 model call 后拿到，则在 model call 前创建 `let raw_trace_id_for_context = uuid::Uuid::new_v4().to_string();`，并在 `ModelClient` 调用路径中复用该 trace id；如果 `ModelClient` 当前不能接收外部 trace id，则只记录 context event，trace_id 使用 `None` 并在 metadata 写入 `window_title/process_name/timestamp`。本轮验收以能捕获真实 `raw_text` 为准。

- [ ] **步骤 3：Evaluation sample 读取 context_pack_built**

把 `eval_commands.rs` 中 `context_pack_summary` 的构造改成优先读取真实 context pack：

```rust
let context_pack_summary = events
    .iter()
    .find(|e| e.event_type == crate::ledger::event_type::CONTEXT_PACK_BUILT)
    .and_then(|e| e.raw_text.clone())
    .unwrap_or_else(|| {
        events
            .iter()
            .rev()
            .take(10)
            .map(|e| {
                format!(
                    "[{}] {}",
                    e.event_type,
                    e.window_title.as_deref().unwrap_or("")
                )
            })
            .collect::<Vec<_>>()
            .join(" | ")
    });
```

- [ ] **步骤 4：增加 evaluation 捕获测试**

在 `eval_commands.rs` tests 中新增一个 trace 包含 `CONTEXT_PACK_BUILT` 的样本，断言：

```rust
assert!(sample.context_pack_summary.contains("runtime_selected_memory_entries:"));
assert!(sample.context_pack_summary.contains("memory_id=mem-1"));
```

- [ ] **步骤 5：运行 evaluation 测试**

执行命令：

```bash
cd cozmio
cargo test -p cozmio -- evaluation
cargo test -p cozmio -- eval_commands
```

预期结果：全部通过。

---

## 任务 6：语义边界回归检查

**涉及文件**：

- 修改：`cozmio/src-tauri/tests/semantic_boundary.rs`

- [ ] **步骤 1：确认新增内容只包含事实字段**

新增允许项：

```text
context_pack_built
runtime_selected_memory_entries
runtime_memory_competition_trace
vector_score
candidate_pool_size
vector_available
vector_provider
source_event_ids
token_estimate
```

继续禁止：

```text
保持沉默
弹窗策略
用户卡住
项目迭代机会
当前阶段
重要记忆
应该提醒
```

- [ ] **步骤 2：运行语义边界测试**

执行命令：

```bash
cd cozmio
cargo test -p cozmio --test semantic_boundary
```

预期结果：全部通过。

---

## 任务 7：端到端验证

**涉及文件**：

- 验证：`cozmio/cozmio_memory`
- 验证：`cozmio/src-tauri`
- 写回：`verification/last_result.json`
- 写回：`feature_list.json`
- 写回：`claude-progress.txt`

- [ ] **步骤 1：运行完整 Rust 测试**

执行命令：

```bash
cd cozmio
cargo test
```

预期结果：全部通过。

- [ ] **步骤 2：运行主 app 构建**

执行命令：

```bash
cd cozmio
cargo build -p cozmio
```

预期结果：构建通过。

- [ ] **步骤 3：验证向量主线证据**

执行命令：

```bash
cd cozmio
cargo test -p cozmio_memory -- vector_score
cargo test -p cozmio_memory -- backfill_candidate_embeddings
cargo test -p cozmio -- prompt_context
```

预期结果：

- `vector_score` 测试证明有 embedding_ref 时能算 cosine similarity。
- `backfill_candidate_embeddings` 测试证明 active candidate 可以被写入 embedding_ref。
- `prompt_context` 测试证明 selected memory entry 进入 runtime context packet。

- [ ] **步骤 4：写回飞轮文件**

更新 `verification/last_result.json`，写入：

```json
{
  "task": "PRACTICE-LOOP-V1-VECTOR-MAINLINE-CLOSURE",
  "status": "pass",
  "evidence": [
    "cargo test",
    "cargo build -p cozmio",
    "cargo test -p cozmio_memory -- vector_score",
    "cargo test -p cozmio_memory -- backfill_candidate_embeddings",
    "cargo test -p cozmio -- prompt_context"
  ],
  "mainline_guarantees": [
    "src-tauri enables cozmio_memory fastembed feature",
    "distillation candidates attempt embedding on write",
    "activity context performs bounded factual embedding backfill",
    "competition uses MemoryCore provider instance",
    "runtime context packet includes selected competition entries",
    "evaluation captures actual context_pack_built raw_text"
  ]
}
```

更新 `feature_list.json`，添加 `PRACTICE-LOOP-V1-VECTOR-MAINLINE-CLOSURE` 条目，状态为 `pass`。

更新 `claude-progress.txt`，写入本轮完成内容、验证命令和未进入本轮的内容：自动 sample capture、Evaluation 自动闭环、prompt 自动调参。

---

## 完成门槛

本方案完成时必须满足：

- `cargo build -p cozmio` 通过。
- `cargo test` 通过。
- `cargo tree -p cozmio -i fastembed` 能看到 `cozmio -> cozmio_memory -> fastembed`。
- `build_popup_context` 测试能看到 `runtime_selected_memory_entries` 和 `vector_score=...`。
- Evaluation sample 的 `context_pack_summary` 能来自 `context_pack_built` 的真实 raw_text。
- semantic boundary 测试继续通过。
- 没有新增 popup 频率限制、冷却、静默、阶段判断、用户意图判断。

## 自我审查

- 产品类型：已标记为 `deterministic_software + model_output_validated`。
- 规格覆盖度：覆盖 P0 feature gate、P0 candidate embedding 主线、P1 prompt entries 注入、P1 evaluation context pack、P2 自动 sample capture 拆出范围。
- 占位符排查：本方案不使用开放式占位描述，每个任务都有明确文件、代码形态、命令和预期结果。
- 类型一致性：使用现有 `ReminderContextDto`、`CompetitionResultEntryDto`、`CompetitionTraceDto`、`MemoryCore`、`EmbeddingProvider`、`LedgerEvent`。
- 语义边界：新增内容均为事实字段、数值、provider 名称、source refs、raw context text。
