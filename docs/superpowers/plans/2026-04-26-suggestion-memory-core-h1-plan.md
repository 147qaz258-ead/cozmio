# Suggestion Memory Core H1 — 实施方案

> **智能执行体须知**：本方案以运行效果、验证资产和飞轮写回为中心。

## 1. Flywheel Context

- active task: Suggestion Memory Core H1
- current phase: 初始阶段，尚未开始实现
- latest verification: SYSTEM-NOTIFICATION-CHAIN pass (2026-04-25)
- blocker (if any): none
- next expected step: 创建 cozmio_memory crate，建立数据库 schema，接入 action_log.jsonl

## 2. Goal

建立 cozmio_memory crate（含 SQLite + FTS5 + sqlite-vec + 本地 EmbeddingProvider），实现 memory events/slices/threads/decision/skill 五张表存储，从真实 action_log.jsonl 导入数据，通过 memory-cli 和 Tauri IPC 双入口暴露 stats/import/search/replay 命令，Memory Competition 从 ActivityNote 输出基于真实检索结果的 ReminderContext（含 imported evidence），src-tauri IPC 可调用并完成旧 reason 与新 ReminderContext 的对比验证。

## 3. Product Type

- type: `deterministic_software`（核心是 Rust 数据库操作和 CLI 命令）
- core risk: SQLite migrations 顺序正确性、sqlite-vec Windows 兼容性、FastEmbed 运行时可用性
- verification style: cargo build + memory-cli 命令实际输出 + Tauri IPC 响应

## 4. Global Roadmap

| Phase | 目标 | 依赖 | 验收意图 |
|-------|------|------|---------|
| H1 | 第一条真实纵向闭环 | — | 从 action_log.jsonl → ReminderContext → src-tauri IPC 可调用；旧 reason vs ReminderContext 对比验证 |
| H2 | src-tauri 深度集成 + 实时 ActivityNote → ReminderContext | H1 | main_loop.rs 调用 memory core，建议链路闭环 |
| H3 | Skill Memory 自动沉淀 + Memory Competition 排序迭代 | H1+H2 | 成功执行流程自动沉淀，建议质量提升 |

## 5. Scope

### In（本次包含）

- cozmio_memory crate 创建（Cargo.toml + lib.rs + db.rs + schema.rs）
- EmbeddingProvider trait + **FastEmbedProvider**（本地，H1 正式）+ MockProvider（测试用）+ DisabledProvider（降级用）
  - OllamaProvider **仅作为 experimental feature**，默认关闭，不参与 H1 验收
- 5 张表：memory_events（含 FTS5 + sqlite-vec）、context_slices、task_threads、decision_memory、skill_memory
- Hybrid Search（search.rs）：FTS5 关键词 + sqlite-vec 语义 + metadata 过滤
- Memory Competition（competition.rs）：build_reminder_context，**基于真实检索结果合成，不硬编码候选建议**
- importer.rs：从 action_log.jsonl 导入（含 action_log.jsonl 格式解析），每条 imported 记录标记 `evidence_source = "imported"`
- memory-cli 二进制（stats / import / search / replay / inspect / rebuild-index）
- Tauri IPC 命令（memory_commands.rs）
- seed data：3 个 task_threads + 3 个 decision_memory + 1 个 skill_memory（标记 `evidence_source = "seed"`）
- **src-tauri IPC 体验验证接点**：build_activity_context 可接收真实 ActivityNote，返回 ReminderContext（含 imported evidence）；提供旧 judgment reason 与新 ReminderContext 的对比样例

### Out（本次不包含）

- **不修改现有 judgment 链路**（main_loop.rs 中 judgment 逻辑保持不变）
- 不做实时弹窗（replay 验证质量后才考虑）
- 不做 Skill Memory 自动沉淀（只支持手动添加）
- 不做 Memory Competition 自动排序算法（hardcoded priority 拼接）
- 不做 embedding model fine-tuning
- **OllamaProvider 不进入 H1 正式架构**（即使保留也只能通过 experimental feature flag 开启，默认 off）

## 6. Current Truth

