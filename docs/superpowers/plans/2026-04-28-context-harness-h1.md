# Context Harness H1 实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标**：把 Cozmio 的弹窗上下文升级为事实底座，让本地模型获得更稳定的事实材料，同时禁止系统代码替模型生成用户意图、任务阶段、弹窗策略或沉默规则。

**架构思路**：本阶段是“传统软件实现型 + 模型输出验证型”的混合任务。软件侧只负责事实采集、事实裁剪、日志来源标注和测试 gate；语义总结、长期记忆、项目阶段判断必须来自模型或执行端 Agent，并带 timestamp/source/provenance。弹窗仍由模型主导，系统不得增加 cooldown、frequency cap、`popup|silence` 决策字段或硬编码沉默规则。

**技术栈**：Rust 2021、Tauri 2、Cargo workspace、Ollama HTTP API、JSONL action log、PowerShell 验证命令。

---

## 产品类型识别

本方案属于混合型：

- 软件实现型部分：事实上下文构建、日志读取、UI 事件降级、语义边界测试。
- 模型输出验证型部分：本地模型输出质量评估，验收必须检查输出内容，不允许只检查命令是否成功。

执行顺序固定：先实现事实底座和边界测试，再进行模型输出质量评估。

## 全局不可违反约束

- 不添加弹窗 cooldown。
- 不添加弹窗 frequency cap。
- 不添加“快速切换窗口所以不弹”的规则。
- 不添加 `decision: popup | silence`、`should_popup`、`should_silence` 这类模型输出字段。
- 不添加 `stuck`、`working`、`project_phase`、`iteration_opportunity`、`user_intent` 这类系统生成语义字段。
- 不让系统代码把路径、窗口标题、进程名推断成“用户正在做某项目”。
- 不把 Claude Code、relay、subagent 大日志直接塞给端侧模型。
- 任何语义总结必须是模型或执行端 Agent 输出，并保存 timestamp、source、provenance。

## 文件结构

- 修改：`D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/prompt_context.rs`
  - 职责：构建端侧模型可见的事实上下文包。
- 修改：`D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/model_client.rs`
  - 职责：构建视觉模型 prompt，明确系统材料只是事实。
- 修改：`D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/main_loop.rs`
  - 职责：把事实上下文接入模型调用，并记录 raw output 的事实痕迹。
- 修改：`D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/commands.rs`
  - 职责：把用户 UI 操作记录为事实事件，不伪装成模型判断。
- 修改：`D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/logging.rs`
  - 职责：提供 action log tail 读取能力，避免热路径扫描整天日志。
- 创建：`D:/C_Projects/Agent/cozmio/cozmio/src-tauri/tests/semantic_boundary.rs`
  - 职责：用集成测试阻止运行时代码重新引入硬编码语义和机械弹窗限制。
- 创建：`D:/C_Projects/Agent/cozmio/verification/context_harness_h1/review_template.md`
  - 职责：人工评审本地模型输出质量，关注有用性和是否幻觉。
- 创建：`D:/C_Projects/Agent/cozmio/verification/context_harness_h1/runbook.md`
  - 职责：记录本地模型实验命令、样本采集方式、失败停止规则。
- 修改：`D:/C_Projects/Agent/cozmio/AGENTS.md`
  - 职责：沉淀语义边界，作为之后 Codex/Agent 默认约束。

---

### 任务1：建立语义边界集成测试

**涉及文件**：

- 创建：`D:/C_Projects/Agent/cozmio/cozmio/src-tauri/tests/semantic_boundary.rs`

- [ ] **步骤1：编写失败的语义边界测试**

创建文件 `D:/C_Projects/Agent/cozmio/cozmio/src-tauri/tests/semantic_boundary.rs`，完整内容如下：

