use super::{LocalHelperStatusSnapshot, LocalRuntimeState};
use crate::models::server::ServerStatus;
use crate::services::server::manager::process::is_process_alive;
use crate::services::server::runtime::i18n::runtime_t;
use crate::services::server::runtime::RuntimeStatusSnapshot;

pub(super) fn fallback_snapshot_from_state_file(
    state: &LocalRuntimeState,
) -> LocalHelperStatusSnapshot {
    fallback_snapshot_from_state_file_with(state, is_process_alive, false)
}

fn fallback_snapshot_from_state_file_with<F>(
    state: &LocalRuntimeState,
    is_alive: F,
    helper_unreachable: bool,
) -> LocalHelperStatusSnapshot
where
    F: Fn(u32) -> bool,
{
    let child_running = if state.running {
        state.child_pid.filter(|pid| is_alive(*pid))
    } else {
        None
    };

    LocalHelperStatusSnapshot {
        running: child_running.is_some(),
        pid: child_running,
        exit_code: state.exit_code,
        detail_message: fallback_detail_message(state, child_running, helper_unreachable),
        error_message: fallback_error_message(state, child_running, helper_unreachable),
    }
}

pub(super) fn fallback_snapshot_from_unreachable_helper(
    state: &LocalRuntimeState,
) -> LocalHelperStatusSnapshot {
    fallback_snapshot_from_state_file_with(state, is_process_alive, true)
}

pub(crate) fn helper_runtime_status(
    snapshot: &LocalHelperStatusSnapshot,
    is_stopping: bool,
) -> ServerStatus {
    if is_stopping {
        ServerStatus::Stopping
    } else if snapshot.running {
        ServerStatus::Running
    } else if snapshot.error_message.is_some() {
        ServerStatus::Error
    } else {
        ServerStatus::Stopped
    }
}

pub(crate) fn runtime_snapshot_from_helper(
    snapshot: LocalHelperStatusSnapshot,
    status: ServerStatus,
) -> RuntimeStatusSnapshot {
    let detail_message = if snapshot.running {
        snapshot.detail_message
    } else {
        stopped_detail_message(snapshot.exit_code)
    };

    RuntimeStatusSnapshot {
        status,
        pid: snapshot.pid,
        detail_message: Some(detail_message),
        error_message: snapshot.error_message,
    }
}

pub(crate) fn stopped_runtime_snapshot() -> RuntimeStatusSnapshot {
    RuntimeStatusSnapshot {
        status: ServerStatus::Stopped,
        pid: None,
        detail_message: Some(
            "runtime=local is_running=false exit_code=none source=state_absent".to_string(),
        ),
        error_message: None,
    }
}

fn fallback_detail_message(
    state: &LocalRuntimeState,
    child_running: Option<u32>,
    helper_unreachable: bool,
) -> String {
    if child_running.is_some() {
        format!(
            "runtime=local running=true source=state_file helper=unavailable exit_code={}",
            format_exit_code(state.exit_code)
        )
    } else if helper_unreachable {
        format!(
            "runtime=local running=false source=state_file helper=unreachable exit_code={}",
            format_exit_code(state.exit_code)
        )
    } else if state.running {
        format!(
            "runtime=local running=false source=state_file helper=exited exit_code={}",
            format_exit_code(state.exit_code)
        )
    } else {
        format!(
            "runtime=local running=false source=state_file exit_code={}",
            format_exit_code(state.exit_code)
        )
    }
}

fn fallback_error_message(
    state: &LocalRuntimeState,
    child_running: Option<u32>,
    helper_unreachable: bool,
) -> Option<String> {
    if child_running.is_some() {
        Some(runtime_t("server.runtime.local_helper.child_running_after_helper_exit"))
    } else if helper_unreachable {
        Some(runtime_t("server.runtime.local_helper.status_probe_failed"))
    } else if state.running
        && state.child_pid.is_none()
        && state.exit_code.is_none()
        && state.error_message.is_none()
    {
        None
    } else {
        state.error_message.clone()
    }
}

fn stopped_detail_message(exit_code: Option<i32>) -> String {
    format!(
        "runtime=local running=false source=helper exit_code={}",
        format_exit_code(exit_code)
    )
}

