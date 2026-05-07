# Relay Engine 真实链路执行记录

日期：2026-04-24

## 审核结论

### 阻塞真实验证的问题

- `relay-engine/build.rs` 强依赖系统 `protoc`，导致无 `protoc` 环境下无法构建 Relay。
- 主请求协议没有显式 request type，`status/progress/result/interrupt` 这类单字段 protobuf 请求会和 `dispatch` 混淆。
- Windows accepted `TcpStream` 继承/表现为 nonblocking，服务端 `recv_exact` 会遇到 `WouldBlock(10035)` 后断开连接。
- Claude connector 使用 `.output()` 等待进程结束，不能实时捕获 progress。
- Claude connector 未注册 PID，`interrupt` 找不到真实执行进程。
- 订阅没有 buffer replay，订阅建立前的 progress 会丢；终态靠轮询状态补空消息。
- `interrupt` 与子进程退出存在竞态，订阅端可能先收到 `Failed` 终态而不是 `Interrupted`。

### 能跑通但不真实的问题

- 旧 `test_v8_subscription.py` 先 dispatch 后 subscribe，只要收到 completion 就 PASS，不能证明实时 progress。
- `ProgressEvent.session_id` 为空，客户端不能证明事件归属。
- `StatusResponse` 时间字段固定为 0。
- `WindowsNamedPipe` 当前仍是 loopback TCP 包装，尚不是真正 Windows Named Pipe。

### 后续优化

- `src-tauri` 宿主已经接入 `relay-client`，但当前 UI 展示的 `result_output` 仍是 Claude stream-json 原始输出，产品层还可以再做清洗。
- Unix transport 仍是 placeholder。
- 协议仍缺正式 error response envelope。
- `WindowsNamedPipe` 当前仍是 loopback TCP 包装。2026-04-24 的决策是：为了先把桌面端真实效果跑通，先保留 `tcp-loopback`，此轮不阻塞在真正 Named Pipe；后续基础设施定型时再补回真实实现。

## 本轮修复

- `build.rs` 改为不依赖系统 `protoc`，使用已提交的 `src/proto/mod.rs`。
- 主请求帧增加显式 request kind：dispatch/status/progress/result/interrupt/subscribe 不再靠 protobuf 首字节猜测。
- Relay accept 后将 stream 切回 blocking，修复 Windows `WouldBlock(10035)` 断链。
- Claude connector 改为真实 `spawn`，注册 PID，逐行捕获 stdout/stderr 写入 session progress。
- Claude connector 对 Windows `claude.cmd` 入口增加多行任务正规化，修复桌面宿主 dispatch 时 `batch file arguments are invalid` 的真实启动失败。
- `interrupt` 改为通过注册 PID 执行 `taskkill /PID <pid> /T /F`，并引入 `interrupt_requested` 标记，避免终态被竞态回写成 `Failed`。
- Session subscription 改为 buffer replay + 后续 live push + terminal event。
- `ProgressEvent` 增加 `terminal` 和 `terminal_status`，并填充真实 `session_id`。
- 旧 V8 脚本改为调用真实 V1-V8 验证脚本。
- `src-tauri` 宿主新增真实 Relay bridge：确认后直接调用 `relay-client` dispatch，订阅 session 进展，回写 `state-update` 给桌面端。
- `src-tauri` 状态面板修复默认空白问题，并增加 `RELAY TEST` 入口与 `RELAY SESSION` 展示区，桌面端可直接看到 session 状态、进展和结果。

## 运行前提说明

- `claude.cmd` 在当前 Codex sandbox 中会报 `spawn EPERM`，因此最终 V1-V8 真实验证使用了脱离 sandbox 的实跑。
- 这不是 Relay 代码路径内的本地 mock 或替身；验证过程仍然是 `relay-engine.exe` 真启动、真监听、真 dispatch、真启动 Claude Code、真读取 progress、真回收 result、真 interrupt 杀进程。

## 验证命令

```powershell
cargo check -p relay-engine -p relay-client
cargo build -p relay-engine -p relay-client --release
python relay-engine\test_real_chain.py
```

## 最新真实运行证据

`python relay-engine\test_real_chain.py`（脱离 sandbox）退出码：0

关键输出：

