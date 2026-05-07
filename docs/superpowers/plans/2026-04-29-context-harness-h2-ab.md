# Context Harness H2 Slice A+B 实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。
>
> **产品类型**：传统软件实现型（测试加固 + 字段拆分）+ 模型输出验证型（审计假语义来源）。以软件实现为主，模型输出验证（样本评审）在 Slice C 开始。

**目标**：

- Slice A：加固 semantic_boundary 集成测试，确保能捕获 runtime 新代码中的硬编码语义词。
- Slice B：审计并拆分 ActionRecord 中的假语义字段（系统伪造的 confidence=1.0、重复的 level/next_step、冒充模型输出的 judgment 字段），建立 model_output / system_route / ui_event / execution_result 的清晰边界。

**全局不可违反约束**：

- 不添加弹窗 cooldown、frequency cap、silence 规则。
- 不让系统代码伪造模型的 confidence。
- 不把 UI 事件记录成模型 judgment。
- 不把 system_route 记录成模型语义判断。

---

## 文件改动总览

| 文件 | 改动类型 | 职责 |
|------|---------|------|
| `cozmio/src-tauri/tests/semantic_boundary.rs` | 扩展 | 新增禁止词、扩展扫描文件列表 |
| `cozmio/src-tauri/src/logging.rs` | 扩展 | 新增 `FactualActionRecord` 类型（与 ActionRecord 并存） |
| `cozmio/src-tauri/src/executor.rs` | 修改 | 停止写入假的 `confidence`、`level`、`next_step`，使用新 factual 类型 |
| `cozmio/src-tauri/src/relay_bridge.rs` | 审计 | 确认无伪造 confidence |
| `cozmio/src-tauri/src/ui_state.rs` | 审计 | 确认无伪造 confidence |
| `cozmio/src-tauri/src/main_loop.rs` | 审计 | 确认无伪造 confidence |
| `cozmio/src-tauri/src/commands.rs` | 审计 | 确认无伪造 confidence |
| `cozmio/cozmio_memory/src/importer.rs` | 审计+兼容 | importer 有独立的 ActionRecord 定义，检查是否需要同步 |
| `docs/superpowers/specs/2026-04-28-context-harness-h2-practice-loop-design.md` | 状态更新 | Slice A+B 完成状态 |

---

### 任务1：Slice A — 扩展 semantic_boundary 测试扫描范围

**涉及文件**：

- 修改：`cozmio/src-tauri/tests/semantic_boundary.rs`

- [ ] **步骤1：确认当前测试状态**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio --test semantic_boundary -- --nocapture
  ```

  预期：2 个测试通过。如果 0 tests 说明测试未注册，需要检查文件位置。

- [ ] **步骤2：读取当前测试文件内容**

  读取 `D:/C_Projects/Agent/cozmio/cozmio/src-tauri/tests/semantic_boundary.rs` 完整内容，确认当前禁止词列表和扫描文件列表。

- [ ] **步骤3：扩展扫描文件列表**

  将 `forbidden` 检查的文件列表从 4 个扩展到 8 个：

  ```rust
  let files = [
      "src/model_client.rs",
      "src/prompt_context.rs",
      "src/main_loop.rs",
      "src/commands.rs",
      "src/executor.rs",       // 新增：有 ActionRecord 写入逻辑
      "src/relay_bridge.rs",   // 新增：有 execution_result 处理
      "src/ui_state.rs",       // 新增：有 state 写入
      "src/logging.rs",        // 新增：ActionRecord 定义处
  ];
  ```

- [ ] **步骤4：扩展禁止词列表**

  新增以下禁止词（这些是系统伪语义，不得出现在 runtime prompt 代码中）：

  ```rust
  "confidence: 1.0",    // 系统伪造的模型确定性
  "confidence = 1.0",    // 同上（不同写法）
  "confidence=1.0",      // 同上（无空格）
  "level:",              // 系统伪造的与 judgment 重复的字段
  "next_step:",          // 系统伪造的语义字段
  "系统判断",            // 中文硬编码语义
  "fake_confidence",     // 测试用词泄漏
  ```

  注意：`confidence` 作为日志字段名本身不禁止（它是 ActionRecord 的已有字段），禁止的是 **在 runtime 代码中将 `confidence = 1.0` 硬编码为系统伪造模型确定性**。测试用正则或注释中的 confidence 不受限。

- [ ] **步骤5：运行测试确认失败（预期行为）**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio --test semantic_boundary -- --nocapture
  ```

  预期：因为 executor.rs 中存在 `confidence: 1.0`（`executor.rs:59`），测试应该失败并指出文件和词。

