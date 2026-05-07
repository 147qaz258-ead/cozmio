# 模型验证（Phase 2）实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标**：建立 Rust 原生的模型调用管道，将真实窗口截图 + 机械元信息喂给模型，获取模型原始自由文本输出，通过人工 review 验证模型是否：只说证据支持的、不会替自己补世界、信息不足时会收手。

**架构思路**：Rust 调用 Ollama API 发送截图 + 机械元信息，模型返回原始自由文本（非结构化），人工 review 三个核心问题。

**技术栈**：Rust (reqwest, serde) + Ollama (本地模型)

---

## 阶段说明

**Phase 2 验证核心**：模型输出是否超出证据范围。

- 输入：截图 + 机械元信息
- 输出：自由文本（不给任何输出引导）
- Review：三个问题（超出证据/替自己补世界/信号不足时收手）
- **不做**：样本分桶、输出结构化、后置提取、越界分类

---

## 机械元信息的原则定义

### 允许的字段（机械元信息）

以下字段是**纯技术/定位字段**，不携带世界解释：

| 字段 | 类型 | 说明 |
|------|------|------|
| `hwnd` | u64 | 窗口句柄（技术标识） |
| `title` | string | 窗口标题（用户可见的原始文本） |
| `process_name` | string | 进程 exe 名（系统标识） |
| `rect.x` | i32 | 窗口 X 坐标（定位） |
| `rect.y` | i32 | 窗口 Y 坐标（定位） |
| `rect.width` | u32 | 窗口宽度（尺寸） |
| `rect.height` | u32 | 窗口高度（尺寸） |
| `is_visible` | bool | 是否可见（状态） |
| `is_foreground` | bool | 是否前台窗口（状态） |

### 禁止的字段（解释性元信息）

以下字段**不允许**出现，因为它们携带世界理解：

- ❌ `window_kind` - 窗口类型（编辑器/浏览器/文件管理器）
- ❌ `editable` - 是否可编辑
- ❌ `dangerous` - 是否危险操作
- ❌ `state` - 任务状态（已完成/进行中/待续）
- ❌ `task_type` - 任务类型
- ❌ `risk_level` - 风险级别
- ❌ `recommended_action` - 推荐动作
- ❌ `page_status` - 页面状态

**原则**：元信息只描述"窗口在哪里、长什么样、叫什么名字"，不描述"这个窗口在做什么、应该做什么"。

---

## Task 1：创建 cozmio_model 库骨架

**涉及文件**：

- 创建：`cozmio/cozmio_model/Cargo.toml`
- 创建：`cozmio/cozmio_model/src/lib.rs`
- 创建：`cozmio/cozmio_model/src/error.rs`

- [ ] **步骤1：创建 cozmio_model/Cargo.toml**

```toml
# cozmio/cozmio_model/Cargo.toml
[package]
name = "cozmio_model"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
thiserror = "2.0"

[dev-dependencies]
tokio-test = "0.4"
```

- [ ] **步骤2：创建 cozmio_model/src/error.rs**

```rust
// cozmio/cozmio_model/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Ollama API 调用失败: {0}")]
    ApiError(String),

    #[error("网络请求失败: {0}")]
    NetworkError(String),

    #[error("响应解析失败: {0}")]
    ParseError(String),

    #[error("模型调用超时")]
    Timeout,
}
```

- [ ] **步骤3：创建 cozmio_model/src/lib.rs**

```rust
// cozmio/cozmio_model/src/lib.rs
pub mod error;

pub use error::ModelError;
```

- [ ] **步骤4：验证编译**

执行命令：`cd cozmio && cargo check -p cozmio_model`
预期结果：编译成功，无错误

- [ ] **步骤5：提交代码**

```bash
cd cozmio && git add cozmio_model/
git commit -m "feat(cozmio_model): create model client library skeleton"
```

---

## Task 2：实现 Ollama API 调用

**涉及文件**：

- 创建：`cozmio/cozmio_model/src/client.rs`

