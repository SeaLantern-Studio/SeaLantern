mod dispatch;
mod protocol;
mod snapshot;
mod state;
mod status;

pub use state::{read_state, state_file_path, LocalHelperStatusSnapshot, LocalRuntimeState};
pub(crate) use status::{
    helper_runtime_status, runtime_snapshot_from_helper, stopped_runtime_snapshot,
};

use self::dispatch::handle_connection;
use self::protocol::{send_request, LocalHelperRequest};
use self::snapshot::detect_terminal_snapshot;
use self::state::{
    current_timestamp_secs, persist_terminal_state, remove_state_file, started_state,
    state_from_requested_stop, state_from_terminal_snapshot, write_state_file,
};
use self::status::fallback_snapshot_from_state_file;
use crate::models::server::ServerInstance;
use crate::services::global;
use crate::services::server::log_pipeline as server_log_pipeline;
use crate::services::server::manager::process::{force_kill_process_tree_by_pid, is_process_alive};
use crate::services::server::runtime::{RuntimeProcessHandle, RuntimeStartRequest};
use crate::utils::logger;
use std::ffi::OsString;
use std::net::TcpListener;
use std::process::Command;
use std::time::Duration;
use sysinfo::{Pid, ProcessesToUpdate, System};

pub fn cleanup_for_new_start(server: &ServerInstance) {
    let current_exe = current_exe_lowercase();

    if let Some(state) = read_state(server) {
        let helper_alive = helper_process_matches_server_pid(
            state.helper_pid,
            &server.id,
            current_exe.as_deref(),
        );
        let child_alive = state.child_pid.is_some_and(is_process_alive);
        if helper_alive || child_alive {
            logger::log_warn(&format!(
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
        logger::log_warn(&format!(
            "local runtime start cleanup skipped stale helper pid because identity no longer matches: server_id={} helper_pid={}",
            server.id, state.helper_pid
        ));
    }

    if let Some(child_pid) = state.child_pid.filter(|pid| is_process_alive(*pid)) {
        logger::log_warn(&format!(
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
            process.exe().map(|path| path.to_string_lossy().to_ascii_lowercase()),
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

fn helper_process_matches_server_pid(
    pid: u32,
    server_id: &str,
    current_exe: Option<&str>,
) -> bool {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    let Some(process) = system.process(Pid::from_u32(pid)) else {
        return false;
    };

    process_matches_server_helper_identity(
        process.cmd(),
        process.exe().map(|path| path.to_string_lossy().to_ascii_lowercase()),
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
    let current_exe =
        std::env::current_exe().map_err(|e| format!("获取当前可执行文件路径失败: {}", e))?;
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
        .map_err(|e| format!("启动本地 runtime helper 失败: {}", e))
}

pub fn wait_for_helper_ready(
    server: &ServerInstance,
    timeout: Duration,
) -> Result<LocalRuntimeState, String> {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if let Some(state) = read_state(server) {
            if state.control_port.is_some() || !state.running {
                return Ok(state);
            }
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    Err(format!(
        "等待本地 runtime helper 就绪超时: server_id={} state_file={}",
        server.id,
        state_file_path(server).display()
    ))
}

pub fn status_snapshot(
    server: &ServerInstance,
) -> Result<Option<LocalHelperStatusSnapshot>, String> {
    let Some(state) = read_state(server) else {
        return Ok(None);
    };

    if is_process_alive(state.helper_pid) {
        match send_request(
            &state,
            LocalHelperRequest::Status { auth_token: state.auth_token.clone() },
        ) {
            Ok(response) => return Ok(response.snapshot),
            Err(error) => {
                logger::log_warn(&format!(
                    "local runtime helper status probe failed, falling back to state file: server_id={} helper_pid={} error={}",
                    server.id, state.helper_pid, error
                ));
            }
        }
    }

    Ok(Some(fallback_snapshot_from_state_file(&state)))
}

pub fn send_command(server: &ServerInstance, command: &str) -> Result<(), String> {
    let state = read_state(server).ok_or_else(|| {
        format!("本地 runtime helper 状态不存在: {}", state_file_path(server).display())
    })?;

    if !is_process_alive(state.helper_pid) {
        return Err("本地 runtime helper 不可用，当前无法跨命令发送本地控制台命令；请改用 force-stop 后重启，或在同一 CLI/Web 会话内操作".to_string());
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

pub fn request_stop(server: &ServerInstance) -> Result<(), String> {
    let state = read_state(server).ok_or_else(|| {
        format!("本地 runtime helper 状态不存在: {}", state_file_path(server).display())
    })?;

    if is_process_alive(state.helper_pid) {
        let response = send_request(
            &state,
            LocalHelperRequest::Stop { auth_token: state.auth_token.clone() },
        )?;
        if response.ok {
            return Ok(());
        }
        return Err(response
            .error
            .unwrap_or_else(|| "helper stop failed".to_string()));
    }

    if let Some(pid) = state.child_pid.filter(|pid| is_process_alive(*pid)) {
        force_kill_process_tree_by_pid(pid)?;
        persist_terminal_state(
            server,
            &state,
            None,
            Some("本地 runtime helper 不可用，已回退为按 PID 强制终止".to_string()),
        )?;
        return Ok(());
    }

    Ok(())
}

pub fn force_stop(server: &ServerInstance) -> Result<(), String> {
    let state = read_state(server).ok_or_else(|| {
        format!("本地 runtime helper 状态不存在: {}", state_file_path(server).display())
    })?;

    if is_process_alive(state.helper_pid) {
        let response = send_request(
            &state,
            LocalHelperRequest::ForceStop { auth_token: state.auth_token.clone() },
        )?;
        if response.ok {
            return Ok(());
        }
        return Err(response
            .error
            .unwrap_or_else(|| "helper force stop failed".to_string()));
    }

    if let Some(pid) = state.child_pid.filter(|pid| is_process_alive(*pid)) {
        force_kill_process_tree_by_pid(pid)?;
        persist_terminal_state(
            server,
            &state,
            None,
            Some("本地 runtime helper 不可用，已回退为按 PID 强制终止".to_string()),
        )?;
    }

    Ok(())
}

pub fn handle_helper_command(args: &[String]) -> i32 {
    let Some(server_id) = args.first().map(String::as_str) else {
        eprintln!("local runtime helper 缺少 server_id");
        return 2;
    };

    match run_helper(server_id) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("local runtime helper 失败: {}", err);
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
        .map_err(|e| format!("绑定本地 runtime helper 控制端口失败: {}", e))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("设置本地 runtime helper 非阻塞失败: {}", e))?;
    let control_port = listener
        .local_addr()
        .map_err(|e| format!("读取本地 runtime helper 端口失败: {}", e))?
        .port();

    let start_result =
        manager.start_local_runtime(RuntimeStartRequest { server_id, server: &server })?;
    let Some(mut child) = start_result
        .process_handle
        .and_then(RuntimeProcessHandle::into_local_child)
    else {
        return Err("本地 runtime helper 未收到有效的 Java 子进程句柄".to_string());
    };

    let child_pid = Some(child.id());
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    manager
        .lock_processes()?
        .insert(server_id.to_string(), child);

    if let Some(stdout) = stdout {
        server_log_pipeline::spawn_server_output_reader(server_id.to_string(), stdout);
    }
    if let Some(stderr) = stderr {
        server_log_pipeline::spawn_server_output_reader(server_id.to_string(), stderr);
    }

    let mut state = started_state(
        server_id,
        helper_pid,
        child_pid,
        control_port,
        auth_token,
        current_timestamp_secs(),
    );

    let state_path = state_file_path(&server);
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
                logger::log_warn(&format!(
                    "local runtime helper accept failed: server_id={} error={}",
                    server_id, err
                ));
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::state::{state_file_path, write_state_file};
    use super::status_snapshot;
    use super::{looks_like_local_runtime_helper_command, process_matches_server_helper_identity};
    use crate::models::server::{LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig};
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
                control_port: Some(25570),
                auth_token: "token".to_string(),
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
}