```rust
use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_runtime_file(relative: &str) -> String {
    let path = crate_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    })
}

fn runtime_region(content: &str) -> &str {
    content
        .split("#[cfg(test)]")
        .next()
        .unwrap_or(content)
}

#[test]
fn runtime_prompt_does_not_hardcode_model_silence_or_popup_policy() {
    let files = [
        "src/model_client.rs",
        "src/prompt_context.rs",
        "src/main_loop.rs",
        "src/commands.rs",
    ];
    let forbidden = [
        "保持沉默",
        "弹窗策略",
        "检索线索",
        "project_phase",
        "iteration_opportunity",
        "should_popup",
        "should_silence",
        "popup|silence",
        "popup | silence",
        "frequency cap",
        "cooldown",
    ];

    for file in files {
        let content = read_runtime_file(file);
        let runtime_content = runtime_region(&content);
        for term in forbidden {
            assert!(
                !runtime_content.contains(term),
                "{file} contains forbidden system semantic or popup-control term: {term}"
            );
        }
    }
}

#[test]
fn runtime_prompt_names_system_material_as_facts_not_conclusions() {
    let content = read_runtime_file("src/model_client.rs");

    assert!(content.contains("Cozmio 只提供事实材料和工具材料，不提供结论"));
    assert!(content.contains("不要把它们当成用户意图、任务阶段或项目结论"));
    assert!(content.contains("不要为了迎合上下文而编造屏幕上或材料中没有出现的内容"));
}
```

- [ ] **步骤2：运行测试确认当前风险暴露**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio --test semantic_boundary -- --nocapture
```

预期结果：输出包含 `Running tests\semantic_boundary.rs`，并执行 2 个测试。如果运行时代码仍含 forbidden 词，测试失败并指出文件名和词；如果当前代码已被清理，测试通过。不得使用 `cargo test -p cozmio semantic_boundary` 代替本命令，因为那会把 `semantic_boundary` 当作测试名过滤器并可能得到 0 个测试的假通过。

- [ ] **步骤3：提交测试文件**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; git add cozmio/src-tauri/tests/semantic_boundary.rs; git commit -m "test: guard semantic boundary in runtime prompt code"
```

预期结果：生成一个只包含 `semantic_boundary.rs` 的提交。

---

### 任务2：把 prompt context 固定为事实上下文包

**涉及文件**：

- 修改：`D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/prompt_context.rs`

- [ ] **步骤1：确认测试覆盖事实字段**

在 `D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/prompt_context.rs` 的 `includes_process_and_recent_feedback` 测试中，保证断言包含以下内容：

```rust
assert!(context.contains("stay_duration_seconds=42"));
assert!(context.contains("feedback=\"ui_closed\""));
assert!(context.contains("timestamp=1000"));
assert!(!context.contains("检索线索"));
assert!(!context.contains("弹窗策略"));
```

- [ ] **步骤2：运行单测确认失败或通过**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio prompt_context -- --nocapture
```

预期结果：如果当前实现没有 `prompt_context` 或仍输出语义提示，测试失败；如果已经是事实上下文包，测试通过。

- [ ] **步骤3：实现事实上下文包**

确保 `D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/prompt_context.rs` 的核心实现满足以下形状。已有同名函数时保留现有测试辅助函数，只替换事实构建逻辑：

```rust
use crate::logging::{ActionLogger, ActionRecord};
use crate::window_monitor::{ProcessContext, SwitchDirection};

const MAX_RECENT_RECORDS: usize = 18;
const MAX_INCLUDED_RECORDS: usize = 6;
const MAX_CONTEXT_CHARS: usize = 1400;
const MAX_FIELD_CHARS: usize = 180;
const ACTION_LOG_TAIL_BYTES: u64 = 64 * 1024;

pub fn build_popup_context(
    logger: &ActionLogger,
    window_title: &str,
    process_name: &str,
    process_context: &ProcessContext,
) -> String {
    let recent = logger
        .get_recent_tail(MAX_RECENT_RECORDS, ACTION_LOG_TAIL_BYTES)
        .unwrap_or_else(|error| {
            log::warn!("Failed to load recent action history for popup context: {error}");
            Vec::new()
        });

    let mut lines = vec![
        format_process_context(process_context),
        format!(
            "current_window: title=\"{}\", process=\"{}\"",
            clip(window_title, 120),
            clip(process_name, 80)
        ),
    ];

    let records: Vec<&ActionRecord> = recent.iter().take(MAX_INCLUDED_RECORDS).collect();
    if !records.is_empty() {
        lines.push(String::from("action_log_tail:"));
        for record in records {
            lines.push(format!("- {}", compact_record(record)));
        }
    }

    clip(&lines.join("\n"), MAX_CONTEXT_CHARS)
}

