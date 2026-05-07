# 模型验证（修订版）实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development` 逐任务落地本方案。

**目标**：验证模型能否基于真实窗口截图 + 机械元信息，输出"继续介入(CONTINUE)"或"收手(ABSTAIN)"的最终判断。

**本轮只验证最终介入判断，不验证中间思考。**

---

## 核心变更说明

原方案问题：
- 系统提示词只给了窗口标题+进程名，让模型自由发挥
- 输出是自由文本，模型输出了页面描述而非判断

修订后：
- 系统提示词明确定义任务：输出 `MODE: CONTINUE/ABSTAIN` + `REASON`
- 输出必须解析成结构化结果，不接受纯页面描述

---

## 系统提示词（必须严格使用）

```text
你是一个窗口判断器。

你的任务不是描述页面，也不是推测用户完整意图。
你的任务是：根据窗口截图和机械元信息，判断当前证据是否足以支持 agent 继续介入。

只允许两种最终结果：

1. CONTINUE
表示：当前可见证据已经足以支持 agent 继续往前一轮判断。

2. ABSTAIN
表示：当前证据不足，agent 不应继续延伸。

要求：
- 只能依据截图和提供的机械元信息作判断。
- 不要补充截图中看不见、元信息中没有的事实。
- 不要输出页面描述作为最终结果。
- 不要输出世界标签，例如"idle状态""高风险操作""用户正在做X"。
- 不要推荐具体动作，不要替用户做决定。
- 不要展示思考过程，只输出最终结果。

输出格式必须严格为：

MODE: CONTINUE 或 MODE: ABSTAIN
REASON: 一句简短理由，理由只能引用可见证据。
```

---

## 输出结构

```rust
pub enum InterventionMode {
    Continue,
    Abstain,
}

pub struct InterventionResult {
    pub mode: InterventionMode,
    pub reason: String,
    pub raw_output: String,  // 保留原始输出用于 review
}
```

---

## Review 四项检查

1. **有没有给出最终判断** - 是 CONTINUE 还是 ABSTAIN，格式是否正确
2. **理由是否贴证据** - REASON 是否引用了截图/元信息中的可见内容
3. **是否偷跑成页面描述** - 输出是否是"这是一个XX页面"而非判断
4. **该收手时是否收手** - 信号不足时是否给了 ABSTAIN

---

## Task 1：修改 client.rs - 更新系统提示词 + 输出解析

**涉及文件**：
- 修改：`cozmio/cozmio_model/src/client.rs`

- [ ] **步骤1：更新 client.rs 系统提示词**

将 prompt 改为上面定义的系统提示词，不再是裸窗口信息。

- [ ] **步骤2：添加输出解析**

添加 `InterventionResult` 结构和解析函数：

```rust
use regex::Regex;

pub enum InterventionMode {
    Continue,
    Abstain,
}

pub struct InterventionResult {
    pub mode: InterventionMode,
    pub reason: String,
    pub raw_output: String,
}

pub fn parse_intervention_result(raw: &str) -> Result<InterventionResult, ModelError> {
    // 解析 MODE: CONTINUE 或 MODE: ABSTAIN
    let mode_re = Regex::new(r"(?i)MODE:\s*(CONTINUE|ABSTAIN)").unwrap();
    // 解析 REASON: ...
    let reason_re = Regex::new(r"(?i)REASON:\s*(.+)").unwrap();

    let mode_cap = mode_re.captures(raw).ok_or_else(|| {
        ModelError::ParseError("Missing MODE line".to_string())
    })?;
    let mode_str = mode_cap.get(1).unwrap().as_ref().to_uppercase();
    let mode = if mode_str == "CONTINUE" {
        InterventionMode::Continue
    } else {
        InterventionMode::Abstain
    };

    let reason = reason_re.captures(raw)
        .map(|c| c.get(1).unwrap().as_ref().trim().to_string())
        .unwrap_or_default();

    Ok(InterventionResult {
        mode,
        reason,
        raw_output: raw.to_string(),
    })
}
```

注意：需要添加 `regex = "1.0"` 依赖到 Cargo.toml。

- [ ] **步骤3：验证编译**

执行命令：`cd cozmio && cargo check -p cozmio_model`
预期结果：编译成功

- [ ] **步骤4：提交代码**

```bash
cd cozmio && git add cozmio_model/src/client.rs cozmio_model/Cargo.toml
git commit -m "fix(cozmio_model): add intervention mode prompt and output parsing"
```

---

## Task 2：修改 cozmio_verify - 保存结构化结果

**涉及文件**：
- 修改：`cozmio/cozmio_verify/src/main.rs`

- [ ] **步骤1：更新 main.rs 使用新输出**

将调用改为：
```rust
use cozmio_model::{ask_model_sync, parse_intervention_result, InterventionResult};

// 调用模型后解析
let raw_output = ask_model_sync(model, &screenshot.image_base64, title, process_name)
    .map_err(|e| {
        eprintln!("模型调用失败: {}", e);
        std::process::exit(1);
    })?;

let result: InterventionResult = parse_intervention_result(&raw_output)
    .map_err(|e| {
        eprintln!("解析输出失败: {}", e);
        std::process::exit(1);
    })?;

println!("\n模型判断: {:?}", result.mode);
println!("理由: {}", result.reason);
```

保存三个文件：
- `screenshot.png` - 原始截图
- `meta.json` - 机械元信息
- `output.json` - 结构化结果（含 raw_output 用于 review）

