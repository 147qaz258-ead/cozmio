# Relay Engine 实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标**：构建独立的执行接力层，支持多执行代理可插拔，通过本地 RPC 通信。

**架构思路**：Relay Engine 作为独立进程运行，通过 Named Pipe（Windows）/ Unix Domain Socket（Unix）与宿主通信。执行代理通过 Trait + 注册表模式实现可插拔。

**技术栈**：Rust + prost + tokio + Named Pipe / UDS

---

## 验收标准（功能成立验证）

**不是"代码骨架验证"，而是"产品能力验证"：**

| 验收 | 验证内容 | 真实验证方式 |
|------|---------|------------|
| V1 | Relay Engine 可独立启动 | 启动进程，检查存活 |
| V2 | 宿主通过 IPC  dispatch 任务到 Relay | 启动 Relay → relay-client IPC 连接 → dispatch → 检查 Relay 端是否收到 |
| V3 | dispatch 返回真实 session_id | session_id 是 Relay 生成的 UUID，可追踪 |
| V4 | Claude Code 进程真实启动 | dispatch 后检查系统进程列表 (tasklist/ps) |
| V5 | 进展写入 session | dispatch 后查 Relay session 的 progress 字段 |
| V6 | 结果写入 session | 执行完成后查 Relay session 的 result 字段 |
| V7 | interrupt 真实 kill 进程 | dispatch 长时间任务 → interrupt → 检查进程列表确认已终止 |
| V8 | 进展实时推送宿主 | IPC 订阅 → 执行代理输出 → 宿主收到进展事件 |

---

## 任务1：Relay Engine 项目初始化

**涉及文件**：

- 创建：`relay-engine/Cargo.toml`
- 创建：`relay-engine/build.rs`
- 创建：`relay-engine/src/main.rs`
- 创建：`relay-engine/src/proto/relay.proto`
- 创建：`relay-engine/src/error.rs`
- 创建：`relay-engine/src/session.rs`
- 创建：`relay-engine/src/agent.rs`
- 创建：`relay-engine/src/transport/mod.rs`
- 创建：`relay-engine/src/transport/windows.rs`
- 创建：`relay-engine/src/transport/unix.rs`
- 创建：`relay-engine/src/agents/mod.rs`
- 创建：`relay-engine/src/agents/claude_code.rs`

- [ ] **步骤1：创建目录结构**

```bash
mkdir -p relay-engine/src/proto
mkdir -p relay-engine/src/transport
mkdir -p relay-engine/src/agents
mkdir -p relay-client/src
mkdir -p relay-client/tests
```

- [ ] **步骤2-13：按设计文档实现所有核心模块**

（详见原方案任务1的步骤2-13）

- [ ] **步骤14：编译验证**

```bash
cd relay-engine && cargo build 2>&1
```

预期结果：编译通过

- [ ] **步骤15：提交代码**

```bash
git add relay-engine/
git commit -m "feat(relay): initial project structure"
```

---

## 任务2：V1 验证 - Relay Engine 独立启动

**真实验证**：启动进程，检查存活

- [ ] **步骤1：启动 Relay Engine（后台）**

```bash
cd relay-engine && cargo run --release 2>&1 &
sleep 3
ps aux | grep relay-engine | grep -v grep
```

预期结果：能看到 `relay-engine` 进程正在运行

- [ ] **步骤2：停止进程**

```bash
taskkill //F //IM relay-engine.exe 2>/dev/null || pkill relay-engine
```

预期结果：进程被成功终止

- [ ] **步骤3：提交验证记录**

```bash
git add -A && git commit -m "verify(relay): V1 - Relay Engine starts independently"
```

---

## 任务3：V2+V3+V4+V5+V6 验证 - Relay 完整执行链

**真实验证**（不拆分，要完整闭环）：

```
启动 Relay Engine
    ↓
relay-client 通过 IPC 连接
    ↓
发送 DispatchRequest (agent=claude-code, task="echo hello")
    ↓
Relay 收到请求，创建 Session
    ↓
Relay 调用 ClaudeCodeConnector.dispatch()
    ↓
ClaudeCodeConnector 启动 claude 子进程
    ↓
验证：子进程在系统进程列表中 (V4)
    ↓
子进程输出被捕获，写入 Session progress (V5)
    ↓
子进程结束，结果写入 Session result (V6)
    ↓
relay-client 通过 IPC 查询 result (V2+V3)
```

