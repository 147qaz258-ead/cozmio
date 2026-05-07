# Cozmio Local Agent Box H1 实施方案

> **智能执行体须知**：本方案是 Cozmio 硬件线的 H1 落地计划。
> 目标：树莓派作为本地 Agent Box 真实进入执行链路，不是 mock，不靠 PC 模型。
> 本计划**仅限硬件线**，不涉及 Suggestion Memory Core、向量检索、长期记忆、Memory Competition。
> 执行时请关注每个阶段的验收标准，而不是文件数量。

---

## 硬件线 Roadmap

| 阶段 | 目标 | 核心问题 |
|------|------|---------|
| **H1** | Local Agent Box 真机闭环 | 树莓派能作为本地推理节点接入现有链路，完成一次真实执行 |
| **H2** | Hardware-first Context Input | 探索 HDMI/USB-C/无线镜像等硬件输入，减少对桌面端完整软件的依赖 |
| **H3** | Hardware Execution Bridge | 探索 USB HID / 系统执行桥 / 浏览器控制桥，让硬件盒参与执行 |
| **H4** | Productized Local AI Device | 外壳、配网、模型管理、安全、更新、稳定运行 |

**为什么叫 H1**：H1 是硬件线的第一个验证点——证明 Cozmio 的推理/计划生成能力可以从 PC 软件中独立出来，运行在一个本地硬件节点上，并进入真实执行闭环。

---

## H1 总目标

把树莓派包装成 Cozmio 的第一代本地 Agent Box，接入现有 Relay Engine 中间层。

H1 必须证明：

```
桌面端 context_bundle
→ Relay inference task
→ 树莓派 cozmio-box-worker
→ 树莓派本地模型 payload_text
→ Relay result
→ 桌面端/执行端完成一个真实动作
→ execution_result
```

**H1 完成标准不是 cargo build，不是 worker 返回文本，而是完整链路跑通且能演示。**

---

## H1 Completion Gate

**H1 只有在以下全部完成时才能宣布完成：**

- [ ] 树莓派真机运行 cozmio-box-worker
- [ ] Relay 显示 `local_agent_box` online
- [ ] 桌面端发送真实 context_bundle
- [ ] Relay 将 task 路由到树莓派
- [ ] 树莓派通过 Box Model Runtime 调用本地 llama.cpp/GGUF provider
- [ ] payload_text 由树莓派本地模型生成（不是 mock，不是固定文本）
- [ ] payload_text 返回桌面端/执行端
- [ ] 执行端完成一个真实可见动作
- [ ] execution_result 被记录
- [ ] trace 能串起 `Desktop → Relay → Pi → Relay → Desktop/Executor → Result`
- [ ] PC 端**没有**运行该模型（日志为证）

**11 条必须全部打勾，缺一不可宣布 H1 完成。**

---

## No Fake Completion

**以下内容不能作为 H1 完成依据：**

```
- cargo build 通过
- worker online
- heartbeat 正常
- mock provider 返回文本
- 固定文本 inference 往返
- 单元测试通过
- 写完文件结构
- 只生成日志但没有真实执行动作
```

**判断标准**：必须有真实模型推理（llama.cpp + GGUF）、真实 payload_text、真实执行动作。不是代码写完了就行，是链路跑通了才算。

---

## 核心原则

```
1. 中间层只传信封 + payload_text，不解析模型语义。
2. 不把模型自然语言回复拆成 current_task / next_plan / confidence 后硬执行。
3. 树莓派必须是真实参与链路的本地 Agent Box，不是 PC mock。
4. PC 端不运行该模型。
5. Ollama 只能作为开发期临时对照 provider，不作为正式架构依赖。
6. 硬件端必须有 Cozmio 自有的 Box Model Runtime 层，对 Relay 只暴露 payload_text。
7. Box Model Runtime 通过可替换 provider 调用本地模型 runtime（H1 默认 llama.cpp + GGUF）。
8. 模型文件由 Cozmio Box 自己管理，不依赖第三方模型管理服务。
9. H1 验收必须包含真实树莓派（不是 x86 mock）。
10. 不重新开发截图/窗口监听；桌面端感知能力继续复用。
11. 不新造中间层；复用现有 Relay Engine。
12. 本计划仅限硬件线，不混入记忆系统、向量检索、Suggestion Memory Core。
```

