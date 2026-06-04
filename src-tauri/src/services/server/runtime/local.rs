use super::{
    control, local_helper, RuntimeForceStopPreparation, RuntimeProcessHandle, RuntimeStartRequest,
    RuntimeStartResult, RuntimeStatusSnapshot, ServerRuntime,
};
use crate::models::server::{LocalRuntimeConfig, ServerInstance, ServerStatus};
use crate::services::server::log_pipeline as server_log_pipeline;
use crate::services::server::manager::ServerManager;
use std::io::Write;
use std::time::Duration;

pub struct LocalServerRuntime;

impl LocalServerRuntime {
    pub fn ensure_config(server: &ServerInstance) -> Result<&LocalRuntimeConfig, String> {
        server
            .local_runtime()
            .ok_or_else(|| format!("当前服务器运行时暂未实现: {}", server.runtime_kind))
    }

    fn is_running(manager: &ServerManager, server_id: &str) -> Result<bool, String> {
        let mut procs = manager.lock_processes()?;
        Ok(if let Some(child) = procs.get_mut(server_id) {
            match child.try_wait() {
                Ok(Some(_)) => {
                    procs.remove(server_id);
                    server_log_pipeline::shutdown_writer(server_id);
                    false
                }
                Ok(None) => true,
                Err(_) => {
                    procs.remove(server_id);
                    server_log_pipeline::shutdown_writer(server_id);
                    false
                }
            }
        } else {
            false
        })
    }

    fn send_command_to_tracked_child(
        manager: &ServerManager,
        server: &ServerInstance,
        command: &str,
    ) -> Result<bool, String> {
        let mut procs = manager.lock_processes()?;
        let Some(child) = procs.get_mut(&server.id) else {
            return Ok(false);
        };

        let stdin = child.stdin.as_mut().ok_or_else(|| {
            let message =
                format!("本地服务端 stdin 不可用，当前进程无法再接收控制台命令: {}", server.id);
            let _ = server_log_pipeline::append_sealantern_log(
                &server.id,
                &format!("[Sea Lantern] {}", message),
            );
            message
        })?;

        writeln!(stdin, "{}", command)
            .map_err(|e| format!("本地服务端 stdin 写入失败（id={}）: {}", server.id, e))?;
        stdin
            .flush()
            .map_err(|e| format!("本地服务端 stdin 刷新失败（id={}）: {}", server.id, e))?;
        Ok(true)
    }

    fn stop_tracked_process(
        &self,
        manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<bool, String> {
        let is_running = Self::is_running(manager, &server.id)?;
        if !is_running {
            return Ok(false);
        }

        let _ = server_log_pipeline::append_sealantern_log(
            &server.id,
            "[Sea Lantern] 正在发送停止命令...",
        );
        let _ = self.send_command_with_manager(manager, server, "stop");

        for _ in 0..20 {
            std::thread::sleep(Duration::from_millis(500));
            let mut procs = manager.lock_processes()?;
            if let Some(child) = procs.get_mut(&server.id) {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        procs.remove(&server.id);
                        control::clear_runtime_flags(manager, &server.id);
                        let _ = server_log_pipeline::append_sealantern_log(
                            &server.id,
                            "[Sea Lantern] 服务器已正常停止",
                        );
                        server_log_pipeline::shutdown_writer(&server.id);
                        return Ok(true);
                    }
                    Ok(None) => {}
                    Err(_) => {
                        procs.remove(&server.id);
                        control::clear_runtime_flags(manager, &server.id);
                        server_log_pipeline::shutdown_writer(&server.id);
                        return Ok(true);
                    }
                }
            } else {
                control::clear_runtime_flags(manager, &server.id);
                let _ = server_log_pipeline::append_sealantern_log(
                    &server.id,
                    "[Sea Lantern] 服务器已停止",
                );
                server_log_pipeline::shutdown_writer(&server.id);
                return Ok(true);
            }
        }