**注意**：此任务依赖传输层的 IPC 通信实现。

### 子任务3a：传输层 IPC 通信实现

**涉及文件**：

- 修改：`relay-engine/src/transport/windows.rs`
- 修改：`relay-engine/src/transport/unix.rs`

- [ ] **步骤1：实现 Windows Named Pipe Server + Client**

```rust
// windows.rs
use std::io::{Read, Write};
use windows::named_pipe::{
    PipeServer, PipeClient, PipeMode, ServerSide, ClientSide,
    SECURITY_ATTRIBUTES, PSECURITY_DESCRIPTOR, ACL,
};

pub struct PipeConnection {
    server: PipeServer,
}

impl PipeConnection {
    pub fn new(pipe_name: &str) -> Result<Self> {
        let security = create_pipe_security()?;
        let server = PipeServer::new(pipe_name, &security, 1024, 1024, 0, 0)?;
        Ok(Self { server })
    }

    pub fn connect_client(&self) -> Result<ClientConnection> {
        let client = self.server.connect()?;
        Ok(ClientConnection { stream: Some(client) })
    }
}

pub struct ClientConnection {
    stream: Option<PipeClient>,
}

impl ClientConnection {
    pub fn connect(pipe_name: &str) -> Result<Self> {
        let client = PipeClient::connect(pipe_name)?;
        Ok(Self { stream: Some(client) })
    }

    pub fn send(&self, data: &[u8]) -> Result<()> {
        if let Some(ref s) = self.stream {
            s.write_all(data)?;
            s.flush()?;
        }
        Ok(())
    }

    pub fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        if let Some(ref s) = self.stream {
            Ok(s.read(buf)?)
        } else {
            Err(Error::Transport("connection closed".to_string()))
        }
    }
}

fn create_pipe_security() -> Result<SECURITY_ATTRIBUTES> {
    // Windows 命名管道安全描述符
    let sd = windows::Win32::Security::SECURITY_DESCRIPTOR::new();
    // 设置为本地系统权限
    Ok(SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: Box::into_raw(Box::new(sd)) as *mut _,
        bInheritHandle: false.into(),
    })
}
```

- [ ] **步骤2：实现 Unix Domain Socket Server + Client**

```rust
// unix.rs
use std::os::unix::net::{UnixListener, UnixStream, SocketAddr};
use std::io::{Read, Write};

pub struct UnixConnection {
    listener: UnixListener,
}

impl UnixConnection {
    pub fn new(socket_path: &str) -> Result<Self> {
        std::fs::remove_file(socket_path).ok();
        let listener = UnixListener::bind(socket_path)?;
        listener.set_nonblocking(true)?;
        Ok(Self { listener })
    }

    pub fn accept(&self) -> Result<UnixStream> {
        match self.listener.accept() {
            Ok((stream, _)) => Ok(stream),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Err(Error::Transport("would block".to_string()))
            }
            Err(e) => Err(Error::Transport(e.to_string())),
        }
    }
}
```

- [ ] **步骤3：修改 main.rs 添加协议解析和请求处理**

```rust
// main.rs - 添加协议解析和请求处理

#[tokio::main]
async fn main() -> Result<()> {
    // ... 初始化代码 ...

    // 事件循环
    loop {
        match transport.accept() {
            Ok(mut conn) => {
                // 处理连接
                tokio::spawn(async move {
                    loop {
                        let mut buf = vec![0u8; 4096];
                        match conn.recv(&mut buf) {
                            Ok(n) if n > 0 => {
                                // 解析请求
                                if let Ok(request) = parse_request(&buf[..n]) {
                                    handle_request(&request, &mut conn);
                                }
                            }
                            Ok(0) | Err(_) => break,
                            Err(e) if is_would_block(&e) => {
                                tokio::time::sleep(Duration::from_millis(10)).await;
                                continue;
                            }
                            Err(e) => break,
                        }
                    }
                });
            }
            Err(e) if is_would_block(&e) => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
            Err(e) => {
                log::error!("accept error: {}", e);
            }
            _ => {}
        }
    }
}

fn parse_request(buf: &[u8]) -> Result<DispatchRequest> {
    // 使用 prost 解析 protobuf
    prost::Message::decode(buf).map_err(|e| Error::Protocol(e.to_string()))
}

fn handle_request(req: &DispatchRequest, conn: &mut Box<dyn Connection>) {
    match req {
        DispatchRequest { agent_name, original_suggestion, dispatched_task } => {
            // 获取 agent
            if let Some(agent) = agent_registry.get(agent_name) {
                let task = ExecutionTask {
                    original_suggestion: original_suggestion.clone(),
                    dispatched_task: dispatched_task.clone(),
                    agent_name: agent_name.clone(),
                };
                let session_id = agent.dispatch(&task).expect("dispatch failed");
                // 发送响应
                let resp = DispatchResponse {
                    session_id: session_id.to_string(),
                    status: SessionStatus::Running as i32,
                };
                let mut buf = vec![];
                prost::Message::encode(&resp, &mut buf).unwrap();
                conn.send(&buf).unwrap();
            }
        }
    }
}
```