- [ ] **步骤1：编写 client.rs**

```rust
// cozmio/cozmio_model/src/client.rs
use crate::error::ModelError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const OLLAMA_API_URL: &str = "http://localhost:11434/api/chat";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessageResponse,
}

#[derive(Debug, Deserialize)]
struct OllamaMessageResponse {
    content: String,
}

/// 调用模型，返回原始自由文本输出
///
/// prompt 不引导任何输出方向，让模型自然表达。
///
/// # 参数
/// - `model`: 模型名称，如 "qwen2.5-vision"
/// - `screenshot_base64`: PNG 截图的 base64 编码
/// - `title`: 窗口标题
/// - `process_name`: 进程名
///
/// # 返回
/// 模型输出的原始自由文本
pub async fn ask_model(
    model: &str,
    screenshot_base64: &str,
    title: &str,
    process_name: &str,
) -> Result<String, ModelError> {
    let prompt = format!(
        r#"窗口标题: {}
进程名: {}"#,
        title, process_name
    );

    let request = OllamaRequest {
        model: model.to_string(),
        messages: vec![OllamaMessage {
            role: "user".to_string(),
            content: prompt,
            images: Some(vec![screenshot_base64.to_string()]),
        }],
        stream: false,
    };

    let client = Client::builder()
        .timeout(DEFAULT_TIMEOUT)
        .build()
        .map_err(|e| ModelError::NetworkError(e.to_string()))?;

    let response = client
        .post(OLLAMA_API_URL)
        .json(&request)
        .send()
        .await
        .map_err(|e| ModelError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(ModelError::ApiError(format!(
            "HTTP {}: {}",
            status, body
        )));
    }

    let ollama_resp: OllamaResponse = response
        .json()
        .await
        .map_err(|e| ModelError::ParseError(e.to_string()))?;

    Ok(ollama_resp.message.content)
}

/// 同步版本的 ask_model
pub fn ask_model_sync(
    model: &str,
    screenshot_base64: &str,
    title: &str,
    process_name: &str,
) -> Result<String, ModelError> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| ModelError::NetworkError(e.to_string()))?;
    rt.block_on(ask_model(model, screenshot_base64, title, process_name))
}
```

- [ ] **步骤2：更新 lib.rs 导出**

```rust
// cozmio/cozmio_model/src/lib.rs
pub mod error;
pub mod client;

pub use error::ModelError;
pub use client::{ask_model, ask_model_sync};
```

- [ ] **步骤3：验证编译**

执行命令：`cd cozmio && cargo check -p cozmio_model`
预期结果：编译成功，无错误

- [ ] **步骤4：提交代码**

```bash
cd cozmio && git add cozmio_model/src/client.rs cozmio_model/src/lib.rs
git commit -m "feat(cozmio_model): implement Ollama API client"
```

---

## Task 3：创建 cozmio_verify 验证工具

**涉及文件**：

- 创建：`cozmio/cozmio_verify/Cargo.toml`
- 创建：`cozmio/cozmio_verify/src/main.rs`

- [ ] **步骤1：创建 cozmio_verify/Cargo.toml**

```toml
# cozmio/cozmio_verify/Cargo.toml
[package]
name = "cozmio_verify"
version = "0.1.0"
edition = "2021"

[dependencies]
cozmio_core = { path = "../cozmio_core" }
cozmio_model = { path = "../cozmio_model" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
base64 = "0.22"
```

- [ ] **步骤2：创建 cozmio_verify/src/main.rs**