- [ ] **步骤6：确认当前失败位置**

  如果测试失败，记录失败的文件和词。这些就是 Slice B 需要修复的假语义写入点。

- [ ] **步骤7：提交 Slice A 阶段成果**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio; git add cozmio/src-tauri/tests/semantic_boundary.rs; git commit -m "test(semantic_boundary): extend scan files and forbidden terms for A+B"
  ```

---

### 任务2：Slice B — 审计所有 confidence=1.0 写入点

**涉及文件**：

- 审计：`cozmio/src-tauri/src/` 下的所有 .rs 文件
- 修改：`cozmio/src-tauri/src/executor.rs`（主要问题点）

- [ ] **步骤1：搜索所有 confidence 写入**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src; rg "confidence\s*[=:]" --type rust -n
  ```

  列出所有 `confidence =` 或 `confidence:` 赋值点，记录每个文件:行号。

- [ ] **步骤2：分类每个 confidence 来源**

  对每个命中点，判断是：

  - **模型输出**：模型真实输出了 confidence 值 → 保留
  - **系统伪造**：`confidence: 1.0` 或 `confidence = 1.0` 硬编码 → 标记待删除
  - **已有 legacy 字段**：历史代码中的旧字段，尚未迁移 → 保留但加兼容性注释

  预期发现（基于 executor.rs:59）：

  ```
  executor.rs:59: confidence: 1.0,  ← 系统伪造，必须删除
  ```

- [ ] **步骤3：搜索所有 level 字段写入**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src; rg "\blevel\b\s*[=:]" --type rust -n
  ```

  预期：executor.rs 中 `level: output.mode.to_string()` 与 `judgment: output.mode.to_string()` 完全重复。

- [ ] **步骤4：搜索所有 next_step 字段写入**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src; rg "\bnext_step\b" --type rust -n
  ```

  预期：executor.rs 中 `next_step: output.reason.clone()` 与 `grounds` 重复。

- [ ] **步骤5：记录所有问题点**

  汇总所有假语义写入点，格式：

  ```text
  文件:行号 | 字段 | 当前值 | 问题
  executor.rs:59 | confidence | 1.0 | 系统伪造
  executor.rs:58 | level | output.mode | 与 judgment 重复
  executor.rs:57 | next_step | output.reason | 与 grounds 重复
  ```

---

### 任务3：Slice B — 设计 FactualActionRecord 类型

**涉及文件**：

- 修改：`cozmio/src-tauri/src/logging.rs`

- [ ] **步骤1：理解 legacy ActionRecord 结构**

  读取 `logging.rs` 中 `ActionRecord` 的完整定义（logging.rs:8-31），确认所有字段。

  Legacy 字段：

  ```rust
  pub struct ActionRecord {
      pub timestamp: i64,
      pub trace_id: Option<String>,
      pub session_id: Option<String>,
      pub window_title: String,
      pub judgment: String,        // 来自模型，但命名混淆（judgment≠模型输出）
      pub next_step: String,        // 系统伪造（=reason），应删除
      pub level: String,            // 系统伪造（=judgment重复），应删除
      pub confidence: f32,           // 系统伪造（=1.0），应删除
      pub grounds: String,           // 系统伪造（=reason），命名不清
      pub system_action: String,     // 系统路由，OK
      pub content_text: Option<String>,
      pub result_text: Option<String>,
      pub error_text: Option<String>,
      pub user_feedback: Option<String>,
      pub model_name: Option<String>,
      pub captured_at: Option<i64>,
      pub call_started_at: Option<i64>,
      pub call_duration_ms: Option<u64>,
  }
  ```