- [ ] **步骤4：实现 relay-client IPC 连接**

```rust
// relay-client/src/client.rs

pub struct RelayClient {
    connection: Box<dyn Connection>,
}

impl RelayClient {
    pub fn connect(address: &str) -> Result<Self> {
        #[cfg(windows)]
        let connection = Box::new(named_pipe::ClientConnection::connect(address)?);
        #[cfg(unix)]
        let connection = Box::new(unix_stream::UnixStream::connect(address)?);

        Ok(Self { connection })
    }

    pub fn dispatch(&self, agent: &str, suggestion: &str, task: &str) -> Result<String> {
        let req = DispatchRequest {
            agent_name: agent.to_string(),
            original_suggestion: suggestion.to_string(),
            dispatched_task: task.to_string(),
        };

        let mut buf = vec![];
        prost::Message::encode(&req, &mut buf).unwrap();
        self.connection.send(&buf)?;

        let mut resp_buf = vec![0u8; 4096];
        let n = self.connection.recv(&mut resp_buf)?;
        let resp: DispatchResponse = prost::Message::decode(&resp_buf[..n])?;

        Ok(resp.session_id)
    }
}
```

- [ ] **步骤5：编写端到端测试脚本**

```bash
#!/bin/bash
# test_relay_e2e.sh

# 1. 启动 Relay Engine
cd relay-engine && cargo run --release &
RELAY_PID=$!
sleep 3

# 2. 发送 dispatch 请求
SESSION_ID=$(curl -X POST http://localhost:8080/dispatch \
  -d '{"agent":"claude-code","suggestion":"test","task":"echo hello"}' 2>/dev/null | jq -r '.session_id')

# 3. 验证 session_id 是真实 UUID
if [[ "$SESSION_ID" =~ ^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$ ]]; then
    echo "V3 PASS: session_id is valid UUID: $SESSION_ID"
else
    echo "V3 FAIL: session_id is not valid: $SESSION_ID"
fi

# 4. 验证 Claude Code 进程启动 (V4)
sleep 1
if pgrep -f "claude.*echo hello" > /dev/null; then
    echo "V4 PASS: claude process found"
else
    echo "V4 FAIL: claude process not found"
fi

# 5. 查询 progress (V5)
sleep 2
PROGRESS=$(curl http://localhost:8080/progress/$SESSION_ID 2>/dev/null | jq -r '.entries | length')
if [ "$PROGRESS" -gt 0 ]; then
    echo "V5 PASS: progress entries: $PROGRESS"
else
    echo "V5 FAIL: no progress entries"
fi

# 6. 等待执行完成，查询 result (V6)
sleep 5
RESULT=$(curl http://localhost:8080/result/$SESSION_ID 2>/dev/null)
if echo "$RESULT" | jq -e '.raw_output' > /dev/null; then
    echo "V6 PASS: result received"
else
    echo "V6 FAIL: no result"
fi

# 清理
kill $RELAY_PID 2>/dev/null
```

- [ ] **步骤6：运行端到端测试**

```bash
chmod +x test_relay_e2e.sh && ./test_relay_e2e.sh
```

预期结果：
- V3: session_id 是有效 UUID
- V4: claude 进程在系统进程列表中
- V5: progress 有条目
- V6: result 有输出

- [ ] **步骤7：提交代码**

```bash
git add relay-engine/ relay-client/
git commit -m "verify(relay): V2+V3+V4+V5+V6 - full execution chain e2e"
```

---

## 任务4：V7 验证 - interrupt 真实终止进程

**真实验证**：