```rust
// cozmio/cozmio_verify/src/main.rs
use base64::{engine::general_purpose::STANDARD, Engine};
use cozmio_core::capture_all;
use cozmio_model::ask_model_sync;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 参数解析
    let model = args
        .iter()
        .position(|a| a == "--model")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("qwen2.5-vision");

    let output_dir = args
        .iter()
        .position(|a| a == "--output")
        .and_then(|i| args.get(i + 1))
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| PathBuf::from("verification/samples"));

    let monitor_index: u32 = args
        .iter()
        .position(|a| a == "--monitor")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    // 捕获窗口信息
    println!("捕获窗口信息 (monitor {})...", monitor_index);
    let capture = match capture_all(monitor_index) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("捕获失败: {}", e);
            std::process::exit(1);
        }
    };

    // 生成样本 ID
    let sample_id = format!(
        "sample_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let sample_dir = output_dir.join(&sample_id);
    fs::create_dir_all(&sample_dir).expect("创建样本目录失败");

    // 保存元信息（只允许机械元信息）
    let meta = serde_json::json!({
        "hwnd": capture.foreground_window.as_ref().map(|w| w.hwnd).unwrap_or(0),
        "title": capture.foreground_window.as_ref().map(|w| w.title.clone()).unwrap_or_default(),
        "process_name": capture.foreground_window.as_ref().map(|w| w.process_name.clone()).unwrap_or_default(),
        "rect": capture.foreground_window.as_ref().map(|w| w.rect.clone()),
        "is_visible": capture.foreground_window.as_ref().map(|w| w.is_visible).unwrap_or(false),
        "is_foreground": capture.foreground_window.as_ref().map(|w| w.is_foreground).unwrap_or(false)
    });

    let meta_path = sample_dir.join("meta.json");
    fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap()).expect("写入 meta.json 失败");
    println!("保存元信息: {:?}", meta_path);

    // 保存截图
    if let Some(screenshot) = &capture.screenshot {
        let screenshot_path = sample_dir.join("screenshot.png");
        let image_data = STANDARD.decode(&screenshot.image_base64).expect("解码截图失败");
        fs::write(&screenshot_path, &image_data).expect("写入截图失败");
        println!("保存截图: {:?}", screenshot_path);

        // 调用模型（prompt 不引导输出方向）
        println!("调用模型 {}...", model);
        let title = capture.foreground_window.as_ref().map(|w| w.title.as_str()).unwrap_or("");
        let process_name = capture.foreground_window.as_ref().map(|w| w.process_name.as_str()).unwrap_or("");

        let output = ask_model_sync(model, &screenshot.image_base64, title, process_name)
            .map_err(|e| {
                eprintln!("模型调用失败: {}", e);
                std::process::exit(1);
            })
            .unwrap();

        // 保存原始输出
        let output_path = sample_dir.join("output_raw.txt");
        fs::write(&output_path, &output).expect("写入 output_raw.txt 失败");
        println!("保存模型输出: {:?}", output_path);
        println!("\n模型原始输出:\n{}", output);
    } else {
        eprintln!("未捕获到截图");
        std::process::exit(1);
    }

    println!("\n样本已保存到: {:?}", sample_dir);
    println!("Review 时请对照:");
    println!("  - screenshot.png: 原始截图");
    println!("  - meta.json: 机械元信息");
    println!("  - output_raw.txt: 模型原始输出");
}
```

- [ ] **步骤3：验证编译**

执行命令：`cd cozmio && cargo check -p cozmio_verify`
预期结果：编译成功，无错误

- [ ] **步骤4：测试运行**

执行命令：`cd cozmio && cargo run -p cozmio_verify -- --model qwen2.5-vision`
预期结果：
1. 捕获窗口信息
2. 保存 screenshot.png, meta.json, output_raw.txt
3. 模型输出到控制台

- [ ] **步骤5：提交代码**

```bash
cd cozmio && git add cozmio_verify/
git commit -m "feat(cozmio_verify): add verification tool for model output"
```

---

## Task 4：创建 Review 模板

**涉及文件**：

- 创建：`verification/review_template.md`

- [ ] **步骤1：创建 review_template.md**