- files inspected: `cozmio/Cargo.toml`, `cozmio/src-tauri/Cargo.toml`
- existing entry points: Tauri IPC commands (commands.rs), relay_bridge.rs, main_loop.rs
- existing runtime path: WindowMonitor → Ollama → Toast → Relay → Claude Code
- existing verification (if any): `cargo build` + cozmio.log + action_log.jsonl
- workspace location: `D:\C_Projects\Agent\cozmio\cozmio\`（Cargo workspace root）

## 7. Implementation Shape（Phase H1 分步）

### Step 1：创建 cozmio_memory crate

**目标**：workspace 骨架可用，`cargo build -p cozmio_memory` 通过。

创建 `cozmio/cozmio_memory/` 目录结构：

```
cozmio_memory/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── error.rs           # MemoryError enum
    ├── db.rs              # Database 连接管理
    └── schema.rs          # migrations
```

**Cargo.toml 依赖**（按顺序重要性）：

```toml
[package]
name = "cozmio_memory"
version = "0.1.0"
edition = "2021"

[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
sqlite-vec = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2"
log = "0.4"

[dependencies.fastembed]
version = "3"
optional = true
default-features = false

[features]
default = ["fastembed"]
fastembed = ["dep:fastembed"]
experimental_ollama = []
```

**注意**：
- `rusqlite` 使用 `bundled` feature，绑定 bundled SQLite 3.x，保证 FTS5 和 sqlite-vec 兼容
- FastEmbed 是 H1 正式依赖，从 Step 1 就引入（不是 Step 5 才引入）
- Step 1-2 用 `MockProvider` 临时占位，待 Step 4-5 FastEmbed 实现后替换

**验证**：`cargo build -p cozmio_memory` 通过。

---

### Step 2：schema.rs — 数据库 migrations

**目标**：5 张表 + FTS5 + sqlite-vec virtual table 创建函数。

在 `schema.rs` 中实现：

```rust
pub fn run_migrations(conn: &Connection) -> Result<(), MemoryError>;

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_create_memory_events", CREATE_MEMORY_EVENTS_SQL),
    ("002_create_context_slices", CREATE_CONTEXT_SLICES_SQL),
    ("003_create_task_threads", CREATE_TASK_THREADS_SQL),
    ("004_create_decision_memory", CREATE_DECISION_MEMORY_SQL),
    ("005_create_skill_memory", CREATE_SKILL_MEMORY_SQL),
    ("006_create_fts5", CREATE_FTS5_SQL),
    ("007_create_vec_index", CREATE_VEC_INDEX_SQL),
];

const CREATE_MEMORY_EVENTS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS memory_events (
    id INTEGER PRIMARY KEY,
    timestamp TEXT NOT NULL,
    source TEXT NOT NULL,
    window_title TEXT,
    content TEXT NOT NULL,
    raw_ref TEXT,
    embedding BLOB,
    thread_id INTEGER REFERENCES task_threads(id),
    created_at TEXT DEFAULT (datetime('now'))
);
"#;
// ... 其余 4 张表类似
```

**FTS5 trigger**（保证 FTS 索引随写入更新）：

```sql
CREATE TRIGGER IF NOT EXISTS memory_events_ai AFTER INSERT ON memory_events BEGIN
    INSERT INTO memory_events_fts(rowid, content, window_title)
    VALUES (new.id, new.content, new.window_title);
END;
```

**验证**：
- `cargo test -p cozmio_memory`（连接 DB 并验证 migrations 可执行）
- 验证 5 张表 + FTS5 virtual table + sqlite-vec virtual table 均创建成功

---

### Step 3：memory store 模块（events / threads / decision / skill / slices）

**目标**：每个 store 实现 CRUD，不实现业务逻辑。

每个模块结构：

```rust
// src/memory_events.rs
pub struct MemoryEvent {
    pub id: Option<i64>,
    pub timestamp: String,
    pub source: String,
    pub window_title: Option<String>,
    pub content: String,
    pub raw_ref: Option<String>,
    pub embedding: Option<Vec<u8>>,
    pub thread_id: Option<i64>,
}

pub struct MemoryEventsStore { db: &Connection }