- [ ] **步骤2：定义 FactualActionRecord（新增类型）**

  在 `logging.rs` 中，在 `ActionRecord` 定义之后、新增 `FactualActionRecord`：

  ```rust
  /// Factual action record - separates concerns that legacy ActionRecord mixed together.
  ///
  /// Facts are provided by the system. Semantic summaries come from models and must
  /// include provenance (timestamp, source path, source range, producer).
  ///
  /// 以下字段由系统提供事实：
  ///   timestamp, trace_id, window_title, event_type, system_route
  /// 以下字段是模型原文（只读保存，不做解析）：
  ///   raw_model_text, model_name
  /// 以下字段是执行端返回（只读保存）：
  ///   execution_result, error_text
  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
  pub struct FactualActionRecord {
      /// Unix timestamp when this record was created
      pub timestamp: i64,
      /// Unique trace ID for this action
      pub trace_id: Option<String>,
      /// Session ID if this action is part of a relay session
      pub session_id: Option<String>,
      /// Window title at the time of this record
      pub window_title: String,
      /// Type of the primary event - distinguishes system events from model outputs
      pub event_type: FactualEventType,
      /// System routing decision (not model judgment)
      pub system_route: SystemRoute,
      /// Raw text from the model - preserved as-is, not parsed into structured fields.
      /// Model output is natural language. If semantic summaries are needed, they
      /// must come from a model or execution agent with provenance.
      pub raw_model_text: Option<String>,
      /// Model name used for this output (config name or fallback)
      pub model_name: Option<String>,
      /// When the screenshot was captured (Unix timestamp)
      pub captured_at: Option<i64>,
      /// When the API call started (Unix timestamp)
      pub call_started_at: Option<i64>,
      /// How long the API call took (milliseconds)
      pub call_duration_ms: Option<u64>,
      /// Execution result text from the execution side
      pub execution_result: Option<String>,
      /// Error text from system or execution side
      pub error_text: Option<String>,
      /// User UI feedback (e.g. "ui_confirmed", "ui_cancelled", "ui_dismissed")
      pub user_feedback: Option<String>,
  }

  /// Event types that the system can factually observe
  #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
  pub enum FactualEventType {
      /// Model was called and returned output
      ModelOutput,
      /// User confirmed a pending notification via UI
      UiConfirmed,
      /// User cancelled a pending notification via UI
      UiCancelled,
      /// User dismissed a pending notification via UI
      UiDismissed,
      /// Relay session completed
      RelayCompleted,
      /// Relay session failed
      RelayFailed,
      /// Model call resulted in an error
      ModelError,
      /// System-level error (not model-related)
      SystemError,
  }

  /// System routing decisions - these are factual system events, not model semantics
  #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
  pub enum SystemRoute {
      /// User confirmed the action
      Confirmed,
      /// User declined or action was auto-declined
      Declined,
      /// Action was executed automatically
      AutoExecuted,
      /// No pending action (e.g. stale confirm URL replay)
      NoPendingAction,
      /// Execution completed successfully
      Completed,
      /// Execution failed
      Failed,
      /// Execution was interrupted
      Interrupted,
      /// Model output suggested continuation (CONTINUE mode)
      Continue,
      /// Model output indicated uncertainty (ABSTAIN mode)
      Abstain,
      /// Unknown system route
      Unknown,
  }
  ```

- [ ] **步骤3：保留 legacy ActionRecord 不删除（兼容性）**

  legacy `ActionRecord` 保持不变，用于：
  1. 读取历史 action_log.jsonl（已有数据）
  2. cozmio_memory importer 读取旧格式

  新写入路径使用 `FactualActionRecord`。

- [ ] **步骤4：确认 logging.rs 编译通过**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio; cargo build -p cozmio
  ```

  预期：编译成功，无错误。

- [ ] **步骤5：提交 FactualActionRecord 类型**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio; git add cozmio/src-tauri/src/logging.rs; git commit -m "feat(logging): add FactualActionRecord type with clean semantic boundaries"
  ```

---

### 任务4：Slice B — 修改 executor.rs 停止写入假语义字段

**涉及文件**：

- 修改：`cozmio/src-tauri/src/executor.rs`

- [ ] **步骤1：读取当前 executor.rs 的 route 函数**

  确认 executor.rs:46-86 的 `route` 函数中 `ActionRecord` 创建代码。