```
启动 Relay Engine
    ↓
dispatch 长时间任务 (sleep 60)
    ↓
进程在系统进程列表中
    ↓
发送 InterruptRequest via IPC
    ↓
检查进程列表确认已终止
```

- [ ] **步骤1：编写 interrupt 测试**

```bash
#!/bin/bash
# test_interrupt.sh

# 1. 启动 Relay
cd relay-engine && cargo run --release &
RELAY_PID=$!
sleep 3

# 2. dispatch 长时间任务
SESSION_ID=$(curl -X POST http://localhost:8080/dispatch \
  -d '{"agent":"claude-code","suggestion":"long","task":"sleep 30"}' 2>/dev/null | jq -r '.session_id')

# 3. 等待进程启动
sleep 2

# 4. 检查进程存在
if pgrep -f "claude.*sleep 30" > /dev/null; then
    echo "Process exists before interrupt"
else
    echo "V7 FAIL: process not found before interrupt"
    kill $RELAY_PID
    exit 1
fi

# 5. 发送 interrupt
curl -X POST http://localhost:8080/interrupt/$SESSION_ID 2>/dev/null

# 6. 等待并检查进程已终止
sleep 2
if pgrep -f "claude.*sleep 30" > /dev/null; then
    echo "V7 FAIL: process still running after interrupt"
else
    echo "V7 PASS: process terminated after interrupt"
fi

# 7. 清理
kill $RELAY_PID 2>/dev/null
```

- [ ] **步骤2：运行测试**

```bash
chmod +x test_interrupt.sh && ./test_interrupt.sh
```

预期结果：进程在 interrupt 后不再存在于系统进程列表

- [ ] **步骤3：提交验证记录**

```bash
git add -A && git commit -m "verify(relay): V7 - interrupt kills process"
```

---

## 任务5：V8 验证 - 进展实时推送

**真实验证**：

```
启动 Relay Engine
    ↓
relay-client 订阅 Session 进展
    ↓
dispatch 任务
    ↓
relay-client 通过 IPC/WebSocket 收到进展事件
```

**注意**：此验证需要传输层支持订阅/回调机制

- [ ] **步骤1：实现进展订阅机制**

```rust
// transport/mod.rs - 添加订阅支持

pub trait Transport: Send + Sync {
    fn listen(&self) -> Result<()>;
    fn accept(&self) -> Result<Box<dyn Connection>>;
    fn address(&self) -> &str;
}

pub trait SubscribableTransport: Transport {
    fn subscribe(&self, session_id: &str, callback: Box<dyn Fn(ProgressEvent) + Send>);
}
```

- [ ] **步骤2：relay-client 订阅测试**

```bash
#!/bin/bash
# test_progress_subscription.sh

# 1. 启动 Relay
cd relay-engine && cargo run --release &
RELAY_PID=$!
sleep 3

# 2. 订阅进展 (使用 websocket 或 long-poll)
SUBSCRIPTION=$(curl -N http://localhost:8080/subscribe/$SESSION_ID 2>/dev/null &)

# 3. dispatch 任务
curl -X POST http://localhost:8080/dispatch \
  -d '{"agent":"claude-code","suggestion":"test","task":"echo line1 && sleep 1 && echo line2"}' 2>/dev/null

# 4. 验证收到多批次进展
sleep 3

# 5. 检查订阅输出
if curl_result_contains "line1" && curl_result_contains "line2"; then
    echo "V8 PASS: progress events received"
else
    echo "V8 FAIL: progress events not received"
fi

# 6. 清理
kill $RELAY_PID 2>/dev/null
```

- [ ] **步骤3：提交验证记录**

```bash
git add -A && git commit -m "verify(relay): V8 - progress subscription works"
```

---

## 总结

| 验收 | 验证方式 | 状态 |
|------|---------|------|
| V1 | 启动进程，检查存活 | pending |
| V2+V3+V4+V5+V6 | 端到端测试：启动 Relay → IPC dispatch → 检查进程/progress/result | pending |
| V7 | 端到端测试：dispatch 长任务 → interrupt → 检查进程列表 | pending |
| V8 | 端到端测试：订阅进展 → dispatch → 验证收到进展事件 | pending |

**关键原则**：
- 不使用本地 mock 替代真实链路
- 验证必须是"进程级"检查，不是"内存状态"检查
- IPC 通信必须是真实的，不是内部调用