---

## 产品边界

| 组件 | 职责 | 不负责 |
|------|------|--------|
| 桌面端 src-tauri | 感知桥、上下文提供、执行桥、结果展示 | 模型推理、回复生成 |
| Relay Engine | 中间路由、worker 注册、状态记录、结果回传 | 解析模型语义 |
| 树莓派 cozmio-box-worker | 本地 Agent 节点、模型运行、自然语言回复生成、状态上报 | 截图/窗口采集、执行 |

---

## 通信模型定案

**Box Worker 主动连接 Relay，保持长连接。**

原因：树莓派在内网/NAT 后，适合作为 client 主动连 Relay，避免 Relay 反连的网络问题。

```
Box Worker → Relay：register（启动时一次）
Box Worker → Relay：heartbeat + status（定时）
Relay → Box Worker：inference task（通过长连接下发）
Box Worker → Relay：payload_text result（同 channel 或关联 response）
Relay → Desktop：result callback
```

---

## Phase 1：硬件接入确认

**目标**：确认树莓派真机可用、可 SSH、可部署。

### 连接信息

> **敏感信息**：连接凭据存储在 `.local-device.toml`（已加入 .gitignore），不进入代码仓库。

读取方式：执行时从 `.local-device.toml` 读取，不在计划/代码/提交中明文出现。

### 任务

1. 从 `.local-device.toml` 读取连接信息
2. 通过 SSH 连接树莓派
3. 运行 hardware probe 收集硬件信息（OS、架构、RAM、磁盘、网络、温度）
4. 确认是否 64-bit（决定交叉编译 target）
5. 确认部署路径（`/opt/cozmio` 或 `/home/pi/cozmio`）
6. 确认 llama.cpp 是否已安装或可安装
7. 确认 GGUF 模型文件是否可用（若无，先准备小尺寸量化模型）

### 交付物

- `cozmio-box-worker/scripts/hardware_probe.sh` — 硬件探测脚本
- 树莓派硬件报告（probe 输出，包含 arch、RAM、disk、IP、温度）

### 验收

```
SSH 连接成功（用 .local-device.toml 凭据）
hardware_probe.sh 输出完整硬件信息
报告包含：OS、架构、RAM、磁盘、IP、温度
确认交叉编译 target
确认部署路径
确认 llama.cpp 可用性或安装方式
```

---

## Phase 2：cozmio-box-worker 空壳上线

**目标**：树莓派作为 Cozmio 硬件节点在 Relay 注册并上报心跳。

### 任务

1. 新建 `cozmio-box-worker/` Cargo 项目
2. 实现配置读取（从 `/opt/cozmio/config/box-worker.toml` 读取 worker_id、relay_addr、heartbeat_interval）
3. 实现 TCP 长连接接入 Relay（复用现有 Relay protobuf 协议，端口 7892）
4. 实现 worker register（启动时发送 worker_type=local_agent_box）
5. 实现 heartbeat + status 上报（online/idle/busy + model_status）
6. 实现 Box Model Runtime 骨架（预留 provider 扩展口，Phase 2 暂接 mock_provider）
7. 实现日志输出（tracing，日志路径 `/opt/cozmio/logs/box-worker.log`）
8. 支持 systemd service

### 交付物

```
cozmio-box-worker/src/main.rs               # 入口
cozmio-box-worker/src/config.rs             # 配置读取
cozmio-box-worker/src/protocol.rs           # TCP 通信层
cozmio-box-worker/src/box_model_runtime.rs  # 运行时骨架（mock provider）
cozmio-box-worker/src/model_provider.rs     # provider trait
cozmio-box-worker/src/providers/mock.rs      # mock provider（仅链路验证）
cozmio-box-worker/deployment/
    cozmio-box-worker.service               # systemd unit（无 Ollama 依赖）
cozmio-box-worker/scripts/
    start.sh / stop.sh / status.sh / logs.sh
```

### 验收

```
Relay 日志显示：box worker register 成功
Relay 日志显示：worker_id、worker_type=local_agent_box
Relay 能看到 heartbeat 定时更新（model_status: mock）
拔掉网线 30s 后 Relay 能感知 offline
重新连接后 Relay 能看到 re-register
```

