# cozmio-cli 统一 CLI 工具实施方案

> **智能执行体须知**：本方案以运行效果、验证资产和飞轮写回为中心。默认不含代码示例。

## 1. Flywheel Context

- active task: COZMIO-CLI（新建统一 CLI，覆盖所有子系统）
- current phase: Plan
- latest verification: `verification/last_result.json` — 短时上下文模块 H2 完成
- blocker: 无
- next expected step: 方案批准后执行 H1（创建 cozmio-cli crate）

## 2. Goal

新建 `cozmio-cli` crate，提供单一 CLI 入口覆盖 window / model / relay / memory / config / run / status 全部子系统，供开发调试和运维使用。

## 3. Product Type

- type: `deterministic_software`
- core risk: 代码正确性、跨 crate 接口兼容性、cargo build 通过
- verification style: `cargo build` + 实际命令执行输出校验

## 4. Global Roadmap

| Phase | 目标 | 依赖 | 验收意图 |
|-------|------|------|---------|
| H1 | 创建 cozmio-cli crate + window/model 子命令 | — | `cargo build -p cozmio-cli` 通过 |
| H2 | relay / memory / config 子命令 | H1 通过 | relay dispatch 可达、memory search 有输出 |
| H3 | run / status 子命令 | H2 通过 | `run --once` 输出 judgment，`status` 显示主应用状态 |

## 5. Scope

### In（本次包含）

- 新建 `cozmio-cli` crate，加入 workspace members
- `window capture` — 调用 `cozmio_core::capture_all()`，输出 JSON
- `window list` — 枚举所有顶层窗口，输出表格
- `model list` — 调用 `cozmio_model::discover_first_model_sync()`
- `model call <text>` — 调用 `cozmio_model::ask_model_sync()`，带截图
- `relay dispatch <task>` — 调用 `relay-client` 的 `dispatch()`
- `relay status <session_id>` — 调用 `relay-client` 的 `status()`
- `memory stats / search / slices / threads / replay` — 复用 memory-cli 逻辑
- `config show / set`
- `run --once / --daemon`
- `status`

### Out（本次不包含）

- `cozmio://` 协议处理（Tauri deep-link）
- GUI 任何部分
- 新增模型验证样本

## 6. Current Truth

- workspace members: `cozmio_core`, `cozmio_capture`, `cozmio_model`, `cozmio_verify`, `src-tauri`, `relay-engine`, `relay-client`, `cozmio_memory`, `cozmio-box-worker`, `memory-cli`, `embedding-probe`
- `cozmio_model::discover_first_model_sync()` / `ask_model_sync()` — 已有同步 API
- `relay-client` 有 `dispatch()`, `status()` 同步方法
- `cozmio_core::capture_all()` — 已有的截图 API
- `memory-cli` 已实现完整 memory 子系统 CLI（stats, search, slices, threads, replay）
- config 存储路径: `%LOCALAPPDATA%/cozmio/config.json`（已有 `dirs` crate）

## 7. Implementation Shape（H1）

1. **添加 crate**：在 `cozmio/Cargo.toml` workspace members 加入 `cozmio-cli`，创建目录结构 `cozmio-cli/src/main.rs` + `Cargo.toml`
2. **依赖声明**：`clap` (derive), `cozmio_core`, `cozmio_model`, `relay-client`, `cozmio_memory` (default-features = false), `anyhow`, `serde_json`, `dirs`
3. **clap 结构**：使用 `#[derive(Subcommand)]` 定义顶层 `Cli` 枚举，每个子系统一个 variant，内嵌子子命令
4. **window capture**：调用 `cozmio_core::capture_all()`，序列化 `WindowSnapshot` 为 JSON 输出
5. **window list**：调用 `cozmio_core::window::enumerate_windows()`，格式化输出
6. **model list**：调用 `discover_first_model_sync()`，输出模型名和 Ollama URL
7. **model call**：截图 + `ask_model_sync(text, screenshot_bytes)`，输出 `InterventionResult` JSON

## 8. Verification Asset

- verification type: `deterministic_software`
- command: `cargo build -p cozmio-cli`
- expected evidence: build 成功，无编译错误
- writeback targets:
  - `verification/last_result.json`
  - `feature_list.json`
  - `claude-progress.txt`

## 9. Phase Gate

H1 只有满足以下条件才能标记为完成：

- [ ] `cargo build -p cozmio-cli` 通过
- [ ] `cozmio-cli window capture` 输出有效 JSON
- [ ] `cozmio-cli window list` 列出至少一个窗口
- [ ] `cozmio-cli model list` 发现可用模型（或给出明确错误）
- [ ] `cozmio-cli --help` 显示完整命令树
- [ ] `verification/last_result.json` 已更新
- [ ] `feature_list.json` 已添加 COZMIO-CLI 条目
- [ ] `claude-progress.txt` 已写入交接内容

## 10. Next Execution Step

- next phase: H1（创建 cozmio-cli crate + window/model 子命令）
- goal: `cargo build -p cozmio-cli` 通过，`cozmio-cli window capture` 和 `model list` 有实际输出
- entry skill: `superpowers:subagent-driven-development`
- stop condition: Phase Gate 全部条件满足