impl MemoryEventsStore {
    pub fn insert(&self, event: &MemoryEvent) -> Result<i64, MemoryError>;
    pub fn get_by_id(&self, id: i64) -> Result<Option<MemoryEvent>, MemoryError>;
    pub fn count(&self) -> Result<i64, MemoryError>;
    pub fn get_recent(&self, limit: usize) -> Result<Vec<MemoryEvent>, MemoryError>;
    pub fn get_in_time_range(&self, start: &str, end: &str) -> Result<Vec<MemoryEvent>, MemoryError>;
}
```

对 `task_threads / decision_memory / skill_memory / context_slices` 分别实现类似结构。

**context_slices 额外方法**：

```rust
pub fn get_recent_slices(&self, since_hours: u32) -> Result<Vec<ContextSlice>, MemoryError>;
```

**验证**：每个 store 有单元测试覆盖基础 CRUD。

---

### Step 4：EmbeddingProvider trait + 本地 Provider 实现

**目标**：接口稳定，**FastEmbedProvider 是 H1 唯一正式 Provider**，Mock/Disabled 仅用于测试和降级。

```rust
// src/embed_provider.rs
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, MemoryError>;
    fn dimension(&self) -> usize;
    fn is_available(&self) -> bool;
}

pub enum EmbeddingProviderType {
    FastEmbed,   // H1 正式方向
    #[cfg(feature = "experimental_ollama")]
    Ollama,      // 仅 experimental，默认关闭
    Mock,        // 仅测试用
    Disabled,    // 降级用
}

impl MemoryCore {
    pub fn with_provider(self, provider: Arc<dyn EmbeddingProvider>) -> Self;
}

impl dyn EmbeddingProvider {
    /// 按优先级尝试创建可用 Provider：
    /// 1. FastEmbed（默认，H1 正式）
    /// 2. Disabled（无可用本地 provider 时降级）
    pub fn try_new() -> Result<Arc<dyn EmbeddingProvider>, MemoryError>;
}
```

**实现要求**：
- `try_new()` 自动检测可用 Provider：优先 FastEmbed，如果编译失败或不可用则降级到 DisabledProvider
- **MockProvider 仅用于单元测试**，不作为 H1 完成标准
- **DisabledProvider 降级时上层 search 纯走 FTS5**，但数据结构保留 embedding 字段

**验证**：`cargo test -p cozmio_memory` 通过，且 `embed("hello")` 返回有效 384 维向量（FastEmbed 或降级 Disabled）。

---

### Step 5：FastEmbedProvider 实现（Windows 兼容优先）

**目标**：FastEmbed 在 Windows 可用，返回有效向量。

```toml
# cozmio_memory/Cargo.toml
[dependencies]
fastembed = "3"   # 或 candlelight / rustformers / 基于 ONNX 的轻量 embedding

[features]
default = ["fastembed"]
fastembed = ["dep:fastembed"]
```

**Windows 兼容性策略**：
- `fastembed` 依赖 tokenizers 和怜处理 UTF-8 文本
- 如果 `fastembed` 编译失败，设置 `default-features = false`，`try_new()` 返回 DisabledProvider
- 数据结构不变：embedding 字段保留，后续环境具备时直接启用

**验证**：`cargo build -p cozmio_memory` 通过，且以下代码返回有效向量：

```rust
let provider = FastEmbedProvider::new()?;
assert!(provider.is_available());
let vec = provider.embed("Claude Code execution");
assert_eq!(vec.len(), 384);
```

如果 Windows 编译失败：
- 记录 warning 到日志
- 自动降级到 DisabledProvider
- 不阻塞 H1 其他步骤

---

### Step 5：FastEmbedProvider 实现（Windows 兼容优先）

**目标**：FastEmbed 可在 Windows 环境加载，返回有效向量。

FastEmbed 的 Windows 风险：
- `fastembed` crate 可能需要 C++ runtime 或额外的本地库
- 如果编译失败，降级策略：检测 `fastembed` feature flag，feature off 时自动用 MockProvider

**Cargo.toml 新增**（带 feature flag）：

```toml
[features]
default = ["fastembed"]
fastembed = ["dep:fastembed"]
```

如果 `fastembed` 编译失败，设置 `default-features = false` 并在 `try_new()` 中返回 DisabledProvider。

**验证**：`cargo build -p cozmio_memory --features fastembed` 通过，且 `embed("hello")` 返回 384 维向量。

---

### Step 6：Hybrid Search 实现（search.rs）

**目标**：FTS5 + sqlite-vec + metadata/time 过滤的 search 接口。

```rust
// src/search.rs
pub struct SearchQuery {
    pub text: Option<String>,
    pub time_range: Option<(String, String)>,  // ISO8601 strings
    pub thread_id: Option<i64>,
    pub memory_types: Option<Vec<MemoryType>>,
    pub limit: usize,
}

