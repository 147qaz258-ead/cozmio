# 桌面端应用设计文档

## 1. 技术架构

**框架**: Tauri (Rust 后端 + Web 前端)

**核心技术栈**:
- 后端: Rust + cozmio_core (底层窗口信息捕获)
- 前端: Web (HTML/CSS/JS)
- 模型: Ollama 本地调用
- 原生对话框: Windows MessageBox (通过 tauri-plugin-dialog)

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri 应用                            │
├─────────────────────────────────────────────────────────┤
│  Frontend (Web)                                         │
│  ┌─────────┐  ┌──────────┐  ┌─────────────────────┐    │
│  │ 托盘菜单 │  │ 主窗口    │  │ Windows 原生对话框   │    │
│  └─────────┘  └──────────┘  └─────────────────────┘    │
├─────────────────────────────────────────────────────────┤
│  Rust Backend                                           │
│  ┌─────────────┐  ┌─────────────┐  ┌───────────────┐  │
│  │ cozmio_core│  │ 模型调用     │  │ 执行路由      │  │
│  │ (窗口监控)  │  │ (reqwest +  │  │ (suggest/     │  │
│  │             │  │  ollama)    │  │  request/     │  │
│  │             │  │             │  │  execute)     │  │
│  └─────────────┘  └─────────────┘  └───────────────┘  │
│                                                         │
│  cozmio_core: xcap + windows-rs Win32 API              │
└─────────────────────────────────────────────────────────┘
```

## 2. UI 设计语言

**视觉风格**：
- 浅灰白纸感底色 + 极淡工程网格纹理
- 细边框、轻阴影、低饱和中性色
- 窗口化拼贴、技术编辑感、系统模块感
- 排版/边界/交叠营造层次，不用重色块和强特效

**配色方案**：
- 背景: #fafafa (浅灰白)
- 卡片背景: #ffffff
- 边框: #e8e8e8 (极淡灰)
- 文字主色: #555555
- 文字辅助: #999999
- 强调色: #666666 (中性灰黑)

**字体**：
- 等宽字体优先: SF Mono, Fira Code, Consolas
- 层级靠字重和大小区分，不靠颜色

**组件风格**：
- 细边框卡片 (1px solid #e8e8e8)
- 极轻阴影 (0 1px 2px rgba(0,0,0,0.02))
- 小组件紧凑排列
- 模块边界清晰但不厚重

**交互风格**：
- 状态变化用 dot 大小/颜色
- hover 用极轻背景色 (#f5f5f5)
- 点击反馈快速 (0.1-0.15s)
- 无动效、无弹跳、无渐变

## 3. 核心模块

### 2.1 窗口监控模块 (`window_monitor`)

**职责**:
- 调用 cozmio_core 获取窗口截图和元数据
- 检测窗口变化（避免重复处理相同窗口）

**接口**:
```rust
pub struct WindowSnapshot {
    pub screenshot_base64: String,  // PNG base64
    pub screenshot_width: u32,
    pub screenshot_height: u32,
    pub window_info: WindowInfo,   // 来自 cozmio_core
    pub timestamp: i64,
}

pub trait WindowMonitor {
    fn capture(&self) -> Result<WindowSnapshot, Error>;
    fn has_changed(&self, prev: &WindowSnapshot) -> bool;
}
```

### 2.2 模型调用模块 (`model_client`)

**职责**:
- 构造 prompt（截图 + 元数据）
- 调用本地 Ollama API
- 解析模型输出（judgment + next_step + level + confidence + grounds）

**模型输出格式**:
```
judgment: <模型对当前局面的判断>
next_step: <最合理的下一步>
level: suggest | request | execute
confidence: <0.0-1.0>
grounds: <判断依据>
```

**接口**:
```rust
pub struct ModelOutput {
    pub judgment: String,
    pub next_step: String,
    pub level: InitiativeLevel,
    pub confidence: f32,
    pub grounds: String,
}

pub enum InitiativeLevel {
    Suggest,   // 只通知用户
    Request,   // 需用户确认
    Execute,   // 高置信度，可直接执行
}

pub trait ModelClient {
    fn call(&self, snapshot: &WindowSnapshot) -> Result<ModelOutput, Error>;
}
```

### 2.3 执行路由模块 (`executor`)

**职责**:
- 根据 `level` 决定处理方式
- `suggest` → 推送系统通知
- `request` → 弹出 Windows 原生确认框 → 执行或放弃
- `execute` → 直接执行（可配置为也需要确认）

**接口**:
```rust
pub trait ActionExecutor {
    fn execute(&self, action: &ModelOutput) -> Result<(), Error>;
}

