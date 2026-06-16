mod dispatch;
mod protocol;
mod snapshot;
mod state;
mod status;

use serde::{Deserialize, Serialize};
use sl_server_info::log::LogStream;
use std::path::{Path, PathBuf};

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
    #[serde(default)]
    pub fallback: Option<LocalRuntimeFallbackInfo>,
    #[serde(default)]
    pub terminal_mode: Option<LocalTerminalMode>,
    #[serde(default)]
    pub terminal_cols: Option<u16>,
    #[serde(default)]
    pub terminal_rows: Option<u16>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalRuntimeFallbackInfo {
    pub from_mode: String,
    pub to_mode: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalHelperStatusSnapshot {
    pub running: bool,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub detail_message: String,
    pub error_message: Option<String>,
    #[serde(default)]
    pub terminal_mode: Option<LocalTerminalMode>,
    #[serde(default)]
    pub terminal_cols: Option<u16>,
    #[serde(default)]
    pub terminal_rows: Option<u16>,
}

pub(crate) use status::{
    helper_runtime_status, runtime_snapshot_from_helper, stopped_runtime_snapshot,
};

use self::dispatch::handle_connection;
use self::protocol::{send_request, LocalHelperRequest};
use self::snapshot::detect_terminal_snapshot;
use self::state::{
    current_timestamp_secs, helper_ready_state, persist_terminal_state, remove_state_file,
    start_failed_state, started_state, state_from_requested_stop, state_from_terminal_snapshot,
    write_state_file,
};
use self::status::{fallback_snapshot_from_state_file, fallback_snapshot_from_unreachable_helper};
use crate::models::server::{LocalTerminalMode, ServerInstance, TerminalStatusInfo};
use crate::services::global;
use crate::services::server::manager::process::{force_kill_process_tree_by_pid, is_process_alive};
use crate::services::server::runtime::local_terminal_reader::spawn_local_terminal_reader;
use crate::services::server::runtime::i18n::{runtime_t, runtime_t1, runtime_t2};
use crate::services::server::runtime::{RuntimeProcessHandle, RuntimeStartRequest};
use crate::services::server::terminal_transcript;
use crate::utils::logger;
use std::ffi::OsString;
use std::net::TcpListener;
use std::process::Command;
use std::time::Duration;
use sysinfo::{Pid, ProcessesToUpdate, System};

pub fn state_file_path(server: &ServerInstance) -> PathBuf {
    Path::new(&server.path).join(crate::utils::constants::LOCAL_RUNTIME_STATE_FILE)
}

pub fn read_state_checked(server: &ServerInstance) -> Result<Option<LocalRuntimeState>, String> {
    let path = state_file_path(server);
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(runtime_t2(
                "server.runtime.local_helper.state_read_failed",
                path.display().to_string(),
                error.to_string(),
            ));
        }
    };

    let state = serde_json::from_str::<LocalRuntimeState>(&content).map_err(|error| {
        runtime_t2(
            "server.runtime.local_helper.state_parse_failed",
            path.display().to_string(),
            error.to_string(),
        )
    })?;

    Ok(Some(state))
}

#[allow(dead_code)]
pub fn read_state(server: &ServerInstance) -> Option<LocalRuntimeState> {
    read_state_checked(server).ok().flatten()
}

pub fn cleanup_for_new_start(server: &ServerInstance) {
    let current_exe = current_exe_lowercase();

    match read_state_checked(server) {
        Ok(Some(state)) => {
            let helper_alive = helper_process_matches_server_pid(
                state.helper_pid,
                &server.id,
                current_exe.as_deref(),
            );
            let child_alive = state.child_pid.is_some_and(is_process_alive);
            if helper_alive || child_alive {
                logger::log_warn_ctx("server.runtime.local_helper", "cleanup_for_new_start", &format!(
                    "local runtime start cleanup detected lingering state: server_id={} helper_alive={} child_alive={} helper_pid={} child_pid={}",
                    server.id,
                    helper_alive,
                    child_alive,
                    state.helper_pid,
                    state
                        .child_pid
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "none".to_string())
                ));
            }

            cleanup_runtime_processes_from_state(server, &state, current_exe.as_deref());
        }
        Ok(None) => {}
        Err(error) => logger::log_warn_ctx(
            "server.runtime.local_helper",
            "cleanup_for_new_start",
            &format!(
                "local runtime start cleanup skipped unreadable state file: server_id={} error={}",
                server.id, error
            ),
        ),
    }

    cleanup_orphan_helper_processes(server, current_exe.as_deref());

    remove_state_file(server);
}