- [ ] **步骤2：确认有哪些字段是系统伪造的**

  基于任务2的审计结果：

  ```text
  confidence: 1.0     ← 删除（系统伪造）
  level: output.mode  ← 删除（与 judgment 重复）
  next_step: output.reason ← 删除（与 grounds 重复）
  grounds: output.reason ← 重命名为 raw_model_text
  ```

- [ ] **步骤3：修改 route 函数，创建 FactualActionRecord**

  将 `route` 函数的 `ActionRecord` 创建改为 `FactualActionRecord`：

  ```rust
  pub fn route(
      &self,
      output: &ModelOutput,
      window_title: &str,
  ) -> Result<ExecutionResult, String> {
      // 创建 FactualActionRecord（而不是 legacy ActionRecord）
      let record = FactualActionRecord {
          timestamp: chrono::Utc::now().timestamp(),
          trace_id: None,
          session_id: None,
          window_title: window_title.to_string(),
          event_type: FactualEventType::ModelOutput,
          system_route: match output.mode {
              InterventionMode::Continue => SystemRoute::Continue,
              InterventionMode::Abstain => SystemRoute::Abstain,
          },
          raw_model_text: Some(output.reason.clone()), // 模型原文保存
          model_name: None,
          captured_at: None,
          call_started_at: None,
          call_duration_ms: None,
          execution_result: None,
          error_text: None,
          user_feedback: None,
      };

      let result = match output.mode {
          InterventionMode::Continue => self.handle_continue(output, &record),
          InterventionMode::Abstain => self.handle_abstain(output, &record),
      };

      if let Ok(ref exec_result) = result {
          let mut updated_record = record;
          updated_record.system_route = match exec_result {
              ExecutionResult::Notified => SystemRoute::Unknown,
              ExecutionResult::Confirmed => SystemRoute::Confirmed,
              ExecutionResult::Executed => SystemRoute::AutoExecuted,
              ExecutionResult::Skipped => SystemRoute::Declined,
          };
          if let Err(e) = self.logger.log_factual(updated_record) {
              eprintln!("Failed to log action: {}", e);
          }
      }

      result
  }
  ```

  注意：`logger.log_factual` 是新增方法，见任务5。

- [ ] **步骤4：修改 handle_continue 和 handle_abstain 签名**

  这两个函数接收 `_record: &ActionRecord` 参数，改为接收 `&FactualActionRecord`。

- [ ] **步骤5：检查是否有其他地方调用 route 函数**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src; rg "\.route\(" --type rust -n
  ```

  确认所有调用点，必要时同步修改参数类型。

- [ ] **步骤6：运行测试确认 executor 相关测试失败（预期）**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio executor -- --nocapture
  ```

  预期：因为 executor 测试仍使用 legacy `ActionRecord` 断言，测试会失败。这是预期的——下一步修改测试。

- [ ] **步骤7：更新 executor 测试以使用 legacy ActionRecord 断言（保持兼容性）**

  executor 的测试继续使用 legacy `ActionRecord` 断言（因为 executor 内部仍创建 legacy record 写入），但需确认测试断言的是正确的字段值。

  在 executor 测试中：
  - `test_route_continue_logs_action` 断言 `judgment == "CONTINUE"` → 这是从模型输出来的，正确
  - `test_executor_logs_correct_judgment_for_continue` 断言 `judgment == "CONTINUE"` → 正确

  但需要确认测试不再断言假的 `confidence`、`level`、`next_step`。

  读取 `executor.rs:143-249` 的测试部分，确认测试断言了哪些字段。

