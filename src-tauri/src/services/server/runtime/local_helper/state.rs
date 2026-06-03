use crate::models::server::ServerInstance;
use crate::utils::constants::LOCAL_RUNTIME_STATE_FILE;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalRuntimeState {
    pub server_id: String,
    pub helper_pid: u32,
    pub child_pid: Option<u32>,
    pub control_port: Option<u16>,
    pub auth_token: String,
    pub running: bool,
    pub exit_code: Option<i32>,
    pub detail_message: String,
    pub error_message: Option<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalHelperStatusSnapshot {
    pub running: bool,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub detail_message: String,
    pub error_message: Option<String>,
}

pub fn state_file_path(server: &ServerInstance) -> PathBuf {
    Path::new(&server.path).join(LOCAL_RUNTIME_STATE_FILE)
}

pub fn read_state(server: &ServerInstance) -> Option<LocalRuntimeState> {
    let path = state_file_path(server);
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<LocalRuntimeState>(&content).ok()
}

pub fn remove_state_file(server: &ServerInstance) {
    let path = state_file_path(server);
    let _ = std::fs::remove_file(path);
}

pub(super) fn write_state_file(path: &Path, state: &LocalRuntimeState) -> Result<(), String> {
    let content = serde_json::to_string_pretty(state)
        .map_err(|e| format!("序列化本地 runtime 状态失败: {}", e))?;
    std::fs::write(path, content)
        .map_err(|e| format!("写入本地 runtime 状态失败 ({}): {}", path.display(), e))
}

pub(super) fn persist_terminal_state(
    server: &ServerInstance,
    state: &LocalRuntimeState,
    exit_code: Option<i32>,
    error_message: Option<String>,
) -> Result<(), String> {
    let next = terminal_state_from_exit(state, exit_code, error_message, current_timestamp_secs());
    write_state_file(&state_file_path(server), &next)
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
        read_state, state_file_path, terminal_state_from_exit, write_state_file, LocalRuntimeState,
    };
    use crate::models::server::{LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig};
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
