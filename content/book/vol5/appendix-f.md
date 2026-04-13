---
appendix: F
title: "术语表"
status: draft
---

# 附录 F：术语表

本术语表收录全书使用的专业技术术语，按主题分组。

---

## 核心概念

| 术语 | 英文 | 说明 |
|:---|:---|:---|
| Agent | Agent | AI 实体，能够使用工具、记忆上下文、与用户交互的自主程序 |
| 通道 | Channel | Agent 与外部世界通讯的媒介，如 WhatsApp、Telegram、Discord 等 |
| Workspace | Workspace | Agent 的工作目录，包含引导文件、记忆和 Skill |
| 网关 | Gateway | OpenClaw 的核心服务进程，负责通道管理、消息路由和 Agent 调度 |
| 节点 | Node | 与网关配对的外部设备，可提供屏幕、摄像头、执行环境等能力 |
| 会话 | Session | Agent 与用户的连续对话上下文，存储在磁盘上 |

---

## 配置和认证

| 术语 | 英文 | 说明 |
|:---|:---|:---|
| 配对 | Pairing | DM 安全机制：未知发送者需通过一次性配对码验证 |
| 白名单 | Allowlist | 预先批准的发送者/群组列表 |
| 提及门控 | Mention Gating | 群聊中仅当 Agent 被 @提及或匹配文本模式时才回复 |
| 认证配置文件 | Auth Profile | 存储在 `auth-profiles.json` 中的认证凭据元数据 |
| SecretRef | SecretRef | 引用外部密钥存储的配置对象格式 |

---

## 模型和推理

| 术语 | 英文 | 说明 |
|:---|:---|:---|
| 主模型 | Primary Model | Agent 默认使用的 LLM |
| 备选模型 | Fallback Model | 主模型失败时自动切换的模型 |
| 思考模式 | Thinking Mode | 模型的推理深度级别（off/low/medium/high） |
| 流式输出 | Streaming | 模型生成过程中逐步返回内容（partial/block/progress） |
| 上下文窗口 | Context Window | 模型单次请求能处理的最大 token 数 |
| 压缩 | Compaction | 会话历史过长时自动摘要以适应上下文窗口 |

---

## 工具和 Skill

| 术语 | 英文 | 说明 |
|:---|:---|:---|
| Skill | Skill | 教导 Agent 如何使用工具的 Markdown 指令文件 |
| Plugin | Plugin | 扩展 OpenClaw 能力的代码包，可注册通道、工具、Provider 等 |
| 提权工具 | Elevated Tool | 具有更高权限的工具（如 Shell 命令），需用户白名单授权 |
| ClawHub | ClawHub | OpenClaw 的公共 Skill 注册表和分发平台 |
| 内置 Skill | Bundled Skill | 随 OpenClaw 安装包附带的预置 Skill |
| 门控 | Gating | 加载时过滤机制，基于 OS、二进制、环境变量等条件决定 Skill 是否可用 |

---

## 记忆和状态

| 术语 | 英文 | 说明 |
|:---|:---|:---|
| 引导文件 | Bootstrap File | Workspace 中的模板文件（SOUL.md、AGENTS.md 等），定义 Agent 行为 |
| 长期记忆 | MEMORY.md | Agent 的策划性记忆，每日日记的精华提炼 |
| 日记文件 | Daily Note | `memory/YYYY-MM-DD.md`，记录当天事件和上下文 |
| 心跳 | Heartbeat | 定期唤醒 Agent 执行检查任务的机制 |
| 记忆刷新 | Memory Flush | 压缩前自动保存重要记忆到文件 |
| 上下文修剪 | Context Pruning | 发送给 LLM 前修剪旧工具结果以节省 token |

---

## 网络和安全

| 术语 | 英文 | 说明 |
|:---|:---|:---|
| Tailscale Serve | Tailscale Serve | 通过 Tailscale 魔法 DNS 暴露本地服务的安全方式 |
| SSH 隧道 | SSH Tunnel | 通过 SSH 转发远程端口到本地 |
| 沙箱 | Sandbox | 隔离的执行环境，限制 Agent 的文件系统和网络访问 |
| 热重载 | Hot Reload | 配置文件变更后自动重新加载，无需重启 |
| 去抖动 | Debounce | 防止短时间内频繁触发（如消息队列收集） |

---

## 会话和路由

| 术语 | 英文 | 说明 |
|:---|:---|:---|
| 会话作用域 | Session Scope | 会话的隔离级别（per-sender/per-channel-peer/global） |
| 消息队列 | Message Queue | 收集多条消息后合并处理的机制 |
| 线程绑定 | Thread Binding | 将 Discord 线程与会话绑定的功能 |
| ACP Agent | ACP Agent | 通过 Agent Control Protocol 连接的外部 Agent（如 Codex） |
| 子 Agent | Subagent | 由主 Agent 生成的并行工作 Agent |

---

## 自动化

| 术语 | 英文 | 说明 |
|:---|:---|:---|
| Cron | Cron | 精确定时任务调度 |
| Webhook | Webhook | 外部事件触发 Agent 处理的 HTTP 端点 |
| BOOT.md | BOOT.md | Gateway 启动时自动执行的任务清单 |
| 自聊模式 | Self-chat Mode | 将自己的号码添加到 `allowFrom`，通过自聊控制 Agent |

---

## 缩写对照

| 缩写 | 全称 |
|:---|:---|
| CDP | Chrome DevTools Protocol |
| CLI | Command Line Interface |
| DM | Direct Message |
| ESM | ECMAScript Modules |
| JSON5 | JSON with comments and trailing commas |
| LLM | Large Language Model |
| RPC | Remote Procedure Call |
| STT | Speech-to-Text |
| TTS | Text-to-Speech |
| TUI | Terminal User Interface |
| UI | User Interface |
| VPS | Virtual Private Server |