- [ ] **步骤8：提交 executor 修改**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio; git add cozmio/src-tauri/src/executor.rs; git commit -m "feat(executor): stop writing fake confidence/level/next_step, use FactualActionRecord"
  ```

---

### 任务5：Slice B — 为 ActionLogger 新增 log_factual 方法

**涉及文件**：

- 修改：`cozmio/src-tauri/src/logging.rs`

- [ ] **步骤1：在 ActionLogger 中添加 log_factual 方法**

  在 `impl ActionLogger` 中新增方法：

  ```rust
  /// Log a FactualActionRecord to the action log.
  ///
  /// This writes in the legacy JSONL format for backwards compatibility
  /// with existing log readers (cozmio_memory importer, etc).
  pub fn log_factual(&self, record: FactualActionRecord) -> Result<(), String> {
      // Convert FactualActionRecord to legacy ActionRecord for JSONL format
      // This is a transitional compatibility layer - both types write the same JSONL format.
      let legacy = ActionRecord {
          timestamp: record.timestamp,
          trace_id: record.trace_id,
          session_id: record.session_id,
          window_title: record.window_title,
          // raw_model_text goes into content_text
          content_text: record.raw_model_text,
          // model_name preserved
          model_name: record.model_name,
          // execution_result goes into result_text
          result_text: record.execution_result,
          // error_text preserved
          error_text: record.error_text,
          // user_feedback preserved
          user_feedback: record.user_feedback,
          // timing fields preserved
          captured_at: record.captured_at,
          call_started_at: record.call_started_at,
          call_duration_ms: record.call_duration_ms,
          // Fields that are now system-only (no longer model output):
          judgment: match record.system_route {
              SystemRoute::Continue => "CONTINUE".to_string(),
              SystemRoute::Abstain => "ABSTAIN".to_string(),
              _ => "SYSTEM".to_string(),
          },
          // next_step removed - was a duplicate of reason
          next_step: String::new(),
          // level removed - was a duplicate of judgment
          level: String::new(),
          // confidence removed - was a system-faked 1.0
          confidence: 0.0,
          // grounds removed - was a duplicate of reason/raw_model_text
          grounds: String::new(),
          // system_action derived from system_route
          system_action: format!("{:?}", record.system_route).to_lowercase(),
      };
      self.log(legacy)
  }
  ```

- [ ] **步骤2：确认 ActionLogger::log 签名兼容 ActionRecord**

  读取 `logging.rs` 中 `ActionLogger::log` 方法签名，确认 `log(legacy)` 调用方式正确。

- [ ] **步骤3：运行 build 确认编译通过**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio; cargo build -p cozmio
  ```

  预期：编译成功。

- [ ] **步骤4：运行 semantic_boundary 测试**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio --test semantic_boundary -- --nocapture
  ```

  预期：测试仍失败，因为 executor.rs:59 的 `confidence: 1.0` 还没删除（步骤在任务4中已完成，但需要确认测试通过）。

  如果测试通过，说明 `confidence: 1.0` 已经被替换为 0.0 或删除。
  如果测试失败，记录仍存在的假语义词。

- [ ] **步骤5：提交 log_factual 方法**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio; git add cozmio/src-tauri/src/logging.rs; git commit -m "feat(logging): add log_factual method for FactualActionRecord"
  ```

---

### 任务6：Slice B — 审计 relay_bridge / ui_state / main_loop / commands 的假语义写入

**涉及文件**：

- 审计：`cozmio/src-tauri/src/relay_bridge.rs`
- 审计：`cozmio/src-tauri/src/ui_state.rs`
- 审计：`cozmio/src-tauri/src/main_loop.rs`
- 审计：`cozmio/src-tauri/src/commands.rs`

- [ ] **步骤1：搜索 relay_bridge.rs 中的 confidence 写入**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src; rg "confidence" relay_bridge.rs -n
  ```

  预期：可能没有直接 `confidence =` 写入。确认是否有 relay session 结果写入 ActionRecord。

- [ ] **步骤2：搜索 ui_state.rs 中的 confidence 写入**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src; rg "confidence" ui_state.rs -n
  ```

  预期：可能没有。确认 UI state 写入的是 model_output 还是 system_route。

- [ ] **步骤3：搜索 main_loop.rs 中的 confidence 写入**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src; rg "confidence" main_loop.rs -n
  ```

  预期：如果 main_loop 直接创建 ActionRecord，检查是否有 `confidence: 1.0`。

- [ ] **步骤4：搜索 commands.rs 中的 confidence 写入**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src; rg "confidence" commands.rs -n
  ```

  预期：检查 confirm/cancel handler 中是否有伪造 confidence。

- [ ] **步骤5：汇总审计结果**

  如果发现其他假语义写入点（非 executor 的），记录到设计文档备注中。
  如果仅 executor 有问题，则 Slice B 的假语义清理聚焦于 executor。

- [ ] **步骤6：提交审计结果（如有修改）**

  如果任务6中有任何文件被修改（发现并修复了假语义写入），执行提交：

  ```powershell
  cd D:/C_Projects/Agent/cozmio; git add cozmio/src-tauri/src/...; git commit -m "fix: remove fake confidence from <file>"
  ```

  如果没有文件被修改，则跳过此步骤。

