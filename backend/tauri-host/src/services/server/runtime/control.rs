use crate::models::server::ServerStatus;
use crate::services::server::log_pipeline as server_log_pipeline;
use crate::services::server::manager::ServerManager;
use crate::utils::logger;

pub fn log_runtime_action(scope: &str, action: &str, detail: impl AsRef<str>) {
    logger::log_user_action(scope, action, detail.as_ref());
}

pub fn log_runtime_action_error(scope: &str, action: &str, detail: impl AsRef<str>, error: &str) {
    logger::log_user_action_error(scope, action, detail.as_ref(), error);
}

pub fn clear_runtime_flags(manager: &ServerManager, server_id: &str) {
    manager.clear_starting(server_id);
    manager.clear_stopping(server_id);
}

pub fn clear_runtime_flags_if_terminal(
    manager: &ServerManager,
    server_id: &str,
    status: &ServerStatus,
) {
    if matches!(status, ServerStatus::Stopped | ServerStatus::Error) {
        clear_runtime_flags(manager, server_id);
    }
}

pub fn spawn_stop_worker<F>(
    scope: &'static str,
    server_id: String,
    failure_log_message: String,
    stop_action: F,
) where
    F: FnOnce() -> Result<(), String> + Send + 'static,
{
    std::thread::spawn(move || {
        if let Err(err) = stop_action() {
            let manager = crate::services::global::server_manager();
            log_runtime_action_error(
                scope,
                "stop_worker",
                format!("server_id={}", server_id),
                &err,
            );
            let _ = server_log_pipeline::append_sealantern_log(
                &server_id,
                &format!("{}: {}", failure_log_message, err),
            );
            clear_runtime_flags(manager, &server_id);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{clear_runtime_flags, clear_runtime_flags_if_terminal};
    use crate::models::server::ServerStatus;
    use crate::services::server::manager::ServerManager;
    use crate::test_support::{lock_env, EnvGuard};
    use tempfile::tempdir;

    #[test]
    fn clear_runtime_flags_removes_starting_and_stopping_markers() {
        let _env_lock = lock_env();
        let temp_dir = tempdir().expect("temp dir should exist");
        let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &temp_dir.path().to_string_lossy());
        let manager = ServerManager::new();
        manager.mark_starting("alpha");
        manager.mark_stopping("alpha");

        clear_runtime_flags(&manager, "alpha");

        assert!(!manager
            .is_starting_checked("alpha")
            .expect("starting flag should read"));
        assert!(!manager
            .is_stopping_checked("alpha")
            .expect("stopping flag should read"));
    }

    #[test]
    fn clear_runtime_flags_if_terminal_only_clears_terminal_statuses() {
        let _env_lock = lock_env();
        let temp_dir = tempdir().expect("temp dir should exist");
        let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &temp_dir.path().to_string_lossy());
        let manager = ServerManager::new();
        manager.mark_starting("alpha");
        manager.mark_stopping("alpha");

        clear_runtime_flags_if_terminal(&manager, "alpha", &ServerStatus::Running);
        assert!(manager
            .is_starting_checked("alpha")
            .expect("starting flag should read"));
        assert!(manager
            .is_stopping_checked("alpha")
            .expect("stopping flag should read"));

        clear_runtime_flags_if_terminal(&manager, "alpha", &ServerStatus::Stopped);
        assert!(!manager
            .is_starting_checked("alpha")
            .expect("starting flag should read"));
        assert!(!manager
            .is_stopping_checked("alpha")
            .expect("stopping flag should read"));
    }
}