fn format_process_context(context: &ProcessContext) -> String {
    format!(
        "process_context: stay_duration_seconds={}, switches_last_minute={}, is_oscillating={}, last_switch_direction={}, just_arrived={}",
        context.stay_duration_seconds,
        context.switches_in_last_minute,
        context.is_oscillating,
        switch_direction_label(context.last_switch_direction),
        context.just_arrived
    )
}

fn switch_direction_label(direction: SwitchDirection) -> &'static str {
    match direction {
        SwitchDirection::Arrived => "arrived",
        SwitchDirection::Left => "left",
        SwitchDirection::None => "none",
    }
}
```

- [ ] **步骤4：确认没有系统语义筛选函数**

在同一文件中删除以下函数或同等职责代码：

```rust
fn workspace_hint(...) -> ...
fn has_workspace_hint(...) -> ...
fn is_relevant_record(...) -> ...
fn is_same_window_family(...) -> ...
```

验收方式：执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; rg "workspace_hint|has_workspace_hint|is_relevant_record|is_same_window_family|检索线索|弹窗策略" cozmio/src-tauri/src/prompt_context.rs
```

预期结果：命令无匹配输出。

- [ ] **步骤5：运行测试确认通过**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio prompt_context semantic_boundary -- --nocapture
```

预期结果：`prompt_context` 和 `semantic_boundary` 相关测试全部通过。

- [ ] **步骤6：提交事实上下文包**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; git add cozmio/src-tauri/src/prompt_context.rs cozmio/src-tauri/tests/semantic_boundary.rs; git commit -m "feat: build factual popup context harness"
```

预期结果：提交包含 `prompt_context.rs` 和必要测试更新。

---

### 任务3：把模型 prompt 改成事实输入说明，不控制模型输出

**涉及文件**：

- 修改：`D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/model_client.rs`

- [ ] **步骤1：更新 prompt 测试断言**

在 `test_build_prompt` 中添加以下断言：

```rust
assert!(prompt.contains("Cozmio 只提供事实材料和工具材料，不提供结论"));
assert!(prompt.contains("不要把它们当成用户意图、任务阶段或项目结论"));
assert!(prompt.contains("不要为了迎合上下文而编造屏幕上或材料中没有出现的内容"));
assert!(!prompt.contains("保持沉默"));
assert!(!prompt.contains("明显推进"));
assert!(!prompt.contains("项目迭代"));
assert!(!prompt.contains("MODE:"));
assert!(!prompt.contains("USER_HOW:"));
```

- [ ] **步骤2：运行测试确认失败或通过**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio model_client::tests::test_build_prompt -- --nocapture
```

预期结果：如果 prompt 仍含旧结构或硬编码语义，测试失败；如果已经清理，测试通过。

- [ ] **步骤3：替换 `build_prompt_with_context` 的 prompt 文案**

在 `D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/model_client.rs` 中，确保 `format!` 使用以下文案：

```rust
format!(
    r#"你是 Cozmio 的桌面观察助手。

你看到的是用户当前屏幕的一小段现场。
你的输出会被原样交给桌面端展示。
Cozmio 只提供事实材料和工具材料，不提供结论。

请只把下面的系统材料当作事实输入，不要把它们当成用户意图、任务阶段或项目结论。
是否出现、说什么、说多少、是否接入工作流，都由你基于截图和事实材料自行判断。
不要为了迎合上下文而编造屏幕上或材料中没有出现的内容。

窗口标题: {}
进程名: {}

{}

local_context:
{}
"#,
    window.title, window.process_name, process_context_block, popup_context_block
)
```

- [ ] **步骤4：保留 legacy parser，但不得让新 prompt 依赖它**

保留 `parse_response`、`ModelOutput`、`InterventionMode` 相关旧测试。不要在本任务删除旧 parser，因为这会扩大行为变更。新热路径必须继续调用 `call_raw_with_context`。

验收方式：执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; rg "call_raw_with_context|parse_response|MODE:|USER_HOW:" cozmio/src-tauri/src/model_client.rs cozmio/src-tauri/src/main_loop.rs
```

预期结果：