pub struct SearchResult {
    pub event_id: i64,
    pub score: f32,
    pub source: String,
    pub content: String,
    pub window_title: Option<String>,
    pub timestamp: String,
}

pub struct SearchResults {
    pub events: Vec<SearchResult>,
    pub total_fts: usize,
    pub total_vec: usize,
}

impl MemoryCore {
    pub fn search(&self, query: &SearchQuery) -> Result<SearchResults, MemoryError>;
}
```

**检索流程**：

```
search(text="Toast 无效")
  ├─ FTS5: SELECT rowid, content, window_title FROM memory_events_fts WHERE content MATCH 'Toast 无效'
  │         ORDER BY rank LIMIT 50
  ├─ sqlite-vec: (如果 provider 可用) SELECT rowid, distance FROM memory_events_vec
  │               WHERE embedding MATCH vec0(?)
  └─ merge: 按 score 排序，取 top limit
```

**metadata 过滤**：`time_range` 和 `thread_id` 在 SQL 层加 WHERE 子句。

**验证**：`memory-cli search "Claude Code"` 返回结果；`memory-cli search "Toast"` 返回结果。

---

### Step 7：Memory Competition 实现（competition.rs）

**目标**：从 ActivityNote 生成 ReminderContext，**完全基于真实检索结果，不硬编码候选建议**。

```rust
// src/competition.rs
pub struct ActivityNote {
    pub window_title: String,
    pub content_text: String,
    pub timestamp: String,
    pub current_thread_id: Option<i64>,
}

pub struct EvidenceRef {
    pub source: String,       // "imported" | "seed" | "generated"
    pub memory_type: String,  // "memory_event" | "context_slice" | "decision" | "skill"
    pub id: i64,
    pub content_snippet: String,
    pub timestamp: Option<String>,
}

pub struct ReminderContext {
    pub current_activity: String,       // 来自 ActivityNote
    pub recent_context: String,        // 来自 context_slices（必须包含至少一条 imported 记录）
    pub related_decisions: String,     // 来自 decision_memory
    pub relevant_skills: String,       // 来自 skill_memory
    pub task_state: Option<String>,    // 来自 task_threads
    pub evidence_refs: Vec<EvidenceRef>, // 所有 evidence 来源，标注 source
}

impl MemoryCore {
    pub fn build_reminder_context(
        &self,
        note: &ActivityNote,
    ) -> Result<ReminderContext, MemoryError>;
}
```

**Competition 合并逻辑（hardcoded priority，不硬编码内容）**：

```
build_reminder_context(ActivityNote)
  ├─ recent_context:
  │    ├─ 获取最近 2h 的 context_slices
  │    └─ 拼接 slice.summary，标注 evidence_refs
  │    （必须至少包含一条 evidence_source="imported" 的 slice，否则返回 error）
  ├─ related_decisions:
  │    ├─ 如果有 current_thread_id，查询对应 decision_memory
  │    └─ 拼接 decision.content，标注 evidence_refs
  ├─ relevant_skills:
  │    ├─ 查询 skill_memory（usage_count > 0 优先）
  │    └─ 拼接 skill.description + procedure，标注 evidence_refs
  ├─ task_state:
  │    └─ 查询 task_thread，拼接 current_state + open_questions
  └─ evidence_refs:
       └─ 收集上述所有 evidence，标注 source（imported/seed/generated）