        let mut procs = manager.lock_processes()?;
        if let Some(mut child) = procs.remove(&server.id) {
            let _ = manager.force_kill_local_process(&mut child);
            let _ = server_log_pipeline::append_sealantern_log(
                &server.id,
                "[Sea Lantern] 服务器超时，已强制终止",
            );
        }
        server_log_pipeline::shutdown_writer(&server.id);
        control::clear_runtime_flags(manager, &server.id);
        Ok(true)
    }

    fn force_stop_tracked_process(
        &self,
        manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<bool, String> {
        let mut procs = manager.lock_processes()?;
        let Some(mut child) = procs.remove(&server.id) else {
            return Ok(false);
        };

        let kill_result = manager.force_kill_local_process(&mut child);
        drop(procs);

        control::clear_runtime_flags(manager, &server.id);

        match kill_result {
            Ok(_) => {
                let _ = server_log_pipeline::append_sealantern_log(
                    &server.id,
                    "[Sea Lantern] 已按用户确认强制终止服务器进程",
                );
                server_log_pipeline::shutdown_writer(&server.id);
                Ok(true)
            }
            Err(err) => {
                let message = format!("强制终止服务器失败: {}", err);
                let _ = server_log_pipeline::append_sealantern_log(
                    &server.id,
                    &format!("[Sea Lantern] {}", message),
                );
                server_log_pipeline::shutdown_writer(&server.id);
                Err(message)
            }
        }
    }

    fn tracked_status_snapshot(
        manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<Option<RuntimeStatusSnapshot>, String> {
        let mut exit_code: Option<i32> = None;
        let mut error_message: Option<String> = None;
        let mut terminated_abnormally = false;

        let has_tracked_process = manager
            .lock_processes()
            .ok()
            .map(|procs| procs.contains_key(&server.id))
            .unwrap_or(false);

        if !has_tracked_process {
            return Ok(None);
        }

        let is_running = manager
            .lock_processes()
            .ok()
            .map(|mut procs| {
                if let Some(child) = procs.get_mut(&server.id) {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            exit_code = status.code();
                            match &exit_code {
                                Some(0) => {
                                    let _ = server_log_pipeline::append_sealantern_log(
                                        &server.id,
                                        "[Sea Lantern] 服务器已正常退出",
                                    );
                                }
                                Some(code) => {
                                    terminated_abnormally = true;
                                    error_message =
                                        Some(format!("服务器异常退出 (退出码：{})", code));
                                    let _ = server_log_pipeline::append_sealantern_log(
                                        &server.id,
                                        &format!("[Sea Lantern] 服务器异常退出 (退出码：{})", code),
                                    );
                                }
                                None => {
                                    terminated_abnormally = true;
                                    error_message = Some("服务器被强制终止".to_string());
                                    let _ = server_log_pipeline::append_sealantern_log(
                                        &server.id,
                                        "[Sea Lantern] 服务器被强制终止",
                                    );
                                }
                            }

                            procs.remove(&server.id);
                            server_log_pipeline::shutdown_writer(&server.id);
                            control::clear_runtime_flags(manager, &server.id);
                            false
                        }
                        Ok(None) => true,
                        Err(_) => {
                            terminated_abnormally = true;
                            error_message = Some("获取服务器状态失败".to_string());
                            let _ = server_log_pipeline::append_sealantern_log(
                                &server.id,
                                "[Sea Lantern] 获取服务器状态失败",
                            );
                            procs.remove(&server.id);
                            server_log_pipeline::shutdown_writer(&server.id);
                            control::clear_runtime_flags(manager, &server.id);
                            false
                        }
                    }
                } else {
                    control::clear_runtime_flags(manager, &server.id);
                    false
                }
            })
            .unwrap_or(false);

        let pid = if is_running {
            manager
                .lock_processes()
                .ok()
                .and_then(|mut procs| procs.get_mut(&server.id).map(|child| child.id()))
        } else {
            None
        };

        let status = if manager.is_stopping(&server.id) {
            ServerStatus::Stopping
        } else if is_running && manager.is_starting(&server.id) {
            manager.clear_starting(&server.id);
            ServerStatus::Running
        } else if !is_running && terminated_abnormally {
            ServerStatus::Error
        } else if is_running {
            ServerStatus::Running
        } else {
            ServerStatus::Stopped
        };

        Ok(Some(RuntimeStatusSnapshot {
            status,
            pid,
            detail_message: Some(format!(
                "runtime=local is_running={} exit_code={} source=tracked_child",
                is_running,
                exit_code
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            )),
            error_message,
        }))
    }

    fn helper_status_snapshot(
        manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<Option<RuntimeStatusSnapshot>, String> {
        let Some(snapshot) = local_helper::status_snapshot(server)? else {
            return Ok(None);
        };

        let is_starting = manager.is_starting(&server.id);
        let status =
            local_helper::helper_runtime_status(&snapshot, manager.is_stopping(&server.id));

        if snapshot.running && is_starting {
            manager.clear_starting(&server.id);
        }

        if !snapshot.running {
            control::clear_runtime_flags(manager, &server.id);
        }

        Ok(Some(local_helper::runtime_snapshot_from_helper(snapshot, status)))
    }
}