```text
PASS: V1 relay-engine process stays alive
PASS: V2 client can connect to Relay main port
PASS: V8 client can connect to Relay subscription port
PASS: V3 dispatch returned real UUID session_id=4f76cb20-4d9f-4d0e-8a41-7a8a91639b50
PASS: dispatch returned active status=1
PASS: V4 Claude Code connector process exists pid=11320
PASS: V4 process command line references Claude Code: cmd.exe ... claude.cmd --print --output-format stream-json --include-partial-messages "Say relay-e2e-ok and nothing else."
PASS: V5 progress query returns buffered entries=1
PASS: V8 subscriber replayed buffered pre-subscription progress
PASS: V8 subscriber received live process output event
PASS: V8 subscriber received terminal event
PASS: V6 session reached completed/failed terminal status=3
PASS: V6 result is available through client query
PASS: V6 result contains captured Claude Code output
PASS: Relay process remains alive after first session
PASS: V7 process exists before interrupt pid=46860
PASS: V7 subscription replayed initial progress before interrupt
PASS: V7 interrupt request returned success
PASS: V7 process was terminated pid=46860
PASS: V7 interrupted terminal event was pushed to subscriber
ALL_REAL_RELAY_CHECKS_PASSED
```

Interrupt 证据：

```text
SUCCESS: The process with PID 35032 (child process of PID 24980) has been terminated.
SUCCESS: The process with PID 24980 (child process of PID 26676) has been terminated.
SUCCESS: The process with PID 26676 (child process of PID 46860) has been terminated.
SUCCESS: The process with PID 46860 (child process of PID 12500) has been terminated.
```

## 桌面宿主实跑证据（2026-04-24）

这轮不再依赖外部 Python 脚本触发宿主验证，而是从 Cozmio 桌面窗口直接点击 `RELAY TEST`，再由宿主自己：

```text
桌面端确认
  ↓
src-tauri 调用 relay-client dispatch
  ↓
Relay 创建真实 session
  ↓
Claude Code 真实启动
  ↓
进展通过订阅流回到 src-tauri
  ↓
桌面状态面板展示进展与终态结果
```

关键宿主日志：

```text
Desktop relay demo confirmed from status panel window='[Manual Relay Demo]' process='cozmio.exe'
Desktop host dispatching Relay request for '[Manual Relay Demo]' (cozmio.exe)
Desktop host connected to Relay at 127.0.0.1:7890
Desktop host subscribed to Relay session b944bd21-b49c-4378-b757-3e2033440792 for '[Manual Relay Demo]' (cozmio.exe)
Desktop host received Relay event session=b944bd21-b49c-4378-b757-3e2033440792 terminal=false status=0 level=info message=Started Claude Code process pid=35576
Desktop host received Relay event session=b944bd21-b49c-4378-b757-3e2033440792 terminal=false status=0 level=info message=stdout: {\"type\":\"result\",\"subtype\":\"success\" ...}
Desktop host received Relay event session=b944bd21-b49c-4378-b757-3e2033440792 terminal=true status=3 level=info message=session terminal: Completed
Desktop host fetched Relay result session=b944bd21-b49c-4378-b757-3e2033440792 success=true summary=Task completed successfully
```

这组证据证明了：

- 桌面端真的调到了 Relay，而不是外部脚本代发。
- 用户在桌面端确认后，宿主真的自动 dispatch 了真实 session。
- 宿主真的收到了 progress stream，不是只靠轮询最终状态。
- 宿主真的收到了 terminal event 和 result，并回写到了桌面状态面板。

## 真实产品路径验证（2026-04-24 夜间）

这一轮不再以 `RELAY TEST` 作为主要证明路径，而是验证真实产品路径：

```text
监控循环发现可继续介入
  ↓
右下角确认卡片出现
  ↓
用户点击确认
  ↓
src-tauri 真实 dispatch 到 Relay
  ↓
Relay 创建真实 session 并启动 Claude Code
  ↓
任务监控页收到 progress / status / result
  ↓
用户点击 STOP
  ↓
Relay 真实 interrupt 执行进程
  ↓
页面状态变为 Interrupted，结果区可取到 interrupted result
```

### 这轮代码落点

- `cozmio/src-tauri/src/main_loop.rs`
  - 去掉阻塞式系统确认框。
  - `ExecutionResult::Confirmed` 改为写入 `pending_confirmation`，并暂停后续新任务生成。
