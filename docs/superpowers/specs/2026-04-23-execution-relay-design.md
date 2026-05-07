# 执行接力层 (Relay Engine) 设计文档

> **智能执行体须知**：设计方案已确认，可进入实施阶段。

**目标**：构建独立的执行接力层，不依赖桌面端，支持多执行代理可插拔。

**架构定位**：独立进程 / 长期基础设施 / 本地 IPC 通信

---

## 1. 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                    桌面端 (Cozmio Host)                      │
│              上游宿主，仅负责观察/建议/确认                     │
└──────────────────────────┬────────────────────────────────┘
                           │ IPC (Named Pipe / UDS)
┌──────────────────────────▼────────────────────────────────┐
│                    Relay Engine (独立进程)                    │
│              核心基础设施，不依赖任何宿主                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │ SessionManager   │  │ AgentRegistry   │                   │
│  │ - 会话生命周期    │  │ - 执行代理注册   │                   │
│  │ - 状态追踪       │  │ - 代理心跳      │                   │
│  │ - 进展日志       │  │                 │                   │
│  └────────┬────────┘  └────────┬────────┘                   │
│           │                    │                             │
│  ┌────────▼────────────────────▼────────┐                   │
│  │         ExecutionAgentTrait           │                   │
│  │  ┌────────────┐  ┌────────────┐     │                   │
│  │  │ClaudeCode  │  │ Future     │     │  ← 可插拔连接器    │
│  │  │Connector   │  │ Connectors │     │                   │
│  │  └────────────┘  └────────────┘     │                   │
│  └─────────────────────────────────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. 技术选型

| 层级 | 技术 | 理由 |
|------|------|------|
| 传输层 | Named Pipe (Win) / UDS (Unix) | 本地 IPC，不暴露网络端口 |
| 协议 | Protobuf | 稳定、跨语言、多端扩展 |
| 执行模型 | 异步非阻塞 | CLI 执行不阻塞 Relay |
| 进展推送 | 订阅模式 | 实时流式更新宿主 |
| 代理扩展 | Trait + 注册表 | 新代理只需实现 Agent trait |

---

## 3. 核心组件

### 3.1 Relay Engine (独立二进制)

```
relay-engine/
├── src/
│   ├── main.rs              # 入口，初始化 Pipe/UDS
│   ├── session.rs           # SessionManager: 会话生命周期
│   ├── agent.rs             # AgentRegistry + ExecutionAgent trait
│   ├── error.rs             # 统一错误类型
│   ├── transport/
│   │   ├── mod.rs           # Transport trait 定义
│   │   ├── windows.rs       # NamedPipe 实现
│   │   └── unix.rs          # UnixDomainSocket 实现
│   └── agents/
│       └── claude_code.rs   # Claude Code CLI 连接器
├── proto/
│   └── relay.proto          # IPC 协议定义
├── tests/
│   └── integration_test.rs  # 集成测试
├── Cargo.toml
└── README.md
```

### 3.2 ExecutionAgent Trait

```rust
/// 执行代理接口
trait ExecutionAgent: Send + Sync {
    /// 代理名称 (e.g., "claude-code")
    fn name(&self) -> &str;

    /// 派发任务，返回会话 ID
    fn dispatch(&self, task: &ExecutionTask) -> Result<SessionId, Error>;

    /// 获取当前状态
    fn status(&self, session: &SessionId) -> Result<SessionStatus, Error>;

    /// 获取进展日志
    fn progress(&self, session: &SessionId) -> Result<Vec<ProgressEntry>, Error>;

    /// 中断执行
    fn interrupt(&self, session: &SessionId) -> Result<(), Error>;

    /// 获取最终结果
    fn result(&self, session: &SessionId) -> Result<ExecutionResult, Error>;
}

/// 执行任务
struct ExecutionTask {
    pub original_suggestion: String,   // 原始建议
    pub dispatched_task: String,       // 派发给代理的任务文本
    pub agent_name: String,           // 执行代理名称
}

/// 会话 ID
struct SessionId(String);

/// 执行状态
enum SessionStatus {
    Pending,     // 已派发
    Running,     // 执行中
    Waiting,     // 等待中
    Completed,   // 已完成
    Failed,      // 执行失败
    Interrupted,  // 已中断
}

/// 进展条目
struct ProgressEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub level: LogLevel,  // INFO, WARN, ERROR
}

/// 执行结果
struct ExecutionResult {
    pub summary: String,           // 结果摘要
    pub raw_output: String,       // 原始输出
    pub duration_secs: u64,       // 执行时长
    pub success: bool,            // 是否成功
    pub error_message: Option<String>,  // 错误信息
}
```

