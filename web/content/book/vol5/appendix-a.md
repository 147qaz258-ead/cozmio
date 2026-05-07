---
appendix: A
title: "openclaw.json 配置完整参考"
status: draft
---

# 附录 A：openclaw.json 配置完整参考

本附录是 `~/.openclaw/openclaw.json` 的完整字段参考。配置格式为 **JSON5**（支持注释和尾逗号）。所有字段均可选——OpenClaw 在省略时使用安全默认值。

配置示例和常见模式参见 [附录 B](appendix-b.md)；配置文件的上下文和最佳实践参见 [第 4 章](../vol2/ch04.md)。

---

## 顶层结构概览

```json5
{
  identity: {},        // Agent 身份信息
  env: {},             // 环境变量和 Shell 配置
  auth: {},            // 认证配置
  logging: {},         // 日志设置
  messages: {},        // 消息格式化
  routing: {},         // 路由与队列
  session: {},         // 会话行为
  channels: {},        // 通道配置
  agents: {},          // Agent 运行时
  tools: {},           // 工具控制
  models: {},          // 自定义模型提供商
  cron: {},            // 定时任务
  hooks: {},           // Webhook
  gateway: {},         // 网关与网络
  skills: {},          // Skill 管理
  commands: {},        // 聊天命令
}
```

---

## identity

定义 Agent 的公开身份。这些信息会出现在系统提示词和部分通道的元数据中。

```json5
{
  identity: {
    name: "Samantha",
    theme: "helpful sloth",
    emoji: "🦥",
  },
}
```

| 字段 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `name` | `string` | `"OpenClaw"` | Agent 名称，出现在系统提示词中 |
| `theme` | `string` | - | 主题描述，用于引导 Agent 的性格倾向 |
| `emoji` | `string` | - | 签名 Emoji，用于 UI 展示 |

---

## env

环境变量管理和 Shell 环境配置。

