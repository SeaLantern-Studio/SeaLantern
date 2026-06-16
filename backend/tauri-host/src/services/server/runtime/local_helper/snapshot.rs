use super::LocalHelperStatusSnapshot;
use crate::models::server::LocalTerminalMode;
use crate::models::server::ServerInstance;
use crate::services::server::log_pipeline as server_log_pipeline;
use crate::services::server::manager::ServerManager;
use crate::services::server::runtime::i18n::runtime_t1;

pub(super) fn detect_terminal_snapshot(
    manager: &ServerManager,
    server: &ServerInstance,
) -> Result<Option<LocalHelperStatusSnapshot>, String> {
    let mut procs = manager.lock_processes()?;
    let Some(child) = procs.get_mut(&server.id) else {
        return Ok(Some(process_missing_snapshot()));
    };

    match child.try_wait() {
        Ok(Some(status)) => {
            let exit_code = status.code();
            let terminal_mode = Some(child.terminal_mode());
            let terminal_size = child.terminal_size();
            procs.remove(&server.id);
            server_log_pipeline::shutdown_writer(&server.id);
            Ok(Some(terminal_snapshot_from_exit(
                exit_code,
                terminal_mode,
                terminal_size,
            )))
        }
        Ok(None) => Ok(None),
        Err(err) => {
            let terminal_mode = Some(child.terminal_mode());
            let terminal_size = child.terminal_size();
            procs.remove(&server.id);
            server_log_pipeline::shutdown_writer(&server.id);
            Ok(Some(status_error_snapshot(
                runtime_t1("server.runtime.local.process_status_read_failed", err.to_string()),
                terminal_mode,
                terminal_size,
            )))
        }
    }
}

pub(super) fn snapshot_from_manager(
    manager: &ServerManager,
    server_id: &str,
) -> Result<LocalHelperStatusSnapshot, String> {
    let mut procs = manager.lock_processes()?;
    let Some(child) = procs.get_mut(server_id) else {
        return Ok(process_missing_snapshot());
    };

    match child.try_wait() {
        Ok(Some(status)) => {
            let exit_code = status.code();
            let terminal_mode = Some(child.terminal_mode());
            let terminal_size = child.terminal_size();
            procs.remove(server_id);
            server_log_pipeline::shutdown_writer(server_id);
            Ok(terminal_snapshot_from_exit(
                exit_code,
                terminal_mode,
                terminal_size,
            ))
        }
        Ok(None) => Ok(LocalHelperStatusSnapshot {
            running: true,
            pid: child.id(),
            exit_code: None,
            detail_message: format!(
                "runtime=local running=true source=helper pid={}",
                child
                    .id()
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ),
            error_message: None,
            terminal_mode: Some(child.terminal_mode()),
            terminal_cols: child.terminal_size().map(|(cols, _)| cols),
            terminal_rows: child.terminal_size().map(|(_, rows)| rows),
        }),
        Err(err) => {
            let terminal_mode = Some(child.terminal_mode());
            let terminal_size = child.terminal_size();
            procs.remove(server_id);
            server_log_pipeline::shutdown_writer(server_id);
            Ok(status_error_snapshot(
                runtime_t1("server.runtime.local.process_status_read_failed", err.to_string()),
                terminal_mode,
                terminal_size,
            ))
        }
    }
}

fn process_missing_snapshot() -> LocalHelperStatusSnapshot {
    LocalHelperStatusSnapshot {
        running: false,
        pid: None,
        exit_code: None,
        detail_message: "runtime=local running=false source=helper process=missing".to_string(),
        error_message: None,
        terminal_mode: None,
        terminal_cols: None,
        terminal_rows: None,
    }
}

fn terminal_snapshot_from_exit(
    exit_code: Option<i32>,
    terminal_mode: Option<LocalTerminalMode>,
    terminal_size: Option<(u16, u16)>,
) -> LocalHelperStatusSnapshot {
    LocalHelperStatusSnapshot {
        running: false,
        pid: None,
        exit_code,
        detail_message: format!(
            "runtime=local running=false source=helper exit_code={}",
            exit_code
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        error_message: terminal_error_message(exit_code),
        terminal_mode,
        terminal_cols: terminal_size.map(|(cols, _)| cols),
        terminal_rows: terminal_size.map(|(_, rows)| rows),
    }
}

fn status_error_snapshot(
    error_message: String,
    terminal_mode: Option<LocalTerminalMode>,
    terminal_size: Option<(u16, u16)>,
) -> LocalHelperStatusSnapshot {
    LocalHelperStatusSnapshot {
        running: false,
        pid: None,
        exit_code: None,
        detail_message: "runtime=local running=false source=helper status=error".to_string(),
        error_message: Some(error_message),
        terminal_mode,
        terminal_cols: terminal_size.map(|(cols, _)| cols),
        terminal_rows: terminal_size.map(|(_, rows)| rows),
    }
}

fn terminal_error_message(exit_code: Option<i32>) -> Option<String> {
    if matches!(exit_code, Some(0)) {
        None
    } else {
        Some(runtime_t1(
            "server.runtime.local.exit_abnormal",
            exit_code
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{process_missing_snapshot, terminal_snapshot_from_exit};
    use crate::models::server::LocalTerminalMode;
    use crate::services::server::runtime::i18n::runtime_t1;

    #[test]
    fn terminal_snapshot_from_exit_reports_clean_exit_without_error() {
        let snapshot = terminal_snapshot_from_exit(
            Some(0),
            Some(LocalTerminalMode::PipeManaged),
            None,
        );

        assert!(!snapshot.running);
        assert_eq!(snapshot.pid, None);
        assert_eq!(snapshot.exit_code, Some(0));
        assert_eq!(
            snapshot.detail_message,
            "runtime=local running=false source=helper exit_code=0"
        );
        assert_eq!(snapshot.error_message, None);
    }

    #[test]
    fn terminal_snapshot_from_exit_reports_non_zero_exit_as_error() {
        let snapshot = terminal_snapshot_from_exit(
            Some(7),
            Some(LocalTerminalMode::PtyManaged),
            Some((80, 24)),
        );

        assert_eq!(snapshot.exit_code, Some(7));
        assert_eq!(
            snapshot.error_message.as_deref(),
            Some(runtime_t1("server.runtime.local.exit_abnormal", "7").as_str())
        );
    }

    #[test]
    fn process_missing_snapshot_marks_helper_as_stopped_without_error() {
        let snapshot = process_missing_snapshot();

        assert!(!snapshot.running);
        assert_eq!(snapshot.pid, None);
        assert_eq!(snapshot.exit_code, None);
        assert_eq!(
            snapshot.detail_message,
            "runtime=local running=false source=helper process=missing"
        );
        assert_eq!(snapshot.error_message, None);
    }
}
