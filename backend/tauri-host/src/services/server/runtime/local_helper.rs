mod dispatch;
mod protocol;
mod snapshot;
mod state;
mod status;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sl_server_info::log::LogStream;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalRuntimeState {
    pub server_id: String,
    pub helper_pid: u32,
    pub child_pid: Option<u32>,
    pub running: bool,
    pub exit_code: Option<i32>,
    pub detail_message: String,
    pub error_message: Option<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalHelperControlState {
    pub server_id: String,
    pub helper_pid: u32,
    pub control_port: u16,
    pub auth_token: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyLocalRuntimeState {
    server_id: String,
    helper_pid: u32,
    child_pid: Option<u32>,
    control_port: Option<u16>,
    auth_token: Option<String>,
    running: bool,
    exit_code: Option<i32>,
    detail_message: String,
    error_message: Option<String>,
    updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalHelperStatusSnapshot {
    pub running: bool,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub detail_message: String,
    pub error_message: Option<String>,
}

pub(crate) use status::{
    helper_runtime_status, runtime_snapshot_from_helper, stopped_runtime_snapshot,
};

use self::dispatch::handle_connection;
use self::protocol::{send_request, LocalHelperRequest};
use self::snapshot::detect_terminal_snapshot;
use self::state::{
    current_timestamp_secs, helper_ready_state, persist_terminal_state,
    remove_control_state_file, remove_state_file,
    start_failed_state, started_state, state_from_requested_stop, state_from_terminal_snapshot,
    write_state_file,
};
use self::status::{fallback_snapshot_from_state_file, fallback_snapshot_from_unreachable_helper};
use crate::models::server::ServerInstance;
use crate::services::global;
use crate::services::server::log_pipeline as server_log_pipeline;
use crate::services::server::manager::process::{force_kill_process_tree_by_pid, is_process_alive};
use crate::services::server::runtime::i18n::{runtime_t, runtime_t1, runtime_t2};
use crate::services::server::runtime::{RuntimeProcessHandle, RuntimeStartRequest};
use crate::utils::logger;
use std::ffi::OsString;
use std::net::TcpListener;
use std::process::Command;
use std::time::Duration;
use sysinfo::{Pid, ProcessesToUpdate, System};

#[cfg(test)]
static TEST_ENV_LOCK: once_cell::sync::Lazy<std::sync::Mutex<()>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(()));

pub fn state_file_path(server: &ServerInstance) -> PathBuf {
    Path::new(&server.path).join(crate::utils::constants::LOCAL_RUNTIME_STATE_FILE)
}

pub fn control_state_file_path(server: &ServerInstance) -> PathBuf {
    let encoded_server_id = URL_SAFE_NO_PAD.encode(server.id.as_bytes());
    crate::utils::path::get_app_data_dir()
        .join("runtime")
        .join("local_helper_control")
        .join(format!("{}.json", encoded_server_id))
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

    let legacy = serde_json::from_str::<LegacyLocalRuntimeState>(&content).map_err(|error| {
        runtime_t2(
            "server.runtime.local_helper.state_parse_failed",
            path.display().to_string(),
            error.to_string(),
        )
    })?;

    let had_legacy_control_fields = legacy.control_port.is_some() || legacy.auth_token.is_some();
    let state = LocalRuntimeState {
        server_id: legacy.server_id,
        helper_pid: legacy.helper_pid,
        child_pid: legacy.child_pid,
        running: legacy.running,
        exit_code: legacy.exit_code,
        detail_message: legacy.detail_message,
        error_message: legacy.error_message,
        updated_at: legacy.updated_at,
    };

    if had_legacy_control_fields {
        if let Err(error) = write_state_file(&path, &state) {
            logger::log_warn_ctx(
                "server.runtime.local_helper",
                "read_state_checked",
                &format!(
                    "failed to sanitize legacy control fields from state file: server_id={} path={} error={}",
                    server.id,
                    path.display(),
                    error
                ),
            );
        }
    }

    Ok(Some(state))
}

pub fn read_control_state_checked(
    server: &ServerInstance,
) -> Result<Option<LocalHelperControlState>, String> {
    let path = control_state_file_path(server);
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

    let state = serde_json::from_str::<LocalHelperControlState>(&content).map_err(|error| {
        runtime_t2(
            "server.runtime.local_helper.state_parse_failed",
            path.display().to_string(),
            error.to_string(),
        )
    })?;

    Ok(Some(state))
}

fn read_live_control_state_checked(
    server: &ServerInstance,
    runtime_state: &LocalRuntimeState,
) -> Result<Option<LocalHelperControlState>, String> {
    if !runtime_state.running {
        remove_control_state_file(server);
        return Ok(None);
    }

    let Some(control_state) = read_control_state_checked(server)? else {
        return Ok(None);
    };

    if control_state.server_id != runtime_state.server_id
        || control_state.helper_pid != runtime_state.helper_pid
    {
        logger::log_warn_ctx(
            "server.runtime.local_helper",
            "read_live_control_state_checked",
            &format!(
                "discarding stale helper control file: server_id={} runtime_helper_pid={} control_helper_pid={} path={}",
                server.id,
                runtime_state.helper_pid,
                control_state.helper_pid,
                control_state_file_path(server).display()
            ),
        );
        remove_control_state_file(server);
        return Ok(None);
    }

    Ok(Some(control_state))
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
    remove_control_state_file(server);
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
        let Some(control_state) = read_live_control_state_checked(server, &state)? else {
            let snapshot = fallback_snapshot_from_unreachable_helper(&state);
            reconcile_terminal_state_after_fallback(server, &state, &snapshot);
            return Ok(Some(snapshot));
        };

        match send_request(
            &control_state,
            LocalHelperRequest::Status { auth_token: control_state.auth_token.clone() },
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

                let snapshot = fallback_snapshot_from_unreachable_helper(&state);
                reconcile_terminal_state_after_fallback(server, &state, &snapshot);
                return Ok(Some(snapshot));
            }
        }
    }