**此阶段用 mock provider，不要求真实模型调用。**

---

## Phase 3：Relay worker registry 与任务通道

**目标**：Relay 能注册、管理、路由 inference task 到 Box Worker。

### 任务

1. Relay 增加 worker registry（内存 HashMap，worker_id → WorkerInfo）
2. Relay 管理 Box Worker 长连接（记录活跃连接）
3. Relay 增加 inference task queue
4. Relay 实现 worker 选择策略（优先选 online + idle 的 local_agent_box）
5. Relay 下发 task 到 Box Worker 长连接
6. Relay 接收 Box Worker payload_text result
7. Relay 处理 timeout、worker offline、worker busy
8. Relay 返回 result 给桌面端

### 交付物

```
relay-engine/src/worker_registry.rs    # worker 注册表
relay-engine/src/worker_session.rs     # 长连接管理
relay-engine/src/inference_router.rs   # task 路由逻辑
relay-engine/src/proto/mod.rs         # 新增 WorkerRegister/Heartbeat/Inference 消息
relay-engine/src/main.rs              # 集成 worker listener（端口 7892）
```

### 验收

```
桌面端发送测试 context_bundle
Relay 生成 inference task
Relay 路由到树莓派 Box Worker（看日志）
Box Worker 收到 task（看 Box Worker 日志）
Box Worker 返回固定测试文本（mock provider）
Relay 收到 result 并回传桌面端
完整往返成功，无丢包，无死锁
```

**此阶段用固定文本/moc回复，不接真实模型。**

---

## Phase 4：Box Model Runtime 与本地 GGUF 模型

**目标**：Box Worker 通过 Cozmio Box Model Runtime 真实调用本地 GGUF 模型生成 payload_text。

**这是 H1 的核心验证点。必须完成真实 GGUF 模型推理，不能只跑 mock。**

### 任务

1. 实现 Box Model Runtime 抽象接口
   - `generate(context: &str) -> Result<(String, u64)>`  // (payload_text, duration_ms)
   - `status() -> ModelStatus`
   - `warmup() -> Result<()>`
   - `shutdown() -> Result<()>`
2. 实现 llama.cpp provider（H1 默认 provider）
   - 调用本地 llama.cpp 二进制或绑定
   - 加载 GGUF 模型文件（路径来自 `/opt/cozmio/config/box-model.toml`）
   - 返回自然语言 payload_text
3. 实现模型配置（`/opt/cozmio/config/box-model.toml`）
   ```toml
   provider = "llama_cpp"
   model_path = "/opt/cozmio/models/current.gguf"
   context_size = 2048
   threads = 4
   timeout_secs = 120
   ```
4. 实现模型 health check（启动时验证模型文件存在且可加载）
5. 实现 warmup（首次推理前预热）
6. 实现推理 timeout
7. 日志必须记录：provider、model_path、trace_id、duration_ms、output_chars

### 交付物

```
cozmio-box-worker/src/
    box_model_runtime.rs     # 运行时管理 + llama_cpp provider
    model_provider.rs        # trait 定义
    providers/
        mod.rs
        llama_cpp.rs        # llama.cpp + GGUF provider
        mock.rs             # mock provider（仅开发验证用）
cozmio-box-worker/config/
    box-model.toml.example  # 模型配置示例
cozmio-box-worker/scripts/
    install_model.sh         # 下载/安装 GGUF 模型脚本
```

### 验收

```
PC 端不动模型（桌面端日志无模型推理记录）
Box Worker 日志显示：BoxModelRuntime initialized with llama_cpp provider
Box Worker 日志显示：Loading model from /opt/cozmio/models/current.gguf
Box Worker 日志显示：Box inference: generating response
Box Worker 日志显示：Inference duration: N ms, output: N chars
payload_text 回到 Relay
日志有明确的 "Box inference" 标记（区分 PC 模型）
至少完成一次真实 GGUF 模型推理（不是 mock）
```

**模型效果可以弱（小尺寸量化模型），但必须是真实 GGUF 推理。**

---

## Phase 5：桌面端发送真实 context_bundle

**目标**：桌面端把当前真实上下文交给树莓派，并接收自然语言结果。

### 任务

