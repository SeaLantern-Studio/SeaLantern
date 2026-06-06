use super::{LocalHelperStatusSnapshot, LocalRuntimeState};
use crate::models::server::ServerInstance;
use crate::services::server::runtime::i18n::{runtime_t1, runtime_t2};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn remove_state_file(server: &ServerInstance) {
    let path = super::state_file_path(server);
    let _ = std::fs::remove_file(path);
}

pub(super) fn write_state_file(path: &Path, state: &LocalRuntimeState) -> Result<(), String> {
    let content = serde_json::to_string_pretty(state).map_err(|e| {
        runtime_t1("server.runtime.local_helper.state_serialize_failed", e.to_string())
    })?;
    std::fs::write(path, content).map_err(|e| {
        runtime_t2(
            "server.runtime.local_helper.state_write_failed",
            path.display().to_string(),
            e.to_string(),
        )
    })
}

pub(super) fn persist_terminal_state(
    server: &ServerInstance,
    state: &LocalRuntimeState,
    exit_code: Option<i32>,
    error_message: Option<String>,
) -> Result<(), String> {
    let next = terminal_state_from_exit(state, exit_code, error_message, current_timestamp_secs());
    write_state_file(&super::state_file_path(server), &next)
}

pub(super) fn started_state(
    server_id: &str,
    helper_pid: u32,
    child_pid: Option<u32>,
    control_port: u16,
    auth_token: String,
    updated_at: u64,
) -> LocalRuntimeState {
    LocalRuntimeState {
        server_id: server_id.to_string(),
        helper_pid,
        child_pid,
        control_port: Some(control_port),
        auth_token,
        running: true,
        exit_code: None,
        detail_message: format!(
            "runtime=local running=true source=helper child_pid={} control_port={}",
            child_pid
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            control_port
        ),
        error_message: None,
        updated_at,
    }
}

pub(super) fn state_from_terminal_snapshot(
    state: &LocalRuntimeState,
    snapshot: &LocalHelperStatusSnapshot,
    updated_at: u64,
) -> LocalRuntimeState {
    LocalRuntimeState {
        server_id: state.server_id.clone(),
        helper_pid: state.helper_pid,
        child_pid: state.child_pid,
        control_port: None,
        auth_token: state.auth_token.clone(),
        running: false,
        exit_code: snapshot.exit_code,
        detail_message: snapshot.detail_message.clone(),
        error_message: snapshot.error_message.clone(),
        updated_at,
    }
}

pub(super) fn state_from_requested_stop(
    state: &LocalRuntimeState,
    updated_at: u64,
) -> LocalRuntimeState {
    LocalRuntimeState {
        server_id: state.server_id.clone(),
        helper_pid: state.helper_pid,
        child_pid: None,
        control_port: None,
        auth_token: state.auth_token.clone(),
        running: false,
        exit_code: state.exit_code,
        detail_message: if state.exit_code == Some(0) {
            "runtime=local running=false source=helper exit_code=0".to_string()
        } else {
            "runtime=local running=false source=helper requested_stop".to_string()
        },
        error_message: None,
        updated_at,
    }
}