fn cleanup_runtime_processes_from_state(
    server: &ServerInstance,
    state: &LocalRuntimeState,
    current_exe: Option<&str>,
) {
    if helper_process_matches_server_pid(state.helper_pid, &server.id, current_exe) {
        match force_kill_process_tree_by_pid(state.helper_pid) {
            Ok(()) => logger::log_user_action(
                "server.runtime.local.helper",
                "cleanup_lingering_helper",
                &format!("server_id={} helper_pid={}", server.id, state.helper_pid),
            ),
            Err(err) => logger::log_user_action_error(
                "server.runtime.local.helper",
                "cleanup_lingering_helper",
                &format!("server_id={} helper_pid={}", server.id, state.helper_pid),
                &err,
            ),
        }
        return;
    }

    if is_process_alive(state.helper_pid) {
        logger::log_warn_ctx("server.runtime.local_helper", "cleanup_runtime_processes_from_state", &format!(
            "local runtime start cleanup skipped stale helper pid because identity no longer matches: server_id={} helper_pid={}",
            server.id, state.helper_pid
        ));
    }

    if let Some(child_pid) = state.child_pid.filter(|pid| is_process_alive(*pid)) {
        logger::log_warn_ctx("server.runtime.local_helper", "cleanup_runtime_processes_from_state", &format!(
            "local runtime start cleanup skipped lingering child without verified helper ownership: server_id={} child_pid={} helper_pid={}",
            server.id, child_pid, state.helper_pid
        ));
    }
}

fn cleanup_orphan_helper_processes(server: &ServerInstance, current_exe: Option<&str>) {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    for process in system.processes().values() {
        if !process_matches_server_helper_identity(
            process.cmd(),
            process
                .exe()
                .map(|path| path.to_string_lossy().to_ascii_lowercase()),
            &server.id,
            current_exe,
        ) {
            continue;
        }

        let pid = process.pid().as_u32();
        if pid == std::process::id() {
            continue;
        }

        match force_kill_process_tree_by_pid(pid) {
            Ok(()) => logger::log_user_action(
                "server.runtime.local.helper",
                "cleanup_orphan_helper",
                &format!("server_id={} helper_pid={}", server.id, pid),
            ),
            Err(err) => logger::log_user_action_error(
                "server.runtime.local.helper",
                "cleanup_orphan_helper",
                &format!("server_id={} helper_pid={}", server.id, pid),
                &err,
            ),
        }
    }
}

fn current_exe_lowercase() -> Option<String> {
    std::env::current_exe()
        .ok()
        .map(|path| path.to_string_lossy().to_ascii_lowercase())
}

fn helper_process_matches_server(server: &ServerInstance, pid: u32) -> bool {
    let current_exe = current_exe_lowercase();
    helper_process_matches_server_pid(pid, &server.id, current_exe.as_deref())
}

fn helper_process_matches_server_pid(pid: u32, server_id: &str, current_exe: Option<&str>) -> bool {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    let Some(process) = system.process(Pid::from_u32(pid)) else {
        return false;
    };

    process_matches_server_helper_identity(
        process.cmd(),
        process
            .exe()
            .map(|path| path.to_string_lossy().to_ascii_lowercase()),
        server_id,
        current_exe,
    )
}

fn process_matches_server_helper_identity(
    cmd: &[OsString],
    process_exe: Option<String>,
    server_id: &str,
    current_exe: Option<&str>,
) -> bool {
    if !looks_like_local_runtime_helper_command(cmd, server_id) {
        return false;
    }

    if let Some(current_exe) = current_exe {
        let process_exe = process_exe.unwrap_or_default();
        if !process_exe.is_empty() && process_exe != current_exe {
            return false;
        }
    }

    true
}