- [ ] **步骤2：验证编译**

执行命令：`cd cozmio && cargo check -p cozmio_verify`
预期结果：编译成功

- [ ] **步骤3：提交代码**

```bash
cd cozmio && git add cozmio_verify/src/main.rs
git commit -m "fix(cozmio_verify): use structured intervention result"
```

---

## Task 3：重写 review_template.md

**涉及文件**：
- 修改：`verification/review_template.md`

- [ ] **步骤1：重写 review_template.md**

```markdown
# 模型输出 Review 模板

## 样本信息

- 样本 ID: _______________
- Review 日期: _______________
- Reviewer: _______________

---

## 四项检查

Review 时，对照 screenshot.png 和 meta.json，检查 output.json 中的 raw_output。

---

### 1. 有没有给出最终判断？

输出是否包含正确的 MODE 格式：`MODE: CONTINUE` 或 `MODE: ABSTAIN`

- [ ] **有最终判断** - Pass
- [ ] **没有/格式错误** - Fail

如果 Fail，描述：_______________

---

### 2. 理由是否贴证据？

REASON 是否引用了截图或元信息中的可见内容（标题、进程名、截图内容），而非编造的事实。

检查点：
- 理由中提到的内容，在 screenshot.png 或 meta.json 中是否可见？
- 是否有"窗口中没有的按钮"、"用户意图"等编造内容？

- [ ] **理由贴证据** - Pass
- [ ] **理由超出证据** - Fail

如果 Fail，描述超出内容：_______________

---

### 3. 是否偷跑成页面描述？

输出是否是"这是一个XX页面"、"从截图看..."等页面描述，而非判断。

**拒绝的输出类型**：
- "这是一个 QQ 邮箱登录页面..."
- "从截图内容来看，该窗口显示的是..."
- "您提供的信息中存在一些混淆点..."

- [ ] **是判断而非描述** - Pass
- [ ] **偷跑成页面描述** - Fail

如果 Fail，描述：_______________

---

### 4. 该收手时是否收手？

当证据不足以支撑 CONTINUE 时，是否给出了 ABSTAIN。

检查点：
- 截图是否模糊/元信息是否不足？
- 如果证据不足，MODE 是否为 ABSTAIN？
- 如果证据充足却给了 ABSTAIN，是否合理？

- [ ] **该收手时收手** - Pass
- [ ] **信号不足却给 CONTINUE** - Fail

如果 Fail，描述：_______________

---

## 最终分类

| 四项检查 | 结果 |
|----------|------|
| 1. 最终判断 | Pass / Fail |
| 2. 理由贴证据 | Pass / Fail |
| 3. 判断非描述 | Pass / Fail |
| 4. 正确收手 | Pass / Fail |

**综合结果**：
- [ ] **Pass** - 四项全部 Pass
- [ ] **Fail** - 任意一项 Fail

**如果 Fail，失败原因**：_______________

---

## Review 备注（可选）

其他观察：_______________
```

- [ ] **步骤2：提交代码**

```bash
cd cozmio && git add verification/review_template.md
git commit -m "fix: rewrite review template for MODE+REASON format"
```

---

## Task 4：重新运行样本验证

- [ ] **步骤1：捕获 3-5 个样本**

```bash
cd cozmio && cargo run -p cozmio_verify -- --model qwen3-vl:4b --output verification/samples
```

每次运行后切换窗口，捕获不同场景。

- [ ] **步骤2：对每个样本进行 Review**

使用新的 review_template.md 逐项检查。

- [ ] **步骤3：更新 review_summary.md**

```markdown
# Review 汇总

## 样本列表

| 样本 ID | 最终判断 | 理由贴证据 | 判断非描述 | 正确收手 | 综合 |
|---------|----------|------------|------------|----------|------|
| sample_XXX | Pass/Fail | Pass/Fail | Pass/Fail | Pass/Fail | Pass/Fail |

## 模式观察

[记录观察到的模式]

## 结论

- 模型能否输出有效的 CONTINUE/ABSTAIN 判断？
- 理由是否贴证据？
- 是否仍有页面描述的问题？
- 是否在该收手时收手？
```

- [ ] **步骤4：提交样本和汇总**

```bash
cd cozmio && git add verification/samples/ verification/review_summary.md
git commit -m "feat: add samples with MODE+REASON output format"
```

---

## 验收标准

| 验证项 | 预期结果 |
|--------|----------|
| `cargo check -p cozmio_model` | 编译成功 |
| `cargo check -p cozmio_verify` | 编译成功 |
| 输出格式 | `MODE: CONTINUE/ABSTAIN` + `REASON:` |
| output.json | 包含 mode + reason + raw_output |
| Review 四项 | 每项都要检查 |
| 拒绝页面描述 | 模型不输出纯描述性内容 |

---

## 明确拒绝的输出类型

1. **纯页面描述** - "这是一个 QQ 邮箱登录页面..."
2. **世界标签** - "idle状态"、"高风险操作"
3. **动作建议** - "建议点击登录"
4. **长篇分析** - "我将从两个角度为您解析..."
5. **中间思考** - "让我分析一下..."

---

## 方案总结

| Task | 描述 | 改动 |
|------|------|------|
| 1 | 修改 client.rs | 新系统提示词 + 输出解析 |
| 2 | 修改 cozmio_verify | 保存 output.json |
| 3 | 重写 review_template.md | 四项检查 |
| 4 | 重新运行样本验证 | 捕获 + Review + 汇总 |