```text
cozmio/src-tauri/src/model_client.rs:... call_raw_with_context
cozmio/src-tauri/src/model_client.rs:... parse_response
cozmio/src-tauri/src/model_client.rs:... MODE:
cozmio/src-tauri/src/model_client.rs:... USER_HOW:
cozmio/src-tauri/src/main_loop.rs:... call_raw_with_context
```

`MODE:` 和 `USER_HOW:` 只允许出现在 legacy parser 测试数据和 parser 正则中，不允许出现在新 prompt 文案中。

- [ ] **步骤5：运行 prompt 与语义边界测试**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio model_client semantic_boundary -- --nocapture
```

预期结果：测试全部通过。

- [ ] **步骤6：提交 prompt 边界修改**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; git add cozmio/src-tauri/src/model_client.rs cozmio/src-tauri/tests/semantic_boundary.rs; git commit -m "feat: make model prompt fact-only and model-led"
```

预期结果：提交只包含模型 prompt 和语义边界测试相关修改。

---

### 任务4：把 UI 操作记录降级为事实事件

**涉及文件**：

- 修改：`D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/commands.rs`

- [ ] **步骤1：定位用户取消和关闭 pending confirmation 的日志记录**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; rg "cancelled_by_user|dismissed_by_user|ui_cancelled|ui_dismissed|USER_ACTION|confidence: 1\.0" cozmio/src-tauri/src/commands.rs
```

预期结果：能看到 pending confirmation 的 UI 操作记录位置。

- [ ] **步骤2：将取消事件记录为事实事件**

把用户取消 pending confirmation 时创建的 `ActionRecord` 字段设置为：

```rust
judgment: "USER_ACTION".to_string(),
next_step: String::new(),
level: "INFO".to_string(),
confidence: 0.0,
grounds: "pending confirmation cancelled through UI".to_string(),
system_action: "cancelled".to_string(),
content_text: None,
result_text: None,
error_text: None,
user_feedback: Some("ui_cancelled".to_string()),
```

- [ ] **步骤3：将关闭事件记录为事实事件**

把用户关闭 pending confirmation 时创建的 `ActionRecord` 字段设置为：

```rust
judgment: "USER_ACTION".to_string(),
next_step: String::new(),
level: "INFO".to_string(),
confidence: 0.0,
grounds: "pending confirmation dismissed through UI".to_string(),
system_action: "dismissed".to_string(),
content_text: None,
result_text: None,
error_text: None,
user_feedback: Some("ui_dismissed".to_string()),
```

- [ ] **步骤4：检查 commands.rs 不再写入旧反馈值**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; rg "cancelled_by_user|dismissed_by_user" cozmio/src-tauri/src/commands.rs
```

预期结果：命令无匹配输出。

- [ ] **步骤5：运行相关测试**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio commands -- --nocapture
```

预期结果：commands 相关测试通过；如果没有独立 commands 测试，cargo 输出中不得出现 test failure。

- [ ] **步骤6：提交 UI 事实事件修改**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; git add cozmio/src-tauri/src/commands.rs; git commit -m "fix: record UI confirmation actions as factual events"
```

预期结果：提交只包含 `commands.rs`。

---

### 任务5：确保 action log tail 读取不扫描整天日志

**涉及文件**：

- 修改：`D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/logging.rs`

- [ ] **步骤1：确认 tail 读取测试存在**

在 `logging.rs` 测试模块中确认存在以下测试函数：

```rust
#[test]
#[serial]
fn test_get_recent_tail_reads_latest_records() {
    let logger = create_test_logger();

    for i in 0..8 {
        logger
            .log(ActionRecord {
                timestamp: 1000 + i,
                trace_id: None,
                session_id: None,
                window_title: format!("Window {}", i),
                judgment: "CONTINUE".to_string(),
                next_step: "continue".to_string(),
                level: InterventionMode::Continue.to_string(),
                confidence: 1.0,
                grounds: "test".to_string(),
                system_action: "confirmed".to_string(),
                content_text: Some("content".to_string()),
                result_text: None,
                error_text: None,
                user_feedback: None,
                model_name: None,
                captured_at: None,
                call_started_at: None,
                call_duration_ms: None,
            })
            .unwrap();
    }

    let recent = logger.get_recent_tail(3, 4096).unwrap();

    assert_eq!(recent.len(), 3);
    assert_eq!(recent[0].window_title, "Window 7");
    assert_eq!(recent[1].window_title, "Window 6");
    assert_eq!(recent[2].window_title, "Window 5");
}
```

