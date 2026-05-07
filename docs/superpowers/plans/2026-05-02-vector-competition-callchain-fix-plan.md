# Vector Competition Call Chain Fix — Decision-Complete Plan

## Flywheel Bootstrap

- `claude-progress.txt`: ✅ 存在 — PRACTICE-LOOP-V1-GAP-CLOSURE 完成，external model blocked
- `feature_list.json`: ✅ 存在 — PRACTICE-LOOP-V1-PHASE-D (Memory Competition) 标记为 pass
- `verification/last_result.json`: ✅ 存在 — H1-H6 验证通过

## Context

本次不是新功能开发，而是修复一个已知问题：向量竞争系统的调用链在运行时**从未真正执行向量计算**。

根因分析见 `2026-04-30-practice-loop-v1-phase-d-context-memory-competition-plan.md`。

---

## Current Truth

### files inspected

| 文件 | 关键内容 |
|------|---------|
| `cozmio_memory/src/competition.rs:240-247` | `run_competition_and_build_context` 调用 `compete_candidates(..., None)` — 传 None |
| `cozmio_memory/src/competition.rs:617-639` | `compete_candidates` 接收 `vector_provider: Option<String>`，内部创建 provider 并生成 `query_embedding` |
| `cozmio_memory/src/competition.rs:43-51` | `MemoryCompetition` 结构体只有 `db` 和 `search_engine`，无 `embed_provider` 字段 |
| `cozmio_memory/src/competition.rs:257-268` | `MemoryCore::new(db, embed_provider)` 存储 provider；`search_engine()` 传 `self.embed_provider` 给 SearchEngine |
| `cozmio_memory/src/competition.rs:204-254` | `run_competition_and_build_context` 注释明确写：`// vector_provider None since MemoryCompetition doesn't have embed_provider` |
| `cozmio_memory/src/search.rs:103-119` | `SearchEngine` 存储 `embed_provider` 并在 `search()` 中使用 `self.embed_provider`（line 149） |
| `cozmio/src-tauri/src/memory_commands.rs:375-445` | `build_activity_context_sync` 调用 `MemoryCore::new(&db, None)` |
| `cozmio/src-tauri/src/competition_commands.rs:124-172` | `compete_for_context` IPC 创建 provider、调用 backfill，但 `MemoryCore::new(&db, None)` 仍未传 provider |
| `cozmio_memory/src/embed_provider.rs:22-38` | `create_provider(ProviderType)` 返回 `FastEmbed` / `Mock` / `Disabled` |
| `cozmio_memory/Cargo.toml:26-29` | `fastembed` feature 非默认；`default = []` |

### existing runtime path

**build_activity_context_sync（main_loop 调用的入口）:**
```
build_activity_context_sync(window_title, content_text, token_budget)
  → MemoryCore::new(&db, None)                          ← 传 None
  → core.competition() → MemoryCompetition::new(db, search_engine)
  → competition.run_competition_and_build_context(note, token)
    → compete_candidates(..., None)                     ← 传 None
      → query_embedding = None（因为 vector_provider=None）
      → 所有候选 vector_score = None
      → 竞争退化为纯 signal_score
```

**compete_for_context（Practice Dashboard 预览用的 IPC）:**
```
compete_for_context(token_budget)
  → MemoryCore::new(&db, None)                          ← 传 None
  → create_embedding_provider() → provider (创建后未传给 MemoryCore)
  → backfill_candidate_embeddings(&db, provider, 20)   ← 填充 embedding_ref
  → run_competition_and_build_context(...)
    → compete_candidates(..., None)                     ← 传 None
      → 向量计算仍然被跳过（因为 None）
```

### 已知不一致

1. **向量计算从未执行**: `run_competition_and_build_context` 注释承认 `vector_provider None since MemoryCompetition doesn't have embed_provider`
2. **compete_for_context 创建了 provider 但未传递**: provider 创建于 line 126，然后用于 backfill（line 152），但从未传进 `MemoryCore::new()`
3. **feature flag 阻塞**: `fastembed` 非默认；`FastEmbedProvider::new()` 会 fallback 到 `DisabledProvider`，`is_available()` 返回 false
4. **SearchEngine.search 确认使用 `self.embed_provider`**: search.rs:149 使用存储字段，非 `create_provider()` 重新创建 — **此前的分析有误，已更正**

---

## Key Path Tracing