1. 桌面端生成真实 context_bundle（包含当前窗口信息、用户最近操作描述）
2. 桌面端通过 Relay 发送 inference task
3. 桌面端接收 payload_text
4. 桌面端展示"来自 Local Agent Box"的标记结果（UI 上区分 PC 模型和 Box 模型）
5. 桌面端记录 trace_id（贯穿桌面→Relay→Box→Relay→桌面）
6. 桌面端写入 action_log.jsonl（追溯完整链路）

### 交付物

```
src-tauri/src/relay_bridge.rs          # 增加 inference 调用方法
src-tauri/src/main_loop.rs             # 集成 Box inference 路径
src-tauri/src/components/
    StatusPanel.js                     # 增加 Box/PC 模型来源标记
```

### 验收

```
打开一个真实 Cozmio 监控窗口
桌面端日志显示：Sending inference to Box Worker, trace_id=xxx
Relay 日志显示：Routing to worker_id=xxx
Box Worker 日志显示：Received context_bundle, calling model
Box Worker 日志显示：Model response: <自然语言>
桌面端 UI 显示：回复来源 = Local Agent Box（不是 Local Mock）
trace_id 在桌面端日志、Relay 日志、Box Worker 日志一致
```

---

## Phase 6：执行闭环

**目标**：树莓派 payload_text 进入真实执行链路，完成一个真实动作。

### 任务

1. 桌面端将 payload_text 传给执行端
2. 执行端生成一个最小真实动作（生成 markdown 任务卡 / 打开面板 / 写入文件）
3. 执行端产生 execution_result
4. result 回传 Relay 并通知桌面端
5. 完整 trace 可追溯

### 交付物

```
relay-engine/src/inference_dispatcher.rs  # inference result 回传通道
src-tauri/src/executor.rs               # 接入 Box payload_text 执行
action_log.jsonl                         # 可追溯 trace
```

### 验收

```
用户打开 Cozmio 监控的真实工作场景
触发一次 inference
树莓派生成自然语言理解/建议
桌面端展示回复
执行端完成一个可见的最小动作（如生成 task_proposal.md 文件）
execution_result 被记录
日志里 trace 能证明全链路：桌面→Relay→树莓派→Relay→桌面→执行端→result
```

**最小动作**：可感知、可验证、非 mock。

---

## Phase 7：设备化部署

**目标**：树莓派像设备节点而非临时脚本。

### 任务

1. systemd service（开机自启动、崩溃重启；**不依赖 Ollama service**）
2. 启动 / 停止 / 重启 / 状态查看 / 日志查看脚本
3. 配置文件（`/opt/cozmio/config/box-worker.toml` 和 `/opt/cozmio/config/box-model.toml`）
4. 断线重连（Box Worker 自动重连 Relay）
5. 日志目录（`/opt/cozmio/logs/`）
6. 模型文件目录（`/opt/cozmio/models/`）

### 交付物

```
cozmio-box-worker/deployment/
    cozmio-box-worker.service
cozmio-box-worker/scripts/
    start.sh / stop.sh / logs.sh / status.sh
cozmio-box-worker/config/
    box-worker.toml.example
    box-model.toml.example
```

### 验收

```
拔掉树莓派网线，30s 后 Relay 显示 offline
重新插上网线，Box Worker 自动重连，Relay 显示 online
重启树莓派，worker 自动上线（无需手动启动）
journalctl -u cozmio-box-worker -f 能看到实时日志
/opt/cozmio/logs/box-worker.log 可查看
```

---

## Phase 8：投资人 Demo

**目标**：能演示，不只是能跑通。

### Demo 1：项目讨论场景

```
打开 Cozmio 监控的真实项目讨论页面
发送上下文
树莓派生成自然语言理解
执行端生成任务说明
展示：推理发生在树莓派，不在 PC
```

### Demo 2：开发交接场景

```
打开 Claude Code 相关页面
树莓派生成自然语言建议
执行端创建任务文档
展示：硬件盒是执行链路中的推理节点
```

### Demo 3：断开/接入对比

```
正常工作时 Relay 显示 local_agent_box online
断开树莓派（拔网线或关机）
Relay 显示 box offline，提示 local agent unavailable
重新连接
Relay 显示 box re-connected
系统恢复远程硬件推理
展示：树莓派是独立硬件节点，不是 PC 假装出来的能力
```