- [ ] **步骤2：实现 `get_recent_tail`**

在 `impl ActionLogger` 中加入以下方法：

```rust
pub fn get_recent_tail(
    &self,
    limit: usize,
    max_tail_bytes: u64,
) -> Result<Vec<ActionRecord>, String> {
    if !self.log_path.exists() {
        return Ok(Vec::new());
    }

    let mut file = OpenOptions::new()
        .read(true)
        .open(&self.log_path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;

    let file_len = file
        .metadata()
        .map_err(|e| format!("Failed to read log metadata: {}", e))?
        .len();
    let read_len = file_len.min(max_tail_bytes.max(1));
    file.seek(SeekFrom::Start(file_len.saturating_sub(read_len)))
        .map_err(|e| format!("Failed to seek log tail: {}", e))?;

    let mut buffer = String::new();
    file.read_to_string(&mut buffer)
        .map_err(|e| format!("Failed to read log tail: {}", e))?;

    let mut records: Vec<ActionRecord> = Vec::new();
    for (idx, line) in buffer.lines().enumerate() {
        if file_len > read_len && idx == 0 {
            continue;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str(line) {
            Ok(record) => records.push(record),
            Err(e) => {
                log::warn!("Skipping malformed action history tail record: {}", e);
            }
        }
    }

    records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    records.truncate(limit);
    Ok(records)
}
```

同时确保文件顶部 import 包含：

```rust
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
```

- [ ] **步骤3：运行 logging 测试**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio logging -- --nocapture
```

预期结果：logging 相关测试全部通过。

- [ ] **步骤4：提交 tail 读取能力**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; git add cozmio/src-tauri/src/logging.rs; git commit -m "perf: read action log tail for popup context"
```

预期结果：提交只包含 `logging.rs`。

---

### 任务6：接入事实上下文到主循环，但不添加弹窗控制规则

**涉及文件**：

- 修改：`D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/main.rs`
- 修改：`D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/main_loop.rs`

- [ ] **步骤1：在 main.rs 注册模块**

在 `D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/main.rs` 的模块声明区加入：

```rust
mod prompt_context;
```

- [ ] **步骤2：在 main_loop.rs 导入事实上下文函数**

在 `D:/C_Projects/Agent/cozmio/cozmio/src-tauri/src/main_loop.rs` 顶部加入：

```rust
use crate::prompt_context::build_popup_context;
```

- [ ] **步骤3：在模型调用前构建 popup_context**

在 `let model_client = ModelClient::new(active_config.clone());` 前加入：

```rust
let popup_context = build_popup_context(
    &logger,
    &snapshot.window_info.title,
    &snapshot.window_info.process_name,
    &process_context,
);
let popup_context_preview: String = popup_context.chars().take(240).collect();
log::debug!(
    "Popup context built ({} chars): {}",
    popup_context.len(),
    popup_context_preview
);
```

- [ ] **步骤4：调用 raw context 接口**

把模型调用改成：

```rust
let raw_output = match model_client.call_raw_with_context(
    &snapshot,
    &process_context,
    Some(&popup_context),
) {
    Ok(output) => output,
    Err(e) => {
        log::error!("Model call failed: {}", e);
        set_tray_state(&app_handle, TrayState::Idle);
        store_error_judgment(
            &app_handle,
            &logger,
            &mut last_error_signature,
            "MODEL_ERROR",
            &snapshot.window_info.title,
            &e,
            Some(process_context.clone()),
        );
        commands::emit_state_update(&app_handle);
        thread::sleep(poll_interval);
        continue;
    }
};
```

- [ ] **步骤5：确认没有添加新的机械弹窗限制**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; rg "cooldown|frequency|rate_limit|debounce|should_silence|should_popup|window.*switch.*skip" cozmio/src-tauri/src/main_loop.rs cozmio/src-tauri/src/prompt_context.rs
```

预期结果：命令无匹配输出，除了已有 `window_change_detection` 配置相关内容；如果匹配到新加的弹窗限制，删除该限制。

- [ ] **步骤6：运行主 crate 测试**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio
```

预期结果：测试全部通过。

