---
appendix: D
title: "Plugin SDK 参考"
status: draft
---

# 附录 D：Plugin SDK 参考

本附录是 OpenClaw Plugin SDK 的快速参考卡，涵盖导入约定、注册 API、入口点定义和测试方法。

关于 Plugin 架构的深入讲解，参见 [第 8 章](../vol3/ch08.md)；关于 Plugin 构建实战，参见 [第 9 章](../vol3/ch09.md)。

---

## 导入约定

始终从特定的子路径导入，而非使用整体根路径：

```typescript
// 正确：从子路径导入
import { definePluginEntry } from "openclaw/plugin-sdk/plugin-entry";
import { createPluginRuntimeStore } from "openclaw/plugin-sdk/runtime-store";

// 错误：整体根路径导入（已弃用）
import { ... } from "openclaw/plugin-sdk";
```

> ⚠️ 永远不要在生产代码中通过 `openclaw/plugin-sdk/<your-plugin>` 导入自己的 Plugin。内部导入应通过 `./api.ts` 或 `./runtime-api.ts` 路由。

---

## 常用子路径速查

### 入口点

| 子路径 | 核心导出 |
|:---|:---|
| `plugin-sdk/plugin-entry` | `definePluginEntry` |
| `plugin-sdk/core` | `defineChannelPluginEntry`、`createChatChannelPlugin`、`createChannelPluginBase` |

### 通道

| 子路径 | 核心导出 |
|:---|:---|
| `plugin-sdk/channel-setup` | `createOptionalChannelSetupSurface` |
| `plugin-sdk/channel-pairing` | `createChannelPairingController` |
| `plugin-sdk/channel-reply-pipeline` | `createChannelReplyPipeline` |
| `plugin-sdk/channel-config-helpers` | `createHybridChannelConfigAdapter` |
| `plugin-sdk/channel-policy` | `resolveChannelGroupRequireMention` |
| `plugin-sdk/channel-lifecycle` | `createAccountStatusSink` |
| `plugin-sdk/channel-inbound` | Debounce、mention 匹配、envelope 辅助函数 |
| `plugin-sdk/channel-actions` | `createMessageToolButtonsSchema`、`createMessageToolCardSchema` |
| `plugin-sdk/channel-targets` | Target 解析/匹配辅助函数 |
| `plugin-sdk/channel-contract` | Channel contract 类型 |
| `plugin-sdk/channel-feedback` | Feedback/reaction 连线 |

### 提供商

| 子路径 | 核心导出 |
|:---|:---|
| `plugin-sdk/provider-auth` | `createProviderApiKeyAuthMethod`、`ensureApiKeyFromOptionEnvOrPrompt` |
| `plugin-sdk/provider-models` | `normalizeModelCompat` |
| `plugin-sdk/provider-catalog` | Catalog 类型重导出 |
| `plugin-sdk/provider-usage` | `fetchClaudeUsage` 等 |
| `plugin-sdk/provider-stream` | Stream wrapper 类型 |
| `plugin-sdk/cli-backend` | CLI backend 默认值 + watchdog 常量 |

### 运行时和存储

| 子路径 | 核心导出 |
|:---|:---|
| `plugin-sdk/runtime-store` | `createPluginRuntimeStore` |
| `plugin-sdk/config-runtime` | Config 加载/写入辅助函数 |
| `plugin-sdk/infra-runtime` | 系统事件/心跳辅助函数 |
| `plugin-sdk/agent-runtime` | Agent 目录/身份/Workspace 辅助函数 |
| `plugin-sdk/directory-runtime` | Config 支持的目录查询/去重 |
| `plugin-sdk/keyed-async-queue` | `KeyedAsyncQueue` |

### 安全和工具

| 子路径 | 核心导出 |
|:---|:---|
| `plugin-sdk/command-auth` | `resolveControlCommandGate` |
| `plugin-sdk/allow-from` | `formatAllowFromLowercase` |
| `plugin-sdk/secret-input` | Secret input 解析辅助函数 |
| `plugin-sdk/webhook-ingress` | Webhook 请求/target 辅助函数 |

### 能力和测试

| 子路径 | 核心导出 |
|:---|:---|
| `plugin-sdk/image-generation` | 图像生成 Provider 类型 |
| `plugin-sdk/media-understanding` | 媒体理解 Provider 类型 |
| `plugin-sdk/speech` | 语音 Provider 类型 |
| `plugin-sdk/testing` | `installCommonResolveTargetErrorCases`、`shouldAckReaction` |

---

## 注册 API

`register(api)` 回调接收一个 `OpenClawPluginApi` 对象。以下按功能分类列出所有注册方法。

### 能力注册

| 方法 | 注册内容 |
|:---|:---|
| `api.registerProvider(...)` | 文本推理（LLM） |
| `api.registerCliBackend(...)` | 本地 CLI 推理后端 |
| `api.registerChannel(...)` | 消息通道 |
| `api.registerSpeechProvider(...)` | 语音合成 / STT |
| `api.registerMediaUnderstandingProvider(...)` | 图像/音频/视频分析 |
| `api.registerImageGenerationProvider(...)` | 图像生成 |
| `api.registerWebSearchProvider(...)` | 网络搜索 |

### 工具和命令

| 方法 | 注册内容 |
|:---|:---|
| `api.registerTool(tool, opts?)` | Agent 工具（必需或可选） |
| `api.registerCommand(def)` | 自定义命令（跳过 LLM） |

### 基础设施