    let snapshot = fallback_snapshot_from_state_file(&state);
    reconcile_terminal_state_after_fallback(server, &state, &snapshot);
    Ok(Some(snapshot))
}

fn reconcile_terminal_state_after_fallback(
    server: &ServerInstance,
    state: &LocalRuntimeState,
    snapshot: &LocalHelperStatusSnapshot,
) {
    if !state.running || snapshot.running {
        return;
    }

    if let Err(error) = persist_terminal_state(
        server,
        state,
        snapshot.exit_code,
        snapshot.error_message.as_ref().cloned(),
    ) {
        logger::log_warn_ctx(
            "server.runtime.local_helper",
            "reconcile_terminal_state_after_fallback",
            &format!(
                "failed to persist terminal fallback state: server_id={} helper_pid={} child_pid={} error={}",
                server.id,
                state.helper_pid,
                state
                    .child_pid
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                error
            ),
        );
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

    let control_state = read_live_control_state_checked(server, &state)?
        .ok_or_else(|| runtime_t("server.runtime.local_helper.send_unavailable"))?;

    let response = send_request(
        &control_state,
        LocalHelperRequest::Send {
            auth_token: control_state.auth_token.clone(),
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

pub fn request_stop(server: &ServerInstance) -> Result<(), String> {
    let state = read_state_checked(server)?.ok_or_else(|| {
        runtime_t1(
            "server.runtime.local_helper.state_missing",
            state_file_path(server).display().to_string(),
        )
    })?;

    if helper_process_matches_server(server, state.helper_pid) {
        let control_state = read_live_control_state_checked(server, &state)?
            .ok_or_else(|| runtime_t("server.runtime.local_helper.send_unavailable"))?;
        let response = match send_request(
            &control_state,
            LocalHelperRequest::Stop { auth_token: control_state.auth_token.clone() },
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
        let control_state = read_live_control_state_checked(server, &state)?
            .ok_or_else(|| runtime_t("server.runtime.local_helper.send_unavailable"))?;
        let response = match send_request(
            &control_state,
            LocalHelperRequest::ForceStop { auth_token: control_state.auth_token.clone() },
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
    let control_state = LocalHelperControlState {
        server_id: server_id.to_string(),
        helper_pid,
        control_port,
        auth_token: auth_token.clone(),
        updated_at: current_timestamp_secs(),
    };
    state::write_control_state_file(&server, &control_state)?;
    let mut state = helper_ready_state(server_id, helper_pid, control_port, current_timestamp_secs());
    write_state_file(&state_path, &state)?;

    let start_result =
        match manager.start_local_runtime(RuntimeStartRequest { server_id, server: &server }) {
            Ok(result) => result,
            Err(error) => {
                state = start_failed_state(&state, error.clone(), current_timestamp_secs());
                write_state_file(&state_path, &state)?;
                remove_control_state_file(&server);
                return Err(error);
            }
        };
    let Some(mut child) = start_result
        .process_handle
        .and_then(RuntimeProcessHandle::into_local_child)
    else {
        return Err(runtime_t("server.runtime.local_helper.child_handle_missing"));
    };

    let child_pid = Some(child.id());
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    manager
        .lock_processes()?
        .insert(server_id.to_string(), child);

    if let Some(stdout) = stdout {
        server_log_pipeline::spawn_server_output_reader(
            server_id.to_string(),
            LogStream::Stdout,
            stdout,
        );
    }
    if let Some(stderr) = stderr {
        server_log_pipeline::spawn_server_output_reader(
            server_id.to_string(),
            LogStream::Stderr,
            stderr,
        );
    }

    state = started_state(
        server_id,
        helper_pid,
        child_pid,
        control_port,
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
            remove_control_state_file(&server);
            break;
        }

        match listener.accept() {
            Ok((stream, _)) => {
                let should_exit =
                    handle_connection(manager, &server, &control_state, &state, stream)?;
                if should_exit {
                    state = state_from_requested_stop(&state, current_timestamp_secs());
                    write_state_file(&state_path, &state)?;
                    remove_control_state_file(&server);
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
    use super::state::{remove_control_state_file, write_control_state_file};
    use super::state::write_state_file;
    use super::status_snapshot;
    use super::{
        control_state_file_path, looks_like_local_runtime_helper_command,
        process_matches_server_helper_identity, read_control_state_checked, read_state_checked,
        send_command, state_file_path, LocalHelperControlState,
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
                running: true,
                exit_code: None,
                detail_message:
                    "runtime=local running=true source=helper child_pid=none control_port=25570"
                        .to_string(),
                error_message: None,
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

        let reconciled = read_state_checked(&server)
            .expect("reconciled state should read")
            .expect("reconciled state should exist");
        assert!(!reconciled.running);
        assert_eq!(reconciled.child_pid, None);
        assert_eq!(
            reconciled.detail_message,
            "runtime=local running=false source=state_file exit_code=none"
        );
        assert_eq!(reconciled.error_message, None);
    }

    #[test]
    fn status_snapshot_reconciles_dead_child_fallback_to_terminal_state() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        let path = state_file_path(&server);

        write_state_file(
            &path,
            &super::LocalRuntimeState {
                server_id: server.id.clone(),
                helper_pid: 999_999,
                child_pid: Some(u32::MAX),
                running: true,
                exit_code: None,
                detail_message:
                    "runtime=local running=true source=helper child_pid=4294967295 control_port=25570"
                        .to_string(),
                error_message: None,
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

        let reconciled = read_state_checked(&server)
            .expect("reconciled state should read")
            .expect("reconciled state should exist");
        assert!(!reconciled.running);
        assert_eq!(reconciled.child_pid, Some(u32::MAX));
        assert_eq!(
            reconciled.detail_message,
            "runtime=local running=false source=state_file exit_code=none"
        );
        assert_eq!(reconciled.error_message, None);
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
    fn read_state_checked_sanitizes_legacy_control_fields_from_state_file() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        let path = state_file_path(&server);

        std::fs::write(
            &path,
            r#"{
  "server_id": "local-helper",
  "helper_pid": 42,
  "child_pid": 24,
  "control_port": 25570,
  "auth_token": "secret",
  "running": true,
  "exit_code": null,
  "detail_message": "runtime=local running=true source=helper",
  "error_message": null,
  "updated_at": 123
}"#,
        )
        .expect("legacy state file should be written");

        let state = read_state_checked(&server)
            .expect("legacy state should read")
            .expect("legacy state should exist");

        assert_eq!(state.server_id, server.id);
        assert_eq!(state.helper_pid, 42);
        assert_eq!(state.child_pid, Some(24));
        assert!(state.running);

        let sanitized = std::fs::read_to_string(&path).expect("sanitized state should be readable");
        assert!(!sanitized.contains("control_port"));
        assert!(!sanitized.contains("auth_token"));
    }

    #[test]
    fn read_live_control_state_rejects_stale_helper_pid_and_deletes_file() {
        let _guard = super::TEST_ENV_LOCK.lock().expect("env lock should work");
        let temp_dir = tempdir().expect("temp dir should exist");
        let app_data_dir = temp_dir.path().join("app-data");
        std::fs::create_dir_all(&app_data_dir).expect("app data dir should exist");
        unsafe {
            std::env::set_var("SEALANTERN_DATA_DIR", &app_data_dir);
        }
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        let runtime_state = super::LocalRuntimeState {
            server_id: server.id.clone(),
            helper_pid: 100,
            child_pid: None,
            running: true,
            exit_code: None,
            detail_message: "runtime=local running=true source=helper".to_string(),
            error_message: None,
            updated_at: 123,
        };
        let control_path = control_state_file_path(&server);
        write_control_state_file(
            &server,
            &LocalHelperControlState {
                server_id: server.id.clone(),
                helper_pid: 101,
                control_port: 25570,
                auth_token: "secret".to_string(),
                updated_at: 123,
            },
        )
        .expect("control state should be written");

        let control = super::read_live_control_state_checked(&server, &runtime_state)
            .expect("stale control state should not error");

        assert_eq!(control, None);
        assert!(!control_path.exists());

        unsafe {
            std::env::remove_var("SEALANTERN_DATA_DIR");
        }
    }

    #[test]
    fn read_live_control_state_drops_terminal_runtime_control_file() {
        let _guard = super::TEST_ENV_LOCK.lock().expect("env lock should work");
        let temp_dir = tempdir().expect("temp dir should exist");
        let app_data_dir = temp_dir.path().join("app-data");
        std::fs::create_dir_all(&app_data_dir).expect("app data dir should exist");
        unsafe {
            std::env::set_var("SEALANTERN_DATA_DIR", &app_data_dir);
        }
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        let runtime_state = super::LocalRuntimeState {
            server_id: server.id.clone(),
            helper_pid: 100,
            child_pid: None,
            running: false,
            exit_code: Some(0),
            detail_message: "runtime=local running=false source=state_file exit_code=0".to_string(),
            error_message: None,
            updated_at: 123,
        };
        let control_path = control_state_file_path(&server);
        write_control_state_file(
            &server,
            &LocalHelperControlState {
                server_id: server.id.clone(),
                helper_pid: 100,
                control_port: 25570,
                auth_token: "secret".to_string(),
                updated_at: 123,
            },
        )
        .expect("control state should be written");

        let control = super::read_live_control_state_checked(&server, &runtime_state)
            .expect("terminal runtime control cleanup should not error");

        assert_eq!(control, None);
        assert!(!control_path.exists());

        unsafe {
            std::env::remove_var("SEALANTERN_DATA_DIR");
        }
    }

    #[test]
    fn control_state_round_trips_in_private_app_data_file() {
        let _guard = super::TEST_ENV_LOCK.lock().expect("env lock should work");
        let temp_dir = tempdir().expect("temp dir should exist");
        let app_data_dir = temp_dir.path().join("app-data");
        std::fs::create_dir_all(&app_data_dir).expect("app data dir should exist");
        unsafe {
            std::env::set_var("SEALANTERN_DATA_DIR", &app_data_dir);
        }

        let server = test_server(temp_dir.path().join("server").to_string_lossy().to_string());
        let expected = LocalHelperControlState {
            server_id: server.id.clone(),
            helper_pid: 42,
            control_port: 25570,
            auth_token: "secret".to_string(),
            updated_at: 123,
        };

        write_control_state_file(&server, &expected).expect("control state should be written");
        let actual = read_control_state_checked(&server)
            .expect("control state should read")
            .expect("control state should exist");

        assert_eq!(actual, expected);
        assert!(control_state_file_path(&server).starts_with(&app_data_dir));

        remove_control_state_file(&server);
        unsafe {
            std::env::remove_var("SEALANTERN_DATA_DIR");
        }
    }
}
