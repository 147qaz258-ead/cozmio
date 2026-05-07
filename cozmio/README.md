# Cozmio Desktop Agent

主动式桌面智能体，监控用户工作状态并在适当时候提供协助。

## 功能

- **窗口监控**: 实时检测前台窗口变化
- **截图捕获**: 自动截取当前屏幕供模型分析
- **智能判断**: 基于 Ollama 本地模型判断是否需要介入
- **系统通知**: Windows Toast 通知 + 确认/取消操作
- **执行路由**: 根据判断级别路由到不同执行策略

## 技术架构

- Rust + Tauri 桌面框架
- cozmio_core: 截图、窗口枚举核心模块
- cozmio_model: Ollama API 封装
- cozmio_memory: 记忆系统（SQLite + FTS5）
- cozmio-box-worker: Claude Code 进程管理

## 构建

```bash
cargo build
```

## 配置

编辑 `src-tauri/config.json` 或通过应用内界面配置：

- `ollama_url`: Ollama 服务地址
- `model_name`: 使用的模型名称
- `poll_interval_secs`: 窗口检测间隔