- `cozmio/src-tauri/src/commands.rs`
  - 新增 `pending_confirmation` / `current_task` 宿主状态。
  - 新增 `confirm_pending_task` / `cancel_pending_task` / `dismiss_pending_task` / `interrupt_current_task`。
  - `relay_execution` 更新时同步回写当前任务状态。
- `cozmio/src-tauri/src/relay_bridge.rs`
  - 新增 `RelayDispatchRequest::from_task_text(...)`，原始任务文本不再为前端显示而拆字段。
  - 新增 `interrupt_session(...)`，桌面端可直接对真实 session 发 interrupt。
- `cozmio/src-tauri/src/components/StatusPanel.js`
  - 状态页改为任务监控页。
  - 右下角确认卡片只保留原始任务文字、来源窗口、时间、确认/取消/关闭按钮。
  - 监控页展示 current task / relay session / recent progress / result，并提供 STOP。
- `cozmio/src-tauri/src/styles.css`
  - 新增浮动确认卡片与任务监控页样式。

### 实跑步骤

```powershell
cargo check -p cozmio
cargo build -p cozmio --target-dir C:\Users\29913\AppData\Local\Temp\cozmio-hosttest -j 2
```

实际运行程序：

```text
C:\Users\29913\AppData\Local\Temp\cozmio-hosttest\debug\cozmio.exe
```

实际用于触发监控循环的前台窗口：

```text
cozmio-product-task.txt - Notepad
内容：
请继续处理这个仓库里的 relay 接入工作：
1. 用户确认后立刻开始执行
2. 把最近进展持续回传到页面
3. 完成后给出结果
```

### 关键日志证据

```text
Window changed: Cozmio - 主动智能体
Model output: CONTINUE - Cozmio进程运行中且显示Foreground Window为Notepad中的任务文件，表明有明确的任务执行上下文。
Execution result: Confirmed
Creating pending confirmation card for window='Cozmio - 主动智能体' process='cozmio.exe'

Desktop host dispatching Relay request for 'Cozmio - 主动智能体' (cozmio.exe)
Relay not reachable, spawning local relay-engine at "D:\C_Projects\Agent\cozmio\cozmio\target\release\relay-engine.exe"
Desktop host connected to Relay at 127.0.0.1:7890
Desktop user confirmed task, relay session started session=c1bd4387-79c8-40a7-8206-74a7c4637231 window='Cozmio - 主动智能体' process='cozmio.exe'
Desktop host subscribed to Relay session c1bd4387-79c8-40a7-8206-74a7c4637231
Desktop host received Relay event ... message=Started Claude Code process pid=26992
Desktop host received Relay event ... message=stdout: {"type":"system","subtype":"init", ...}

Desktop user requested interrupt for relay session c1bd4387-79c8-40a7-8206-74a7c4637231
Desktop host received Relay event ... level=warn message=Interrupted Claude Code process pid=26992
Desktop host interrupted Relay session c1bd4387-79c8-40a7-8206-74a7c4637231
Desktop host received Relay event ... terminal=true status=5 ... message=session terminal: Interrupted
Desktop host fetched Relay result session=c1bd4387-79c8-40a7-8206-74a7c4637231 success=false summary=Task interrupted
```

### 截图证据

- 右下角确认卡片出现：
  - `D:\C_Projects\Agent\cozmio\tmp_cozmio_confirmation_front.png`
- 监控页显示真实 session 正在运行：
  - `D:\C_Projects\Agent\cozmio\tmp_monitor_progress.png`
- 监控页展示 recent progress，并且页面上的 `STOP` 可点击：
  - `D:\C_Projects\Agent\cozmio\tmp_monitor_progress_pagedown.png`
- 点击 STOP 后，当前任务状态变为 `INTERRUPTED`：
  - `D:\C_Projects\Agent\cozmio\tmp_monitor_interrupted_top.png`

### 结论