```

**关键约束**：
- recent_context **必须至少包含一条 imported evidence**（来自 action_log.jsonl），否则 return `Err(MemoryError::InsufficientImportedData)`
- 不生成硬编码建议文本；ReminderContext 只包含真实检索到的上下文
- 所有 evidence_ref 必须标注 source：imported（来自真实导入）/ seed（来自手动创建）/ generated（模型生成，如 slice.summary）

**验证**：

```rust
// 单元测试
let ctx = core.build_reminder_context(&note).unwrap();
// assert ctx.recent_context 不是空的
// assert ctx.evidence_refs.iter().any(|e| e.source == "imported")
```

---

### Step 8：importer.rs — 从 action_log.jsonl 导入

**目标**：解析并导入真实数据到 memory_events，每条记录标注 `evidence_source = "imported"`；生成 context_slices 时标注 `evidence_source = "generated"`。

**action_log.jsonl 格式**（来自 `src-tauri/src/logging.rs` 的 ActionRecord）：

```json
{"timestamp":"2026-04-25T15:25:21.123Z","trace_id":"...","session_id":"...","window_title":"Claude Code","judgment":"CONTINUE","next_step":"suggest","level":"info","confidence":0.95,"grounds":"...","system_action":"Toast notification sent","content_text":"...","result_text":"...","user_feedback":"..."}
```

**memory_events 表新增字段**（修改 schema）：

```sql
ALTER TABLE memory_events ADD COLUMN evidence_source TEXT DEFAULT 'imported';
-- 可选值：imported（来自 action_log.jsonl）/ seed（手动）/ generated（模型生成）
```

**importer 实现**：

```rust
// src/importer.rs
pub struct ActionLogImporter<'a> {
    path: &'a Path,
}

pub struct ImportStats {
    pub events_imported: usize,
    pub slices_generated: usize,
    pub errors: Vec<String>,
    pub time_range: (String, String),
}

impl ActionLogImporter {
    pub fn import(&self, db: &Connection) -> Result<ImportStats, MemoryError>;
}
```

**导入流程**：

```
import()
  ├─ 打开 action_log.jsonl，逐行解析 ActionRecord
  ├─ 对每条记录：
  │    ├─ 插入 memory_events (source="action_log", evidence_source="imported")
  │    ├─ 生成 embedding（如 provider 可用）并插入 sqlite-vec
  │    └─ 批量提交（每 100 条提交一次）
  ├─ 导入完成后，扫描所有事件，按 15 分钟窗口生成 context_slices
  │    └─ context_slices.evidence_source = "generated"
  └─ 返回 ImportStats（含 imported 数量）
```

**容错**：字段缺失不报错，跳过该字段；JSON 解析失败跳过该行并记录到 errors。

**验证**：
- `memory-cli import` 执行后，`memory-cli stats` 显示 events_imported > 0
- `memory-cli replay --since 2h` 的 ReminderContext.evidence_refs 中至少一条 source="imported"

---

### Step 9：memory-cli 二进制

**目标**：6 个命令全部可执行。

```rust
// memory-cli/src/main.rs
#[derive(Clap)]
enum Command {
    Stats,
    Import,
    Search { query: String },
    Replay { since_hours: Option<u32> },
    Inspect { slice_id: i64 },
    RebuildIndex,
}

fn main() {
    // 初始化 cozmio_memory::MemoryCore
    // 根据命令调用对应方法
    // 输出格式化结果
}
```

**Cargo.toml**：

```toml
[[bin]]
name = "memory-cli"
path = "src/main.rs"

[dependencies]
cozmio_memory = { path = "../cozmio_memory" }
clap = { version = "4", features = ["derive"] }
```

**rebuild-index**：删除并重建 FTS5 和 sqlite-vec virtual tables（紧急修复用）。

**验证**：

```bash
cargo build -p memory-cli
./target/debug/memory-cli stats
./target/debug/memory-cli import
./target/debug/memory-cli search "Toast"
./target/debug/memory-cli replay --since 2h
```

---

### Step 10：Tauri IPC 命令 + src-tauri 集成

**目标**：src-tauri 可调用 cozmio_memory；提供体验验证接点：给定真实 ActivityNote，返回含 imported evidence 的 ReminderContext。

**src-tauri/Cargo.toml 新增依赖**：

```toml
cozmio_memory = { path = "../../cozmio_memory" }
```

**memory_commands.rs**：

```rust
#[tauri::command]
fn get_memory_stats() -> Result<cozmio_memory::MemoryStats, String>;

#[tauri::command]
fn import_existing_logs() -> Result<cozmio_memory::ImportStats, String>;

#[tauri::command]
fn search_memory(query: cozmio_memory::SearchQuery) -> Result<cozmio_memory::SearchResults, String>;

#[tauri::command]
fn build_activity_context(
    note: cozmio_memory::ActivityNote,
) -> Result<cozmio_memory::ReminderContext, String>;
// 验证方法：传入当前 window_title + content_text，返回 ReminderContext，
// evidence_refs 中至少一条 source="imported"