fn looks_like_local_runtime_helper_command(cmd: &[OsString], server_id: &str) -> bool {
    let mut saw_helper_marker = false;
    let mut saw_server_id = false;

    for arg in cmd {
        let value = arg.to_string_lossy();
        if value == "__local-runtime-helper" {
            saw_helper_marker = true;
        }
        if value == server_id {
            saw_server_id = true;
        }
    }

    saw_helper_marker && saw_server_id
}

pub fn spawn_helper_process(server: &ServerInstance) -> Result<(), String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| runtime_t1("server.runtime.local_helper.current_exe_failed", e.to_string()))?;
    let mut command = Command::new(current_exe);
    command.arg("__local-runtime-helper").arg(&server.id);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
        .spawn()
        .map(|_| ())
        .map_err(|e| runtime_t1("server.runtime.local_helper.spawn_failed", e.to_string()))
}

pub fn wait_for_helper_ready(
    server: &ServerInstance,
    timeout: Duration,
) -> Result<LocalRuntimeState, String> {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if let Some(state) = read_state_checked(server)? {
            if state.child_pid.is_some() || !state.running {
                return Ok(state);
            }
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    Err(runtime_t2(
        "server.runtime.local_helper.ready_timeout",
        server.id.clone(),
        state_file_path(server).display().to_string(),
    ))
}

pub fn status_snapshot(
    server: &ServerInstance,
) -> Result<Option<LocalHelperStatusSnapshot>, String> {
    let Some(state) = read_state_checked(server)? else {
        return Ok(None);
    };

    if helper_process_matches_server(server, state.helper_pid) {
        match send_request(
            &state,
            LocalHelperRequest::Status { auth_token: state.auth_token.clone() },
        ) {
            Ok(response) => return Ok(response.snapshot),
            Err(error) => {
                logger::log_warn_ctx(
                    "server.runtime.local_helper",
                    "status_snapshot",
                    &format!(
                        "status probe failed; falling back to state file: server_id={} helper_pid={} error={}",
                        server.id, state.helper_pid, error
                    ),
                );

                return Ok(Some(fallback_snapshot_from_unreachable_helper(&state)));
            }
        }
    }

    Ok(Some(fallback_snapshot_from_state_file(&state)))
}

pub(crate) fn build_terminal_status(
    terminal_mode: LocalTerminalMode,
    cols: Option<u16>,
    rows: Option<u16>,
) -> TerminalStatusInfo {
    match terminal_mode {
        LocalTerminalMode::PipeManaged => TerminalStatusInfo {
            backend_kind: "pipe".to_string(),
            interactive_supported: false,
            transcript_supported: false,
            attach_supported: false,
            cols,
            rows,
        },
        LocalTerminalMode::PtyManaged => TerminalStatusInfo {
            backend_kind: "pty".to_string(),
            interactive_supported: true,
            transcript_supported: true,
            attach_supported: true,
            cols,
            rows,
        },
    }
}

pub(crate) fn terminal_status_from_state(state: &LocalRuntimeState) -> Option<TerminalStatusInfo> {
    state.terminal_mode.map(|terminal_mode| {
        build_terminal_status(terminal_mode, state.terminal_cols, state.terminal_rows)
    })
}

pub(crate) fn effective_terminal_mode(server: &ServerInstance) -> Result<LocalTerminalMode, String> {
    let configured = server
        .local_runtime()
        .map(|runtime| runtime.terminal_mode)
        .unwrap_or_default();

    let Some(state) = read_state_checked(server)? else {
        return Ok(configured);
    };

    Ok(state
        .terminal_mode
        .or_else(|| {
            state
                .fallback
                .as_ref()
                .and_then(|fallback| parse_terminal_mode_name(&fallback.to_mode))
        })
        .unwrap_or(configured))
}

fn parse_terminal_mode_name(mode: &str) -> Option<LocalTerminalMode> {
    match mode {
        "pipe_managed" => Some(LocalTerminalMode::PipeManaged),
        "pty_managed" => Some(LocalTerminalMode::PtyManaged),
        _ => None,
    }
}

pub fn send_command(server: &ServerInstance, command: &str) -> Result<(), String> {
    let state = read_state_checked(server)?.ok_or_else(|| {
        runtime_t1(
            "server.runtime.local_helper.state_missing",
            state_file_path(server).display().to_string(),
        )
    })?;

    if !helper_process_matches_server(server, state.helper_pid) {
        return Err(runtime_t("server.runtime.local_helper.send_unavailable"));
    }

    let response = send_request(
        &state,
        LocalHelperRequest::Send {
            auth_token: state.auth_token.clone(),
            command: command.to_string(),
        },
    )?;
    if response.ok {
        Ok(())
    } else {
        Err(response
            .error
            .unwrap_or_else(|| "helper send failed".to_string()))
    }
}

pub fn send_terminal_input(server: &ServerInstance, input: &str) -> Result<(), String> {
    let state = read_state_checked(server)?.ok_or_else(|| {
        runtime_t1(
            "server.runtime.local_helper.state_missing",
            state_file_path(server).display().to_string(),
        )
    })?;

    if !helper_process_matches_server(server, state.helper_pid) {
        return Err(runtime_t("server.runtime.local_helper.send_unavailable"));
    }

    let response = send_request(
        &state,
        LocalHelperRequest::TerminalInput {
            auth_token: state.auth_token.clone(),
            input: input.to_string(),
        },
    )?;
    if response.ok {
        Ok(())
    } else {
        Err(response.error.unwrap_or_else(|| "helper terminal input failed".to_string()))
    }
}

pub fn resize_terminal(server: &ServerInstance, cols: u16, rows: u16) -> Result<(), String> {
    let state = read_state_checked(server)?.ok_or_else(|| {
        runtime_t1(
            "server.runtime.local_helper.state_missing",
            state_file_path(server).display().to_string(),
        )
    })?;

    if !helper_process_matches_server(server, state.helper_pid) {
        return Err(runtime_t("server.runtime.local_helper.send_unavailable"));
    }

    let response = send_request(
        &state,
        LocalHelperRequest::ResizeTerminal {
            auth_token: state.auth_token.clone(),
            cols,
            rows,
        },
    )?;
    if response.ok {
        Ok(())
    } else {
        Err(response.error.unwrap_or_else(|| "helper terminal resize failed".to_string()))
    }
}

pub fn request_stop(server: &ServerInstance) -> Result<(), String> {
    let state = read_state_checked(server)?.ok_or_else(|| {
        runtime_t1(
            "server.runtime.local_helper.state_missing",
            state_file_path(server).display().to_string(),
        )
    })?;

    if helper_process_matches_server(server, state.helper_pid) {
        let response = match send_request(
            &state,
            LocalHelperRequest::Stop { auth_token: state.auth_token.clone() },
        ) {
            Ok(response) => response,
            Err(error) => {
                if state.child_pid.is_some_and(is_process_alive) {
                    return Err(runtime_t1(
                        "server.runtime.local_helper.stop_failed_child_running",
                        error,
                    ));
                }

                return Err(error);
            }
        };
        if response.ok {
            return Ok(());
        }
        let error = response
            .error
            .unwrap_or_else(|| "helper stop failed".to_string());

        if state.child_pid.is_some_and(is_process_alive) {
            return Err(runtime_t1("server.runtime.local_helper.stop_failed_child_running", error));
        }

        return Err(error);
    }

    if let Some(pid) = state.child_pid.filter(|pid| is_process_alive(*pid)) {
        let _ = pid;
        return Err(runtime_t("server.runtime.local_helper.stop_unavailable_child_running"));
    }

    Ok(())
}

pub fn force_stop(server: &ServerInstance) -> Result<(), String> {
    let state = read_state_checked(server)?.ok_or_else(|| {
        runtime_t1(
            "server.runtime.local_helper.state_missing",
            state_file_path(server).display().to_string(),
        )
    })?;

    if helper_process_matches_server(server, state.helper_pid) {
        let response = match send_request(
            &state,
            LocalHelperRequest::ForceStop { auth_token: state.auth_token.clone() },
        ) {
            Ok(response) => response,
            Err(error) => {
                if let Some(pid) = state.child_pid.filter(|pid| is_process_alive(*pid)) {
                    force_kill_process_tree_by_pid(pid)?;
                    persist_terminal_state(
                        server,
                        &state,
                        None,
                        Some(runtime_t1(
                            "server.runtime.local_helper.force_stop_fallback_with_error",
                            error,
                        )),
                    )?;
                    return Ok(());
                }

                return Err(error);
            }
        };
        if response.ok {
            return Ok(());
        }
        let error = response
            .error
            .unwrap_or_else(|| "helper force stop failed".to_string());

        if let Some(pid) = state.child_pid.filter(|pid| is_process_alive(*pid)) {
            force_kill_process_tree_by_pid(pid)?;
            persist_terminal_state(
                server,
                &state,
                None,
                Some(runtime_t1(
                    "server.runtime.local_helper.force_stop_fallback_with_error",
                    error,
                )),
            )?;
            return Ok(());
        }

        return Err(error);
    }

    if let Some(pid) = state.child_pid.filter(|pid| is_process_alive(*pid)) {
        force_kill_process_tree_by_pid(pid)?;
        persist_terminal_state(
            server,
            &state,
            None,
            Some(runtime_t("server.runtime.local_helper.force_stop_fallback")),
        )?;
    }

    Ok(())
}

pub fn handle_helper_command(args: &[String]) -> i32 {
    let Some(server_id) = args.first().map(String::as_str) else {
        eprintln!("{}", runtime_t("server.runtime.local_helper.server_id_missing"));
        return 2;
    };

    match run_helper(server_id) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{}", runtime_t1("server.runtime.local_helper.run_failed", err));
            2
        }
    }
}

