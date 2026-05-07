# 短时过程上下文模块设计方案

## 问题背景

当前 main_loop 每 3 秒抓一张截图，独立发给 Ollama 模型，模型判断 CONTINUE 或 ABSTAIN 后弹窗。

**根本限制：**

- 每帧之间没有"过程"概念
- 模型只能看到"当前这一刻"，不知道"用户刚经历过什么"
- 从 action_log 样本看，存在自触发（cozmio.exe 自身窗口）、调试窗口混入等系统噪音

**体验问题：**

- 弹窗太频繁、太像事后描述
- 弹窗文字看不懂（只描述证据，不解释为什么现在值得打扰）
- 用户感受不到"agent 在理解我的过程"

## 设计目标

让系统拥有"最近一小段过程信息"，使弹窗能回答：

1. 为什么是现在出现，而不是因为当前页面有内容？
2. 用户当前处于什么行为模式（稳定 / 震荡 / 刚切换）？
3. 这次弹窗的依据是否超过了"窗口有内容"这一层？

**不追求：**

- 完整视频流架构
- 长期记忆
- 复杂行为模式识别
- 前置裁决（上下文模块不决定弹不弹）

**当前阶段追求：**

- 短时过程 buffer（最近 20 个快照，约 60 秒）
- 简单行为事实（停留时长、切换次数、震荡检测）
- 仅清理明显的系统噪音（不依赖行为信号做拦截判断）
- 过程事实作为判断依据注入后续链路，不直接决定结果

## 架构设计

```
main_loop poll
    │
    ▼
WindowMonitor.capture() → 快照
    │
    ▼
┌─────────────────────────────────────────┐
│  短时过程上下文层                        │
│                                         │
│  输入：当前快照 + 短时 buffer            │
│  输出：过程事实（行为信号）              │
│                                         │
│  产出内容：                              │
│  - stay_duration_seconds                 │
│  - switches_in_last_N_seconds           │
│  - is_oscillating                       │
│  - last_switch_direction                │
│  - just_arrived                         │
│                                         │
│  职责边界：                              │
│  - 只产出事实，不做判断                  │
│  - 不决定弹不弹                          │
│  - 不拦截模型调用                        │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│  系统噪音清理层（独立于上下文模块）        │
│                                         │
│  仅处理明确的系统自触噪音：               │
│  - cozmio.exe 自身窗口                   │
│  - 调试终端窗口（cmd.exe 含 cozmio 路径）│
│                                         │
│  职责边界：                              │
│  - 只清理明确的系统噪音，不涉及行为判断   │
│  - 不使用停留/震荡/刚切换等信号          │
└─────────────────────────────────────────┘
    │
    ▼
ModelClient.call(snapshot, context)
    │
    ▼
Executor.route()
    │
    ▼
handle_execution_result()
```

## 模块详细设计

### 过程上下文对象（ProcessContext）

```rust
pub struct ProcessContext {
    /// 在当前窗口停留时长（秒）
    pub stay_duration_seconds: u32,
    /// 最近 60 秒内窗口切换次数
    pub switches_in_last_minute: u32,
    /// 是否处于震荡模式（>=2次切换且间隔<5秒）
    pub is_oscillating: bool,
    /// 上次切换方向：切来(Arrived) / 切走(Left) / 无(None)
    pub last_switch_direction: SwitchDirection,
    /// 当前窗口是否刚刚切换到（刚切到 <= 5秒）
    pub just_arrived: bool,
}
```

**语义说明：**

- `stay_duration_seconds`：用户在同一窗口驻留多久。停留长可能表示深度工作，也可能表示卡住了——**模型判断，不预设结论**。
- `is_oscillating`：是否在来回切换。震荡可能表示排错过程，也可能表示注意力分散——**模型判断，不预设结论**。
- `just_arrived`：是否刚切换过来。刚切换可能是误触，也可能用户正在找信息——**模型判断，不预设结论**。

### compute_context 计算顺序（关键约束）

**不正确的做法：**
1. 先将当前快照写入 buffer
2. 再用当前 buffer（含当前条目）计算停留时长
3. 结果：stay_duration_seconds 永远接近 0

**正确的做法：**
1. 先用当前快照 + **旧 buffer**（不含当前条目）计算 ProcessContext
2. 再将当前快照写入 buffer

**计算 stay_duration_seconds 时：**
- 在 buffer 中查找当前窗口的**上一条**记录（排除当前条目）
- stay = 当前时间戳 - 上一条记录的时间戳
- 如果 buffer 中没有当前窗口的历史记录，stay_duration_seconds = 0

### 短时 Buffer

```rust
struct ProcessBuffer {
    entries: VecDeque<BufferedEntry>,  // 最大 20 条，约 60 秒
    capacity: usize,                  // 20
}

struct BufferedEntry {
    window_title: String,
    process_name: String,
    timestamp: i64,
}
```

每 `capture()` 一次，将结果写入 buffer。buffer 超过容量时移除最老的条目。

### 系统噪音清理（独立步骤，非上下文模块职责）

