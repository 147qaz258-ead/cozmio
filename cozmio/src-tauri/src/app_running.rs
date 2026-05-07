use std::sync::atomic::{AtomicBool, Ordering};

/// 单一权威运行状态
/// - true = Running（应用在分析窗口）
/// - false = Stopped（应用驻留但不分析）
static RUNNING_STATE: AtomicBool = AtomicBool::new(false);

/// 获取当前运行状态
pub fn is_running() -> bool {
    RUNNING_STATE.load(Ordering::SeqCst)
}

/// 设置运行状态
/// 注意：不在此处发送事件，调用方负责发送 running-state-changed 事件
pub fn set_running(running: bool) {
    RUNNING_STATE.store(running, Ordering::SeqCst);
}
