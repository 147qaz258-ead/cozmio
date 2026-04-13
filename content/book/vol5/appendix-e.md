---
appendix: E
title: "常见问题排查手册"
status: draft
---

# 附录 E：常见问题排查手册

本手册按症状组织，提供从快速诊断到深度修复的完整排查流程。

关于系统监控和健康检查，参见 [第 10 章](../vol4/ch10.md)；关于各通道的详细配置，参见 [第 4 章](../vol2/ch04.md)。

---

## 通用诊断流程

遇到任何问题时，按以下顺序执行命令：

```bash
openclaw status
openclaw gateway status
openclaw channels status --probe
openclaw logs --follow
openclaw doctor
```

健康基线：

- `Runtime: running`
- `RPC probe: ok`
- 通道探针显示 `connected` 或 `ready`
- 日志中无重复的致命错误

---

## 症状 1：Agent 不回复

### 诊断步骤

```bash
openclaw status
openclaw gateway status
openclaw channels status --probe
openclaw pairing list --channel <channel>
openclaw logs --follow
```

### 常见原因和修复

| 日志特征 | 原因 | 修复 |
|:---|:---|:---|
| `drop guild message (mention required` | Discord 群聊提及门控拦截 | @提及 Bot 或放宽提及策略 |
| `pairing request` | 发送者未批准，等待 DM 配对 | 批准配对请求或切换 DM 策略 |
| `blocked` / `allowlist` | 发送者/房间/群组被过滤 | 将发送者添加到 `allowFrom` 或切换到 `open` 策略 |
| Gateway 未运行 | 服务未启动 | `openclaw gateway restart` |
| `RPC probe: failed` | Gateway 不可达 | 检查端口、Tailscale 连接、防火墙 |

> 💡 如果 Gateway 在远程服务器上运行，确保 SSH 隧道或 Tailscale 连接正常。`openclaw status --all` 可生成可分享的诊断报告。

---

## 症状 2：控制面板无法连接

### 诊断步骤

```bash
openclaw status
openclaw gateway status
openclaw logs --follow
openclaw doctor
```

### 常见原因和修复

| 日志特征 | 原因 | 修复 |
|:---|:---|:---|
| `device identity required` | HTTP/非安全上下文无法完成设备认证 | 使用 HTTPS 或 localhost |
| `AUTH_TOKEN_MISMATCH` | Token 不匹配或过期 | 检查 `gateway.auth.token`，重新生成 |
| `gateway connect failed:` | UI 目标 URL/端口错误 | 确认 Gateway 地址和端口 |
| 无 `Dashboard:` 输出 | 控制面板未启用 | 设置 `gateway.controlUi.enabled: true` |

### 远程访问认证

**Tailscale Serve（推荐）：**

```json5
{
  gateway: {
    bind: "loopback",
    auth: { allowTailscale: true },
  },
}
```

运行 `openclaw gateway --tailscale serve`，通过 `https://<magicdns>/` 访问。

**SSH 隧道：**

```bash
ssh -N -L 18789:127.0.0.1:18789 user@host
```

---

## 症状 3：Gateway 无法启动

### 诊断步骤

```bash
openclaw gateway status
openclaw logs --follow
openclaw doctor
```

### 常见原因和修复

| 日志特征 | 原因 | 修复 |
|:---|:---|:---|
| `set gateway.mode=local` | Gateway 模式未设置或设为 remote | 在 `openclaw.json` 中设置 `gateway.mode: "local"` |
| `refusing to bind ... without auth` | 非回环绑定但未设置认证 | 添加 `gateway.auth.token` 或改用 `bind: "loopback"` |
| `EADDRINUSE` | 端口已被占用 | 更换端口或终止占用进程 |
| `another gateway instance` | 已有 Gateway 实例运行 | 先停止现有实例 |

> ⚠️ 在非回环地址绑定（`bind: "all"`）时必须设置认证，否则 Gateway 拒绝启动。

---

## 症状 4：通道已连接但消息不流通

### 诊断步骤

```bash
openclaw status
openclaw gateway status
openclaw channels status --probe
openclaw logs --follow
```

### 常见原因和修复

| 日志特征 | 原因 | 修复 |
|:---|:---|:---|
| `mention required` | 群聊提及门控拦截 | 提及 Bot 或设置 `requireMention: false` |
| `not_in_channel` / `403` | 通道权限或 Token 问题 | 检查 Bot Token 和权限范围 |
| `Forbidden` | API 权限不足 | 重新授权 Bot |
| `pairing` / `pending` | DM 发送者未批准 | 批准配对 |

---

## 症状 5：Cron 或心跳未触发

### 诊断步骤

```bash
openclaw status
openclaw cron status
openclaw cron list
openclaw cron runs --id <jobId> --limit 20
openclaw logs --follow
```

### 常见原因和修复

| 日志特征 | 原因 | 修复 |
|:---|:---|:---|
| `scheduler disabled` | Cron 被禁用 | 设置 `cron.enabled: true` |
| `heartbeat skipped` + `quiet-hours` | 在安静时段外 | 检查活跃时间配置 |
| `requests-in-flight` | 主通道忙碌 | 心跳被延迟，等待当前请求完成 |
| `unknown accountId` | 投递目标账户不存在 | 检查 `heartbeat.to` 配置 |

---

## 症状 6：节点已配对但工具失败

