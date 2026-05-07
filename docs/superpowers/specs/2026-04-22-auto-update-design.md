# Cozmio 自动更新功能设计

**日期**: 2026-04-22
**状态**: 已批准

---

## 概述

为 Cozmio 桌面应用添加自动更新功能，实现后台静默更新 + 用户确认重启的体验。更新检查、下载、安装默认不打扰用户，仅在需要用户参与时轻提示。

---

## 模块划分

| 模块 | 功能 | 优先级 |
|------|------|--------|
| 1 | 更新检查 + 后台下载 + 后台安装 | P0 |
| 2 | 更新提示 + 用户确认重启 | P0 |
| 3 | 空闲检测 + 自动重启 | P1（后续阶段） |

---

## 模块 1: 更新检查 + 后台下载 + 后台安装

### 检查触发条件

- **启动时检查**: 应用启动时判断 `now - last_check_at > 24h`，超过则检查
- **定时检查**: 应用运行时每 24 小时自动检查一次
- 统一受 24 小时节流控制，避免频繁请求

### 服务端接口

```
GET /updates/check?version={current_version}

响应：
{
  "needs_update": true/false,
  "latest_version": "1.2.0",
  "channel": "stable",
  "notes": "修复了xxx问题",
  "download_url": "https://updates.example.com/cozmio-1.2.0.msi",
  "signature": "sha256:abc123..."
}
```

### 更新源地址

- **正式版**: 硬编码 `https://updates.example.com`
- **开发/内测**: 环境变量 `COZMIO_UPDATE_URL` 覆盖（不暴露给普通用户）

### 后台流程

1. 检查更新 → 收到 `needs_update: true`
2. 后台下载 MSI 到临时目录
3. 校验 signature（SHA256）
4. 后台静默安装（`msiexec /quiet`）
5. 记录状态为 `UpdatePending`

### 错误处理（分级策略）

| 错误类型 | 处理方式 |
|---------|---------|
| 网络/临时下载失败 | 日志 + 指数退避重试（1h, 2h, 4h...） |
| 连续失败达到 3 次 | 轻提示（气泡通知"更新待安装"），继续后台重试 |
| 不可自动恢复（权限/磁盘/签名失败） | 明确告知用户原因 |

---

## 模块 2: 更新提示 + 用户确认重启

### 待重启状态

- 安装完成后，状态记录为 `UpdatePending`
- 托盘图标显示微小标记表明有待生效更新

### 重启提示

- 气泡通知:
  > "Cozmio 已更新至 1.2.0，重启后生效"
  > `[立即重启]` `[稍后]`
- 点击"立即重启"：执行 `shutdown /r /t 5`
- 点击"稍后"：保持状态

### 不打断原则

- 重启提示仅在非活跃时段弹出（或用户点击托盘图标时）
- 不在用户全屏游戏、演示等场景强制弹窗

---

## 模块 3: 空闲检测 + 自动重启（后续阶段）

本期不做实现，预留接口和状态定义。

### 预留状态定义

```rust
pub enum UpdateState {
    None,
    Pending { version: String, installed_at: DateTime },
    ReadyToRestart,
}
```

### 预留托盘菜单

托盘菜单预留 "检查更新" 入口（供调试/手动触发）

---

## 数据结构

### Config 新增字段

```json
{
  "last_check_at": "2026-04-22T10:00:00Z",
  "update_channel": "stable"
}
```

### AppState 新增

```rust
pub struct AppState {
    pub config: Config,
    pub logger: ActionLogger,
    pub tray_state: RwLock<TrayState>,
    pub update_state: RwLock<UpdateState>,  // 新增
}
```

### UpdateState 枚举

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateState {
    None,
    Pending { version: String, installed_at: DateTime<Utc> },
    ReadyToRestart,
}
```

---

## 日志要求

每次检查/下载/安装/成功/失败都要有结构化日志，便于排查问题。

---

## 后续扩展

- 模块 3: 空闲检测 + 自动重启生效
- Beta/stable 通道支持
- 灰度发布
- 坏版本跳过
- 回滚策略

以上均由服务端 `needs_update` 布尔值统一控制，客户端不做复杂版本判断。