pub(super) fn terminal_state_from_exit(
    state: &LocalRuntimeState,
    exit_code: Option<i32>,
    error_message: Option<String>,
    updated_at: u64,
) -> LocalRuntimeState {
    LocalRuntimeState {
        server_id: state.server_id.clone(),
        helper_pid: state.helper_pid,
        child_pid: state.child_pid,
        control_port: None,
        auth_token: state.auth_token.clone(),
        running: false,
        exit_code,
        detail_message: format!(
            "runtime=local running=false source=state_file exit_code={}",
            exit_code
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        error_message,
        updated_at,
    }
}

pub(super) fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        started_state, state_from_requested_stop, state_from_terminal_snapshot,
        terminal_state_from_exit, write_state_file, LocalHelperStatusSnapshot, LocalRuntimeState,
    };
    use crate::models::server::{LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig};
    use crate::services::server::runtime::i18n::runtime_t1;
    use crate::services::server::runtime::local_helper::{read_state, state_file_path};
    use tempfile::tempdir;

    fn test_server(path: String) -> ServerInstance {
        ServerInstance {
            id: "local-state".to_string(),
            name: "Local State".to_string(),
            aliases: Vec::new(),
            core_type: "fabric".to_string(),
            core_version: "fabric".to_string(),
            mc_version: "1.20.1".to_string(),
            path,
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "java".to_string(),
                jvm_args: Vec::new(),
                cpu_policy: crate::models::server::CpuPolicyConfig::default(),
                jvm_preset: crate::models::server::JvmPresetConfig::default(),
            }),
        }
    }

    fn sample_state() -> LocalRuntimeState {
        LocalRuntimeState {
            server_id: "local-state".to_string(),
            helper_pid: 11,
            child_pid: Some(22),
            control_port: Some(25570),
            auth_token: "token".to_string(),
            running: true,
            exit_code: None,
            detail_message: "runtime=local running=true source=helper pid=22".to_string(),
            error_message: None,
            updated_at: 123,
        }
    }

    fn stopped_snapshot(exit_code: Option<i32>) -> LocalHelperStatusSnapshot {
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
            error_message: exit_code
                .map(|code| runtime_t1("server.runtime.local.exit_abnormal", code.to_string())),
        }
    }

    #[test]
    fn started_state_populates_initial_helper_control_plane_fields() {
        let state = started_state("local-state", 11, Some(22), 25570, "token".to_string(), 456);

        assert_eq!(state.server_id, "local-state");
        assert_eq!(state.helper_pid, 11);
        assert_eq!(state.child_pid, Some(22));
        assert_eq!(state.control_port, Some(25570));
        assert_eq!(state.auth_token, "token");
        assert!(state.running);
        assert_eq!(state.exit_code, None);
        assert_eq!(state.error_message, None);
        assert_eq!(state.updated_at, 456);
        assert_eq!(
            state.detail_message,
            "runtime=local running=true source=helper child_pid=22 control_port=25570"
        );
    }

    #[test]
    fn terminal_state_from_exit_clears_control_plane_fields() {
        let next = terminal_state_from_exit(
            &sample_state(),
            Some(7),
            Some("helper unavailable".to_string()),
            456,
        );

        assert!(!next.running);
        assert_eq!(next.control_port, None);
        assert_eq!(next.child_pid, Some(22));
        assert_eq!(next.exit_code, Some(7));
        assert_eq!(next.error_message.as_deref(), Some("helper unavailable"));
        assert_eq!(next.updated_at, 456);
        assert_eq!(
            next.detail_message,
            "runtime=local running=false source=state_file exit_code=7"
        );
    }

    #[test]
    fn state_from_terminal_snapshot_keeps_identity_and_uses_snapshot_payload() {
        let next = state_from_terminal_snapshot(&sample_state(), &stopped_snapshot(Some(7)), 789);

        assert_eq!(next.server_id, "local-state");
        assert_eq!(next.helper_pid, 11);
        assert_eq!(next.child_pid, Some(22));
        assert_eq!(next.control_port, None);
        assert_eq!(next.auth_token, "token");
        assert!(!next.running);
        assert_eq!(next.exit_code, Some(7));
        assert_eq!(next.updated_at, 789);
        assert_eq!(next.detail_message, "runtime=local running=false source=helper exit_code=7");
        assert_eq!(
            next.error_message.as_deref(),
            Some(runtime_t1("server.runtime.local.exit_abnormal", "7").as_str())
        );
    }

    #[test]
    fn state_from_requested_stop_clears_live_control_plane_fields() {
        let next = state_from_requested_stop(&sample_state(), 999);

        assert_eq!(next.server_id, "local-state");
        assert_eq!(next.helper_pid, 11);
        assert_eq!(next.child_pid, None);
        assert_eq!(next.control_port, None);
        assert_eq!(next.auth_token, "token");
        assert!(!next.running);
        assert_eq!(next.exit_code, None);
        assert_eq!(next.error_message, None);
        assert_eq!(next.updated_at, 999);
        assert_eq!(next.detail_message, "runtime=local running=false source=helper requested_stop");
    }

    #[test]
    fn state_from_requested_stop_preserves_clean_exit_detail_when_exit_code_is_zero() {
        let mut state = sample_state();
        state.exit_code = Some(0);

        let next = state_from_requested_stop(&state, 1001);

        assert_eq!(next.exit_code, Some(0));
        assert_eq!(next.detail_message, "runtime=local running=false source=helper exit_code=0");
    }

    #[test]
    fn read_state_round_trips_written_state_file() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        let path = state_file_path(&server);
        let expected = sample_state();

        write_state_file(&path, &expected).expect("state file should be written");

        let actual = read_state(&server).expect("state file should deserialize");
        assert_eq!(actual, expected);
    }
}
