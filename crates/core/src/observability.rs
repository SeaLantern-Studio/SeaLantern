use std::fmt::Display;

/// 进程守护进程生命周期事件的稳定追踪目标。
pub const PROCESS_DAEMON_TARGET: &str = "sealantern.core.process.daemon";

/// 主机适配器可映射到面向前端事件的稳定事件名称。
pub const EVENT_DAEMON_TERMINATION_FAILED: &str = "daemon_termination_failed";

pub(crate) fn daemon_termination_failed(process_id: u32, sign: &str, error: &dyn Display) {
    tracing::error!(
        target: PROCESS_DAEMON_TARGET,
        event_name = EVENT_DAEMON_TERMINATION_FAILED,
        process_id,
        sign,
        error = %error,
        "daemon process tree termination failed"
    );
}