- [ ] **步骤7：提交主循环接入**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; git add cozmio/src-tauri/src/main.rs cozmio/src-tauri/src/main_loop.rs; git commit -m "feat: feed factual context into model calls"
```

预期结果：提交包含 `main.rs` 和 `main_loop.rs`。

---

### 任务7：沉淀 AGENTS 语义边界

**涉及文件**：

- 修改：`D:/C_Projects/Agent/cozmio/AGENTS.md`

- [ ] **步骤1：在 Architecture Overview 后加入边界章节**

在 `## Architecture Overview` 段落后加入：

```markdown
## Semantic Boundary

Cozmio treats models as the semantic layer. System code must provide facts, tools, traces, and provenance, but must not hardcode semantic conclusions for the model.

Allowed system-authored material:

- Observed facts: timestamps, window titles, process names, trace ids, session ids, UI action ids, durations, counts, raw outputs, error text, source paths.
- Mechanical retrieval metadata: file paths, byte ranges, record ids, token counts, timestamps, source names.
- Tool affordances: what a tool can do, what input it requires, and what raw result it returned.

Forbidden system-authored material unless the user explicitly approves a specific design:

- Hardcoded user intent such as "the user is working on the project" or "the user is stuck".
- Hardcoded workflow stages such as `project_phase`, `task_stage`, or `iteration_opportunity`.
- Mechanical popup limits such as cooldowns, frequency caps, or window-switch suppression rules.
- Structured output constraints such as `decision: popup | silence` for model-led popup content.
- System-generated summaries that claim meaning without model or execution-agent provenance.

Context packs should be factual harnesses, not interpretations. If Cozmio needs semantic summaries, those summaries must be produced by a model or execution agent and stored with timestamp, source references, and provenance.
```

- [ ] **步骤2：确认章节存在**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; rg "Semantic Boundary|Cozmio treats models as the semantic layer|Context packs should be factual harnesses" AGENTS.md
```

预期结果：三条文本均有匹配。

- [ ] **步骤3：提交边界文档**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; git add AGENTS.md; git commit -m "docs: define Cozmio semantic boundary"
```

预期结果：提交只包含 `AGENTS.md`。

---

### 任务8：建立本地模型输出评审材料

**涉及文件**：

- 创建：`D:/C_Projects/Agent/cozmio/verification/context_harness_h1/review_template.md`
- 创建：`D:/C_Projects/Agent/cozmio/verification/context_harness_h1/runbook.md`

- [ ] **步骤1：创建评审模板**

创建 `D:/C_Projects/Agent/cozmio/verification/context_harness_h1/review_template.md`，完整内容如下：

```markdown
# Context Harness H1 模型输出评审模板

## 样本信息

- sample_id:
- timestamp:
- model_name:
- source_window:
- process_name:
- screenshot_path:
- prompt_context_path:
- output_path:

## 硬失败检查

- [ ] 输出不是 `[ERROR]`。
- [ ] 输出不是空白，除非模型自然输出空白且记录了 raw output 长度为 0。
- [ ] 输出没有声称截图或 factual context 中不存在的事实。
- [ ] 输出没有依赖系统生成的用户意图、任务阶段、项目阶段或弹窗策略。

## 有用性检查

- [ ] 输出具体到当前屏幕或当前 factual context。
- [ ] 输出可以进入用户正在做的工作流。
- [ ] 输出没有泛泛建议。
- [ ] 输出没有把 Claude Code、relay、subagent 日志当作已经注入端侧模型的长期记忆。
- [ ] 输出没有要求系统限制弹窗次数。

## 结论

- verdict: PASS / PARTIAL / FAIL
- reason:
- evidence:
- next_prompt_or_context_change:
```

- [ ] **步骤2：创建实验 runbook**

创建 `D:/C_Projects/Agent/cozmio/verification/context_harness_h1/runbook.md`，完整内容如下：

```markdown
# Context Harness H1 本地模型实验 Runbook

## 实验目标

验证 factual context 是否提升弹窗输出质量。验收重点是输出是否具体、有依据、能进入工作流，而不是弹窗数量是否减少。

## 启动前检查

1. 确认 Ollama 可访问：

```powershell
Invoke-RestMethod http://localhost:11434/api/tags
```

2. 确认 Cozmio 配置使用本地 Ollama：

```powershell
Get-Content "$env:LOCALAPPDATA/cozmio/config.json"
```

配置中的 `ollama_url` 必须是 `http://localhost:11434`，`model_name` 必须是本机 `api/tags` 返回的视觉模型名称。