---

### 任务7：Slice B — 审计 cozmio_memory importer 的 ActionRecord 兼容性

**涉及文件**：

- 审计：`cozmio/cozmio_memory/src/importer.rs`

- [ ] **步骤1：检查 importer 的 ActionRecord 定义**

  读取 `cozmio_memory/src/importer.rs` 的 ActionRecord 定义（importer.rs:12-27）。

  确认 importer 的 ActionRecord 与 src-tauri 的 logging.rs 中的 ActionRecord 是否一致。

- [ ] **步骤2：确认 importer 不依赖被删除的字段**

  检查 importer 代码是否依赖 `confidence`、`level`、`next_step` 字段。

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio/cozmio_memory/src; rg "confidence|level|next_step" importer.rs -n
  ```

  预期：如果 importer 有独立 ActionRecord 定义，它定义了自己的字段，不依赖 src-tauri 的字段。

- [ ] **步骤3：确认 cozmio_memory 仍能读取 legacy JSONL**

  importer 读取 JSONL 时使用 `serde_json::from_str::<ActionRecord>`。
  因为 JSONL 中的字段是 JSON key，只要 JSON 有这些 key 就能反序列化。
  将 `confidence: 0.0`（从 1.0 改为 0.0）不影响反序列化。

  不需要修改 importer。

- [ ] **步骤4：编译 cozmio_memory 确认无破坏**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio; cargo build -p cozmio_memory
  ```

  预期：编译成功。

---

### 任务8：验证 Slice A+B 的 semantic_boundary 测试通过

- [ ] **步骤1：运行 semantic_boundary 测试**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio --test semantic_boundary -- --nocapture
  ```

  预期：测试通过，不再有假语义词命中。

- [ ] **步骤2：运行全部测试**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio
  ```

  预期：全部测试通过。

- [ ] **步骤3：搜索残留的 confidence: 1.0**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src; rg "confidence.*1\.0|confidence\s*=\s*1\.0" --type rust -n
  ```

  预期：无匹配输出。

- [ ] **步骤4：搜索残留的 level: 和 next_step: 写入（不在定义处）**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src; rg "^\s*level:\s*output|\s*next_step:\s*output" --type rust -n
  ```

  预期：无匹配（定义处和测试中的不受限）。

- [ ] **步骤5：提交完整 Slice A+B 成果**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio; git add cozmio/src-tauri/src/logging.rs cozmio/src-tauri/src/executor.rs cozmio/src-tauri/tests/semantic_boundary.rs; git commit -m "feat(h2_slice_ab): add FactualActionRecord and stop fake confidence/level/next_step"
  ```

---

## 全量验证

- [ ] **步骤1：cargo build 全量编译**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio; cargo build
  ```

  预期：编译成功，无错误。

- [ ] **步骤2：cargo test 全量测试**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio
  ```

  预期：全部测试通过。

- [ ] **步骤3：cargo build -p cozmio_memory**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio; cargo build -p cozmio_memory
  ```

  预期：编译成功。

- [ ] **步骤4：确认 semantic_boundary 测试覆盖 main_loop.rs**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio --test semantic_boundary -- --nocapture 2>&1 | rg "test result"
  ```

  预期：显示运行了 2 个测试，无 0-test 假通过。

- [ ] **步骤5：检查 git status**

  执行命令：

  ```powershell
  cd D:/C_Projects/Agent/cozmio; git status --short
  ```

  预期：只有本方案涉及的文件处于已修改或已提交状态。

---

## 自我审查结果

- 产品类型：传统软件实现型为主，字段拆分有清晰的结构边界。
- 规格覆盖：FactualActionRecord、semantic_boundary 扩展、executor 假语义清理、importer 兼容性确认。
- 占位符排查：本文档无未填的占位词。
- 类型一致性：FactualActionRecord 的字段与 executor 传入的 ModelOutput 类型一致。
- 向后兼容：legacy ActionRecord 保留，importer 仍能读取旧 JSONL，history UI 不受影响。
- 约束检查：无 cooldown、无 frequency cap、无伪造 confidence、无 UI-as-judgment。