| 方法 | 注册内容 |
|:---|:---|
| `api.registerHook(events, handler, opts?)` | 事件钩子 |
| `api.registerHttpRoute(params)` | Gateway HTTP 端点 |
| `api.registerGatewayMethod(name, handler)` | Gateway RPC 方法 |
| `api.registerCli(registrar, opts?)` | CLI 子命令 |
| `api.registerService(service)` | 后台服务 |
| `api.registerInteractiveHandler(registration)` | 交互处理器 |

### 独占槽位

| 方法 | 注册内容 |
|:---|:---|
| `api.registerContextEngine(id, factory)` | 上下文引擎（同时仅一个活跃） |
| `api.registerMemoryPromptSection(builder)` | 记忆提示词段落构建器 |
| `api.registerMemoryFlushPlan(resolver)` | 记忆刷新计划解析器 |
| `api.registerMemoryRuntime(runtime)` | 记忆运行时适配器 |
| `api.registerMemoryEmbeddingProvider(adapter)` | 记忆嵌入适配器 |

### 生命周期

| 方法 | 用途 |
|:---|:---|
| `api.on(hookName, handler, opts?)` | 类型化生命周期钩子 |
| `api.onConversationBindingResolved(handler)` | 会话绑定回调 |

---

## API 对象字段

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `api.id` | `string` | Plugin ID |
| `api.name` | `string` | 显示名称 |
| `api.version` | `string?` | Plugin 版本 |
| `api.description` | `string?` | Plugin 描述 |
| `api.source` | `string` | Plugin 源路径 |
| `api.rootDir` | `string?` | Plugin 根目录 |
| `api.config` | `OpenClawConfig` | 当前配置快照 |
| `api.pluginConfig` | `Record<string, unknown>` | Plugin 专属配置 |
| `api.runtime` | `PluginRuntime` | 运行时辅助 |
| `api.logger` | `PluginLogger` | 作用域日志（`debug`、`info`、`warn`、`error`） |
| `api.registrationMode` | `string` | `"full"`、`"setup-only"` 或 `"setup-runtime"` |
| `api.resolvePath(input)` | `(string) => string` | 解析相对于 Plugin 根目录的路径 |

---

## Hook 决策语义

Hook 返回值的语义至关重要：

**`before_tool_call`：**

- `{ block: true }` —— 终止决策，跳过低优先级处理程序
- `{ block: false }` —— 不做决策（等同于省略 `block`）
- 不返回 `block` —— 不做决策

**`message_sending`：**

- `{ cancel: true }` —— 终止决策，跳过低优先级处理程序
- `{ cancel: false }` —— 不做决策（等同于省略 `cancel`）
- 不返回 `cancel` —— 不做决策

---

## 快速上手：工具 Plugin

### 1. 创建包和清单

```json
// package.json
{
  "name": "@myorg/openclaw-my-plugin",
  "version": "1.0.0",
  "type": "module",
  "openclaw": {
    "extensions": ["./index.ts"]
  }
}
```

```json
// openclaw.plugin.json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "description": "Adds a custom tool to OpenClaw",
  "configSchema": {
    "type": "object",
    "additionalProperties": false
  }
}
```

### 2. 编写入口点

```typescript
// index.ts
import { definePluginEntry } from "openclaw/plugin-sdk/plugin-entry";
import { Type } from "@sinclair/typebox";

export default definePluginEntry({
  id: "my-plugin",
  name: "My Plugin",
  description: "Adds a custom tool to OpenClaw",
  register(api) {
    // 必需工具 —— 始终可用
    api.registerTool({
      name: "my_tool",
      description: "Do a thing",
      parameters: Type.Object({ input: Type.String() }),
      async execute(_id, params) {
        return { content: [{ type: "text", text: `Got: ${params.input}` }] };
      },
    });

    // 可选工具 —— 用户需在 allow 列表中添加
    api.registerTool(
      {
        name: "workflow_tool",
        description: "Run a workflow",
        parameters: Type.Object({ pipeline: Type.String() }),
        async execute(_id, params) {
          return { content: [{ type: "text", text: params.pipeline }] };
        },
      },
      { optional: true },
    );
  },
});
```

用户在配置中启用可选工具：

```json5
{
  tools: { allow: ["workflow_tool"] },
}
```

### 3. Plugin 内部模块约定

```
my-plugin/
  api.ts            # 公共导出（外部消费者使用）
  runtime-api.ts    # 内部运行时导出
  index.ts          # Plugin 入口点
  setup-entry.ts    # 轻量级 setup-only 入口（可选）
```

---

## 测试

### 运行测试

```bash
# 仓库内 Plugin
pnpm test -- extensions/my-plugin/

# 代码检查
pnpm check
```

### 测试工具

从 `plugin-sdk/testing` 导入的测试辅助函数：

```typescript
import { installCommonResolveTargetErrorCases, shouldAckReaction } from "openclaw/plugin-sdk/testing";
```

- `installCommonResolveTargetErrorCases` —— 安装目标解析的通用错误用例
- `shouldAckReaction` —— Reaction 行为断言

---

## 发布前检查清单

- [ ] `package.json` 包含正确的 `openclaw` 元数据
- [ ] `openclaw.plugin.json` 清单存在且有效
- [ ] 入口点使用 `defineChannelPluginEntry` 或 `definePluginEntry`
- [ ] 所有导入使用聚焦的 `plugin-sdk/<subpath>` 路径
- [ ] 内部导入使用本地模块，而非 SDK 自引用
- [ ] 测试通过（`pnpm test -- extensions/my-plugin/`）
- [ ] `pnpm check` 通过（仓库内 Plugin）

---

## 安装和使用

```bash
# 从 ClawHub 或 npm 安装
openclaw plugins install @myorg/openclaw-my-plugin

# OpenClaw 优先检查 ClawHub，回退到 npm
```

仓库内 Plugin 放置在 `extensions/` 目录下，自动发现。