fn run_helper(server_id: &str) -> Result<(), String> {
    let manager = global::server_manager();
    let server = manager.find_server_clone(server_id)?;
    let helper_pid = std::process::id();
    let auth_token = uuid::Uuid::new_v4().to_string();
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|e| runtime_t1("server.runtime.local_helper.bind_failed", e.to_string()))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| runtime_t1("server.runtime.local_helper.nonblocking_failed", e.to_string()))?;
    let control_port = listener
        .local_addr()
        .map_err(|e| runtime_t1("server.runtime.local_helper.local_addr_failed", e.to_string()))?
        .port();

    let state_path = state_file_path(&server);
    let configured_terminal_mode = server
        .local_runtime()
        .map(|runtime| runtime.terminal_mode)
        .unwrap_or_default();
    let mut state = helper_ready_state(
        server_id,
        helper_pid,
        control_port,
        auth_token.clone(),
        configured_terminal_mode,
        current_timestamp_secs(),
    );
    write_state_file(&state_path, &state)?;

    let start_result =
        match manager.start_local_runtime(RuntimeStartRequest { server_id, server: &server }) {
            Ok(result) => result,
            Err(error) => {
                state = start_failed_state(&state, error.clone(), current_timestamp_secs());
                write_state_file(&state_path, &state)?;
                return Err(error);
            }
        };
    let Some(RuntimeProcessHandle::LocalProcess {
        process: child,
        readers,
    }) = start_result.process_handle
    else {
        return Err(runtime_t("server.runtime.local_helper.child_handle_missing"));
    };

    let child_pid = child.id();
    let actual_terminal_mode = child.terminal_mode();
    let actual_terminal_size = child.terminal_size();

    let _ = terminal_transcript::reset_transcript(&server);

    manager
        .lock_processes()?
        .insert(server_id.to_string(), child);

    if let Some(stdout) = readers.stdout {
        spawn_local_terminal_reader(
            server_id.to_string(),
            std::path::PathBuf::from(&server.path),
            LogStream::Stdout,
            stdout,
        );
    }
    if let Some(stderr) = readers.stderr {
        spawn_local_terminal_reader(
            server_id.to_string(),
            std::path::PathBuf::from(&server.path),
            LogStream::Stderr,
            stderr,
        );
    }

    state = started_state(
        server_id,
        helper_pid,
        child_pid,
        control_port,
        auth_token,
        start_result.fallback.map(|fallback| LocalRuntimeFallbackInfo {
            from_mode: fallback.from_mode,
            to_mode: fallback.to_mode,
            reason: fallback.reason,
        }),
        actual_terminal_mode,
        actual_terminal_size,
        current_timestamp_secs(),
    );
    write_state_file(&state_path, &state)?;
    logger::log_user_action(
        "server.runtime.local.helper",
        "started",
        &format!(
            "server_id={} helper_pid={} child_pid={} control_port={}",
            server_id,
            helper_pid,
            child_pid
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            control_port
        ),
    );

    loop {
        if let Some(snapshot) = detect_terminal_snapshot(manager, &server)? {
            state = state_from_terminal_snapshot(&state, &snapshot, current_timestamp_secs());
            write_state_file(&state_path, &state)?;
            break;
        }

        match listener.accept() {
            Ok((stream, _)) => {
                let should_exit = handle_connection(manager, &server, &state, stream)?;
                if should_exit {
                    state = state_from_requested_stop(&state, current_timestamp_secs());
                    write_state_file(&state_path, &state)?;
                    break;
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(err) => {
                logger::log_warn_ctx(
                    "server.runtime.local_helper",
                    "run_helper",
                    &format!("accept failed server_id={} error={}", server_id, err),
                );
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::state::write_state_file;
    use super::status_snapshot;
    use super::{
        looks_like_local_runtime_helper_command, process_matches_server_helper_identity,
        read_state_checked, resize_terminal, send_command, send_terminal_input, state_file_path,
    };
    use crate::models::server::{LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig};
    use crate::services::server::runtime::i18n::runtime_t;
    use std::ffi::OsString;
    use tempfile::tempdir;

    fn test_server(path: String) -> ServerInstance {
        ServerInstance {
            id: "local-helper".to_string(),
            name: "Local Helper".to_string(),
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
                terminal_mode: crate::models::server::LocalTerminalMode::PipeManaged,
                cpu_policy: crate::models::server::CpuPolicyConfig::default(),
                jvm_preset: crate::models::server::JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn helper_status_snapshot_falls_back_to_helper_exited_state_file_when_helper_is_unavailable() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        let path = state_file_path(&server);

        write_state_file(
            &path,
            &super::LocalRuntimeState {
                server_id: server.id.clone(),
                helper_pid: 999_999,
                child_pid: None,
                control_port: Some(25570),
                auth_token: "token".to_string(),
                running: true,
                exit_code: None,
                detail_message:
                    "runtime=local running=true source=helper child_pid=none control_port=25570"
                        .to_string(),
                error_message: None,
                fallback: None,
                terminal_mode: Some(crate::models::server::LocalTerminalMode::PipeManaged),
                terminal_cols: None,
                terminal_rows: None,
                updated_at: 123,
            },
        )
        .expect("state file should be written");

        let snapshot = status_snapshot(&server)
            .expect("status should succeed")
            .expect("fallback snapshot should exist");

        assert!(!snapshot.running);
        assert_eq!(snapshot.pid, None);
        assert_eq!(snapshot.exit_code, None);
        assert_eq!(
            snapshot.detail_message,
            "runtime=local running=false source=state_file helper=exited exit_code=none"
        );
        assert_eq!(snapshot.error_message, None);
    }

    #[test]
    fn helper_identity_match_requires_helper_marker_and_server_id() {
        let cmd = vec![
            OsString::from("sea-lantern.exe"),
            OsString::from("__local-runtime-helper"),
            OsString::from("server-1"),
        ];

        assert!(looks_like_local_runtime_helper_command(&cmd, "server-1"));
        assert!(!looks_like_local_runtime_helper_command(&cmd, "server-2"));
    }

    #[test]
    fn helper_identity_match_rejects_pid_reuse_with_different_executable() {
        let cmd = vec![
            OsString::from("other.exe"),
            OsString::from("__local-runtime-helper"),
            OsString::from("server-1"),
        ];

        assert!(!process_matches_server_helper_identity(
            &cmd,
            Some("c:/windows/system32/not-helper.exe".to_string()),
            "server-1",
            Some("e:/repo/sealantern/target/debug/sea-lantern.exe"),
        ));
    }

    #[test]
    fn helper_identity_match_accepts_same_executable_for_server_helper() {
        let cmd = vec![
            OsString::from("sea-lantern.exe"),
            OsString::from("__local-runtime-helper"),
            OsString::from("server-1"),
        ];

        assert!(process_matches_server_helper_identity(
            &cmd,
            Some("e:/repo/sealantern/target/debug/sea-lantern.exe".to_string()),
            "server-1",
            Some("e:/repo/sealantern/target/debug/sea-lantern.exe"),
        ));
    }

    #[test]
    fn read_state_checked_surfaces_invalid_state_file_json() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        let path = state_file_path(&server);

        std::fs::write(&path, "{").expect("broken state file should be written");

        let error = read_state_checked(&server)
            .expect_err("invalid local helper state should not be treated as absent");

        assert!(error.contains(&runtime_t("server.runtime.local_helper.state_parse_failed_prefix")));
        assert!(error.contains(path.to_string_lossy().as_ref()));
    }

    #[test]
    fn status_snapshot_surfaces_invalid_state_file_json() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());

        std::fs::write(state_file_path(&server), "{").expect("broken state file should be written");

        let error = status_snapshot(&server)
            .expect_err("invalid local helper state should abort status snapshot");

        assert!(error.contains(&runtime_t("server.runtime.local_helper.state_parse_failed_prefix")));
    }

    #[test]
    fn send_command_surfaces_invalid_state_file_json() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());

        std::fs::write(state_file_path(&server), "{").expect("broken state file should be written");

        let error = send_command(&server, "say hi")
            .expect_err("invalid local helper state should abort send command");

        assert!(error.contains(&runtime_t("server.runtime.local_helper.state_parse_failed_prefix")));
    }

    #[test]
    fn send_terminal_input_surfaces_invalid_state_file_json() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());

        std::fs::write(state_file_path(&server), "{").expect("broken state file should be written");

        let error = send_terminal_input(&server, "hi")
            .expect_err("invalid local helper state should abort terminal input");

        assert!(error.contains(&runtime_t("server.runtime.local_helper.state_parse_failed_prefix")));
    }

    #[test]
    fn resize_terminal_surfaces_invalid_state_file_json() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());

        std::fs::write(state_file_path(&server), "{").expect("broken state file should be written");

        let error = resize_terminal(&server, 80, 24)
            .expect_err("invalid local helper state should abort terminal resize");

        assert!(error.contains(&runtime_t("server.runtime.local_helper.state_parse_failed_prefix")));
    }

    #[test]
    fn effective_terminal_mode_prefers_fallback_target_when_present() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());

        write_state_file(
            &state_file_path(&server),
            &super::LocalRuntimeState {
                server_id: server.id.clone(),
                helper_pid: 1,
                child_pid: Some(2),
                control_port: Some(25570),
                auth_token: "token".to_string(),
                running: true,
                exit_code: None,
                detail_message: "runtime=local running=true source=helper".to_string(),
                error_message: None,
                fallback: Some(super::LocalRuntimeFallbackInfo {
                    from_mode: "pty_managed".to_string(),
                    to_mode: "pipe_managed".to_string(),
                    reason: "PTY init failed".to_string(),
                }),
                terminal_mode: Some(crate::models::server::LocalTerminalMode::PipeManaged),
                terminal_cols: None,
                terminal_rows: None,
                updated_at: 123,
            },
        )
        .expect("state file should be written");

        let actual_mode = super::effective_terminal_mode(&server)
            .expect("effective terminal mode should resolve");

        assert_eq!(actual_mode, crate::models::server::LocalTerminalMode::PipeManaged);
    }
}