#[tauri::command]
fn run_suggestion_replay(since_hours: u32) -> Result<cozmio_memory::ReplayOutput, String>;

#[tauri::command]
fn get_task_threads() -> Result<Vec<cozmio_memory::TaskThread>, String>;

#[tauri::command]
fn update_task_thread(thread: cozmio_memory::TaskThreadUpdate) -> Result<cozmio_memory::TaskThread, String>;

#[tauri::command]
fn get_decision_memory() -> Result<Vec<cozmio_memory::Decision>, String>;

#[tauri::command]
fn add_decision(decision: cozmio_memory::DecisionInput) -> Result<cozmio_memory::Decision, String>;

#[tauri::command]
fn get_skill_memory() -> Result<Vec<cozmio_memory::Skill>, String>;
```

**H1 不修改现有 judgment 链路**，但提供以下验证接点：

```rust
// main_loop.rs - 新增集成点（注释，不改现有 judgment 逻辑）
// TODO H2: judgment 完成后，生成 ActivityNote 并调用 memory core
//   let activity_note = ActivityNote {
//       window_title: current_window.title.clone(),
//       content_text: judgment_reason.clone(),
//       timestamp: chrono::Utc::now().to_rfc3339(),
//       current_thread_id: None,
//   };
//   let ctx = memory_core.build_reminder_context(&activity_note)?;
//   // ctx 进入后续建议生成流程
//
// H1 验证方式：在 Tauri dev 模式或 memory-cli 中手动传入 ActivityNote，
// 对比 judgment reason 和 ReminderContext，证明新上下文更丰富
```

**commands.rs 注册**：将 memory_commands 注册到 Tauri app builder。

**验证**：
1. `cargo build -p cozmio` 通过
2. `build_activity_context` IPC 调用返回含 imported evidence 的 ReminderContext
3. `run_suggestion_replay` 输出 evidence_refs 中至少一条 source="imported"

---

### Step 11：seed data + 验收验证

**目标**：system 能用真实 imported 数据跑出有意义的 ReminderContext；replay 输出标注每条 evidence 的 source。

**seed data 插入**（通过 memory-cli init 或代码中的 init_fn，标注 `evidence_source = "seed"`）：

**task_threads**：

| name | current_state | open_questions | decisions |
|------|--------------|----------------|-----------|
| Cozmio 使用体验改造 | 已有弹窗和执行链路，但提醒不准 | 如何形成有效建议；如何构建最小记忆底座 | 端侧负责整理小范围信息；云端处理更大上下文 |
| Cozmio 硬件线 Local Agent Box | 树莓派 + 端侧部署 | 硬件规格；Agent Box 软件栈 | — |
| Claude Code 执行链路 | Relay dispatch + subprocess 管理 | — | Toast → Relay → Claude Code 已跑通 |

**decision_memory**（evidence_source="seed"）：

| memory_type | content | evidence |
|-------------|---------|----------|
| rejected_direction | 用户反对为了方便绕开向量检索、长期记忆等核心技术 | 2026-04-26 brainstorm |
| rejected_direction | 用户不接受牺牲终局体验换取实现便利 | 2026-04-26 brainstorm |
| user_preference | 用户希望技术方案从好用出发，不从省事出发 | 2026-04-26 brainstorm |

**skill_memory**（evidence_source="seed"）：

| name | description | procedure | success_context |
|------|-------------|-----------|----------------|
| Toast → Relay → Claude Code 执行流程 | 完整的端侧提醒到执行链路 | 1. judgment CONTINUE 2. 发送 Toast 带 confirm/cancel 3. 用户确认 4. Relay dispatch 5. Claude Code subprocess 6. progress 回传 7. result 通知 | 2026-04-25 E2E 验证通过 |

**最终验收**：

```bash
# 1. build
cargo build -p cozmio_memory && cargo build -p memory-cli && cargo build -p cozmio

# 2. import（必须先执行，导入真实 action_log.jsonl）
./target/debug/memory-cli import
# 期望：events_imported > 0，slices_generated > 0

# 3. stats
./target/debug/memory-cli stats
# 期望：显示 imported 事件数量、时间范围、来源分布