## ERROR 停止规则

如果模型输出、应用日志或评审样本中出现 `[ERROR]`、`HTTP request failed`、`Ollama API error`、`model not found`，立即停止实验并修复环境。不得把 ERROR 样本计入输出质量评估。

## 样本桶

- workflow_active：用户正在写代码、调试、对话或处理任务。
- workflow_ambiguous：屏幕信息不足，模型可以自然少说或只说不确定。
- execution_trace_visible：屏幕上可见 Claude Code、relay、终端执行结果。
- high_risk_action：删除、发布、覆盖、提交等高风险动作。
- context_tail_useful：action log tail 中有最近 UI 反馈或执行结果。

## 评审流程

1. 每个样本保存 screenshot、prompt_context、raw_output、metadata。
2. 用 `review_template.md` 逐样本人工评审。
3. PASS 表示输出具体、有依据、可进入工作流。
4. PARTIAL 表示输出有依据但泛、迟滞或帮助不足。
5. FAIL 表示输出幻觉、依赖系统语义、误解事实材料或只是泛泛建议。

## 本阶段不评估

- 不以弹窗次数减少作为成功标准。
- 不要求模型输出结构化字段。
- 不测试 cooldown、frequency cap、window switch suppression。
```

- [ ] **步骤3：提交评审材料**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; git add verification/context_harness_h1/review_template.md verification/context_harness_h1/runbook.md; git commit -m "docs: add context harness model review runbook"
```

预期结果：提交包含两份 verification 文档。

---

### 任务9：执行模型输出验证实验

**涉及文件**：

- 读取：`D:/C_Projects/Agent/cozmio/verification/context_harness_h1/runbook.md`
- 读取：`D:/C_Projects/Agent/cozmio/verification/context_harness_h1/review_template.md`
- 创建：`D:/C_Projects/Agent/cozmio/verification/context_harness_h1/results-2026-04-28.md`

- [ ] **步骤1：检查 Ollama 模型列表**

执行命令：

```powershell
Invoke-RestMethod http://localhost:11434/api/tags | ConvertTo-Json -Depth 5
```

预期结果：输出 JSON 中包含至少一个视觉模型，例如 `qwen3-vl:4b` 或 `qwen3-vl:8b`。如果请求失败，停止任务9，记录环境问题。

- [ ] **步骤2：检查 Cozmio 配置**

执行命令：

```powershell
Get-Content "$env:LOCALAPPDATA/cozmio/config.json"
```

预期结果：`ollama_url` 是 `http://localhost:11434`，`model_name` 是步骤1返回的可用视觉模型。若配置仍是 `http://test:11434` 或 `test_model`，停止实验，先把配置改为真实本地模型。

- [ ] **步骤3：运行测试确保代码基线通过**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio
```

预期结果：测试全部通过。

- [ ] **步骤4：采集至少5个样本**

样本必须覆盖以下桶，每桶至少一个：

```text
workflow_active
workflow_ambiguous
execution_trace_visible
high_risk_action
context_tail_useful
```

每个样本至少保存四类材料：

```text
screenshot.png
prompt_context.txt
raw_output.txt
metadata.json
```

保存目录格式：

```text
D:/C_Projects/Agent/cozmio/verification/context_harness_h1/samples/<sample_id>/
```

- [ ] **步骤5：逐样本人工评审**

对每个样本复制 `review_template.md` 的内容，填写 verdict、reason、evidence、next_prompt_or_context_change。

硬失败标准：

```text
输出包含 [ERROR] => FAIL
输出编造未出现在截图或 factual context 的事实 => FAIL
输出依赖系统生成的用户意图或任务阶段 => FAIL
输出要求系统加 cooldown/frequency cap/沉默规则 => FAIL
```

- [ ] **步骤6：写结果汇总**

创建 `D:/C_Projects/Agent/cozmio/verification/context_harness_h1/results-2026-04-28.md`，内容使用以下结构：

```markdown
# Context Harness H1 Results

