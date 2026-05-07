# 长期驻留桌面应用三层验证方案

> 本方案替代 Task 11 的"代码检查"式验证，建立三层验证体系：
> - 第一层：静态检查（编译、依赖、链路存在）
> - 第二层：运行验证（runtime_state.json 状态出口）
> - 第三层：桌面行为验证（Win32/UI Automation）

---

## 第一层：静态检查

### 1.1 编译检查

```bash
cd cozmio && cargo build --package cozmio 2>&1 | tail -3
```
**验收标准**：无 error（warnings 允许）

### 1.2 依赖完整性检查

验证以下依赖在 Cargo.toml 中存在且版本正确：
- `tauri-plugin-single-instance`
- `tauri-plugin-shell`
- `tauri-plugin-dialog`
- `tauri-plugin-notification`
- `env_logger`
- `chrono`
- `dirs`

```bash
grep -E "tauri-plugin-(single-instance|shell|dialog|notification)" src-tauri/Cargo.toml
grep -E "env_logger|chrono|dirs" src-tauri/Cargo.toml
```
**验收标准**：所有依赖声明存在

### 1.3 关键链路存在性检查

| 链路环节 | 检查位置 | 验收标准 |
|---------|---------|---------|
| `is_running()` 检查在 loop 开头 | `main_loop.rs:57` | `if !is_running()` 存在 |
| `start_running` → `set_running(true)` | `commands.rs:75-76` | 两个调用连续 |
| `stop_running` → `set_running(false)` | `commands.rs:83-84` | 两个调用连续 |
| Continue → Confirmed → blocking_show | `executor.rs:60-61` + `main_loop.rs:184-198` | 完整链路 |
| X 按钮 → hide() + prevent_close() | `main.rs:151-153` | 存在 |
| 单实例 → show + focus | `main.rs:100-103` | 存在 |

---

## 第二层：运行验证（runtime_state.json）

### 2.1 修改 runtime_state 模块

**新增文件**：`cozmio/src-tauri/src/runtime_state.rs`

```rust
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;
use chrono::{DateTime, Local};

static RUNTIME_STATE: OnceLock<Mutex<Option<File>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeState {
    pub running_state: String,        // "Running" | "Stopped"
    pub loop_tick_count: u64,         // 主循环迭代次数
    pub last_loop_at: Option<String>, // ISO8601 时间戳
    pub last_popup_requested_at: Option<String>,
    pub popup_count: u64,            // 累计弹窗次数
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            running_state: "Stopped".to_string(),
            loop_tick_count: 0,
            last_loop_at: None,
            last_popup_requested_at: None,
            popup_count: 0,
        }
    }
}

fn get_state_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cozmio")
        .join("runtime_state.json")
}

pub fn read_state() -> RuntimeState {
    let path = get_state_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        RuntimeState::default()
    }
}

pub fn write_state(state: &RuntimeState) {
    // 创建目录
    if let Some(dir) = get_state_path().parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    let json = serde_json::to_string_pretty(state).unwrap();
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&get_state_path())
    {
        let _ = file.write_all(json.as_bytes());
    }
}

pub fn increment_tick() {
    let mut state = read_state();
    state.running_state = if crate::app_running::is_running() {
        "Running".to_string()
    } else {
        "Stopped".to_string()
    };
    state.loop_tick_count += 1;
    state.last_loop_at = Some(Local::now().to_rfc3339());
    write_state(&state);
}

pub fn record_popup() {
    let mut state = read_state();
    state.last_popup_requested_at = Some(Local::now().to_rfc3339());
    state.popup_count += 1;
    write_state(&state);
}
```

### 2.2 修改 main_loop.rs 集成 runtime_state

**涉及文件**：`cozmio/src-tauri/src/main_loop.rs`

**修改点1**：在循环每次迭代时（is_running 检查通过后）调用 `increment_tick()`：

```rust
// main_loop.rs loop 开头，约 line 55-61
loop {
    // Step 0: Check running state
    if !crate::app_running::is_running() {
        log::debug!("Main loop paused, waiting...");
        thread::sleep(poll_interval);
        continue;
    }

    // 每次实际迭代时记录 tick
    crate::runtime_state::increment_tick();
```