- 验收项“出现右下角确认卡片”成立。
- 验收项“点击确认后产生真实 relay session”成立，真实 `session_id=c1bd4387-79c8-40a7-8206-74a7c4637231`。
- 验收项“执行端真实开始工作”成立，真实 Claude Code 进程 `pid=26992` 被启动。
- 验收项“页面能看到 progress”成立，监控页 recent progress 已展示真实 stdout 流。
- 验收项“页面能看到最终 result / error”成立；终态日志已证明页面状态同步到了 interrupted，且宿主已成功取回 `summary=Task interrupted`。
- 验收项“点击停止能真实 interrupt”成立；真实进程被 interrupt，终态事件为 `Interrupted`，不是仅本地改状态。

## 系统 Toast 产品链路验证（2026-04-25）

这轮不再把“代码接上”和“build 通过”当验收，而是专门收口系统层提醒到 Relay 的真实产品链路。

### 这轮先后修掉的真实断点

- `notification_manager.rs`
  - 确认/结果 Toast 原来把 `cozmio://...&token=...` 直接塞进 XML 属性，WinRT 真实报错 `Failed to load toast XML: 要求分号 (0xC00CE50D)`。
  - 已修为统一 XML escape，确认 Toast 和结果 Toast 都能被系统真正接受。
- `main.rs`
  - 单实例 deep-link 回调原来只做“不拉起主窗口”，但**没有处理 deep-link 本身**。
  - 已修为在 single-instance 回调里直接解析 `cozmio://...` 并调用 `process_protocol_action(...)`。
- `protocol_handler.rs`
  - Windows Toast action 实际回传的是 `cozmio://confirm/?...` / `cozmio://cancel/?...`，action 带尾斜杠。
  - 原逻辑把它识别成未知动作 `confirm/` / `cancel/`。
  - 已修为 `trim_end_matches('/')` 后再分发。
- `src-tauri` 控制面字段
  - `PendingConfirmationInfo`
  - `CurrentTaskInfo`
  - `RelayExecutionInfo`
  - `ActionRecord`
  - 都已补齐 `trace_id`，且未新增 `task_title / task_summary / why_now / source_hint / confidence / executor_prompt` 这类内容字段。

### 真实系统 Toast 证据

隐藏 Cozmio 主窗口后，以真实监控循环捕获前台 `continue-trigger.txt - Notepad`，系统日志记录：

```text
Window changed: continue-trigger.txt - Notepad
Model output: CONTINUE ...
Execution result: Confirmed
Creating pending notification for window='continue-trigger.txt - Notepad' process='Notepad.exe' trace_id=18a987d87e14eaec-3f8e4e11b3880e7c
Confirmation toast sent trace_id=18a987d87e14eaec-3f8e4e11b3880e7c token=18a987d87e1b8c303fb53a614dedd070
confirm_url=cozmio://confirm?trace_id=18a987d87e14eaec-3f8e4e11b3880e7c&token=18a987d87e1b8c303fb53a614dedd070
cancel_url=cozmio://cancel?trace_id=18a987d87e14eaec-3f8e4e11b3880e7c&token=18a987d87e1b8c303fb53a614dedd070
```

Toast 屏幕证据：

- `D:\C_Projects\Agent\cozmio\tmp\toastframes3\frame_11.png`
- `D:\C_Projects\Agent\cozmio\tmp\toastframes3\frame_12.png`

这两帧直接证明：

- 主窗口隐藏时，系统 Toast 真出现。
- Toast 标题为 `Cozmio - 任务确认`。
- Toast 上真实显示 `确认 / 取消` 两个 action button。

### 系统 Toast 取消链路证据

这轮先验证取消，不允许 dispatch。

真实协议日志：

```text
Single-instance deep-link activation received without forcing main window: cozmio://cancel/?trace_id=18a9889ab0fd31b8-59f2d8003c197b58&token=18a9889ab102d5dc5a13f643d928b6ac
protocol_handler: Parsing URL: cozmio://cancel/?trace_id=18a9889ab0fd31b8-59f2d8003c197b58&token=18a9889ab102d5dc5a13f643d928b6ac
cancel_pending_task_by_token action=cancel trace_id=18a9889ab0fd31b8-59f2d8003c197b58 token=18a9889ab102d5dc5a13f643d928b6ac
Notification token consumed trace_id=18a9889ab0fd31b8-59f2d8003c197b58 token=18a9889ab102d5dc5a13f643d928b6ac
PROTOCOL: Cancel processed trace_id=18a9889ab0fd31b8-59f2d8003c197b58
```