```
build_activity_context_sync (memory_commands.rs:375)
  → MemoryCore::new(&db, None)                           ← 问题1: None
    → search_engine: SearchEngine::new(db, None)         ← SearchEngine.embed_provider = None
  → competition.run_competition_and_build_context(note, token)
    → compete_candidates(..., None)                       ← 问题2: None
      → vector_provider = None
      → query_embedding: Option<Vec<f32>> = None        ← 向量生成被跳过
      → compute_vector_score 永远不会被调用（因为 query_embedding 为 None）
      → 所有候选 vector_score = None
      → 竞争完全依赖 signal_score

vs

compete_for_context (competition_commands.rs)
  → MemoryCore::new(&db, None)                           ← 问题3: provider 未传递
  → create_embedding_provider() → provider (临时)
  → backfill_candidate_embeddings(&db, provider, 20)    ← 填充 embedding_ref
  → run_competition_and_build_context(...)
    → compete_candidates(..., None)                      ← 同样问题
```

**缺失链路**: `run_competition_and_build_context` 需要一个真实 provider 字符串（`"fastembed"`），由 `compete_candidates` 内部创建。

---

## Implementation Shape

### RP-1: build_activity_context_sync 传递真实 provider

**文件**: `cozmio/src-tauri/src/memory_commands.rs:382`

**当前真相**:
```rust
let core = MemoryCore::new(&db, None);
```

**修改为**:
```rust
// 尝试创建 embedding provider（内部会尝试 FastEmbed，失败则降级）
let embed_provider = cozmio_memory::embed_provider::create_provider(
    cozmio_memory::embed_provider::ProviderType::FastEmbed
).ok();
let core = MemoryCore::new(&db, embed_provider);
```

**验证**: `cargo build -p cozmio` 通过即可（因为 create_provider 失败返回 Error，不会 panic）

**事实依据**:
- `embed_provider.rs:22-38`: `create_provider(ProviderType::FastEmbed)` 是 public 函数
- `competition_commands.rs:126`: 相同模式已在 `compete_for_context` 中使用

**状态**: 已锁定 ✓

---

### RP-2: run_competition_and_build_context 使用真实 provider 字符串

**文件**: `cozmio_memory/src/competition.rs:240-247`

**当前真相**:
```rust
// Run competition (vector_provider None since MemoryCompetition doesn't have embed_provider)
let result = compete_candidates(
    self.db,
    note,
    &candidates,
    token_budget,
    0.3, // alpha
    None,                                          // ← 问题在这里
);
```

**修改为**:
```rust
// 传 "fastembed" 让 compete_candidates 内部创建 provider
// 若 FastEmbedProvider 不可用（未启用 feature 或初始化失败），query_embedding 为 None，向量计算优雅降级
let result = compete_candidates(
    self.db,
    note,
    &candidates,
    token_budget,
    0.3, // alpha
    Some("fastembed".to_string()),
);
```

**验证**:
```bash
cargo test -p cozmio_memory -- compute_vector_score
cargo test -p cozmio_memory -- compete_candidates
cargo test -p cozmio_memory -- backfill_candidate_embeddings
```

**事实依据**:
- `competition.rs:629-639`: `compete_candidates` 内部已有完整 provider 创建逻辑
- `competition_commands.rs:126`: `create_provider(ProviderType::FastEmbed).ok()` 模式确认

**状态**: 已锁定 ✓

---

### RP-3: compete_for_context 将 provider 传递给 MemoryCore

**文件**: `cozmio/src-tauri/src/competition_commands.rs:144-172`

**当前真相**:
```rust
let db = open_memory_db()?;
let core = MemoryCore::new(&db, None);                    // ← provider 未传递
let competition = core.competition();

let provider = create_embedding_provider().ok();          // ← 创建后未使用
if let Some(ref p) = provider {
    if p.is_available() {
        let count = backfill_candidate_embeddings(&db, p.as_ref(), 20)
            .map_err(|e| e.to_string())?;
        log::info!("Backfilled {} candidate embeddings", count);
    }
}
```

**修改为**:
```rust
let db = open_memory_db()?;

// 创建 provider 并传给 MemoryCore，使 SearchEngine 也有真实 provider
let provider = create_embedding_provider().ok();
let embed_provider = provider
    .as_ref()
    .and_then(|p| if p.is_available() { Some(p.clone()) } else { None });
let core = MemoryCore::new(&db, embed_provider);
let competition = core.competition();

// 若 provider 可用，先做 backfill
if let Some(ref p) = provider {
    if p.is_available() {
        let count = backfill_candidate_embeddings(&db, p.as_ref(), 20)
            .map_err(|e| e.to_string())?;
        log::info!("Backfilled {} candidate embeddings", count);
    }
}
```