### 3.3 传输层 Trait

```rust
/// 传输层 trait
trait Transport: Send + Sync {
    /// 发送消息
    fn send(&self, msg: &ProtoMessage) -> Result<(), Error>;

    /// 接收消息 (blocking)
    fn recv(&self) -> Result<ProtoMessage, Error>;

    /// 接收消息 (non-blocking)
    fn try_recv(&self) -> Result<Option<ProtoMessage>, Error>;

    /// 订阅会话进展 (async callback)
    fn subscribe(&self, session: SessionId, callback: Box<dyn Fn(ProgressEntry) + Send>);
}
```

### 3.4 SessionManager

```rust
struct SessionManager {
    sessions: RwLock<HashMap<SessionId, Session>>,
    agent_registry: AgentRegistry,
}

impl SessionManager {
    /// 创建新会话
    fn create_session(&self, task: ExecutionTask) -> Result<SessionId, Error>;

    /// 派发到执行代理
    fn dispatch(&self, session_id: &SessionId) -> Result<(), Error>;

    /// 获取会话状态
    fn status(&self, session_id: &SessionId) -> Result<SessionStatus, Error>;

    /// 获取进展日志
    fn progress(&self, session_id: &SessionId) -> Result<Vec<ProgressEntry>, Error>;

    /// 中断会话
    fn interrupt(&self, session_id: &SessionId) -> Result<(), Error>;

    /// 获取最终结果
    fn result(&self, session_id: &SessionId) -> Result<ExecutionResult, Error>;

    /// 推进状态 (内部调用)
    fn advance_status(&self, session_id: &SessionId, status: SessionStatus);
}
```

---

## 4. IPC 协议

### 4.1 Protocol Buffer 定义

```protobuf
syntax = "proto3";

package relay;

message DispatchRequest {
    string agent_name = 1;
    string original_suggestion = 2;
    string dispatched_task = 3;
}

message DispatchResponse {
    string session_id = 1;
    SessionStatus status = 2;
}

message StatusRequest {
    string session_id = 1;
}

message StatusResponse {
    string session_id = 1;
    SessionStatus status = 2;
    int64 started_at = 3;
    int64 updated_at = 4;
    int64 duration_secs = 5;
}

message ProgressRequest {
    string session_id = 1;
}

message ProgressResponse {
    string session_id = 1;
    repeated ProgressEntry entries = 2;
}

message InterruptRequest {
    string session_id = 1;
}

message InterruptResponse {
    bool success = 1;
}

message ResultRequest {
    string session_id = 1;
}

message ResultResponse {
    string session_id = 1;
    ExecutionResult result = 2;
}

message ProgressEvent {
    string session_id = 1;
    int64 timestamp = 2;
    string message = 3;
    LogLevel level = 4;
}

enum SessionStatus {
    PENDING = 0;
    RUNNING = 1;
    WAITING = 2;
    COMPLETED = 3;
    FAILED = 4;
    INTERRUPTED = 5;
}

enum LogLevel {
    INFO = 0;
    WARN = 1;
    ERROR = 2;
}

message ExecutionResult {
    string summary = 1;
    string raw_output = 2;
    int64 duration_secs = 3;
    bool success = 4;
    string error_message = 5;
}

message ProgressEntry {
    int64 timestamp = 1;
    string message = 2;
    LogLevel level = 3;
}
```

---

## 5. Claude Code 连接器实现