impl ServerRuntime for LocalServerRuntime {
    fn start(&self, _request: RuntimeStartRequest<'_>) -> Result<RuntimeStartResult, String> {
        Err("local runtime adapter is not wired yet".to_string())
    }

    fn start_with_manager(
        &self,
        manager: &ServerManager,
        request: RuntimeStartRequest<'_>,
    ) -> Result<RuntimeStartResult, String> {
        let runtime = Self::ensure_config(request.server)?;
        control::log_runtime_action(
            "server.runtime.local",
            "start",
            format!(
                "server_id={} startup_mode={} entry={} java={}",
                request.server.id, runtime.startup_mode, runtime.jar_path, runtime.java_path
            ),
        );

        local_helper::cleanup_for_new_start(request.server);
        local_helper::spawn_helper_process(request.server)?;
        manager.mark_starting(&request.server.id);
        let ready_state =
            local_helper::wait_for_helper_ready(request.server, Duration::from_secs(5))?;

        if !ready_state.running {
            return Err(ready_state
                .error_message
                .unwrap_or(ready_state.detail_message));
        }

        Ok(RuntimeStartResult { process_handle: None, fallback: None })
    }

    fn send_command(&self, _server: &ServerInstance, _command: &str) -> Result<(), String> {
        Err("local runtime adapter is not wired yet".to_string())
    }

    fn send_command_with_manager(
        &self,
        manager: &ServerManager,
        server: &ServerInstance,
        command: &str,
    ) -> Result<(), String> {
        let runtime = Self::ensure_config(server)?;
        control::log_runtime_action(
            "server.runtime.local",
            "send_command",
            format!(
                "server_id={} startup_mode={} entry={} command={}",
                server.id,
                runtime.startup_mode,
                runtime.jar_path,
                command.trim()
            ),
        );

        if Self::send_command_to_tracked_child(manager, server, command)? {
            return Ok(());
        }

        local_helper::send_command(server, command)
    }

    fn request_stop(&self, _server: &ServerInstance) -> Result<(), String> {
        Err("local runtime adapter is not wired yet".to_string())
    }

