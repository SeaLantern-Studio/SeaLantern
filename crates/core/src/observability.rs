use std::fmt::Display;

/// 进程守护进程生命周期事件的稳定追踪目标。
pub const PROCESS_DAEMON_TARGET: &str = "sealantern.core.process.daemon";

/// 主机适配器可映射到面向前端事件的稳定事件名称。
pub const EVENT_DAEMON_TERMINATION_FAILED: &str = "daemon_termination_failed";

/// 实例生命周期操作的稳定追踪目标。
pub const INSTANCE_LIFECYCLE_TARGET: &str = "sealantern.core.instance.lifecycle";

/// 实例生命周期观察中表示尚未读取前一状态的稳定字段值。
pub const INSTANCE_PREVIOUS_STATE_UNAVAILABLE: &str = "state_unavailable";

/// 实例生命周期观察中表示尚未观察到终止状态的稳定字段值。
pub const INSTANCE_TERMINAL_STATE_NOT_OBSERVED: &str = "not_observed";

/// 主机可映射的实例重启请求事件名称。
pub const EVENT_INSTANCE_RESTART_REQUESTED: &str = "instance_restart_requested";

/// 主机可映射的实例重启完成事件名称。
pub const EVENT_INSTANCE_RESTART_COMPLETED: &str = "instance_restart_completed";

/// 主机可映射的实例重启失败事件名称。
pub const EVENT_INSTANCE_RESTART_FAILED: &str = "instance_restart_failed";

/// 重启在读取当前状态时失败的稳定阶段名称。
pub const RESTART_PHASE_READ_STATE: &str = "read_state";

/// 重启在请求停止时失败的稳定阶段名称。
pub const RESTART_PHASE_REQUEST_STOP: &str = "request_stop";

/// 重启在等待停止时失败的稳定阶段名称。
pub const RESTART_PHASE_AWAIT_STOP: &str = "await_stop";

/// 重启因未达到停止状态而终止的稳定阶段名称。
pub const RESTART_PHASE_VERIFY_STOPPED: &str = "verify_stopped";

/// 重启在启动新实例时失败的稳定阶段名称。
pub const RESTART_PHASE_START: &str = "start";

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

pub(crate) fn instance_restart_requested(instance_id: &str, previous_state: &str) {
    tracing::info!(
        target: INSTANCE_LIFECYCLE_TARGET,
        event_name = EVENT_INSTANCE_RESTART_REQUESTED,
        instance_id,
        previous_state,
        "instance restart requested"
    );
}

pub(crate) fn instance_restart_completed(instance_id: &str, previous_state: &str) {
    tracing::info!(
        target: INSTANCE_LIFECYCLE_TARGET,
        event_name = EVENT_INSTANCE_RESTART_COMPLETED,
        instance_id,
        previous_state,
        "instance restart completed"
    );
}

pub(crate) fn instance_restart_failed(
    instance_id: &str,
    previous_state: &str,
    phase: &str,
    terminal_state: Option<&str>,
    error: &dyn Display,
) {
    tracing::error!(
        target: INSTANCE_LIFECYCLE_TARGET,
        event_name = EVENT_INSTANCE_RESTART_FAILED,
        instance_id,
        previous_state,
        phase,
        terminal_state = terminal_state.unwrap_or(INSTANCE_TERMINAL_STATE_NOT_OBSERVED),
        error = %error,
        "instance restart failed"
    );
}
