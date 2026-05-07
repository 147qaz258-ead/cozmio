# Cozmio

Cozmio 是一个主动式桌面智能体，能够观察用户行为并在其需要帮助时主动提供协助。

## 项目结构

```
cozmio/          # Rust 桌面应用（Tauri）
cozmio-api/      # Go API 服务（可选依赖）
web/             # Next.js 前端网站
docs/            # 项目文档
```

## 快速开始

### 桌面应用

```bash
cd cozmio
cargo build
cargo run
```

或使用 Tauri 开发模式：

```bash
cd cozmio/src-tauri
npm run tauri dev
```

### 前端网站

```bash
cd web
npm install
npm run dev
```

## 技术栈

- **桌面应用**: Rust + Tauri + Ollama
- **前端**: Next.js + React + TypeScript
- **后端**: Go + PostgreSQL

## 主要功能

- 窗口监控与截图捕获
- 本地大模型调用（Ollama）
- 系统级通知与确认流程
- 任务执行与进度追踪

## 许可证

MIT