| 噪音类型 | 检测条件 | 处理动作 |
|----------|----------|----------|
| 自环 | 进程名为 cozmio.exe | 跳过本次调用，直接 ABSTAIN |
| 调试窗口 | cmd.exe/powershell.exe 且窗口标题含 cozmio 路径 | 跳过本次调用，直接 ABSTAIN |

**仅限明确的系统自噪声，不包含停留/震荡/刚切换等行为条件。那些是过程事实，不用于裁决。**

### 过程事实如何使用

**不这样做：**
- `if context.stay_duration_seconds < 10 { return ABSTAIN }`（用停留时长做 Rust 层裁决）
- `if context.is_oscillating { return ABSTAIN }`（用震荡做 Rust 层裁决）
- ProcessContext 只作为 ModelOutput 上的附加字段（不参与判断链路）

**这样做：**

保持"模型判断"和"系统观察事实"分离，同时让 ProcessContext 作为**调用上下文**进入判断链路：

```rust
// ModelOutput：模型输出（不变）
pub struct ModelOutput {
    pub mode: InterventionMode,
    pub reason: String,
    pub user_how: Option<String>,
}

// ProcessContext：系统观察事实（独立结构）
pub struct ProcessContext {
    pub stay_duration_seconds: u32,
    pub switches_in_last_minute: u32,
    pub is_oscillating: bool,
    pub last_switch_direction: SwitchDirection,
    pub just_arrived: bool,
}

// JudgmentInput：调用上下文中包含过程事实
pub struct JudgmentInput {
    pub snapshot: WindowSnapshot,
    pub process_context: ProcessContext,
}
```

**调用链路：**

1. main_loop 调用 `model_client.call(judgment_input)`，其中 `judgment_input.snapshot = 截图`，`judgment_input.process_context = ProcessContext`
2. model_client 的 prompt 保持当前格式（本次不修改 prompt）
3. 执行结果（ExecutionResult + ProcessContext）一起推送到 UI state
4. UI 层展示 process_context 信号，供用户判断"这个弹窗时机是否合理"

**边界说明：**

- ProcessContext 是**调用上下文**，不是模型输出，但它参与判断链路
- 本次 prompt 不改，ProcessContext 在当前阶段不出现在 prompt 里
- 模型是否基于 ProcessContext 推理，取决于后续 prompt 设计，不在本次实现范围内
- 行为事实（停留/震荡/刚切换）不作为 Rust 层裁决规则，但作为判断链路的上下文事实存在

### 自触发等噪音为何单独处理

系统自触（监控自己的窗口）和调试窗口混入，是**系统自身 bug**，不是行为判断问题。

它们在任何情况下都应该被过滤，不需要模型参与判断，也不需要用停留/震荡等行为信号来判断。

## 验证方案

**对比基准：** action_log 里的旧样本（CONTINUE 输出）

**验证方法：**

1. 用 memory-cli replay 取一组真实近期窗口序列
2. 对比两种情况下的判断结果：
   - 旧：单帧判断
   - 新：单帧 + 过程事实
3. 记录系统噪音清理是否命中了已知的自触发/调试窗口样本

**判断标准：**

- 系统噪音清理是否正确过滤了 cozmio.exe 自触和调试窗口
- 过程事实是否附加到 ModelOutput 而不影响模型原有判断逻辑
- UI 层是否能看到 process_context 信号（供用户判断时机是否合理）

**验证目标不是减少弹窗，而是：**

- 弹窗是否更有上下文依据（用户能看到为什么是现在）
- 系统噪音是否不再混入有效判断
- 模型判断是否仍保持原有语义（不受行为信号前置干预）

## 涉及文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/window_monitor.rs` | 新增 ProcessBuffer、ProcessContext、compute_context()、push_snapshot() |
| `src-tauri/src/main_loop.rs` | 调用 compute_context()，系统噪音清理（独立步骤），附加 ProcessContext 到 UI state |
| `src-tauri/src/ui_state.rs` | PendingConfirmationInfo 新增 process_context 字段 |

## 不涉及

- memory crate（本次不接 memory/context_slices）
- relay_bridge（执行透明度问题独立）
- 视频流/录制（当前阶段不需要）
- prompt 模板改动（本次 ProcessContext 作为结构化上下文传入，不写入 prompt 文案）
- 停留/震荡/刚切换作为 Rust 层拦截规则（本次不实现）

## 风险

- Buffer 占用内存：20 条快照元数据，极小
- 震荡检测阈值（5 秒间隔）是经验值，可能需要调参
- ProcessContext 目前仅作为调用上下文传入，模型是否主动使用这些事实取决于后续 prompt 设计

## 下一步

1. 实现 ProcessContext 结构和 ProcessBuffer
2. 实现 compute_context() 函数
3. 在 main_loop 中实现系统噪音清理（独立步骤，非上下文模块）
4. ModelOutput 新增 process_context 字段
5. 用 memory-cli replay 样本验证系统噪音清理效果
6. 基于验证结果决定是否需要调整阈值