```rust
struct ClaudeCodeConnector {
    cli_path: String,
}

impl ExecutionAgent for ClaudeCodeConnector {
    fn name(&self) -> &str {
        "claude-code"
    }

    fn dispatch(&self, task: &ExecutionTask) -> Result<SessionId, Error> {
        let session_id = SessionId::new();

        // 启动: claude -p "<task>" --verbose
        // stderr -> stdout 合并，实时捕获
        let child = Command::new(&self.cli_path)
            .args(["-p", &task.dispatched_task, "--verbose"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // 异步读取输出，更新 session 进展
        // ...

        Ok(session_id)
    }

    fn status(&self, session: &SessionId) -> Result<SessionStatus, Error> {
        // 查询进程状态
    }

    fn interrupt(&self, session: &SessionId) -> Result<(), Error> {
        // 杀进程
    }

    fn result(&self, session: &SessionId) -> Result<ExecutionResult, Error> {
        // 收集最终输出
    }
}
```

---

## 6. 宿主绑定 SDK

```rust
/// 桌面端使用的轻量 SDK
pub struct RelayClient {
    transport: Box<dyn Transport>,
}

impl RelayClient {
    /// 连接 Relay Engine
    pub fn connect(pipe_path: &str) -> Result<Self, Error> {
        #[cfg(windows)]
        let transport = NamedPipeTransport::connect(pipe_path)?;
        #[cfg(unix)]
        let transport = UnixSocketTransport::connect(pipe_path)?;
        Ok(Self { transport })
    }

    /// 派发任务
    pub fn dispatch(&self, agent: &str, suggestion: &str, task: &str) -> Result<String, Error> {
        let req = DispatchRequest {
            agent_name: agent.to_string(),
            original_suggestion: suggestion.to_string(),
            dispatched_task: task.to_string(),
        };
        self.transport.send(&req)?;
        let resp = self.transport.recv()?;
        // 解析 DispatchResponse
    }

    /// 订阅进展 (async)
    pub fn subscribe<F>(&self, session: &str, handler: F) -> Result<(), Error>
    where F: Fn(ProgressEntry) + Send + 'static;

    /// 获取最终结果
    pub fn result(&self, session: &str) -> Result<ExecutionResult, Error>;
}
```

---

## 7. 状态流转

```
                    ┌──────────────┐
                    │   PENDING    │
                    └──────┬───────┘
                           │ dispatch()
                           ▼
                    ┌──────────────┐
              ┌─────│   RUNNING    │─────┐
              │     └──────────────┘     │
              │            │              │
        interrupt()   progress()    exit code
              │            │              │
              ▼            ▼              ▼
       ┌───────────┐ ┌───────────┐ ┌─────────────┐
       │INTERRUPTED│ │ WAITING   │ │  COMPLETED  │
       └───────────┘ └───────────┘ │  or FAILED  │
                                  └─────────────┘
```

---

## 8. 文件结构

```
cozmio/
├── relay-engine/                    # 独立 Relay Engine
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── session.rs
│   │   ├── agent.rs
│   │   ├── error.rs
│   │   ├── transport/
│   │   │   ├── mod.rs
│   │   │   ├── windows.rs
│   │   │   └── unix.rs
│   │   └── agents/
│   │       └── claude_code.rs
│   └── proto/
│       └── relay.proto
│
├── src-tauri/                       # 桌面端 (宿主)
│   └── src/
│       ├── main_loop.rs            # 修改：派发到 Relay
│       └── ...
│
└── docs/
    └── superpowers/
        └── specs/
            └── 2026-04-23-execution-relay-design.md
```

---

## 9. 实施顺序

1. **Relay Engine 核心**
   - 定义 `ExecutionAgent` trait
   - 实现 `SessionManager`
   - 实现 Named Pipe / UDS 传输层

2. **Claude Code 连接器**
   - 实现 `ClaudeCodeConnector`
   - 支持 `dispatch` / `status` / `progress` / `interrupt`

3. **宿主 SDK**
   - 实现 `RelayClient`
   - 连接、派发、订阅、获取结果

4. **桌面端集成**
   - 修改 `executor.rs` 派发到 Relay
   - 添加执行面板 UI

---

## 10. 成功标准

- Relay Engine 可独立启动，不依赖桌面端
- Claude Code 连接器可正常派发任务并回收结果
- 多宿主可同时连接（Session 隔离）
- 进展日志实时推送给宿主
- 宿主可中断执行