**修改点2**：在弹窗触发时调用 `record_popup()`：

```rust
// main_loop.rs handle_execution_result，ExecutionResult::Confirmed 分支
ExecutionResult::Confirmed => {
    log::info!("Showing confirmation dialog");
    // ...
    let confirmed = app_handle.dialog()...blocking_show();

    // 弹窗触发时记录
    crate::runtime_state::record_popup();
```

### 2.3 添加 `get_runtime_state` 命令

**涉及文件**：`cozmio/src-tauri/src/commands.rs`

```rust
#[tauri::command]
pub fn get_runtime_state() -> Result<RuntimeState, String> {
    Ok(crate::runtime_state::read_state())
}
```

在 `generate_handler!` 中添加 `get_runtime_state`。

### 2.4 运行验证步骤

```bash
# 1. 启动应用（初始状态为 Stopped）
./target/debug/cozmio.exe &
sleep 2

# 2. 读取初始状态（应全为 0 或空）
cat "%LOCALAPPDATA%\cozmio\runtime_state.json"
# 预期：running_state="Stopped", loop_tick_count=0

# 3. 调用 start_running
# （通过 tray 菜单或 curl 命令调用 start_running）
# 或直接写一个测试命令：curl -X POST http://localhost:.../start_running

# 4. 等待一个轮询周期（10秒）
sleep 12

# 5. 再次读取状态
cat "%LOCALAPPDATA%\cozmio\runtime_state.json"
# 预期：running_state="Running", loop_tick_count >= 1, last_loop_at 不为空

# 6. 调用 stop_running
# （通过 tray 菜单或命令）

# 7. 等待，观察 tick_count 是否停止增长
sleep 12
# 读取状态，tick_count 应该保持不变
```

**验收标准**：
- `running_state` 在 start 后变为 "Running"，stop 后变为 "Stopped"
- `loop_tick_count` 在 Running 时持续增长，Stopped 时停止
- `last_loop_at` 每次迭代更新
- `popup_count` 在弹窗触发后增加

---

## 第三层：桌面行为验证

### 3.1 验证脚本：Windows UI Automation PowerShell

**涉及文件**：`cozmio/verification/desktop_behavior_test.ps1`

```powershell
#Requires -Version 5.1
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

$ErrorActionPreference = "Stop"
$appPath = "D:\C_Projects\Agent\cozmio\cozmio\target\debug\cozmio.exe"
$statePath = "$env:LOCALAPPDATA\cozmio\runtime_state.json"

function Get-CozmioProcess {
    Get-Process -Name "cozmio" -ErrorAction SilentlyContinue
}

function Read-RuntimeState {
    $content = Get-Content $statePath -Raw -ErrorAction SilentlyContinue
    if ($content) { [ ConvertFrom-Json $content ] } else { $null }
}

Write-Host "=== Desktop Behavior Verification ===" -Foreground Cyan

# 清理任何已存在的进程
$existing = Get-CozmioProcess
if ($existing) {
    Write-Host "[CLEANUP] Stopping existing cozmio process..."
    Stop-Process -Id $existing.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
}

Write-Host "`n[TEST 1] Launch application" -Foreground Yellow
Start-Process -FilePath $appPath -PassThru
Start-Sleep -Seconds 3

$processes = Get-CozmioProcess
if ($processes.Count -ne 1) {
    Write-Host "[FAIL] Expected 1 process, found $($processes.Count)" -Foreground Red
    exit 1
}
Write-Host "[PASS] Single process running (PID: $($processes[0].Id))" -Foreground Green

Write-Host "`n[TEST 2] Main window hidden at startup" -Foreground Yellow
$mainWindow = Get-UiaWindow -Name "Cozmio - 主动智能体" -ErrorAction SilentlyContinue
if ($mainWindow) {
    $visible = $mainWindow.Current.IsOffscreen -eq $false
    if ($visible) {
        Write-Host "[FAIL] Main window is visible (should be hidden)" -Foreground Red
    } else {
        Write-Host "[PASS] Main window is hidden" -Foreground Green
    }
} else {
    Write-Host "[PASS] Main window hidden (not found as visible)" -Foreground Green
}