fn format_exit_code(exit_code: Option<i32>) -> String {
    exit_code
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        fallback_snapshot_from_state_file, fallback_snapshot_from_state_file_with,
        fallback_snapshot_from_unreachable_helper, helper_runtime_status,
        runtime_snapshot_from_helper, stopped_runtime_snapshot,
    };
    use crate::models::server::ServerStatus;
    use crate::services::server::runtime::i18n::runtime_t;
    use crate::services::server::runtime::local_helper::LocalRuntimeState;

    const INVALID_PID: u32 = u32::MAX;

    fn sample_state() -> LocalRuntimeState {
        LocalRuntimeState {
            server_id: "local-status".to_string(),
            helper_pid: u32::MAX,
            child_pid: Some(INVALID_PID),
            running: true,
            exit_code: None,
            detail_message: "runtime=local running=true source=helper".to_string(),
            error_message: None,
            updated_at: 123,
        }
    }

    #[test]
    fn fallback_snapshot_from_state_file_reports_running_child_when_helper_is_unavailable() {
        let snapshot = fallback_snapshot_from_state_file_with(
            &sample_state(),
            |pid| pid == INVALID_PID,
            false,
        );

        assert!(snapshot.running);
        assert_eq!(snapshot.pid, Some(INVALID_PID));
        assert_eq!(snapshot.exit_code, None);
        assert_eq!(
            snapshot.detail_message,
            "runtime=local running=true source=state_file helper=unavailable exit_code=none"
        );
        assert!(snapshot
            .error_message
            .as_deref()
            .is_some_and(|message| message
                == runtime_t("server.runtime.local_helper.child_running_after_helper_exit")));
    }

    #[test]
    fn fallback_snapshot_from_state_file_keeps_persisted_error_for_stopped_runtime() {
        let mut state = sample_state();
        state.exit_code = Some(7);
        state.error_message = Some("server crashed".to_string());

        let snapshot = fallback_snapshot_from_state_file_with(&state, |_| false, false);

        assert!(!snapshot.running);
        assert_eq!(snapshot.pid, None);
        assert_eq!(snapshot.exit_code, Some(7));
        assert_eq!(
            snapshot.detail_message,
            "runtime=local running=false source=state_file helper=exited exit_code=7"
        );
        assert_eq!(snapshot.error_message.as_deref(), Some("server crashed"));
    }

    #[test]
    fn fallback_snapshot_from_state_file_uses_real_process_probe_for_stopped_runtime() {
        let snapshot = fallback_snapshot_from_state_file(&sample_state());

        assert!(!snapshot.running);
        assert_eq!(snapshot.pid, None);
        assert_eq!(
            snapshot.detail_message,
            "runtime=local running=false source=state_file helper=exited exit_code=none"
        );
    }

    #[test]
    fn fallback_snapshot_from_unreachable_helper_marks_control_plane_failure() {
        let snapshot = fallback_snapshot_from_unreachable_helper(&sample_state());

        assert!(!snapshot.running);
        assert_eq!(snapshot.pid, None);
        assert_eq!(
            snapshot.detail_message,
            "runtime=local running=false source=state_file helper=unreachable exit_code=none"
        );
        assert!(snapshot.error_message.as_deref().is_some_and(
            |message| message == runtime_t("server.runtime.local_helper.status_probe_failed")
        ));
    }

    #[test]
    fn helper_runtime_status_prefers_stopping_then_errors_then_stopped() {
        let mut snapshot = fallback_snapshot_from_state_file(&sample_state());
        assert_eq!(helper_runtime_status(&snapshot, true), ServerStatus::Stopping);

        snapshot.running = false;
        snapshot.pid = None;
        snapshot.error_message = Some("server crashed".to_string());
        assert_eq!(helper_runtime_status(&snapshot, false), ServerStatus::Error);

        snapshot.error_message = None;
        assert_eq!(helper_runtime_status(&snapshot, false), ServerStatus::Stopped);
    }

    #[test]
    fn runtime_snapshot_from_helper_rewrites_stopped_detail_to_runtime_shape() {
        let snapshot = crate::services::server::runtime::local_helper::LocalHelperStatusSnapshot {
            running: false,
            pid: None,
            exit_code: Some(7),
            detail_message:
                "runtime=local running=false source=state_file helper=exited exit_code=7"
                    .to_string(),
            error_message: Some("server crashed".to_string()),
        };

        let runtime = runtime_snapshot_from_helper(snapshot, ServerStatus::Error);

        assert_eq!(runtime.status, ServerStatus::Error);
        assert_eq!(runtime.pid, None);
        assert_eq!(
            runtime.detail_message.as_deref(),
            Some("runtime=local running=false source=helper exit_code=7")
        );
        assert_eq!(runtime.error_message.as_deref(), Some("server crashed"));
    }

    #[test]
    fn stopped_runtime_snapshot_reports_absent_state_consistently() {
        let snapshot = stopped_runtime_snapshot();

        assert_eq!(snapshot.status, ServerStatus::Stopped);
        assert_eq!(snapshot.pid, None);
        assert_eq!(
            snapshot.detail_message.as_deref(),
            Some("runtime=local is_running=false exit_code=none source=state_absent")
        );
        assert_eq!(snapshot.error_message, None);
    }
}
