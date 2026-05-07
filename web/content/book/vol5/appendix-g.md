---
appendix: G
title: "资源索引"
status: draft
---

# 附录 G：资源索引

本附录汇总 OpenClaw 的官方资源、社区渠道和精选展示项目。

---

## 官方资源

| 资源 | 链接 |
|:---|:---|
| 官方文档 | https://docs.openclaw.ai |
| GitHub 仓库 | https://github.com/openclaw/openclaw |
| CHANGELOG | https://github.com/openclaw/openclaw/blob/main/CHANGELOG.md |
| ClawHub Skill 注册表 | https://clawhub.com |
| 安装脚本（macOS/Linux） | https://openclaw.ai/install.sh |
| 安装脚本（Windows） | https://openclaw.ai/install.ps1 |

---

## 社区渠道

| 渠道 | 链接 |
|:---|:---|
| Discord 社区 | https://discord.gg/clawd |
| X (Twitter) | https://x.com/openclaw |

> 💡 Discord 的 `#showcase` 频道是分享项目和获取反馈的最佳去处。

---

## 部署指南

| 平台 | 文档链接 |
|:---|:---|
| 通用 Linux | https://docs.openclaw.ai/platforms/linux |
| Windows / WSL2 | https://docs.openclaw.ai/platforms/windows |
| Hetzner VPS | https://docs.openclaw.ai/install/hetzner |
| Fly.io | https://docs.openclaw.ai/install/fly |
| exe.dev | https://docs.openclaw.ai/install/exe-dev |
| Nix 打包 | https://github.com/openclaw/nix-openclaw |

---

## 精选展示项目

以下是从社区中选出的代表性项目，按类别分组。

### 自动化与工作流

- **Tesco 自动购物** —— 每周食谱 -> 常规商品 -> 预约配送 -> 确认订单，纯浏览器控制
- **Jira Skill Builder** —— Agent 连接 Jira 后即时生成 Skill
- **TradingView 技术分析** —— 浏览器自动化登录 TradingView，截图并分析图表
- **Slack 自动支持** —— 监控公司 Slack 频道，自动回复并转发通知到 Telegram
- **Winix 空气净化器控制** —— 通过自然语言管理室内空气质量

### 开发工具

- **PR Review Telegram 反馈** —— 代码变更 -> 打开 PR -> Agent 审阅并回复评审意见
- **SNAG 截图转 Markdown** —— 快捷键截屏 -> Gemini 视觉分析 -> 剪贴板 Markdown
- **Linear CLI** —— 与 Agent 工作流集成的 Linear 问题管理命令行
- **CodexMonitor** —— Homebrew 安装的本地 Codex 会话监控工具

### 语音与电话

- **Clawdia Phone Bridge** —— Vapi 语音助手与 OpenClaw 的 HTTP 桥接，近实时电话通话
- **OpenRouter 多语言转录** —— 通过 OpenRouter 的多语言音频转录 Skill

### 硬件与家庭

- **Bambu 3D 打印机控制** —— 状态查询、任务管理、摄像头、校准
- **GoHome 家庭自动化** —— Nix 原生的家庭自动化，OpenClaw 作为交互界面
- **Roborock 扫地机器人** —— 通过自然对话控制扫地机器人
- **Home Assistant Add-on** —— 在 Home Assistant OS 上运行的 OpenClaw 网关

### 基础设施

- **Home Assistant Skill** —— 通过自然语言控制和自动化 Home Assistant 设备
- **Nix 打包** —— 可复现的 Nix 化 OpenClaw 部署配置
- **iOS App via Telegram** —— 完整 iOS 应用开发到 TestFlight 部署，全程通过 Telegram 对话

### 多 Agent 架构

- **Dream Team (14+ Agents)** —— Opus 4.5 编排器 + Codex 工作节点，包含沙箱、Webhook、心跳和委派流的完整技术方案

---

## 提交你的项目

在 Discord 的 [#showcase](https://discord.gg/clawd) 频道分享你的项目，或在 X 上 [@openclaw](https://x.com/openclaw) 标记。包含项目描述、仓库链接和截图即可。
