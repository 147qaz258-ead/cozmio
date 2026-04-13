---
appendix: C
title: "SKILL.md 编写规范"
status: draft
---

# 附录 C：SKILL.md 编写规范

本附录是 SKILL.md 文件的完整编写规范，涵盖格式要求、Frontmatter 字段、内容编写指南和实际示例。

关于 Skill 系统的架构设计，参见 [第 6 章](../vol2/ch06.md)；关于 ClawHub Skill 注册表，参见 [第 7 章](../vol3/ch07.md)。

---

## SKILL.md 格式概述

每个 Skill 是一个包含 `SKILL.md` 文件的目录。`SKILL.md` 使用 YAML Frontmatter + Markdown 正文的格式，符合 [AgentSkills](https://agentskills.io) 规范。

最小可用 SKILL.md：

```markdown
---
name: hello_world
description: A simple skill that says hello.
---

# Hello World Skill

When the user asks for a greeting, use the `echo` tool to say
"Hello from your custom skill!".
```

---

## Frontmatter 字段参考

### 必填字段

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `name` | `string` | 唯一标识符，使用 snake_case 格式 |
| `description` | `string` | 单行描述，展示给 Agent 用于判断何时调用此 Skill |

### 可选字段

| 字段 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `homepage` | `string` | - | 项目主页 URL |
| `user-invocable` | `boolean` | `true` | 是否作为用户斜杠命令暴露 |
| `disable-model-invocation` | `boolean` | `false` | 为 `true` 时，Skill 不注入模型提示词（仅通过用户调用） |
| `command-dispatch` | `string` | - | 设为 `tool` 时，斜杠命令直接分发到工具，跳过模型 |
| `command-tool` | `string` | - | 直接分发时调用的工具名称 |
| `command-arg-mode` | `string` | `raw` | 工具分发时的参数传递模式 |
| `metadata` | `string` | - | 单行 JSON 对象，包含加载过滤条件 |

### metadata 对象

`metadata` 必须是单行 JSON 字符串：

```yaml
metadata: {"openclaw": {"requires": {"bins": ["uv"], "env": ["GEMINI_API_KEY"]}, "primaryEnv": "GEMINI_API_KEY"}}
```

`metadata.openclaw` 下的字段：

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `always` | `boolean` | 始终包含此 Skill，跳过其他门控检查 |
| `emoji` | `string` | macOS Skills UI 使用的 Emoji |
| `homepage` | `string` | 项目主页 URL |
| `os` | `string[]` | 操作系统过滤：`darwin`、`linux`、`win32` |
| `requires.bins` | `string[]` | PATH 中必须存在的可执行文件（全部满足） |
| `requires.anyBins` | `string[]` | PATH 中必须存在的可执行文件（任一满足） |
| `requires.env` | `string[]` | 必须存在的环境变量 |
| `requires.config` | `string[]` | `openclaw.json` 中必须为真的配置路径 |
| `primaryEnv` | `string` | 与 `skills.entries.<name>.apiKey` 关联的环境变量名 |
| `install` | `object[]` | 安装器规格，用于 macOS Skills UI 和自动安装 |

### 安装器规格

```yaml
metadata: {"openclaw": {"requires": {"bins": ["gemini"]}, "install": [{"id": "brew", "kind": "brew", "formula": "gemini-cli", "bins": ["gemini"], "label": "Install Gemini CLI (brew)"}]}}
```

安装器类型：

| 类型 | 说明 |
|:---|:---|
| `brew` | Homebrew 安装 |
| `node` | npm/pnpm/yarn/bun 安装 |
| `go` | Go 模块安装 |
| `download` | 下载归档文件（支持 `url`、`archive`、`extract`、`stripComponents`、`targetDir`） |

安装器可通过 `os` 字段过滤平台。Node 安装受 `skills.install.nodeManager` 控制（默认 npm）。

---

## Skill 加载位置和优先级

| 位置 | 优先级 | 作用域 |
|:---|:---|:---|
| `<workspace>/skills/` | 最高 | 单 Agent |
| `~/.openclaw/skills/` | 中 | 全部 Agent 共享 |
| 内置 Skill（随安装包附带） | 最低 | 全局 |
| `skills.load.extraDirs` | 最低 | 自定义共享目录 |

同名冲突时，高优先级覆盖低优先级。会话启动时快照 Skill 列表，后续轮次复用同一列表。变更在下次新会话生效，或通过文件监听器热重载。

---

## 编写内容的原则

### 指令风格

SKILL.md 的 Markdown 正文是给 Agent 的指令，不是给人看的文档。遵循以下原则：

**告诉 Agent 做什么，而非如何做 AI：**

```markdown
# 好的写法
When the user asks about the weather, use the `exec` tool to run `curl wttr.in/{{location}}?format=3` and relay the result.

# 差的写法
You are a weather assistant. Be helpful and accurate when answering weather questions.
```

**具体明确，避免模糊指令：**

```markdown
# 好的写法
Run `rsync -avz --delete ~/projects/{{project}}/ server:/backups/{{project}}/` and report the summary line.

# 差的写法
Back up the project to the server using rsync.
```

**包含错误处理指导：**

```markdown
# 包含错误处理
If `rsync` exits with code 23 (partial transfer), report which files failed.
If it exits with code 30 (timeout), retry once with `--timeout=120`.
On any other non-zero exit, report the full error and ask the user what to do.
```

### 安全注意事项

使用 `exec` 工具的 Skill 必须防止命令注入：

```markdown
# 危险
Run `echo {{user_input}} > /tmp/output.txt`

# 安全
The user input must be shell-escaped before passing to exec.
Use `shellescape()` or wrap in single quotes, replacing ' with '\''.
```

> ⚠️ 第三方 Skill 应视为不受信任的代码。安装前务必阅读 SKILL.md 和相关脚本。

### 使用 {baseDir} 引用

在指令中使用 `{baseDir}` 引用 Skill 文件夹路径：

```markdown
---
name: my-skill
description: Uses a local helper script.
---

# My Skill

When the user asks to process data:
1. Run `{baseDir}/scripts/process.sh {{input_file}}`
2. Read the output from `{baseDir}/output/result.json`
```

---

## 配置覆盖

通过 `openclaw.json` 控制内置和已安装 Skill 的行为：

```json5
{
  skills: {
    entries: {
      "image-lab": {
        enabled: true,
        apiKey: "GEMINI_KEY_HERE",
        env: { GEMINI_API_KEY: "GEMINI_KEY_HERE" },
        config: {
          endpoint: "https://example.invalid",
          model: "nano-pro",
        },
      },
      sag: { enabled: false },
    },
  },
}
```

配置规则：

- `enabled: false` —— 即使 Skill 已安装/内置也禁用
- `env` —— 注入环境变量（仅当变量未设置时生效）
- `apiKey` —— 简写，等价于设置 `metadata.openclaw.primaryEnv` 指定的环境变量
- `config` —— 自定义配置，Skill 在指令中通过 `{config.key}` 引用

---

## 完整示例

### 示例 1：带环境门控的 Skill

```markdown
---
name: image-lab
description: Generate or edit images via a provider-backed image workflow.
metadata: {"openclaw": {"requires": {"env": ["GEMINI_API_KEY"], "config": ["browser.enabled"]}, "primaryEnv": "GEMINI_API_KEY"}}
---

# Image Lab Skill

When the user asks to generate or edit an image:

1. Determine the task type:
   - **Generation**: User wants a new image from a text prompt
   - **Editing**: User wants to modify an existing image

2. For generation:
   - Use the `image_generate` tool with the user's prompt
   - Apply any style modifiers the user mentions

3. For editing:
   - Use the `image` tool to analyze the source image first
   - Then use `image_generate` with the edit instruction

4. Always confirm the result was received before assuming success.
   If the tool returns an error, explain what went wrong and suggest alternatives.
```

### 示例 2：需要本地二进制的 Skill

```markdown
---
name: gemini
description: Use Gemini CLI for coding assistance and Google search lookups.
metadata: {"openclaw": {"emoji": "♊", "requires": {"bins": ["gemini"]}, "install": [{"id": "brew", "kind": "brew", "formula": "gemini-cli", "bins": ["gemini"], "label": "Install Gemini CLI (brew)"}]}}
---

# Gemini CLI Skill

When the user asks you to use Gemini:

1. Run `gemini "{{user_prompt}}"` via the `exec` tool.
2. Relay the output to the user, preserving code blocks and formatting.
3. If Gemini is not found, suggest installing via: `brew install gemini-cli`

For Google searches, add `--search` flag:
`gemini --search "{{query}}"`
```

### 示例 3：带安装器的 Skill

```markdown
---
name: summarize
description: Summarize long documents or conversations.
metadata: {"openclaw": {"requires": {"bins": ["summarize"]}, "install": [{"id": "brew", "kind": "brew", "formula": "summarize-cli", "bins": ["summarize"], "label": "Install Summarize CLI (brew)", "os": ["darwin"]}, {"id": "npm", "kind": "node", "package": "@openclaw/summarize-cli", "bins": ["summarize"], "label": "Install via npm"}]}}
---

# Summarize Skill

When the user asks to summarize:
1. If the input is a file path, read it first
2. Run `summarize --format markdown --max-paragraphs 5 <input>`
3. Present the summary with a word count and key topics
```

### 示例 4：用户可调用 + 直接分发

```markdown
---
name: backup
description: Create a git backup of the current project.
user-invocable: true
command-dispatch: tool
command-tool: git_backup
command-arg-mode: raw
---

# Backup Skill

This skill is invoked directly via the `/backup` command.
No model interaction needed — the command dispatches straight to the tool.
```

---

## Token 开销估算

当 Skill 有资格加载时，OpenClaw 将可用 Skill 的紧凑 XML 列表注入系统提示词。开销是确定性的：

- **基础开销**（至少 1 个 Skill 时）：195 个字符
- **每个 Skill**：97 个字符 + name、description、location 的 XML 转义长度

```
总字符数 = 195 + Σ (97 + len(name_escaped) + len(description_escaped) + len(location_escaped))
```

粗略估算（OpenAI 风格分词器，约 4 字符/token）：每个 Skill 约占 24 token 加上实际字段长度。

---

## 测试和发布

### 本地测试

```bash
# 创建 Skill 目录
mkdir -p ~/.openclaw/workspace/skills/my-skill

# 启动新会话使 Skill 生效
openclaw gateway restart

# 验证 Skill 已加载
openclaw skills list

# 测试
openclaw agent --message "test my skill"
```

### 发布到 ClawHub

```bash
# 同步（扫描 + 发布更新）
clawhub sync --all
```

访问 [https://clawhub.com](https://clawhub.com) 浏览和安装社区 Skill。
