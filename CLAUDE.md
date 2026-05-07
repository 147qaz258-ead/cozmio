# CLAUDE.md

## 构建

```bash
cd cozmio && cargo build          # 构建
cd cozmio/src-tauri && npm run tauri dev  # 开发模式
cd cozmio && cargo test          # 测试
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
- `src-tauri/src/model_client.rs` — 调用 Ollama API
- `src-tauri/src/executor.rs` — 根据模型输出路由动作
- `src-tauri/src/memory_consolidation.rs` — 记忆 consolidation

## 语义边界原则

> 不要让代码传递"意图"，要让 Agent 合成"事实"。代码只提供时间、来源、窗口、原文、反馈、执行结果这些事实边界；语义解释权必须属于 Agent。

传入模型或记忆层的，必须是可验证的时间序列事实（窗口标题、进程名、时间戳、切换次数），不是代码已做出的推断（`is_oscillating`、`just_arrived`、`last_switch_direction`）。

违规位置：`model_client.rs:parse_response()`、`main_loop.rs:265+283`、`window_monitor.rs:52-58`、`memory_consolidation.rs:830`。

## 完成标准

实现 ≠ 完成。任务只有在以下全部满足时才视为完成：

- 验证命令已运行
- 结果记录在 `verification/last_result.json`
- `feature_list.json` 反映实际状态
- `claude-progress.txt` 包含下一 session 交接

## 仓库飞轮

Session 开始：读 `claude-progress.txt`、`feature_list.json`、git history，选一个 failing/incomplete feature。

Session 结束：运行验证、更新 `feature_list.json`、更新 `claude-progress.txt`。