# 4. search
./target/debug/memory-cli search "Claude Code"
./target/debug/memory-cli search "Toast"
# 期望：返回相关历史记录

# 5. replay（含 evidence source 标注）
./target/debug/memory-cli replay --since 2h
# 期望输出格式：
#   ReminderContext {
#     current_activity: "...",
#     recent_context: "...",
#     evidence_refs: [
#       { source: "imported", memory_type: "memory_event", id: 42, content_snippet: "...", timestamp: "..." },
#       { source: "seed", memory_type: "decision", id: 1, content_snippet: "...", timestamp: null },
#     ]
#   }
# 期望：evidence_refs 中至少一条 source="imported"

# 6. 旧 reason vs ReminderContext 对比验证
# 在 memory-cli 或 Tauri IPC 中手动传入一个 ActivityNote（如 window_title="Claude Code", content_text="模型判断：CONTINUE - 可以执行"），
# 对比：
#   - 旧 judgment reason: "CONTINUE - 可以执行"（无历史上下文）
#   - 新 ReminderContext: 包含 imported historical events + seed decisions + task state（有历史上下文）
# 记录对比结果到 verification/ 文件

# 7. Tauri IPC
# 在 Tauri dev 模式下调用 get_memory_stats / build_activity_context
# 期望：ReminderContext.evidence_refs 包含 source="imported" 的记录
```

---

## 8. Verification Asset

### 8.1 Build 验证

```yaml
verification type: deterministic_software
command: cargo build -p cozmio_memory && cargo build -p memory-cli && cargo build -p cozmio
expected evidence: 所有 crate 编译通过，无错误
evidence location: stdout
failure condition: 任何 crate 编译错误
```

### 8.2 CLI 功能验证

```yaml
verification type: deterministic_software
command: |
  ./target/debug/memory-cli import
  ./target/debug/memory-cli stats
  ./target/debug/memory-cli search "Claude Code"
  ./target/debug/memory-cli replay --since 2h
expected evidence: |
  import 报告 events_imported > 0
  stats 显示 imported 事件数量 > 0
  search 返回包含 "Claude Code" 的记录
  replay 输出 ReminderContext，
    - evidence_refs 中至少一条 source="imported"
    - evidence_refs 中至少一条 source="seed"
    - 不含硬编码候选建议
evidence location: stdout
failure condition: imported 记录为 0，或 evidence_refs 无 source="imported"，或存在硬编码建议
```

### 8.3 Tauri IPC 验证

```yaml
verification type: deterministic_software
command: |
  # 在 Tauri dev 模式或 memory-cli 中调用 build_activity_context
  # 传入: { window_title: "Claude Code", content_text: "CONTINUE - 模型认为可以执行", timestamp: "...", current_thread_id: null }
expected evidence: |
  返回 ReminderContext，
    - recent_context 非空
    - evidence_refs 至少一条 source="imported"
    - evidence_refs 至少一条 source="seed"
evidence location: Tauri app 日志或 memory-cli 输出
failure condition: 无 imported evidence，或返回空 recent_context
```

### 8.4 本地 Embedding Provider 验证

```yaml
verification type: deterministic_software
command: |
  # 在 cozmio_memory 测试或 memory-cli 中
  let provider = FastEmbedProvider::new().unwrap();
  let vec = provider.embed("Claude Code execution chain");
expected evidence: |
  provider.is_available() == true
  vec.len() == 384
evidence location: test output 或 memory-cli stats 中的 provider 状态
failure condition: provider 不可用，或向量维度不为 384
降级处理：如果 FastEmbed 不可用，验证 DisabledProvider 被自动选中，且日志中有 warning
```

### 8.5 旧 reason vs ReminderContext 对比验证

```yaml
verification type: desktop_ui_runtime + execution_trace
command: |
  # 从 action_log.jsonl 取一条真实 judgment reason
  # 例如: "CONTINUE - 可以执行"
  #
  # 调用 build_activity_context
  # 对比输出 ReminderContext：
  #   - 旧 reason：只有模型判断，无历史
  #   - 新 ReminderContext：含 imported historical events + seed decisions + task state