    fn request_stop_with_manager(
        &self,
        manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<(), String> {
        let runtime = Self::ensure_config(server)?;
        control::log_runtime_action(
            "server.runtime.local",
            "request_stop",
            format!(
                "server_id={} startup_mode={} entry={}",
                server.id, runtime.startup_mode, runtime.jar_path
            ),
        );

        if manager.is_stopping(&server.id) {
            return Ok(());
        }

        manager.mark_stopping(&server.id);
        let sid = server.id.clone();
        control::spawn_stop_worker(
            "server.runtime.local",
            sid.clone(),
            "[Sea Lantern] 停止失败".to_string(),
            move || {
                let manager = crate::services::global::server_manager();
                manager.stop_server(&sid)
            },
        );

        Ok(())
    }

    fn stop(&self, _server: &ServerInstance) -> Result<(), String> {
        Err("local runtime adapter is not wired yet".to_string())
    }

    fn stop_with_manager(
        &self,
        manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<(), String> {
        let runtime = Self::ensure_config(server)?;
        control::log_runtime_action(
            "server.runtime.local",
            "stop",
            format!(
                "server_id={} startup_mode={} entry={}",
                server.id, runtime.startup_mode, runtime.jar_path
            ),
        );

        if self.stop_tracked_process(manager, server)? {
            return Ok(());
        }

        let helper_snapshot = local_helper::status_snapshot(server)?;
        if helper_snapshot
            .as_ref()
            .is_none_or(|snapshot| !snapshot.running)
        {
            control::clear_runtime_flags(manager, &server.id);
            let _ = server_log_pipeline::append_sealantern_log(
                &server.id,
                "[Sea Lantern] 服务器未运行",
            );
            server_log_pipeline::shutdown_writer(&server.id);
            return Ok(());
        }

        let _ = server_log_pipeline::append_sealantern_log(
            &server.id,
            "[Sea Lantern] 正在请求本地 runtime helper 停止服务器...",
        );
        local_helper::request_stop(server)?;

        for _ in 0..20 {
            std::thread::sleep(Duration::from_millis(500));
            match local_helper::status_snapshot(server)? {
                Some(snapshot) if snapshot.running => {}
                _ => {
                    control::clear_runtime_flags(manager, &server.id);
                    let _ = server_log_pipeline::append_sealantern_log(
                        &server.id,
                        "[Sea Lantern] 服务器已停止",
                    );
                    server_log_pipeline::shutdown_writer(&server.id);
                    return Ok(());
                }
            }
        }

        local_helper::force_stop(server)?;
        server_log_pipeline::shutdown_writer(&server.id);
        control::clear_runtime_flags(manager, &server.id);
        let _ = server_log_pipeline::append_sealantern_log(
            &server.id,
            "[Sea Lantern] 服务器超时，已强制终止",
        );
        Ok(())
    }

    fn prepare_force_stop(
        &self,
        _server: &ServerInstance,
    ) -> Result<RuntimeForceStopPreparation, String> {
        Ok(RuntimeForceStopPreparation { supported: true })
    }

    fn force_stop(&self, _server: &ServerInstance) -> Result<(), String> {
        Err("local runtime adapter is not wired yet".to_string())
    }

    fn force_stop_with_manager(
        &self,
        manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<(), String> {
        let runtime = Self::ensure_config(server)?;
        control::log_runtime_action(
            "server.runtime.local",
            "force_stop",
            format!(
                "server_id={} startup_mode={} entry={}",
                server.id, runtime.startup_mode, runtime.jar_path
            ),
        );

        manager.mark_stopping(&server.id);

        if self.force_stop_tracked_process(manager, server)? {
            return Ok(());
        }

        local_helper::force_stop(server)?;
        control::clear_runtime_flags(manager, &server.id);
        let _ = server_log_pipeline::append_sealantern_log(
            &server.id,
            "[Sea Lantern] 已按用户确认强制终止服务器进程",
        );
        server_log_pipeline::shutdown_writer(&server.id);
        Ok(())
    }

    fn status(&self, server: &ServerInstance) -> Result<RuntimeStatusSnapshot, String> {
        Ok(local_helper::status_snapshot(server)?.map_or_else(
            local_helper::stopped_runtime_snapshot,
            |snapshot| {
                let status = local_helper::helper_runtime_status(&snapshot, false);
                local_helper::runtime_snapshot_from_helper(snapshot, status)
            },
        ))
    }

    fn status_with_manager(
        &self,
        manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<RuntimeStatusSnapshot, String> {
        let runtime = Self::ensure_config(server)?;

        if let Some(snapshot) = Self::tracked_status_snapshot(manager, server)? {
            control::log_runtime_action(
                "server.runtime.local",
                "status",
                format!(
                    "server_id={} startup_mode={} entry={} status={} detail={} source=tracked_child",
                    server.id,
                    runtime.startup_mode,
                    runtime.jar_path,
                    snapshot.status.as_str(),
                    snapshot.detail_message.as_deref().unwrap_or(""),
                ),
            );
            return Ok(snapshot);
        }

        if let Some(snapshot) = Self::helper_status_snapshot(manager, server)? {
            control::log_runtime_action(
                "server.runtime.local",
                "status",
                format!(
                    "server_id={} startup_mode={} entry={} status={} detail={} source=helper",
                    server.id,
                    runtime.startup_mode,
                    runtime.jar_path,
                    snapshot.status.as_str(),
                    snapshot.detail_message.as_deref().unwrap_or(""),
                ),
            );
            return Ok(snapshot);
        }

        control::clear_runtime_flags(manager, &server.id);
        Ok(local_helper::stopped_runtime_snapshot())
    }
}

impl RuntimeProcessHandle {
    pub fn into_local_child(self) -> Option<std::process::Child> {
        match self {
            RuntimeProcessHandle::LocalChild(child) => Some(child),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LocalServerRuntime;
    use crate::models::server::{
        LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig, ServerStatus,
    };
    use crate::services::server::manager::ServerManager;
    use crate::services::server::runtime::local_helper::{state_file_path, LocalRuntimeState};
    use crate::services::server::runtime::ServerRuntime;
    use std::process::Command;
    use tempfile::tempdir;

    fn local_server_at(path: String) -> ServerInstance {
        ServerInstance {
            id: "local-alpha".to_string(),
            name: "Local Alpha".to_string(),
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

    fn local_server() -> ServerInstance {
        local_server_at("E:/tmp/local-alpha".to_string())
    }

    fn write_state_fixture(server: &ServerInstance, state: &LocalRuntimeState) {
        let content =
            serde_json::to_string_pretty(state).expect("test state should serialize successfully");
        std::fs::write(state_file_path(server), content)
            .expect("test state file should be written successfully");
    }

    #[test]
    fn local_status_clears_stopping_when_no_process_is_present() {
        let manager = ServerManager::new();
        manager.mark_starting("local-alpha");
        manager.mark_stopping("local-alpha");
        let server = local_server();

        let snapshot = LocalServerRuntime
            .status_with_manager(&manager, &server)
            .expect("status should succeed");

        assert_eq!(snapshot.status, ServerStatus::Stopped);
        assert!(!manager.is_starting("local-alpha"));
        assert!(!manager.is_stopping("local-alpha"));
    }

    #[test]
    fn local_status_reports_error_when_process_exits_non_zero() {
        let manager = ServerManager::new();
        let server = local_server();

        let mut child = if cfg!(windows) {
            Command::new("powershell")
                .args(["-NoProfile", "-Command", "exit 7"])
                .spawn()
                .expect("test child should spawn")
        } else {
            Command::new("sh")
                .args(["-c", "exit 7"])
                .spawn()
                .expect("test child should spawn")
        };

        let _ = child.wait();
        manager
            .lock_processes()
            .expect("process map")
            .insert(server.id.clone(), child);

        let snapshot = LocalServerRuntime
            .status_with_manager(&manager, &server)
            .expect("status should succeed");

        assert_eq!(snapshot.status, ServerStatus::Error);
        assert!(snapshot
            .error_message
            .as_deref()
            .is_some_and(|message| message.contains("退出码：7")));
    }

    #[test]
    fn local_send_command_reports_error_when_stdin_is_missing() {
        let manager = ServerManager::new();
        let server = local_server();

        let child = if cfg!(windows) {
            Command::new("powershell")
                .args(["-NoProfile", "-Command", "Start-Sleep -Seconds 3"])
                .stdin(std::process::Stdio::null())
                .spawn()
                .expect("test child should spawn")
        } else {
            Command::new("sh")
                .args(["-c", "sleep 3"])
                .stdin(std::process::Stdio::null())
                .spawn()
                .expect("test child should spawn")
        };

        let pid = child.id();
        manager
            .lock_processes()
            .expect("process map")
            .insert(server.id.clone(), child);

        let err = LocalServerRuntime
            .send_command_with_manager(&manager, &server, "say hello")
            .expect_err("missing stdin should be reported");

        assert!(err.contains("stdin 不可用"));

        let removed_child = {
            manager
                .lock_processes()
                .expect("process map")
                .remove(&server.id)
        };

        if let Some(mut child) = removed_child {
            let _ = child.kill();
            let _ = child.wait();
        } else {
            panic!("expected child process to remain tracked for pid={}", pid);
        }
    }

    #[test]
    fn local_status_uses_state_file_fallback_when_helper_is_unavailable() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = local_server_at(temp_dir.path().to_string_lossy().to_string());
        let state = LocalRuntimeState {
            server_id: server.id.clone(),
            helper_pid: u32::MAX,
            child_pid: Some(u32::MAX),
            control_port: Some(25570),
            auth_token: "token".to_string(),
            running: true,
            exit_code: Some(7),
            detail_message: "runtime=local running=true source=helper".to_string(),
            error_message: Some("server crashed".to_string()),
            updated_at: 123,
        };
        write_state_fixture(&server, &state);

        let snapshot = LocalServerRuntime
            .status(&server)
            .expect("status should succeed");

        assert_eq!(snapshot.status, ServerStatus::Error);
        assert_eq!(snapshot.pid, None);
        assert_eq!(
            snapshot.detail_message.as_deref(),
            Some("runtime=local running=false source=helper exit_code=7")
        );
        assert_eq!(snapshot.error_message.as_deref(), Some("server crashed"));
    }

    #[test]
    fn local_status_with_manager_reports_error_from_state_file_fallback() {
        let manager = ServerManager::new();
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = local_server_at(temp_dir.path().to_string_lossy().to_string());
        let state = LocalRuntimeState {
            server_id: server.id.clone(),
            helper_pid: u32::MAX,
            child_pid: Some(u32::MAX),
            control_port: Some(25570),
            auth_token: "token".to_string(),
            running: true,
            exit_code: Some(7),
            detail_message: "runtime=local running=true source=helper".to_string(),
            error_message: Some("server crashed".to_string()),
            updated_at: 123,
        };
        write_state_fixture(&server, &state);

        let snapshot = LocalServerRuntime
            .status_with_manager(&manager, &server)
            .expect("status should succeed");

        assert_eq!(snapshot.status, ServerStatus::Error);
        assert_eq!(snapshot.pid, None);
        assert_eq!(
            snapshot.detail_message.as_deref(),
            Some("runtime=local running=false source=helper exit_code=7")
        );
        assert_eq!(snapshot.error_message.as_deref(), Some("server crashed"));
    }
}