---

## 文件影响总览

### 新增文件

```
cozmio-box-worker/
    Cargo.toml
    src/
        main.rs
        config.rs
        protocol.rs
        worker.rs
        box_model_runtime.rs     # Cozmio 自有的模型运行时
        model_provider.rs        # provider trait
        providers/
            mod.rs
            llama_cpp.rs        # H1 默认 provider
            mock.rs             # 仅开发验证用
        heartbeat.rs
        status.rs
    models/
        README.md                # 模型文件管理说明
    scripts/
        hardware_probe.sh        # Phase 1
        install_model.sh         # Phase 4（安装 GGUF 模型）
        start.sh / stop.sh / logs.sh / status.sh  # Phase 7
    deployment/
        cozmio-box-worker.service
    config/
        box-worker.toml.example
        box-model.toml.example
relay-engine/src/
    worker_registry.rs            # Phase 3
    worker_session.rs             # Phase 3
    inference_router.rs           # Phase 3
    proto/mod.rs                 # Phase 3（新增消息）
```

### 修改文件

```
relay-engine/src/main.rs          # Phase 3
relay-client/src/lib.rs           # Phase 3（新增 inference API）
src-tauri/src/
    relay_bridge.rs               # Phase 5
    main_loop.rs                  # Phase 5
    components/StatusPanel.js     # Phase 5
```

---

## 验收总览

| Phase | 验收标准 |
|-------|---------|
| Phase 1 | SSH 连接成功，probe 输出完整硬件信息，target 和路径确定 |
| Phase 2 | Relay 日志显示 worker register + heartbeat（mock），断线显示 offline |
| Phase 3 | 完整 inference 往返成功（固定文本），无丢包，无死锁 |
| Phase 4 | PC 不跑模型；树莓派通过 Box Model Runtime + llama_cpp 调用 GGUF；日志有 provider/model_path/duration_ms/output_chars；至少一次真实推理 |
| Phase 5 | 真实 context_bundle，trace_id 全链路一致，UI 显示来源 = Local Agent Box |
| Phase 6 | 执行端完成一个可见最小动作，execution_result 记录，trace 完整 |
| Phase 7 | 重启后自动上线，断线自动重连，日志可实时查看 |
| Phase 8 | 3 个 demo 场景可演示，旁观者能理解树莓派是推理节点 |

---

## 风险处理

| 风险 | 处理方式 |
|------|---------|
| 交叉编译 target 不对 | Phase 1 probe 确认 arch 后再编译；先用 x86 本地验证软件逻辑 |
| 模型跑不动大模型 | Phase 4 用小尺寸 GGUF 量化模型（Qwen2-0.5B / Phi-2-mini / Gemma-2B）；H1 不追求模型质量，追求链路跑通 |
| 真机性能不够 | 临时用 mock_provider 验证链路，但 H1 正式验收必须包含 llama_cpp_provider 真实调用 |
| Relay 和 Box Worker 协议不兼容 | 先对齐 protobuf 定义；两边的消息结构必须完全一致 |
| 执行端接入复杂 | Phase 6 用最简单的文件写入动作，不改动现有 executor 结构 |
| 模型文件管理 | 统一放到 `/opt/cozmio/models/`；由 Cozmio Box Model Runtime 自己管理，不依赖 Ollama |
| Ollama 误入架构 | 明确 Ollama 只是开发期参考；不写进任何部署配置；systemd unit 不依赖 ollama.service |

---

## 执行方式

**每个 Phase 完成后必须按验收标准审查，不能因为某个阶段通过就宣布 H1 完成。**

建议执行方式：

1. **子智能体驱动（推荐）**——每个 Phase 调度独立子智能体，每阶段完成后审核验收
2. **内联执行**——按 Phase 依次执行，每阶段确认验收再进入下阶段

---

## 连接信息管理

> **安全警示**：设备连接凭据存储在 `.local-device.toml`（已加入 .gitignore），不进入代码仓库。
>
> 执行前先确认 `.local-device.toml` 存在且包含正确凭据。

配置文件位置：`D:\C_Projects\Agent\cozmio\.local-device.toml`