`action_log.jsonl` 对应记录：

```json
{"trace_id":"18a9889ab0fd31b8-59f2d8003c197b58","session_id":null,"system_action":"cancelled_by_protocol", ...}
```

这组证据证明：

- `trace_id + token` 真实进入后端。
- token 被一次性消费。
- cancel 不会 dispatch Relay，`session_id` 仍为 `null`。

### 系统 Toast 确认 -> Relay -> Claude -> Result 链路证据

真实协议日志：

```text
Single-instance deep-link activation received without forcing main window: cozmio://confirm/?trace_id=18a988a99d547f40-f66a6010b64dae40&token=18a988a99d5b5ea0f692b902126b0a20
protocol_handler: Parsing URL: cozmio://confirm/?trace_id=18a988a99d547f40-f66a6010b64dae40&token=18a988a99d5b5ea0f692b902126b0a20
confirm_pending_task_by_token action=confirm trace_id=18a988a99d547f40-f66a6010b64dae40 token=18a988a99d5b5ea0f692b902126b0a20
Notification token consumed trace_id=18a988a99d547f40-f66a6010b64dae40 token=18a988a99d5b5ea0f692b902126b0a20
Desktop host dispatching Relay request for 'continue-trigger.txt - Notepad' (Notepad.exe)
Desktop host connected to Relay at 127.0.0.1:7890
Protocol confirm dispatched relay trace_id=18a988a99d547f40-f66a6010b64dae40 session_id=120b7bc2-d424-429f-9829-242e8a92c237
Desktop host subscribed to Relay session 120b7bc2-d424-429f-9829-242e8a92c237
PROTOCOL: Confirm processed trace_id=18a988a99d547f40-f66a6010b64dae40 session_id=120b7bc2-d424-429f-9829-242e8a92c237
Desktop host received Relay event ... message=Started Claude Code process pid=44108
Desktop host received Relay event ... message=stdout: {"type":"system","subtype":"init", ...}
Desktop host received Relay event ... message=stdout: {"type":"result","subtype":"success", ...}
Desktop host received Relay event ... terminal=true status=3 ... message=session terminal: Completed
Desktop host fetched Relay result session=120b7bc2-d424-429f-9829-242e8a92c237 success=true summary=Task completed successfully
Result toast sent trace_id=18a988a99d547f40-f66a6010b64dae40 status=completed title=Cozmio - 任务完成
```

`action_log.jsonl` 对应 trace 贯通记录：

```json
{"trace_id":"18a988a99d547f40-f66a6010b64dae40","session_id":null,"system_action":"awaiting-confirmation", ...}
{"trace_id":"18a988a99d547f40-f66a6010b64dae40","session_id":"120b7bc2-d424-429f-9829-242e8a92c237","system_action":"relay-dispatched", ...}
{"trace_id":"18a988a99d547f40-f66a6010b64dae40","session_id":"120b7bc2-d424-429f-9829-242e8a92c237","system_action":"completed", ...}
```

这组证据证明：

- 同一个 `trace_id` 已经贯通：
  - judgment
  - system notification
  - confirmation
  - relay dispatch
  - session_id
  - progress
  - result
  - ActionRecord / history
- `dispatch` 返回真实 `session_id=120b7bc2-d424-429f-9829-242e8a92c237`。
- Claude Code 真实起进程 `pid=44108`。
- progress 真回到桌面宿主。
- completed result 真回到桌面宿主。
- completed 系统通知真实发出。

### 当前尚未补齐的验证项

- `failed` 结果系统通知：代码路径已具备，但这轮还没有拿到一条**真实 failed session** 的运行证据。
- `interrupted` 结果系统通知：代码路径已具备，但这轮还没有拿到一条**真实 interrupted session** 的运行证据。
- “直接点击 Toast 按钮并落到后端”已经拿到两部分真证据：
  - Toast 上按钮真实存在（截图）
  - 按钮对应 deep-link 被 single-instance 回调真实接收并处理（日志）
  - 但由于桌面自动化在部分轮次存在命中时序抖动，这轮最终用于闭环确认/取消的是 Toast 打出来的**真实 URL**；后续如果要把“按钮点击动作”也做成完全自动化证据，需要再补一层更稳定的 Windows UIA 点击脚本。
