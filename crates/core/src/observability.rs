use std::fmt::Display;

/// Stable tracing target for process-daemon lifecycle events.
pub const PROCESS_DAEMON_TARGET: &str = "sealantern.core.process.daemon";

/// Stable event name that host adapters may map to a frontend-facing event.
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
