# Suggestion Memory Core H1 — 设计文档

> 状态：已批准
> 日期：2026-04-26

---

## 1. 背景与目标

### 1.1 当前状态

Cozmio 已有完整的采集和执行链路：

- WindowMonitor → Screenshot → Ollama Vision → Judgment
- Toast → Relay dispatch → Claude Code subprocess → Result notification

**缺失环节**：系统能看到当前页面，但无法形成有效建议。根因是缺少一个能支撑建议生成的记忆底座。

### 1.2 H1 目标

不是做完整长期记忆系统，而是先建立一套能支持"当前页面 → 相关历史 → 当前任务线程 → 候选建议"的本地记忆底座。

**核心验收逻辑**：当前页面出现时，系统能找到相关历史、当前任务线程和用户已决策内容，从而生成一条有依据的候选建议。

---

## 2. 架构决策

### 2.1 存储层

| 层级 | 技术 | 职责 |
|------|------|------|
| 事实日志 | Raw JSONL | append-only 原始记录保留 |
| 结构化存储 | SQLite | memory_events / context_slices / task_threads / decision_memory / skill_memory |
| 关键词检索 | FTS5 | 全文检索 |
| 语义召回 | sqlite-vec | 向量索引 |
| Embedding | 本地轻量 runtime (FastEmbed/ONNX) | Ollama 仅作实验/测试用 |

**不使用 Ollama 作为正式 embedding provider。**

### 2.2 crate 结构

```
cozmio/
├── cozmio_memory/           # 核心库（workspace member）
│   ├── src/lib.rs
│   ├── src/db.rs
│   ├── src/schema.rs
│   ├── src/embed_provider.rs
│   ├── src/embed_fastreembed.rs
│   ├── src/memory_events.rs
│   ├── src/context_slices.rs
│   ├── src/task_threads.rs
│   ├── src/decision_memory.rs
│   ├── src/skill_memory.rs
│   ├── src/search.rs
│   ├── src/competition.rs
│   ├── src/importer.rs
│   └── Cargo.toml
│
├── memory-cli/               # 调试/验收 CLI
│   ├── src/main.rs
│   └── Cargo.toml
│
└── src-tauri/                # Tauri 应用（调用 cozmio_memory）
    └── src/
        └── memory_commands.rs  # Tauri IPC（只调用 cozmio_memory）
```

### 2.3 EmbeddingProvider 抽象

```rust
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, MemoryError>;
    fn dimension(&self) -> usize;
    fn is_available(&self) -> bool;
}
```

实现优先级：
1. **FastEmbedProvider**（默认，本地，H1 正式方向）
2. **OllamaProvider**（仅实验/测试用）
3. **MockProvider**（测试用）
4. **DisabledProvider**（性能不足时降级，纯 FTS 检索）

向量维度：FastEmbed 默认 384 维（H1 采用）。

---

## 3. 数据库 Schema

### 3.1 memory_events

```sql
CREATE TABLE memory_events (
    id          INTEGER PRIMARY KEY,
    timestamp   DATETIME NOT NULL,
    source      TEXT NOT NULL,         -- action_log | toast | relay | claude_code
    window_title TEXT,
    content     TEXT NOT NULL,
    raw_ref     TEXT,
    embedding   BLOB,
    thread_id   INTEGER REFERENCES task_threads(id),
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE VIRTUAL TABLE memory_events_fts USING fts5(content, window_title, content=memory_events);
CREATE VIRTUAL TABLE memory_events_vec USING vec0(embedding[384]);
```

### 3.2 context_slices