expected evidence: |
  对比样例写入 verification/memory-core-h1-context-comparison.md
  格式：
    ## 样例 1
    - 时间：...
    - ActivityNote: window_title="...", content_text="CONTINUE - 可以执行"
    - 旧 judgment reason: "CONTINUE - 可以执行"（无历史上下文）
    - 新 ReminderContext:
      - recent_context: "00:20-00:35 用户持续讨论 Cozmio 端侧提醒无效..."（含 imported evidence）
      - evidence_refs: [imported event id=42, seed decision id=1, ...]
      - task_state: "Cozmio 使用体验改造 - 已有弹窗但提醒不准"
    - 结论：新 ReminderContext 比旧 reason 多出真实历史上下文，能支撑更有效的执行 brief
evidence location: verification/memory-core-h1-context-comparison.md
failure condition: 无对比样例，或对比中 imported evidence 为空
```

---

## 9. Phase Gate

本 Phase 只有满足以下条件才能标记为完成：

- [ ] `cargo build -p cozmio_memory -p memory-cli -p cozmio` 全部通过
- [ ] **本地真实 embedding provider 可用**（FastEmbed 或降级到 DisabledProvider 但数据结构完整）
- [ ] `memory-cli import` 成功导入 action_log.jsonl，events_imported > 0
- [ ] `memory-cli stats` 显示 imported 事件数量 > 0
- [ ] `memory-cli search "Claude Code"` 返回结果
- [ ] `memory-cli replay --since 2h` 输出 ReminderContext，**不含硬编码候选建议**
- [ ] `ReminderContext.evidence_refs` 中**至少一条 source="imported"**
- [ ] **Memory Competition 结果不是 hardcoded candidate**（基于真实检索）
- [ ] `add_decision` IPC 可保存决策记忆
- [ ] `build_activity_context` IPC 返回 ReminderContext（含 current_activity / recent_context / related_decisions）
- [ ] **Tauri IPC 能基于真实 ActivityNote 返回含 imported evidence 的 ReminderContext**
- [ ] **已记录旧 judgment reason 与新 ReminderContext 的对比样例**（写入 verification/memory-core-h1-context-comparison.md）
- [ ] `verification/last_result.json` 已更新
- [ ] `feature_list.json` 已添加 `SUGGESTION-MEMORY-CORE-H1` 条目
- [ ] `claude-progress.txt` 已更新下一轮交接内容

---

## 10. Next Execution Step

- next phase: H1 开始执行
- goal: 完成 Step 1-2（crate 创建 + schema migrations），验证 build 通过
- entry skill: `execute-plan` 或直接实现
- stop condition: `cargo build -p cozmio_memory` 通过，单元测试通过 migrations

---

## 附录：文件创建清单

```
cozmio/cozmio_memory/Cargo.toml          [新建]
cozmio/cozmio_memory/src/lib.rs          [新建]
cozmio/cozmio_memory/src/error.rs        [新建]
cozmio/cozmio_memory/src/db.rs           [新建]
cozmio/cozmio_memory/src/schema.rs       [新建]
cozmio/cozmio_memory/src/embed_provider.rs  [新建]
cozmio/cozmio_memory/src/embed_fastreembed.rs [新建]   # H1 正式
cozmio/cozmio_memory/src/embed_mock.rs   [新建]        # 仅测试
cozmio/cozmio_memory/src/embed_disabled.rs [新建]      # 降级用
cozmio/cozmio_memory/src/embed_ollama.rs [新建 - experimental，默认关闭，不进 H1 验收]
cozmio/cozmio_memory/src/memory_events.rs [新建]
cozmio/cozmio_memory/src/context_slices.rs [新建]
cozmio/cozmio_memory/src/task_threads.rs  [新建]
cozmio/cozmio_memory/src/decision_memory.rs [新建]
cozmio/cozmio_memory/src/skill_memory.rs  [新建]
cozmio/cozmio_memory/src/search.rs        [新建]
cozmio/cozmio_memory/src/competition.rs   [新建]
cozmio/cozmio_memory/src/importer.rs      [新建]

cozmio/memory-cli/Cargo.toml              [新建]
cozmio/memory-cli/src/main.rs             [新建]

cozmio/src-tauri/src/memory_commands.rs   [新建]
cozmio/src-tauri/src/main_loop.rs         [修改：添加 H2 集成点注释，不改现有 judgment 逻辑]

cozmio/Cargo.toml                         [修改：添加 cozmio_memory 和 memory-cli 到 workspace members]
```