pub struct Executor {
    suggest_handler: NotifyHandler,
    request_handler: ConfirmHandler,
    execute_handler: Box<dyn ActionExecutor>,
}

impl Executor {
    pub fn route(&self, output: &ModelOutput) -> Result<ExecutionResult, Error> {
        match output.level {
            InitiativeLevel::Suggest => self.suggest_handler.handle(output),
            InitiativeLevel::Request => self.request_handler.handle(output),
            InitiativeLevel::Execute => self.execute_handler.execute(output),
        }
    }
}
```

### 2.4 用户交互模块

**托盘**:
- 显示运行状态（监控中/处理中/空闲）
- 右键菜单：设置、历史、暂停、退出

**主窗口**:
- 状态面板（当前窗口、最后判断）
- 历史记录列表
- 设置页面（Ollama 地址、轮询间隔、执行确认策略）

**Windows 原生对话框**:
- `request` 级别：确认框（是/否）
- `execute` 级别（如果启用确认）：确认框
- 显示 judgment + next_step 内容

### 2.5 行为日志

**记录内容**:
```
timestamp / window_id / judgment / next_step / level / confidence / grounds / system_action / user_feedback
```

**存储**: 本地 JSONL 文件

## 3. 数据流

```
窗口变化检测
    ↓
截取窗口截图 + 元数据
    ↓
调用 Ollama 模型
    ↓
解析输出 (judgment/next_step/level/confidence/grounds)
    ↓
根据 level 路由
    ├── suggest → Windows 通知气泡
    ├── request → Windows 原生确认框 → 执行或放弃
    └── execute → 直接执行（或确认后执行，配置可选）
    ↓
记录行为日志
```

## 4. 目录结构

```
cozmio/
├── Cargo.toml                    # Workspace 配置
├── cozmio_core/                  # 底层窗口捕获库 (real-info-acquisition 计划产出)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── error.rs
│       ├── monitor.rs
│       ├── screenshot.rs
│       ├── window.rs
│       └── types.rs
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # 入口 + Tauri setup
│   │   ├── window_monitor.rs    # 窗口监控（调用 cozmio_core）
│   │   ├── model_client.rs      # Ollama 调用
│   │   ├── executor.rs          # 执行路由
│   │   ├── tray.rs              # 系统托盘
│   │   ├── commands.rs          # Tauri IPC 命令
│   │   ├── logging.rs           # 行为日志
│   │   ├── config.rs            # 配置管理
│   │   └── main_loop.rs         # 主监控循环
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                         # Web 前端
│   ├── index.html
│   ├── main.js
│   ├── styles.css
│   └── components/
│       ├── App.js
│       ├── StatusPanel.js
│       ├── HistoryList.js
│       └── Settings.js
├── src/                         # Python (Phase 1 保留)
│   ├── screenshot.py
│   └── model_call.py
├── verification/                # Phase 1 验证产物
├── docs/
│   └── superpowers/
│       ├── specs/
│       └── plans/
└── ui-prototype.html            # UI 原型
```

## 5. 关键设计决策

1. **执行层抽象**: `executor.rs` 定义 `ActionExecutor` trait，具体执行逻辑作为后续模块实现
2. **Windows 原生对话框**: 使用 `tauri-plugin-dialog` 插件
3. **配置持久化**: `config.json` 存储在应用数据目录
4. **模型输出解析**: 正则提取各字段，允许模型输出有一定灵活性
5. **窗口变化检测**: 相同窗口 5 秒内不重复处理（可配置）

## 6. Phase 1 到桌面端的演进

| Phase 1 | 桌面端 |
|---------|--------|
| Python 截图 | cozmio_core (xcap + windows-rs) |
| Python Ollama 调用 | Rust reqwest |
| 手动运行验证 | 后台自动运行 |
| 人工查看输出 | 自动路由处理 |
| 无执行 | 执行路由框架 |
| 无 UI | Tauri Web UI |

## 7. 成功标准

- 应用可正常启动、后台运行、退出
- 托盘图标正确显示，菜单可交互
- 窗口变化时自动截取并调用模型
- 根据 level 正确路由（通知/确认/执行）
- Windows 原生确认框可正常弹出并返回结果
- 行为日志正确记录
- 设置页面可修改配置并生效