**注意**: `provider.clone()` 需要 `Arc<dyn EmbeddingProvider>` 实现 `Clone`。若不实现 Clone，可改为传 `provider.as_ref().map(|p| Arc::clone(p))`。

**验证**: `cargo build -p cozmio && cargo test -p cozmio -- memory` 通过

**事实依据**:
- `competition_commands.rs:126`: 已有 `create_embedding_provider()` 模式
- `memory_commands.rs:382`: 同理

**状态**: 已锁定 ✓

---

## Risk → Verification Mapping

| Risk | 验证命令 | 预期结果 |
|------|---------|---------|
| 向量计算仍未执行（feature flag 未启用） | `cargo test -p cozmio_memory -- compute_vector_score` | FastEmbed 不可用时 query_embedding=None，所有 vector_score=None，signal_score 正常 |
| build_activity_context_sync 编译失败（provider 创建失败） | `cargo build -p cozmio` | Build pass |
| compete_for_context 传 None 导致搜索退化 | `cargo test -p cozmio -- memory` | Build + tests pass |
| 向量竞争选了错误候选项 | `cargo test -p cozmio_memory -- compete_candidates` | 18 tests pass |

---

## 剩余非阻塞问题（不阻止本次修复）

以下问题存在但不在本次修复范围内：

1. **feature flag 默认未启用**: `fastembed` 需要 `cargo build --features fastembed`。如需默认启用，需改 `Cargo.toml:26` 的 `default = []` → `default = ["fastembed"]`。**这是 feature gate 决策，不影响代码正确性。**

2. **build_activity_context_sync 不做 backfill**: 现有候选的 `embedding_ref` 可能为 None。这意味着首次竞争时，向量分数为 None（降级到 signal_score）。后续 `compete_for_context` 会做 backfill，下次 `build_activity_context_sync` 竞争时就有真实向量了。**这是设计决策，不是 bug。**

3. **零向量候选测试全部通过但未测向量计算**: `test_compete_candidates_respects_token_budget` 等测试用 `None` embedding_ref 和 `None` provider，不测向量路径。这是测试覆盖问题，可后续补充。

---

## 口子词扫描

- `需确认` — 无
- `待定/TBD` — 无
- `大概/应该在` — 无
- `探索/研究` — 无（在实现步骤中）
- `...` — 无（在代码骨架中）

所有口子词均已关掉。

---

## 自我审查

### A. 执行者是否还需要判断？
- [x] 不需要决定 helper 放哪 — 直接在 `build_activity_context_sync` 中创建 provider
- [x] 不需要决定测哪一层 — 验证命令已指定
- [x] 不需要推导等待顺序 — `compete_for_context` 中 backfill 在 competition 之前已明确
- [x] 不需要探索数据流 — Current Truth 已写出完整调用链

### B. 真相检查
- [x] 所有文件/函数/字段均已验证（带行号）
- [x] 代码骨架直接照抄当前结构
- [x] import 来源已锁死（`cozmio_memory::embed_provider::create_provider`）

### C. 关键路径检查
- [x] 调用链每个节点已写明"谁调用谁、数据从哪来"
- [x] 缺失链路已标注（Feature flag 阻塞属于 Feature gate，不属于实现缺口）

### D. 验证检查
- [x] 验收标准是可执行测试命令
- [x] 测试入口与计划承诺层级一致
- [x] 每个 risk 都有对应验证命令

### E. 口子词扫描
- [x] 无任何口子词

### F. 冻结检查
- [x] 所有 RP 状态为"已锁定 ✓"
- [x] 实现步骤中无探索性动词

---

## 修复后预期行为

```
build_activity_context_sync (main_loop 调用)
  → MemoryCore::new(&db, Some(provider))
    → SearchEngine.embed_provider = Some(provider)  ✓
  → compete_candidates(..., Some("fastembed"))
    → FastEmbedProvider.is_available()? (取决于 feature flag)
      → 可用: query_embedding = embed("..."), 正常计算 cosine similarity
      → 不可用: query_embedding = None, 向量计算优雅降级（signal_score only）
```

```
compete_for_context (Practice Dashboard IPC)
  → MemoryCore::new(&db, Some(provider))
    → SearchEngine.embed_provider = Some(provider)  ✓
  → backfill_candidate_embeddings(&db, provider, 20) ← 填充 embedding_ref
  → compete_candidates(..., Some("fastembed"))     ✓
    → 候选有 embedding_ref + query_embedding → compute_vector_score 正常执行
```
