# AGENTS.md

This file provides guidance to Codex (codex.ai/code) when working with code in this repository.

## Build

```bash
cd cozmio && cargo build
cd cozmio/src-tauri && npm run tauri dev
cd cozmio && cargo test
```

## 项目架构

```
cozmio/                          # 主应用
├── cozmio_core/                 # 截图、窗口枚举
├── cozmio_memory/               # 记忆系统
├── cozmio_model/                # 模型封装
├── cozmio-box-worker/           # Box 工作进程
├── cozmio-cli/                  # CLI 工具
├── relay-client/                # 中继客户端
├── relay-engine/                # 中继引擎
├── src-tauri/                   # Tauri 桌面端
├── cozmio_capture/              # 截图模块
├── cozmio_verify/              # 验证模块
└── memory-cli/                  # 记忆 CLI

cozmio-api/                      # Go API 服务
├── src/
└── dist/

web/                             # Next.js 前端
├── src/
│   ├── app/
│   ├── components/
│   └── lib/
└── content/                     # 书籍内容
```

**模块职责**：

- `cozmio_core/` — 截图、窗口枚举
- `src-tauri/src/window_monitor.rs` — 窗口监控、截图
- `src-tauri/src/model_client.rs` — 调用 Ollama API，返回原始输出
- `src-tauri/src/main_loop.rs` — 主循环：捕获、调用模型、日志、通知
- `src-tauri/src/memory_consolidation.rs` — 记忆 consolidation

## 语义边界原则

> 不要让代码传递"意图"，要让 Agent 合成"事实"。代码只提供时间、来源、窗口、原文、反馈、执行结果这些事实边界；语义解释权必须属于 Agent。

传入模型或记忆层的，必须是可验证的时间序列事实（窗口标题、进程名、时间戳、切换次数），不是代码已做出的推断（`is_oscillating`、`just_arrived`、`last_switch_direction`）。

**允许在代码中传入模型的内容**：
- 可验证事实：时间戳、窗口标题、进程名、操作 ID、中继状态、原始输出、文件路径、时长、计数
- 机械索引：source id、session id、trace id、时间戳、新近度、存储键、检索分数
- 工具描述：relay、本地模型、执行 agent 能做什么

**禁止在代码中传入模型的内容**：
- 代码推断的语义标签：`is_oscillating`、`just_arrived`、`last_switch_direction`、`stuck/not stuck`、`task stage`
- 代码解析模型输出为 `Continue/Abstain` 枚举再路由行为
- `auto_remember_model_output` 在用户反馈前写入记忆
- 硬编码的用户意图、工作流阶段、弹出限制

如果需要语义摘要，必须由模型或执行 agent 生成，并附带来源、时间戳和 provenance 一起存储。

**违规位置**：`model_client.rs:parse_response()`、`main_loop.rs:265+283`、`window_monitor.rs:52-58`、`memory_consolidation.rs:830`

## 完成标准

实现 ≠ 完成。任务只有在以下全部满足时才视为完成：

- 验证命令已运行
- 结果记录在 `verification/last_result.json`
- `feature_list.json` 反映实际状态
- `claude-progress.txt` 包含下一 session 交接

## 仓库飞轮

Session 开始：读 `claude-progress.txt`、`feature_list.json`、git history，选一个 failing/incomplete feature。

Session 结束：运行验证、更新 `feature_list.json`、更新 `claude-progress.txt`。