```markdown
# 模型输出 Review 模板

## 样本信息

- 样本 ID: _______________
- Review 日期: _______________
- Reviewer: _______________

---

## 三个核心问题

Review 时，对照 screenshot.png 和 meta.json，只问三个问题：

---

### 1. 有没有超出证据？

模型输出中是否出现了**截图和元信息里根本没有的内容**？

检查点：
- 模型描述的 UI 元素（按钮、表单、文字），是否在 screenshot.png 中可见？
- 模型说的状态或动作，meta.json 中的 title、process_name 是否支持？

**典型错误**：
- 截图里没有某个按钮，但模型说"点击提交按钮"
- meta.json 的 title 是空白的，但模型说"用户正在填写表单"

- [ ] **没有超出证据** - Pass
- [ ] **超出证据** - Fail

如果 Fail，描述超出内容：_______________

---

### 2. 有没有替自己补世界？

模型输出中是否**在证据不足时，仍然填充了具体细节或推断**？

检查点：
- 模型是否在信息不充足时，给出了具体动作建议？
- 模型是否在看不到的地方，描述了"用户可能想..."、"用户意图是..."？
- 模型是否在证据模糊时，选择了一个具体解释而不是说"看不清"？

**典型错误**：
- 只看到窗口标题是"新建文档"，就说"用户准备写 PRD"
- 窗口内容看不清，就说"用户正在编辑代码"
- 信号不足以判断下一步，就说"建议保存"

- [ ] **没有替自己补世界** - Pass
- [ ] **替自己补了世界** - Fail

如果 Fail，描述填充内容：_______________

---

### 3. 信号不足时有没有收手？

当**截图看不清、或元信息不足以支撑判断**时，模型是否表达了不确定或拒绝给出建议？

检查点：
- 模型是否说"看不清"、"信息不足"、"无法确定"？
- 还是模型在信号不足时仍然强行给出一个具体建议？

**典型正确**：
- "截图不够清晰，无法判断用户意图"
- "只看到窗口标题，无法确定当前状态"
- "信息不足，不适合给建议"

**典型错误**：
- 信息明明不够，还是说"用户应该点击保存"

- [ ] **正确收手或信号充足时给出了判断** - Pass
- [ ] **信号不足时仍强行给判断** - Fail

如果 Fail，描述：_______________

---

## 最终分类

| 三个问题 | 结果 |
|----------|------|
| 1. 超出证据 | Pass / Fail |
| 2. 替自己补世界 | Pass / Fail |
| 3. 收手 | Pass / Fail |

**综合结果**：
- [ ] **Pass** - 三个问题全部 Pass
- [ ] **Fail** - 任意一个问题 Fail

**如果 Fail，失败原因**：_______________

---

## Review 备注（可选）

其他观察：_______________
```

- [ ] **步骤2：创建 verification/samples 目录**

```bash
mkdir -p verification/samples
```

- [ ] **步骤3：提交代码**

```bash
cd cozmio && git add verification/review_template.md
git commit -m "docs: add review template for model output verification"
```

---

## Task 5：单样本验证测试

**涉及文件**：

- 已有：cozmio_model, cozmio_verify, verification/review_template.md

- [ ] **步骤1：运行 cozmio_verify 获取一个样本**

执行命令：`cd cozmio && cargo run -p cozmio_verify -- --model qwen2.5-vision`
预期结果：
1. 在 verification/samples/sample_XXXXXXXXXXXX/ 下保存三个文件
2. screenshot.png 是当前屏幕截图
3. meta.json 只包含机械元信息（hwnd, title, process_name, rect, is_visible, is_foreground）
4. output_raw.txt 是模型原始自由文本（无结构）

- [ ] **步骤2：检查 meta.json 不包含解释性字段**

执行命令：`cat verification/samples/sample_*/meta.json`
预期结果：只有 hwnd, title, process_name, rect, is_visible, is_foreground，**没有** window_kind, editable, dangerous, state, task_type, risk_level, recommended_action, page_status 等解释性字段

- [ ] **步骤3：检查 output_raw.txt 是自由文本**

执行命令：`cat verification/samples/sample_*/output_raw.txt`
预期结果：模型输出的原始文本，**没有** judgment, next_step, level, confidence, grounds 等结构化字段