```json5
{
  env: {
    OPENROUTER_API_KEY: "sk-or-...",
    vars: {
      GROQ_API_KEY: "gsk-...",
    },
    shellEnv: {
      enabled: true,
      timeoutMs: 15000,
    },
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| 顶层键值 | `string` | 直接注入到 `process.env` 的环境变量 |
| `vars` | `object` | 额外环境变量，语义等同于顶层键值 |
| `shellEnv.enabled` | `boolean` | 是否启用 Shell 环境加载 |
| `shellEnv.timeoutMs` | `number` | Shell 环境加载超时时间（毫秒） |

---

## auth

认证配置。密钥存储在 `auth-profiles.json` 中，此处仅定义配置文件的元数据和优先级。

```json5
{
  auth: {
    profiles: {
      "anthropic:me@example.com": {
        provider: "anthropic",
        mode: "oauth",
        email: "me@example.com",
      },
      "anthropic:work": { provider: "anthropic", mode: "api_key" },
      "openai:default": { provider: "openai", mode: "api_key" },
    },
    order: {
      anthropic: ["anthropic:me@example.com", "anthropic:work"],
      openai: ["openai:default"],
    },
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `profiles` | `object` | 认证配置文件，键格式为 `<provider>:<id>` |
| `profiles.<key>.provider` | `string` | 提供商标识（`anthropic`、`openai` 等） |
| `profiles.<key>.mode` | `string` | 认证模式：`oauth` 或 `api_key` |
| `profiles.<key>.email` | `string` | OAuth 模式下的账户邮箱 |
| `order` | `object` | 按提供商分组的配置文件优先级列表 |

> 💡 `auth.order` 定义了同一提供商下多个配置文件的尝试顺序。主配置失败时，OpenClaw 按列表顺序尝试下一个。

---

## logging

日志输出配置。

```json5
{
  logging: {
    level: "info",
    file: "/tmp/openclaw/openclaw.log",
    consoleLevel: "info",
    consoleStyle: "pretty",
    redactSensitive: "tools",
  },
}
```

| 字段 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `level` | `string` | `"info"` | 文件日志级别 |
| `file` | `string` | - | 日志文件路径 |
| `consoleLevel` | `string` | `"info"` | 控制台日志级别 |
| `consoleStyle` | `string` | `"pretty"` | 控制台输出样式 |
| `redactSensitive` | `string` | - | 敏感信息脱敏策略 |

---

## messages

消息格式化和响应行为。

```json5
{
  messages: {
    messagePrefix: "[openclaw]",
    responsePrefix: ">",
    ackReaction: "👀",
    ackReactionScope: "group-mentions",
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `messagePrefix` | `string` | 用户消息前缀 |
| `responsePrefix` | `string` | 响应消息前缀 |
| `ackReaction` | `string` | 收到消息时的确认 Reaction |
| `ackReactionScope` | `string` | 确认 Reaction 的作用范围 |

---

## routing

消息路由、队列和群聊配置。

```json5
{
  routing: {
    groupChat: {
      mentionPatterns: ["@openclaw", "openclaw"],
      historyLimit: 50,
    },
    queue: {
      mode: "collect",
      debounceMs: 1000,
      cap: 20,
      drop: "summarize",
      byChannel: {
        whatsapp: "collect",
        telegram: "collect",
        discord: "collect",
        slack: "collect",
      },
    },
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `groupChat.mentionPatterns` | `string[]` | 触发 Agent 回复的文本匹配模式 |
| `groupChat.historyLimit` | `number` | 群聊历史消息加载数量 |
| `queue.mode` | `string` | 消息队列模式：`collect`（收集合并）、`immediate`（立即处理） |
| `queue.debounceMs` | `number` | 队列去抖动时间（毫秒） |
| `queue.cap` | `number` | 单次队列最大消息数 |
| `queue.drop` | `string` | 超出上限时的处理策略：`summarize`（摘要）、`tail`（保留尾部） |
| `queue.byChannel` | `object` | 按通道覆盖队列模式 |

---

## session

会话作用域、重置策略和存储配置。

```json5
{
  session: {
    scope: "per-sender",
    reset: {
      mode: "daily",
      atHour: 4,
      idleMinutes: 60,
    },
    resetTriggers: ["/new", "/reset"],
    store: "~/.openclaw/agents/default/sessions/sessions.json",
    maintenance: {
      mode: "warn",
      pruneAfter: "30d",
      maxEntries: 500,
      rotateBytes: "10mb",
      maxDiskBytes: "500mb",
    },
    typingIntervalSeconds: 5,
    sendPolicy: {
      default: "allow",
      rules: [
        { action: "deny", match: { channel: "discord", chatType: "group" } },
      ],
    },
  },
}
```

| 字段 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `scope` | `string` | `"per-sender"` | 会话作用域 |
| `dmScope` | `string` | - | DM 专用作用域，覆盖 `scope` |
| `reset.mode` | `string` | `"daily"` | 重置模式：`daily`、`idle`、`manual`、`off` |
| `reset.atHour` | `number` | `4` | 每日重置的小时（UTC，0-23） |
| `reset.idleMinutes` | `number` | - | 空闲重置的分钟数 |
| `resetTriggers` | `string[]` | `["/new", "/reset"]` | 触发重置的命令列表 |
| `store` | `string` | - | 会话存储文件路径 |
| `maintenance.pruneAfter` | `string` | `"30d"` | 会话清理周期 |
| `maintenance.maxEntries` | `number` | `500` | 最大会话条目数 |
| `maintenance.maxDiskBytes` | `string` | - | 会话存储磁盘上限 |
| `typingIntervalSeconds` | `number` | `5` | 打字指示器间隔（秒） |

会话作用域的可选值：

| 作用域 | 行为 |
|:---|:---|
| `per-sender` | 同一发送者共享会话（默认） |
| `per-channel-peer` | 按通道+对等方隔离 |
| `global` | 所有来源共享同一会话 |

> ⚠️ 多用户 DM 场景下推荐 `dmScope: "per-channel-peer"`，可防止不同发送者共享上下文。

---

## channels

通道配置是 `openclaw.json` 中最复杂的部分。每个通道在配置段存在时自动启动（除非显式设置 `enabled: false`）。

### DM 和群聊策略

所有通道共享以下策略模型：

| DM 策略 | 行为 |
|:---|:---|
| `pairing`（默认） | 未知发送者收到一次性配对码，需管理员批准 |
| `allowlist` | 仅允许 `allowFrom` 列表中的发送者 |
| `open` | 允许所有 DM（需配合 `allowFrom: ["*"]`） |
| `disabled` | 忽略所有 DM |

| 群聊策略 | 行为 |
|:---|:---|
| `allowlist`（默认） | 仅允许配置的群组 |
| `open` | 跳过群组白名单（提及门控仍生效） |
| `disabled` | 阻止所有群组消息 |

配对码在 1 小时后过期。待处理的 DM 配对请求上限为每个通道 3 个。

### 通道默认值和心跳

```json5
{
  channels: {
    defaults: {
      groupPolicy: "allowlist",
      heartbeat: {
        showOk: false,
        showAlerts: true,
        useIndicator: true,
      },
    },
  },
}
```

- `defaults.groupPolicy`：当提供商级别 `groupPolicy` 未设置时的回退策略。
- `defaults.heartbeat.showOk`：心跳输出中是否包含健康通道状态。
- `defaults.heartbeat.showAlerts`：是否包含异常/错误状态。
- `defaults.heartbeat.useIndicator`：是否使用紧凑的指示器样式。

### WhatsApp

```json5
{
  channels: {
    whatsapp: {
      dmPolicy: "pairing",
      allowFrom: ["+15555550123", "+447700900123"],
      textChunkLimit: 4000,
      chunkMode: "length",
      mediaMaxMb: 50,
      sendReadReceipts: true,
      groups: { "*": { requireMention: true } },
      groupPolicy: "allowlist",
      groupAllowFrom: ["+15551234567"],
    },
  },
}
```

| 字段 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `dmPolicy` | `string` | `"pairing"` | DM 策略 |
| `allowFrom` | `string[]` | `[]` | 允许的发送者列表 |
| `textChunkLimit` | `number` | `4000` | 文本分块上限（字符） |
| `chunkMode` | `string` | `"length"` | 分块模式：`length`、`newline` |
| `mediaMaxMb` | `number` | `50` | 媒体文件大小上限（MB） |
| `sendReadReceipts` | `boolean` | `true` | 是否发送已读回执（自聊模式下默认 `false`） |
| `groups` | `object` | - | 群组配置，`"*"` 匹配所有群组 |
| `groupPolicy` | `string` | `"allowlist"` | 群聊策略 |
| `groupAllowFrom` | `string[]` | `[]` | 允许的群组成员 |

WhatsApp 通过网关的 Web 通道（Baileys Web）运行，当已链接会话存在时自动启动。支持多账户配置：

```json5
{
  channels: {
    whatsapp: {
      accounts: {
        default: {},
        personal: {},
        biz: {},
      },
    },
  },
}
```

### Telegram

```json5
{
  channels: {
    telegram: {
      enabled: true,
      botToken: "your-bot-token",
      dmPolicy: "pairing",
      allowFrom: ["tg:123456789"],
      groups: {
        "*": { requireMention: true },
        "-1001234567890": {
          allowFrom: ["@admin"],
          systemPrompt: "Keep answers brief.",
          topics: {
            "99": {
              requireMention: false,
              skills: ["search"],
            },
          },
        },
      },
      customCommands: [
        { command: "backup", description: "Git backup" },
      ],
      historyLimit: 50,
      replyToMode: "first",
      linkPreview: true,
      streaming: "partial",
      actions: { reactions: true, sendMessage: true },
      mediaMaxMb: 100,
      proxy: "socks5://localhost:9050",
      retry: {
        attempts: 3,
        minDelayMs: 400,
        maxDelayMs: 30000,
        jitter: 0.1,
      },
    },
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `botToken` | `string` | Bot Token，也可通过 `tokenFile` 或环境变量 `TELEGRAM_BOT_TOKEN` |
| `replyToMode` | `string` | 回复引用模式：`off`、`first`、`all` |
| `streaming` | `string` | 流式输出：`off`、`partial`、`block`、`progress` |
| `proxy` | `string` | 代理地址（如 `socks5://localhost:9050`） |
| `customCommands` | `object[]` | 自定义 Bot 菜单命令 |
| `reactionNotifications` | `string` | Reaction 通知模式：`off`、`own`、`all` |
| `retry` | `object` | API 调用重试策略 |

### Discord

```json5
{
  channels: {
    discord: {
      enabled: true,
      token: "your-bot-token",
      mediaMaxMb: 8,
      allowBots: false,
      actions: {
        reactions: true,
        stickers: true,
        polls: true,
        messages: true,
        threads: true,
        pins: true,
        search: true,
        memberInfo: true,
        voiceStatus: true,
      },
      replyToMode: "off",
      dmPolicy: "pairing",
      dm: { enabled: true, groupEnabled: false, groupChannels: ["openclaw-dm"] },
      guilds: {
        "123456789012345678": {
          slug: "friends-of-openclaw",
          requireMention: false,
          ignoreOtherMentions: true,
          channels: {
            general: { allow: true },
            help: { allow: true, requireMention: true, skills: ["docs"] },
          },
        },
      },
      textChunkLimit: 2000,
      maxLinesPerMessage: 17,
      streaming: "off",
      voice: {
        enabled: true,
        autoJoin: [{ guildId: "123", channelId: "234" }],
        daveEncryption: true,
        tts: { provider: "openai", openai: { voice: "alloy" } },
      },
    },
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `guilds` | `object` | 服务器配置，键为服务器 ID |
| `guilds.<id>.slug` | `string` | 服务器 URL 友好名称 |
| `guilds.<id>.ignoreOtherMentions` | `boolean` | 忽略提及他人但未提及 Bot 的消息 |
| `allowBots` | `boolean` | 是否接受 Bot 消息（`"mentions"` 仅接受提及 Bot 的 Bot 消息） |
| `maxLinesPerMessage` | `number` | 消息最大行数（默认 17），即使未达 2000 字符也会拆分 |
| `voice.enabled` | `boolean` | 是否启用 Discord 语音频道对话 |

### Slack

```json5
{
  channels: {
    slack: {
      enabled: true,
      botToken: "xoxb-...",
      appToken: "xapp-...",
      dmPolicy: "pairing",
      channels: {
        "#general": { allow: true, requireMention: true },
      },
      dm: { enabled: true, groupEnabled: false, groupChannels: ["G123"] },
      slashCommand: {
        enabled: true,
        name: "openclaw",
        sessionPrefix: "slack:slash",
        ephemeral: true,
      },
      streaming: "partial",
      nativeStreaming: true,
      mediaMaxMb: 20,
      typingReaction: "hourglass_flowing_sand",
      thread: {
        historyScope: "thread",
        inheritParent: false,
      },
    },
  },
}
```

Socket Mode 需要 `botToken`（`xoxb-`）和 `appToken`（`xapp-`）两个令牌。HTTP Mode 需要 `botToken` 加 `signingSecret`。

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `slashCommand.ephemeral` | `boolean` | 斜杠命令响应是否仅发起者可见 |
| `nativeStreaming` | `boolean` | 是否使用 Slack 原生流式 API |
| `typingReaction` | `string` | 处理中显示的临时 Reaction |
| `thread.historyScope` | `string` | 线程会话隔离：`thread`（默认）、`channel` |

### Signal

```json5
{
  channels: {
    signal: {
      enabled: true,
      account: "+15555550123",
      dmPolicy: "pairing",
      allowFrom: ["+15551234567"],
      reactionNotifications: "own",
      historyLimit: 50,
    },
  },
}
```

### iMessage

```json5
{
  channels: {
    imessage: {
      enabled: true,
      cliPath: "imsg",
      dbPath: "~/Library/Messages/chat.db",
      remoteHost: "user@gateway-host",
      dmPolicy: "pairing",
      allowFrom: ["+15555550123", "user@example.com", "chat_id:123"],
      historyLimit: 50,
      includeAttachments: false,
      mediaMaxMb: 16,
    },
  },
}
```

iMessage 需要 Full Disk Access 权限。`cliPath` 可指向 SSH 包装脚本。优先使用 `chat_id:<id>` 格式的目标。

### 其他扩展通道

Mattermost、Microsoft Teams、IRC、Matrix、Google Chat、BlueBubbles、LINE、Nostr 等通道均以 Plugin 形式提供，配置方式参见各自的通道文档页面。

### 通道模型覆盖

使用 `channels.modelByChannel` 将特定通道绑定到指定模型：

```json5
{
  channels: {
    modelByChannel: {
      discord: {
        "123456789012345678": "anthropic/claude-opus-4-6",
      },
      telegram: {
        "-1001234567890": "openai/gpt-4.1-mini",
        "-1001234567890:topic:99": "anthropic/claude-sonnet-4-6",
      },
    },
  },
}
```

### 多账户配置

所有通道均支持多账户：

```json5
{
  channels: {
    telegram: {
      accounts: {
        default: { botToken: "123456:ABC..." },
        alerts: { botToken: "987654:XYZ..." },
      },
    },
  },
}
```

`default` 账户在未指定 `accountId` 时使用。环境变量令牌仅应用于默认账户。

---

## agents

Agent 运行时配置，包含默认参数和每个 Agent 的独立覆盖。

### agents.defaults

```json5
{
  agents: {
    defaults: {
      workspace: "~/.openclaw/workspace",
      userTimezone: "America/Chicago",
      timeFormat: "auto",
      model: {
        primary: "anthropic/claude-sonnet-4-6",
        fallbacks: ["anthropic/claude-opus-4-6", "openai/gpt-5.2"],
      },
      imageModel: {
        primary: "openrouter/anthropic/claude-sonnet-4-6",
      },
      imageGenerationModel: {
        primary: "openai/gpt-image-1",
      },
      pdfModel: {
        primary: "anthropic/claude-opus-4-6",
      },
      models: {
        "anthropic/claude-opus-4-6": { alias: "opus" },
        "anthropic/claude-sonnet-4-6": { alias: "sonnet" },
        "openai/gpt-5.2": { alias: "gpt" },
      },
      thinkingDefault: "low",
      verboseDefault: "off",
      elevatedDefault: "on",
      timeoutSeconds: 600,
      mediaMaxMb: 5,
      maxConcurrent: 3,
      skipBootstrap: false,
      bootstrapMaxChars: 20000,
      bootstrapTotalMaxChars: 150000,
    },
  },
}
```

| 字段 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `workspace` | `string` | `"~/.openclaw/workspace"` | Workspace 路径 |
| `userTimezone` | `string` | 系统时区 | 系统提示词中的时区 |
| `timeFormat` | `string` | `"auto"` | 时间格式：`auto`、`12`、`24` |
| `model.primary` | `string` | - | 主模型，格式 `provider/model` |
| `model.fallbacks` | `string[]` | - | 有序备选模型列表 |
| `imageModel.primary` | `string` | - | 图像分析模型 |
| `imageGenerationModel.primary` | `string` | - | 图像生成模型 |
| `pdfModel.primary` | `string` | - | PDF 工具模型路由 |
| `models` | `object` | - | 模型目录，支持 `alias` 和 `params` |
| `thinkingDefault` | `string` | `"low"` | 思考级别：`off`、`low`、`medium`、`high` |
| `maxConcurrent` | `number` | `1` | 最大并行 Agent 运行数 |
| `skipBootstrap` | `boolean` | `false` | 跳过自动创建 Workspace 引导文件 |
| `bootstrapMaxChars` | `number` | `20000` | 单个引导文件最大字符数 |
| `bootstrapTotalMaxChars` | `number` | `150000` | 所有引导文件总字符数上限 |
| `imageMaxDimensionPx` | `number` | `1200` | 图像最长边的最大像素数 |

模型格式为 `provider/model`，省略提供商时默认为 `anthropic`（已弃用）。内置别名仅在模型存在于 `agents.defaults.models` 中时生效：

| 别名 | 模型 |
|:---|:---|
| `opus` | `anthropic/claude-opus-4-6` |
| `sonnet` | `anthropic/claude-sonnet-4-6` |
| `gpt` | `openai/gpt-5.4` |
| `gpt-mini` | `openai/gpt-5-mini` |
| `gemini` | `google/gemini-3.1-pro-preview` |
| `gemini-flash` | `google/gemini-3-flash-preview` |

用户配置的别名始终优先于内置别名。

### agents.defaults.heartbeat

```json5
{
  agents: {
    defaults: {
      heartbeat: {
        every: "30m",
        model: "openai/gpt-5.2-mini",
        target: "last",
        directPolicy: "allow",
        to: "+15555550123",
        prompt: "HEARTBEAT",
        ackMaxChars: 300,
        isolatedSession: false,
        lightContext: false,
        suppressToolErrorWarnings: false,
      },
    },
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `every` | `string` | 心跳间隔（`"0m"` 禁用），支持 `ms`/`s`/`m`/`h` 单位 |
| `model` | `string` | 心跳使用的模型（可独立于主模型） |
| `target` | `string` | 投递目标：`last`、`none` 或指定通道名 |
| `directPolicy` | `string` | DM 投递策略：`allow`、`block` |
| `isolatedSession` | `boolean` | 隔离会话运行（每次心跳 ~2-5K token，而非 ~100K） |
| `lightContext` | `boolean` | 仅加载 HEARTBEAT.md 引导文件 |

### agents.defaults.sandbox

```json5
{
  agents: {
    defaults: {
      sandbox: {
        mode: "non-main",
        backend: "docker",
        scope: "agent",
        workspaceAccess: "none",
        workspaceRoot: "~/.openclaw/sandboxes",
        docker: {
          image: "openclaw-sandbox:bookworm-slim",
          workdir: "/workspace",
          readOnlyRoot: true,
          tmpfs: ["/tmp", "/var/tmp", "/run"],
          network: "none",
          user: "1000:1000",
          capDrop: ["ALL"],
          memory: "1g",
          memorySwap: "2g",
          cpus: 1,
          setupCommand: "apt-get update && apt-get install -y git curl jq",
          dns: ["1.1.1.1", "8.8.8.8"],
          binds: ["/home/user/source:/source:rw"],
        },
      },
    },
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `mode` | `string` | 沙箱模式：`off`、`non-main`、`all` |
| `backend` | `string` | 后端：`docker`、`ssh`、`openshell` |
| `scope` | `string` | 作用域：`session`、`agent`、`shared` |
| `workspaceAccess` | `string` | Workspace 访问：`none`、`ro`、`rw` |
| `docker.network` | `string` | 容器网络，默认 `"none"` |
| `docker.setupCommand` | `string` | 容器创建后执行一次的安装命令 |
| `docker.binds` | `string[]` | 额外主机目录挂载 |

> ⚠️ 容器默认 `network: "none"`。Agent 需要出站网络时设为 `"bridge"`。`"host"` 被阻止。

### agents.defaults.compaction

```json5
{
  agents: {
    defaults: {
      compaction: {
        mode: "safeguard",
        timeoutSeconds: 900,
        reserveTokensFloor: 24000,
        identifierPolicy: "strict",
        model: "openrouter/anthropic/claude-sonnet-4-6",
        memoryFlush: {
          enabled: true,
          softThresholdTokens: 6000,
          systemPrompt: "Session nearing compaction. Store durable memories now.",
        },
      },
    },
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `mode` | `string` | 压缩模式：`default`、`safeguard` |
| `identifierPolicy` | `string` | 标识符保留：`strict`、`off`、`custom` |
| `memoryFlush.enabled` | `boolean` | 压缩前静默保存记忆 |

### agents.defaults.contextPruning

在发送给 LLM 之前修剪旧的工具结果。不修改磁盘上的会话历史。

```json5
{
  agents: {
    defaults: {
      contextPruning: {
        mode: "cache-ttl",
        ttl: "1h",
        keepLastAssistants: 3,
        softTrimRatio: 0.3,
        hardClearRatio: 0.5,
        tools: { deny: ["browser", "canvas"] },
      },
    },
  },
}
```

### agents.list

每个 Agent 的独立覆盖配置：

```json5
{
  agents: {
    list: [
      {
        id: "main",
        default: true,
        model: "anthropic/claude-opus-4-6",
        thinkingDefault: "high",
        reasoningDefault: "on",
        fastModeDefault: false,
        identity: {
          name: "Samantha",
          theme: "helpful sloth",
          emoji: "🦥",
        },
        groupChat: { mentionPatterns: ["@openclaw"] },
        sandbox: { mode: "off" },
      },
      {
        id: "quick",
        fastModeDefault: true,
        thinkingDefault: "off",
      },
    ],
  },
}
```

---

## tools

工具访问控制和执行参数。

```json5
{
  tools: {
    allow: ["exec", "process", "read", "write", "edit", "apply_patch"],
    deny: ["browser", "canvas"],
    exec: {
      backgroundMs: 10000,
      timeoutSec: 1800,
      cleanupMs: 1800000,
    },
    elevated: {
      enabled: true,
      allowFrom: {
        whatsapp: ["+15555550123"],
        telegram: ["123456789"],
        discord: ["123456789012345678"],
      },
    },
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `allow` | `string[]` | 允许的工具列表 |
| `deny` | `string[]` | 禁止的工具列表 |
| `exec.backgroundMs` | `number` | 后台命令阈值（毫秒），超过此值自动后台执行 |
| `exec.timeoutSec` | `number` | 命令执行超时（秒） |
| `exec.cleanupMs` | `number` | 后台进程清理时间（毫秒） |
| `elevated.enabled` | `boolean` | 是否启用提权工具（`!` Shell 命令等） |
| `elevated.allowFrom` | `object` | 按通道的提权用户白名单 |

---

## models

自定义模型提供商注册。

```json5
{
  models: {
    mode: "merge",
    providers: {
      "custom-proxy": {
        baseUrl: "http://localhost:4000/v1",
        apiKey: "LITELLM_KEY",
        api: "openai-responses",
        authHeader: true,
        headers: { "X-Proxy-Region": "us-west" },
        models: [
          {
            id: "llama-3.1-8b",
            name: "Llama 3.1 8B",
            api: "openai-responses",
            reasoning: false,
            input: ["text"],
            cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
            contextWindow: 128000,
            maxTokens: 32000,
          },
        ],
      },
    },
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `mode` | `string` | `"merge"`（与内置合并）或 `"replace"`（完全替换） |
| `providers.<id>.baseUrl` | `string` | API 基础 URL |
| `providers.<id>.api` | `string` | API 协议：`openai-responses`、`anthropic-messages` 等 |
| `providers.<id>.apiKey` | `string` | API Key，支持环境变量引用 `${ENV_VAR}` |
| `providers.<id>.models` | `object[]` | 模型列表 |
| `models[].contextWindow` | `number` | 上下文窗口大小（token） |
| `models[].maxTokens` | `number` | 最大输出 token 数 |

---

## cron

定时任务调度配置。

```json5
{
  cron: {
    enabled: true,
    store: "~/.openclaw/cron/cron.json",
    maxConcurrentRuns: 2,
    sessionRetention: "24h",
    runLog: {
      maxBytes: "2mb",
      keepLines: 2000,
    },
  },
}
```

---

## hooks

Webhook 和事件钩子配置。

```json5
{
  hooks: {
    enabled: true,
    path: "/hooks",
    token: "shared-secret",
    transformsDir: "~/.openclaw/hooks/transforms",
    mappings: [
      {
        id: "gmail-hook",
        match: { path: "gmail" },
        action: "agent",
        wakeMode: "now",
        name: "Gmail",
        sessionKey: "hook:gmail:{{messages[0].id}}",
        messageTemplate: "From: {{messages[0].from}}\nSubject: {{messages[0].subject}}",
        textTemplate: "{{messages[0].snippet}}",
        deliver: true,
        channel: "last",
        to: "+15555550123",
        thinking: "low",
        timeoutSeconds: 300,
      },
    ],
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `enabled` | `boolean` | 是否启用 Webhook |
| `path` | `string` | Webhook 挂载路径 |
| `token` | `string` | Webhook 验证令牌 |
| `mappings` | `object[]` | 事件到 Agent 的映射规则 |
| `mappings[].action` | `string` | 触发动作：`agent`、`deliver` |
| `mappings[].sessionKey` | `string` | 会话键模板（支持 Mustache 语法） |

---

## gateway

网关服务、网络绑定和认证。

```json5
{
  gateway: {
    mode: "local",
    port: 18789,
    bind: "loopback",
    controlUi: { enabled: true, basePath: "/openclaw" },
    auth: {
      mode: "token",
      token: "gateway-token",
      allowTailscale: true,
    },
    tailscale: { mode: "serve", resetOnExit: false },
    remote: { url: "ws://gateway.tailnet:18789", token: "remote-token" },
    reload: { mode: "hybrid", debounceMs: 300 },
  },
}
```

| 字段 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `mode` | `string` | `"local"` | 网关模式：`local`、`remote` |
| `port` | `number` | `18789` | 监听端口 |
| `bind` | `string` | `"loopback"` | 绑定地址：`loopback`、`tailnet`、`all` |
| `controlUi.enabled` | `boolean` | `true` | 是否启用控制面板 UI |
| `controlUi.basePath` | `string` | - | UI 基础路径 |
| `auth.mode` | `string` | `"token"` | 认证模式 |
| `auth.token` | `string` | - | 认证令牌 |
| `auth.allowTailscale` | `boolean` | `true` | 是否信任 Tailscale 身份头 |
| `reload.mode` | `string` | `"hybrid"` | 配置热重载模式 |
| `reload.debounceMs` | `number` | `300` | 重载去抖动时间 |

---

## skills

Skill 发现、加载和配置。

```json5
{
  skills: {
    allowBundled: ["gemini", "peekaboo"],
    load: {
      extraDirs: ["~/Projects/agent-scripts/skills"],
      watch: true,
      watchDebounceMs: 250,
    },
    install: {
      preferBrew: true,
      nodeManager: "npm",
    },
    entries: {
      "image-lab": {
        enabled: true,
        apiKey: "GEMINI_KEY_HERE",
        env: { GEMINI_API_KEY: "GEMINI_KEY_HERE" },
      },
      peekaboo: { enabled: true },
      sag: { enabled: false },
    },
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `allowBundled` | `string[]` | 内置 Skill 白名单，未列出则禁用 |
| `load.extraDirs` | `string[]` | 额外 Skill 搜索目录（最低优先级） |
| `load.watch` | `boolean` | 是否监听 Skill 文件变更（自动热重载） |
| `entries.<name>.enabled` | `boolean` | 启用或禁用指定 Skill |
| `entries.<name>.apiKey` | `string` | Skill API Key（支持 SecretRef） |
| `entries.<name>.env` | `object` | 注入的环境变量（仅当变量未设置时生效） |

---

## commands

聊天命令控制。

```json5
{
  commands: {
    native: "auto",
    text: true,
    bash: false,
    bashForegroundMs: 2000,
    config: false,
    debug: false,
    restart: false,
    allowFrom: {
      "*": ["user1"],
      discord: ["user:123"],
    },
    useAccessGroups: true,
  },
}
```

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `native` | `string` | 原生命令注册：`auto`（Discord/Telegram 启用，Slack 关闭）、`true`、`false` |
| `text` | `boolean` | 是否解析文本 `/` 命令 |
| `bash` | `boolean` | 是否启用 `!` Shell 命令（需 `tools.elevated.enabled`） |
| `config` | `boolean` | 是否允许 `/config` 命令 |
| `restart` | `boolean` | 是否允许 `/restart` 和网关重启工具 |
| `allowFrom` | `object` | 按通道的命令授权用户 |

---

## 群聊提及门控

群聊消息默认需要提及才触发回复（平台元数据提及或文本模式匹配）。适用于 WhatsApp、Telegram、Discord、Google Chat 和 iMessage 群聊。

提及类型：

- **元数据提及**：平台原生 @-提及。WhatsApp 自聊模式下忽略。
- **文本模式**：`agents.list[].groupChat.mentionPatterns` 中的安全正则表达式。

```json5
{
  messages: {
    groupChat: { historyLimit: 50 },
  },
  agents: {
    list: [{ id: "main", groupChat: { mentionPatterns: ["@openclaw", "openclaw"] } }],
  },
}
```

`messages.groupChat.historyLimit` 设置全局默认值，通道可通过 `channels.<channel>.historyLimit` 覆盖。设为 `0` 禁用历史加载。

DM 历史限制支持按发送者覆盖（`telegram`、`whatsapp`、`discord`、`slack`、`signal`、`imessage`）：

```json5
{
  channels: {
    telegram: {
      dmHistoryLimit: 30,
      dms: { "123456789": { historyLimit: 50 } },
    },
  },
}
```