```sql
CREATE TABLE context_slices (
    id              INTEGER PRIMARY KEY,
    start_time      DATETIME NOT NULL,
    end_time        DATETIME NOT NULL,
    summary         TEXT NOT NULL,
    entities        TEXT,               -- JSON array
    topics          TEXT,               -- JSON array
    raw_refs        TEXT,               -- JSON array of event IDs
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

**生成时机**：导入时批量生成（基于时间窗口 5-15 分钟），不实时生成。

### 3.3 task_threads

```sql
CREATE TABLE task_threads (
    id              INTEGER PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    current_state   TEXT,
    open_questions  TEXT,              -- JSON array
    decisions        TEXT,              -- JSON array
    recent_slice_ids TEXT,              -- JSON array
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 3.4 decision_memory

```sql
CREATE TABLE decision_memory (
    id              INTEGER PRIMARY KEY,
    memory_type     TEXT NOT NULL,     -- rejected_direction | accepted_decision | user_preference
    content         TEXT NOT NULL,
    evidence        TEXT,
    related_thread_id INTEGER REFERENCES task_threads(id),
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 3.5 skill_memory

```sql
CREATE TABLE skill_memory (
    id              INTEGER PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    procedure       TEXT NOT NULL,     -- 自然语言执行步骤
    success_context TEXT,
    usage_count     INTEGER DEFAULT 0,
    last_used_at    DATETIME,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## 4. 核心接口

### 4.1 Hybrid Search

```rust
pub struct SearchQuery {
    pub text: Option<String>,
    pub time_range: Option<(DateTime, DateTime)>,
    pub thread_id: Option<i64>,
    pub memory_types: Option<Vec<MemoryType>>,
    pub limit: usize,
}

pub struct SearchResults {
    pub events: Vec<MemoryEvent>,
    pub slices: Vec<ContextSlice>,
    pub decisions: Vec<Decision>,
    pub skills: Vec<Skill>,
    pub score_breakdown: ScoreBreakdown,
}
```

检索流程：FTS5 关键词 → sqlite-vec 语义 → metadata/time/thread 过滤 → 合并 Ranking。

### 4.2 Memory Competition

```rust
pub struct ActivityNote {
    pub window_title: String,
    pub content_text: String,
    pub timestamp: DateTime,
    pub current_thread_id: Option<i64>,
}

pub struct ReminderContext {
    pub current_activity: String,
    pub recent_context: String,
    pub related_decisions: String,
    pub relevant_skills: String,
    pub task_state: Option<String>,
    pub suggestion_candidates: Vec<String>,
}

impl MemoryCore {
    pub fn build_reminder_context(
        &self,
        activity_note: &ActivityNote,
    ) -> Result<ReminderContext, MemoryError>;
}
```

Competition 流程：
1. 从 FTS/vector 检索相关 events
2. 获取相关 context_slices
3. 获取相关 decision_memory
4. 获取相关 skill_memory
5. 获取关联 task_thread
6. 合并为 ReminderContext

---

## 5. 数据导入

### 5.1 action_log.jsonl 导入

从 `%LOCALAPPDATA%/cozmio/action_log.jsonl` 导入：

- timestamp, trace_id, judgment, window_title, next_step, level, confidence
- system_action, content_text, result_text, user_feedback

导入时：
1. 解析每行 ActionRecord
2. 插入 memory_events（source = "action_log"）
3. 生成 embedding（如果 provider 可用）
4. 批量提交

### 5.2 初始数据（H1 演示用）

手动创建以下 seed data：

**task_threads：**
- `Cozmio 使用体验改造` — 从 CONTINUE/ABSTAIN 弹窗升级为有效建议
- `Cozmio 硬件线 Local Agent Box` — 树莓派 + 端侧部署
- `Claude Code 执行链路` — Relay dispatch + subprocess 管理

**decision_memory：**
- 用户反对为了方便绕开向量检索、长期记忆等核心技术
- 用户不接受牺牲终局体验换取实现便利
- 用户希望技术方案从好用出发，不从省事出发

**skill_memory：**
- Toast → Relay → Claude Code 执行流程（已跑通）

---

## 6. CLI 命令

| 命令 | 作用 |
|------|------|
| `memory-cli stats` | 统计已有记录数量、时间范围、来源分布 |
| `memory-cli import` | 从 action_log.jsonl 导入 |
| `memory-cli search <query>` | 搜索记忆 |
| `memory-cli replay --since 2h` | 生成候选建议（离线，不弹窗） |
| `memory-cli inspect <slice_id>` | 查看 slice 详情 |
| `memory-cli rebuild-index` | 重建 FTS/vector 索引 |

---

## 7. Tauri IPC 命令

| 命令 | 作用 |
|------|------|
| `get_memory_stats` | 获取记忆库统计 |
| `import_existing_logs` | 触发日志导入 |
| `search_memory` | 搜索记忆 |
| `build_activity_context` | 从 ActivityNote 构建 ReminderContext |
| `run_suggestion_replay` | 生成候选建议 |
| `get_task_threads` | 获取任务线程列表 |
| `update_task_thread` | 创建/更新任务线程 |
| `get_decision_memory` | 获取决策记忆 |
| `add_decision` | 添加决策记忆 |
| `get_skill_memory` | 获取技能记忆 |

**所有 Tauri IPC 命令只调用 cozmio_memory，不实现记忆逻辑。**

---

## 8. src-tauri 集成点

### 8.1 ActivityNote 生成

在 main_loop.rs 中，每次捕获窗口后，生成 ActivityNote：

```rust
struct ActivityNote {
    window_title: String,
    content_text: String,     // 来自 Ollama judgment 或 window info
    timestamp: DateTime,
    current_thread_id: Option<i64>,
}
```

### 8.2 ReminderContext 获取

```rust
// main_loop.rs
let context = memory_core.build_reminder_context(&activity_note)?;
// 将 context 传入后续建议生成流程
```

### 8.3 不修改现有 judgment 链路

Memory Core H1 **不修改**现有 Ollama judgment 链路。ActivityNote 的 content_text 初始版本可来自 window_title + 已有 judgment 结果（如果可用），后续随 Memory Core 成熟再升级。

---

## 9. 验收标准

| # | 标准 | 验证方式 |
|---|------|----------|
| 1 | 能从 action_log.jsonl 导入真实数据 | `memory-cli import` + `memory-cli stats` |
| 2 | 能统计已积累事件数量 | `memory-cli stats` 输出 |
| 3 | 搜索关键词返回真实历史 | `memory-cli search "Toast 无效"` |
| 4 | 生成最近 2 小时 context_slices | `memory-cli replay --since 2h` |
| 5 | 创建/更新 task_thread | `get_task_threads` IPC |
| 6 | decision_memory 保存否定/接受方向 | `add_decision` IPC |
| 7 | skill_memory 保存至少一条可复用经验 | seed data |
| 8 | Memory Competition 返回压缩上下文 | `build_activity_context` IPC |
| 9 | src-tauri IPC 能调用 cozmio_memory | Tauri 命令响应验证 |
| 10 | Reminder Context 能进入提醒链路 | 日志/输出验证 |

---

## 10. H1 不做的事项（明确边界）

- 不做多设备同步
- 不做 Skill Memory 的自动沉淀（只支持手动添加）
- 不做 Memory Competition 的自动排序算法（先 hardcoded merge，后续迭代）
- 不做完整的 embedding model fine-tuning
- 不做实时弹窗（replay 验证质量后才考虑）

---

## 11. 技术风险

| 风险 | 缓解 |
|------|------|
| FastEmbed 在 Windows 环境编译困难 | 预先验证，或降级到 Mock/Disabled |
| sqlite-vec 与 SQLite 版本兼容 | 使用 bundled SQLite，验证版本 |
| 向量检索性能不足 | FTS5 作为 primary，vector 作为 secondary |
| 现有 action_log.jsonl 字段不全 | importer 做好字段兼容，缺失字段不报错 |

---

## 12. 文件位置

- 设计文档：`docs/superpowers/specs/2026-04-26-suggestion-memory-core-h1-design.md`
- 实施方案：`docs/superpowers/plans/2026-04-26-suggestion-memory-core-h1-plan.md`