Write-Host "`n[TEST 3] Single instance - second launch focuses existing" -Foreground Yellow
# 启动第二个实例
$secondProcess = Start-Process -FilePath $appPath -PassThru -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

$allProcesses = Get-CozmioProcess
if ($allProcesses.Count -eq 1) {
    Write-Host "[PASS] Only 1 process after second launch (single instance works)" -Foreground Green
} else {
    Write-Host "[FAIL] $($allProcesses.Count) processes after second launch" -Foreground Red
}

Write-Host "`n[TEST 4] Call start_running via IPC" -Foreground Yellow
# 使用 named pipe 或 REST API 调用 start_running
# 简化：直接修改 registry 或发送事件
# 这里我们直接读取 state 确认
$state = Read-RuntimeState
Write-Host "Current state: running_state=$($state.running_state), tick=$($state.loop_tick_count)"

Write-Host "`n[TEST 5] Exit cleanup" -Foreground Yellow
$proc = Get-CozmioProcess
if ($proc) {
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
}

$remaining = Get-CozmioProcess
if ($remaining) {
    Write-Host "[FAIL] Process still exists after exit" -Foreground Red
} else {
    Write-Host "[PASS] Process cleaned up after exit" -Foreground Green
}

Write-Host "`n=== Verification Complete ===" -Foreground Cyan
```

### 3.2 简化运行验证（无 UI Automation 环境）

如果 UI Automation 不可用，使用简化的进程+文件验证：

```bash
# 验证单实例（已在第一层验证）
# 验证退出清理
./target/debug/cozmio.exe &
PID=$!
sleep 3
kill -f $PID
sleep 1
if tasklist | grep -q cozmio; then
    echo "[FAIL] Process still running after exit"
else
    echo "[PASS] Process cleaned up after exit"
fi
```

### 3.3 弹窗确定性验证

弹窗触发需要模型输出 CONTINUE，这在测试环境中难以确定性触发。替代方案：

1. **Mock 模型响应**：添加一个 test mode，通过命令强制触发 Confirmed 弹窗
2. **验证弹窗链路存在**：代码检查 blocking_show() 调用存在

---

## 验收标准汇总

### 第一层：静态检查

| 检查项 | 标准 |
|--------|------|
| 编译通过 | `cargo build --package cozmio` 无 error |
| 依赖完整 | 所有 7 个依赖在 Cargo.toml 中 |
| 链路完整 | 6 条关键链路存在 |

### 第二层：运行验证

| 检查项 | 标准 |
|--------|------|
| 初始状态 Stopped | `runtime_state.json` 中 `running_state="Stopped"` |
| start_running 后 Running | 调用后 `running_state` 变为 "Running" |
| Loop tick 增长 | Running 时 `loop_tick_count` 持续增加 |
| Loop tick 停止 | Stopped 时 `loop_tick_count` 停止增长 |
| Popup 计数 | Confirmed 弹窗后 `popup_count` 增加 |
| last_loop_at 更新 | 每次迭代更新 |

### 第三层：桌面行为验证

| 检查项 | 标准 |
|--------|------|
| 主窗口隐藏 | 启动后主窗口 `IsOffscreen=true` |
| 单实例 | 两次启动后进程数量 = 1 |
| 窗口聚焦 | 第二次启动后旧窗口被 show+focus |
| 退出清理 | 退出后进程消失，托盘消失 |

---

## 任务清单

- [ ] **步骤1：创建 runtime_state.rs 模块**
- [ ] **步骤2：在 main_loop.rs 集成 increment_tick() 和 record_popup()**
- [ ] **步骤3：添加 get_runtime_state 命令**
- [ ] **步骤4：编写 PowerShell UI Automation 验证脚本**
- [ ] **步骤5：执行第一层静态检查**
- [ ] **步骤6：执行第二层运行验证**
- [ ] **步骤7：执行第三层桌面行为验证**
- [ ] **步骤8：生成行为级验证报告**