## Environment

- ollama_url:
- model_name:
- cozmio_commit:
- sample_count:

## Summary

- PASS:
- PARTIAL:
- FAIL:

## Findings

| sample_id | bucket | verdict | evidence | next change |
|---|---|---|---|---|

## Stop Conditions Hit

- none

## Decision

- proceed_to_execution_agent_memory_loop: yes/no
- reason:
```

- [ ] **步骤7：提交验证结果**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; git add verification/context_harness_h1; git commit -m "test: evaluate context harness model outputs"
```

预期结果：提交包含样本、评审和结果汇总。

---

### 任务10：规划执行端记忆沉淀接口，不接入端侧长日志

**涉及文件**：

- 创建：`D:/C_Projects/Agent/cozmio/docs/superpowers/specs/2026-04-28-execution-agent-memory-loop-design.md`

- [ ] **步骤1：创建执行端记忆循环设计文档**

创建 `D:/C_Projects/Agent/cozmio/docs/superpowers/specs/2026-04-28-execution-agent-memory-loop-design.md`，完整内容如下：

```markdown
# Execution Agent Memory Loop Design

> Status: ready for implementation planning
> Date: 2026-04-28

## Goal

Use stronger execution-side agents to reduce large logs into model-generated summaries with provenance, then expose only small factual references or selected summary snippets to the local model.

## Boundary

System code does not summarize meaning. System code schedules, stores, indexes, clips, and records provenance.

Semantic summaries may come from:

- execution agent output
- local model output
- user-authored notes

Semantic summaries must include:

- timestamp
- source path
- source byte range or record id
- producer name
- raw summary text

## Inputs

- Cozmio action log JSONL
- relay session outputs
- Claude Code project conversation logs
- subagent logs

## Outputs

- daily_summary records
- project_summary records
- source_index records

## Local Model Exposure Rule

The local model may receive only small selected records. It must not receive raw full-day logs or complete Claude Code conversations.

## Non-Goals

- No system-authored user intent.
- No popup cooldown.
- No structured silence field.
- No direct injection of large execution logs into the 4k local model context.
```

- [ ] **步骤2：提交设计文档**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; git add docs/superpowers/specs/2026-04-28-execution-agent-memory-loop-design.md; git commit -m "docs: design execution agent memory loop"
```

预期结果：提交只包含执行端记忆循环设计文档。

---

## 全量验证

- [ ] **步骤1：格式化 Rust 代码**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio/cozmio; cargo fmt
```

预期结果：命令成功退出。

- [ ] **步骤2：运行主测试**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio/cozmio; cargo test -p cozmio
```

预期结果：全部测试通过；如果出现 warning，可以记录但不阻塞本方案，除非 warning 指向本次新增死代码。

- [ ] **步骤3：运行语义边界搜索**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; rg "保持沉默|弹窗策略|检索线索|project_phase|iteration_opportunity|should_popup|should_silence|popup \| silence|cooldown|frequency cap" cozmio/src-tauri/src AGENTS.md docs/superpowers/specs docs/superpowers/plans
```

预期结果：运行时代码区域无 forbidden 命中。`#[cfg(test)]` 测试断言、文档中的“禁止项”“边界说明”语境可以命中；最终以 `cargo test -p cozmio --test semantic_boundary -- --nocapture` 的 2 个测试通过作为运行时边界 gate。

- [ ] **步骤4：检查工作树**

执行命令：

```powershell
cd D:/C_Projects/Agent/cozmio; git status --short
```

预期结果：只有本方案明确允许的文件处于已修改或已提交状态。若有无关文件，记录文件名，不要 revert 用户改动。

## 自我审查结果

- 产品类型：混合型，已包含软件实现测试和模型输出验证任务。
- 规格覆盖：覆盖事实上下文、prompt 边界、UI 事实事件、日志 tail、AGENTS 沉淀、模型评审、执行端记忆循环设计。
- 占位符排查：本文没有保留需要执行端自行补全的占位词集合。
- 类型一致性：`build_popup_context`、`get_recent_tail`、`call_raw_with_context`、`ActionRecord` 字段均与当前 Rust 代码一致。
- 用户边界：未要求 cooldown、frequency cap、structured silence、系统阶段语义或系统意图推断。
