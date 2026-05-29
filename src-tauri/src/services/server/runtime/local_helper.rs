use crate::models::server::ServerInstance;
use crate::services::global;
use crate::services::server::log_pipeline as server_log_pipeline;
use crate::services::server::manager::process::{force_kill_process_tree_by_pid, is_process_alive};
use crate::services::server::runtime::ServerRuntime;
use crate::services::server::runtime::{RuntimeProcessHandle, RuntimeStartRequest};
use crate::utils::constants::LOCAL_RUNTIME_STATE_FILE;
use crate::utils::logger;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::{ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalHelperStatusSnapshot {
    pub running: bool,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub detail_message: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
enum LocalHelperRequest {
    Status { auth_token: String },
    Send { auth_token: String, command: String },
    Stop { auth_token: String },
    ForceStop { auth_token: String },
}

#[derive(Debug, Serialize, Deserialize)]
struct LocalHelperResponse {
    ok: bool,
    snapshot: Option<LocalHelperStatusSnapshot>,
    error: Option<String>,
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

pub fn cleanup_for_new_start(server: &ServerInstance) {
    if let Some(state) = read_state(server) {
        let helper_alive = is_process_alive(state.helper_pid);
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

        cleanup_runtime_processes_from_state(server, &state);
    }

    cleanup_orphan_helper_processes(server);

    remove_state_file(server);
}

fn cleanup_runtime_processes_from_state(server: &ServerInstance, state: &LocalRuntimeState) {
    if let Some(child_pid) = state.child_pid.filter(|pid| is_process_alive(*pid)) {
        match force_kill_process_tree_by_pid(child_pid) {
            Ok(()) => logger::log_user_action(
                "server.runtime.local.helper",
                "cleanup_lingering_child",
                &format!("server_id={} child_pid={}", server.id, child_pid),
            ),
            Err(err) => logger::log_user_action_error(
                "server.runtime.local.helper",
                "cleanup_lingering_child",
                &format!("server_id={} child_pid={}", server.id, child_pid),
                &err,
            ),
        }
    }

    if is_process_alive(state.helper_pid) {
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
    }
}

fn cleanup_orphan_helper_processes(server: &ServerInstance) {
    let current_exe = std::env::current_exe()
        .ok()
        .map(|path| path.to_string_lossy().to_ascii_lowercase());
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    for process in system.processes().values() {
        let cmd = process.cmd();
        if !looks_like_local_runtime_helper_command(cmd, &server.id) {
            continue;
        }

        if let Some(current_exe) = current_exe.as_deref() {
            let process_exe = process
                .exe()
                .map(|path| path.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default();
            if !process_exe.is_empty() && process_exe != current_exe {
                continue;
            }
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
        let response = send_request(
            &state,
            LocalHelperRequest::Status { auth_token: state.auth_token.clone() },
        )?;
        return Ok(response.snapshot);
    }

    let child_running = if state.running {
        state.child_pid.filter(|pid| is_process_alive(*pid))
    } else {
        None
    };
    let detail_message = if child_running.is_some() {
        format!(
            "runtime=local running=true source=state_file helper=unavailable exit_code={}",
            state
                .exit_code
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        )
    } else if state.running {
        format!(
            "runtime=local running=false source=state_file helper=exited exit_code={}",
            state
                .exit_code
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        )
    } else {
        format!(
            "runtime=local running=false source=state_file exit_code={}",
            state
                .exit_code
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        )
    };
    let error_message = if child_running.is_some() {
        Some(
            "本地 runtime helper 已退出，但 Java 进程仍在运行；当前无法继续发送命令，建议执行 force-stop 后重新启动"
                .to_string(),
        )
    } else {
        state.error_message.clone()
    };

    Ok(Some(LocalHelperStatusSnapshot {
        running: child_running.is_some(),
        pid: child_running,
        exit_code: state.exit_code,
        detail_message,
        error_message,
    }))
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

    let mut state = LocalRuntimeState {
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
        updated_at: current_timestamp_secs(),
    };

    write_state_file(state_file_path(&server), &state)?;
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
            state.running = false;
            state.control_port = None;
            state.exit_code = snapshot.exit_code;
            state.detail_message = snapshot.detail_message;
            state.error_message = snapshot.error_message;
            state.updated_at = current_timestamp_secs();
            write_state_file(state_file_path(&server), &state)?;
            break;
        }

        match listener.accept() {
            Ok((stream, _)) => {
                let should_exit = handle_connection(manager, &server, &state, stream)?;
                if should_exit {
                    state.running = false;
                    state.child_pid = None;
                    state.control_port = None;
                    state.detail_message = if state.exit_code == Some(0) {
                        "runtime=local running=false source=helper exit_code=0".to_string()
                    } else {
                        "runtime=local running=false source=helper requested_stop".to_string()
                    };
                    state.error_message = None;
                    state.updated_at = current_timestamp_secs();
                    write_state_file(state_file_path(&server), &state)?;
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

fn handle_connection(
    manager: &crate::services::server::manager::ServerManager,
    server: &ServerInstance,
    state: &LocalRuntimeState,
    mut stream: TcpStream,
) -> Result<bool, String> {
    let request: LocalHelperRequest = {
        let mut reader = BufReader::new(
            stream
                .try_clone()
                .map_err(|e| format!("复制 helper 连接失败: {}", e))?,
        );
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("读取 helper 请求失败: {}", e))?;
        serde_json::from_str(&line).map_err(|e| format!("解析 helper 请求失败: {}", e))?
    };

    let auth_token = match &request {
        LocalHelperRequest::Status { auth_token }
        | LocalHelperRequest::Send { auth_token, .. }
        | LocalHelperRequest::Stop { auth_token }
        | LocalHelperRequest::ForceStop { auth_token } => auth_token,
    };
    if auth_token != &state.auth_token {
        write_response(
            &mut stream,
            LocalHelperResponse {
                ok: false,
                snapshot: None,
                error: Some("本地 runtime helper 鉴权失败".to_string()),
            },
        )?;
        return Ok(false);
    }

    match request {
        LocalHelperRequest::Status { .. } => {
            let snapshot = snapshot_from_manager(manager, server.id.as_str())?;
            write_response(
                &mut stream,
                LocalHelperResponse {
                    ok: true,
                    snapshot: Some(snapshot),
                    error: None,
                },
            )?;
            Ok(false)
        }
        LocalHelperRequest::Send { command, .. } => {
            let response = match manager.send_command(&server.id, &command) {
                Ok(()) => LocalHelperResponse { ok: true, snapshot: None, error: None },
                Err(err) => LocalHelperResponse {
                    ok: false,
                    snapshot: None,
                    error: Some(err),
                },
            };
            write_response(&mut stream, response)?;
            Ok(false)
        }
        LocalHelperRequest::Stop { .. } => {
            let response = match manager.stop_server(&server.id) {
                Ok(()) => LocalHelperResponse { ok: true, snapshot: None, error: None },
                Err(err) => LocalHelperResponse {
                    ok: false,
                    snapshot: None,
                    error: Some(err),
                },
            };
            let should_exit = response.ok;
            write_response(&mut stream, response)?;
            Ok(should_exit)
        }
        LocalHelperRequest::ForceStop { .. } => {
            let response = match crate::services::server::runtime::local::LocalServerRuntime
                .force_stop_with_manager(manager, server)
            {
                Ok(()) => LocalHelperResponse { ok: true, snapshot: None, error: None },
                Err(err) => LocalHelperResponse {
                    ok: false,
                    snapshot: None,
                    error: Some(err),
                },
            };
            let should_exit = response.ok;
            write_response(&mut stream, response)?;
            Ok(should_exit)
        }
    }
}

fn write_response(stream: &mut TcpStream, response: LocalHelperResponse) -> Result<(), String> {
    let payload =
        serde_json::to_string(&response).map_err(|e| format!("序列化 helper 响应失败: {}", e))?;
    writeln!(stream, "{}", payload).map_err(|e| format!("写入 helper 响应失败: {}", e))
}

fn send_request(
    state: &LocalRuntimeState,
    request: LocalHelperRequest,
) -> Result<LocalHelperResponse, String> {
    let control_port = state
        .control_port
        .ok_or_else(|| "本地 runtime helper 当前未暴露控制端口".to_string())?;
    let mut stream = TcpStream::connect(("127.0.0.1", control_port)).map_err(|e| {
        format!(
            "连接本地 runtime helper 失败: helper_pid={} port={} error={}",
            state.helper_pid, control_port, e
        )
    })?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("设置 helper 读取超时失败: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("设置 helper 写入超时失败: {}", e))?;

    let payload =
        serde_json::to_string(&request).map_err(|e| format!("序列化 helper 请求失败: {}", e))?;
    writeln!(stream, "{}", payload).map_err(|e| format!("写入 helper 请求失败: {}", e))?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("读取 helper 响应失败: {}", e))?;

    serde_json::from_str(&line).map_err(|e| format!("解析 helper 响应失败: {}", e))
}

fn write_state_file(path: PathBuf, state: &LocalRuntimeState) -> Result<(), String> {
    let content = serde_json::to_string_pretty(state)
        .map_err(|e| format!("序列化本地 runtime 状态失败: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("写入本地 runtime 状态失败 ({}): {}", path.display(), e))
}

fn persist_terminal_state(
    server: &ServerInstance,
    state: &LocalRuntimeState,
    exit_code: Option<i32>,
    error_message: Option<String>,
) -> Result<(), String> {
    let next = LocalRuntimeState {
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
        updated_at: current_timestamp_secs(),
    };
    write_state_file(state_file_path(server), &next)
}

fn detect_terminal_snapshot(
    manager: &crate::services::server::manager::ServerManager,
    server: &ServerInstance,
) -> Result<Option<LocalHelperStatusSnapshot>, String> {
    let mut procs = manager.lock_processes()?;
    let Some(child) = procs.get_mut(&server.id) else {
        return Ok(Some(LocalHelperStatusSnapshot {
            running: false,
            pid: None,
            exit_code: None,
            detail_message: "runtime=local running=false source=helper process=missing".to_string(),
            error_message: None,
        }));
    };

    match child.try_wait() {
        Ok(Some(status)) => {
            let exit_code = status.code();
            procs.remove(&server.id);
            server_log_pipeline::shutdown_writer(&server.id);
            Ok(Some(LocalHelperStatusSnapshot {
                running: false,
                pid: None,
                exit_code,
                detail_message: format!(
                    "runtime=local running=false source=helper exit_code={}",
                    exit_code
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "none".to_string())
                ),
                error_message: if matches!(exit_code, Some(0)) {
                    None
                } else {
                    Some(format!(
                        "服务器异常退出 (退出码：{})",
                        exit_code
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    ))
                },
            }))
        }
        Ok(None) => Ok(None),
        Err(err) => {
            procs.remove(&server.id);
            server_log_pipeline::shutdown_writer(&server.id);
            Ok(Some(LocalHelperStatusSnapshot {
                running: false,
                pid: None,
                exit_code: None,
                detail_message: "runtime=local running=false source=helper status=error"
                    .to_string(),
                error_message: Some(format!("获取本地进程状态失败: {}", err)),
            }))
        }
    }
}

fn snapshot_from_manager(
    manager: &crate::services::server::manager::ServerManager,
    server_id: &str,
) -> Result<LocalHelperStatusSnapshot, String> {
    let mut procs = manager.lock_processes()?;
    let Some(child) = procs.get_mut(server_id) else {
        return Ok(LocalHelperStatusSnapshot {
            running: false,
            pid: None,
            exit_code: None,
            detail_message: "runtime=local running=false source=helper process=missing".to_string(),
            error_message: None,
        });
    };

    match child.try_wait() {
        Ok(Some(status)) => {
            let exit_code = status.code();
            procs.remove(server_id);
            server_log_pipeline::shutdown_writer(server_id);
            Ok(LocalHelperStatusSnapshot {
                running: false,
                pid: None,
                exit_code,
                detail_message: format!(
                    "runtime=local running=false source=helper exit_code={}",
                    exit_code
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "none".to_string())
                ),
                error_message: if matches!(exit_code, Some(0)) {
                    None
                } else {
                    Some(format!(
                        "服务器异常退出 (退出码：{})",
                        exit_code
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    ))
                },
            })
        }
        Ok(None) => Ok(LocalHelperStatusSnapshot {
            running: true,
            pid: Some(child.id()),
            exit_code: None,
            detail_message: format!("runtime=local running=true source=helper pid={}", child.id()),
            error_message: None,
        }),
        Err(err) => {
            procs.remove(server_id);
            server_log_pipeline::shutdown_writer(server_id);
            Ok(LocalHelperStatusSnapshot {
                running: false,
                pid: None,
                exit_code: None,
                detail_message: "runtime=local running=false source=helper status=error"
                    .to_string(),
                error_message: Some(format!("获取本地进程状态失败: {}", err)),
            })
        }
    }
}

fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