### 诊断步骤

```bash
openclaw status
openclaw nodes status
openclaw nodes describe --node <idOrNameOrIp>
openclaw logs --follow
```

### 常见原因和修复

| 日志特征 | 原因 | 修复 |
|:---|:---|:---|
| `NODE_BACKGROUND_UNAVAILABLE` | 节点 App 在后台 | 将节点 App 切换到前台 |
| `*_PERMISSION_REQUIRED` | OS 权限被拒绝 | 在系统设置中授予权限 |
| `SYSTEM_RUN_DENIED: approval required` | exec 审批待处理 | 批准执行请求 |
| `SYSTEM_RUN_DENIED: allowlist miss` | 命令不在执行白名单 | 将命令添加到白名单 |

---

## 症状 7：浏览器工具失败

### 诊断步骤

```bash
openclaw status
openclaw browser status
openclaw logs --follow
openclaw doctor
```

### 常见原因和修复

| 日志特征 | 原因 | 修复 |
|:---|:---|:---|
| `Failed to start Chrome CDP on port` | 本地浏览器启动失败 | 检查 Chrome 安装和 CDP 端口 |
| `executablePath not found` | 配置的二进制路径错误 | 更正 `browser.executablePath` |
| `No Chrome tabs found` | Chrome MCP 配置无打开的标签页 | 打开至少一个 Chrome 标签页 |
| `attachOnly ... not reachable` | attach-only 模式无 CDP 目标 | 确认 Chrome 远程调试已启用 |

---

## 症状 8：Anthropic 429 长上下文错误

```
HTTP 429: rate_limit_error: Extra usage is required for long context requests
```

这表示 Anthropic API 的长上下文配额已用尽。修复方法：

1. 缩短会话历史（使用 `/new` 重置）
2. 降低上下文窗口（切换到较小模型）
3. 增加 Anthropic 账户的使用额度
4. 配置 fallback 模型以自动切换

```json5
{
  agents: {
    defaults: {
      model: {
        primary: "anthropic/claude-sonnet-4-6",
        fallbacks: ["openai/gpt-5.2"],
      },
    },
  },
}
```

---

## 通道特定排查

### WhatsApp

| 症状 | 快速检查 | 修复 |
|:---|:---|:---|
| 已连接但无 DM 回复 | `openclaw pairing list whatsapp` | 批准发送者或切换 DM 策略 |
| 群消息被忽略 | 检查 `requireMention` 和提及模式 | 提及 Bot 或放宽提及策略 |
| 随机断连/重新登录循环 | `openclaw channels status --probe` + 日志 | 重新登录，验证凭据目录健康 |

### Telegram

| 症状 | 快速检查 | 修复 |
|:---|:---|:---|
| `/start` 后无回复流 | `openclaw pairing list telegram` | 批准配对或更改 DM 策略 |
| Bot 在线但群聊无响应 | 验证提及要求和 Bot 隐私模式 | 关闭隐私模式或 @提及 Bot |
| 网络错误 | 检查 Telegram API 调用失败日志 | 修复 DNS/IPv6/代理路由到 `api.telegram.org` |
| `BOT_COMMANDS_TOO_MUCH` | 检查日志 | 减少自定义命令数量或禁用原生菜单 |

### Discord

| 症状 | 快速检查 | 修复 |
|:---|:---|:---|
| Bot 在线但无服务器回复 | `openclaw channels status --probe` | 允许服务器/频道，验证消息内容 |
| 群消息被忽略 | 检查日志中的提及门控 | 提及 Bot 或设置 `requireMention: false` |
| DM 回复缺失 | `openclaw pairing list discord` | 批准 DM 配对或调整 DM 策略 |

### Slack

| 症状 | 快速检查 | 修复 |
|:---|:---|:---|
| Socket Mode 已连接但无响应 | `openclaw channels status --probe` | 验证 App Token + Bot Token 和所需权限范围 |
| DM 被阻止 | `openclaw pairing list slack` | 批准配对或放宽 DM 策略 |
| 频道消息被忽略 | 检查 `groupPolicy` 和频道白名单 | 允许频道或切换策略为 `open` |

---

## Plugin 安装失败

**错误信息：** `package.json missing openclaw extensions`

修复方法：

1. 在 Plugin 的 `package.json` 中添加 `openclaw.extensions` 字段
2. 将入口指向构建后的运行时文件（通常是 `./dist/index.js`）
3. 重新发布 Plugin 并运行 `openclaw plugins install <package>`

```json
{
  "name": "@openclaw/my-plugin",
  "version": "1.2.3",
  "openclaw": {
    "extensions": ["./dist/index.js"]
  }
}
```

---

## Windows 特定问题

### exec 输出乱码（中文）

症状：`system.run` / `exec` 输出显示为乱码。

修复：在 PowerShell 中设置 UTF-8 编码：

```powershell
chcp 65001
[Console]::InputEncoding = [System.Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$OutputEncoding = [System.Text.UTF8Encoding]::new($false)
```

### `openclaw` 命令未识别

1. 确认 `git` 在 PATH 中（安装 Git for Windows）
2. 检查 npm 全局 bin 目录：

```powershell
npm config get prefix
```

3. 将该目录添加到用户 PATH
4. 关闭并重新打开 PowerShell

> 💡 推荐使用 WSL2 而非原生 Windows，可获得更流畅的体验。