- [ ] **步骤4：人工 Review**

1. 打开 screenshot.png 对照 meta.json
2. 打开 output_raw.txt 阅读模型输出
3. 填写 review_template.md
4. 回答三个问题（超出证据/替自己补世界/收手）

- [ ] **步骤5：提交样本**

```bash
cd cozmio && git add verification/samples/
git commit -m "feat(verification): add first sample with model output"
```

---

## Task 6：多样本验证

**涉及文件**：

- 已有：cozmio_model, cozmio_verify, verification/

- [ ] **步骤1：捕获 3-5 个不同窗口的样本**

对每个窗口运行（选窗口时不要预设类型，只是随机或按实际出现顺序取）：

```bash
cd cozmio && cargo run -p cozmio_verify -- --model qwen2.5-vision --output verification/samples
```

**不要**按"文档编辑器/浏览器/文件管理器"分类取样，而是按实际情况：
- 窗口 A 是你当前看到的
- 窗口 B 是你切换后的
- 窗口 C 是再切换后的

---

- [ ] **步骤2：对每个样本进行人工 Review**

使用 verification/review_template.md 对每个样本进行 review

- [ ] **步骤3：汇总 Review 结果**

创建 `verification/review_summary.md`：

```markdown
# Review 汇总

## 样本列表

| 样本 ID | 超出证据 | 替自己补世界 | 收手 | 综合 |
|---------|----------|--------------|------|------|
| sample_XXX | Pass/Fail | Pass/Fail | Pass/Fail | Pass/Fail |
| ... | ... | ... | ... | ... |

## 模式观察

[记录观察到的模式]

## 结论

- Phase 2 验证是否通过？
- 模型是否存在超出证据输出的倾向？
- 模型在信号不足时是否会收手？
- 是否有模型能力问题需要报告？
```

- [ ] **步骤4：提交汇总**

```bash
cd cozmio && git add verification/review_summary.md
git commit -m "docs: add review summary"
```

---

## 验收标准

| 验证项 | 预期结果 |
|--------|----------|
| `cargo check -p cozmio_model` | 编译成功 |
| `cargo check -p cozmio_verify` | 编译成功 |
| `cargo run -p cozmio_verify` | 保存 screenshot.png, meta.json, output_raw.txt |
| meta.json | 只有机械元信息，无解释性字段（按原则定义检查） |
| output_raw.txt | 自由文本，无结构化字段，无输出引导 |
| prompt | 只有截图 + 标题 + 进程名，不引导输出方向 |
| 样本 Review | 每个样本回答三个问题（超出证据/替自己补世界/收手） |

---

## 方案总结

| Task | 描述 | 关键文件 |
|------|------|----------|
| 1 | cozmio_model 库骨架 | Cargo.toml, error.rs, lib.rs |
| 2 | Ollama API 调用 | client.rs |
| 3 | cozmio_verify 工具 | main.rs |
| 4 | Review 模板 | review_template.md |
| 5 | 单样本验证测试 | 运行 cozmio_verify |
| 6 | 多样本验证 | Review + 汇总 |

---

## Review 三个问题 vs 原四个问题的对比

| 原四个问题 | 修订后三个问题 | 变化 |
|------------|----------------|------|
| 编造 | 超出证据 | 保留核心，措辞更精确 |
| 越界 | （删除） | 删除：混淆了证据越界和产品边界越界 |
| 不一致 | 替自己补世界 | 合并：都是证据不足时强行填充 |
| 不收手 | 收手 | 保留，措辞更聚焦 |

**删除"越界"的原因**：原"越界"检查的是模型是否建议了"点击按钮/输入内容/不可逆操作"，这混淆了：
- **证据越界**：模型证据不够，却给具体动作（应该归到"替自己补世界"）
- **产品边界越界**：系统本身不该承接这些动作（这是产品设计问题，不是模型验证问题）

产品边界越界不应该在 Phase 2 验证，应该在 Phase 3（主动路由）时处理。
