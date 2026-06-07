use super::{
    control, local_helper, RuntimeForceStopPreparation, RuntimeProcessHandle, RuntimeStartRequest,
    RuntimeStartResult, RuntimeStatusSnapshot, ServerRuntime,
};
use crate::models::server::{LocalRuntimeConfig, ServerInstance, ServerStatus};
use crate::services::server::log_pipeline as server_log_pipeline;
use crate::services::server::manager::ServerManager;
use crate::services::server::runtime::i18n::{runtime_t, runtime_t1};
use std::io::Write;
use std::time::Duration;

// Starter installs may need to download libraries and generate launch scripts
// before the helper can report a child pid, so their ready wait is longer.
const STARTER_HELPER_READY_TIMEOUT: Duration = Duration::from_secs(300);
const DEFAULT_HELPER_READY_TIMEOUT: Duration = Duration::from_secs(10);

pub struct LocalServerRuntime;

impl LocalServerRuntime {
    fn helper_ready_timeout(startup_mode: &str) -> Duration {
        if startup_mode.eq_ignore_ascii_case("starter") {
            STARTER_HELPER_READY_TIMEOUT
        } else {
            DEFAULT_HELPER_READY_TIMEOUT
        }
    }

    pub fn ensure_config(server: &ServerInstance) -> Result<&LocalRuntimeConfig, String> {
        server.local_runtime().ok_or_else(|| {
            runtime_t1("server.manager.runtime_not_supported", server.runtime_kind.clone())
        })
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
            let message = runtime_t1("server.runtime.local.stdin_unavailable", server.id.clone());
            let _ = server_log_pipeline::append_sealantern_log(
                &server.id,
                &format!("[Sea Lantern] {}", message),
            );
            message
        })?;

        writeln!(stdin, "{}", command).map_err(|e| {
            runtime_t1(
                "server.runtime.local.stdin_write_failed",
                format!("id={} error={}", server.id, e),
            )
        })?;
        stdin.flush().map_err(|e| {
            runtime_t1(
                "server.runtime.local.stdin_flush_failed",
                format!("id={} error={}", server.id, e),
            )
        })?;
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
            &format!("[Sea Lantern] {}", runtime_t("server.runtime.local.stopping_log")),
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
                            &format!(
                                "[Sea Lantern] {}",
                                runtime_t("server.runtime.local.stopped_cleanly_log")
                            ),
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
                    &format!("[Sea Lantern] {}", runtime_t("server.runtime.local.stopped_log")),
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
                &format!(
                    "[Sea Lantern] {}",
                    runtime_t("server.runtime.local.stop_timeout_force_killed_log")
                ),
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
                    &format!(
                        "[Sea Lantern] {}",
                        runtime_t("server.runtime.local.force_stopped_by_confirmation_log")
                    ),
                );
                server_log_pipeline::shutdown_writer(&server.id);
                Ok(true)
            }
            Err(err) => {
                let message = runtime_t1("server.runtime.local.force_stop_failed", err);
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

        let has_tracked_process = manager.lock_processes()?.contains_key(&server.id);

        if !has_tracked_process {
            return Ok(None);
        }

        let is_running = {
            let mut procs = manager.lock_processes()?;
            if let Some(child) = procs.get_mut(&server.id) {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        exit_code = status.code();
                        match &exit_code {
                            Some(0) => {
                                let _ = server_log_pipeline::append_sealantern_log(
                                    &server.id,
                                    &format!(
                                        "[Sea Lantern] {}",
                                        runtime_t("server.runtime.local.exited_cleanly_log")
                                    ),
                                );
                            }
                            Some(code) => {
                                terminated_abnormally = true;
                                error_message = Some(runtime_t1(
                                    "server.runtime.local.exit_abnormal",
                                    code.to_string(),
                                ));
                                let _ = server_log_pipeline::append_sealantern_log(
                                    &server.id,
                                    &format!(
                                        "[Sea Lantern] {}",
                                        runtime_t1(
                                            "server.runtime.local.exit_abnormal",
                                            code.to_string()
                                        )
                                    ),
                                );
                            }
                            None => {
                                terminated_abnormally = true;
                                error_message = Some(runtime_t("server.runtime.local.killed"));
                                let _ = server_log_pipeline::append_sealantern_log(
                                    &server.id,
                                    &format!(
                                        "[Sea Lantern] {}",
                                        runtime_t("server.runtime.local.killed")
                                    ),
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
                        error_message =
                            Some(runtime_t("server.runtime.local.status_read_failed_short"));
                        let _ = server_log_pipeline::append_sealantern_log(
                            &server.id,
                            &format!(
                                "[Sea Lantern] {}",
                                runtime_t("server.runtime.local.status_read_failed_short")
                            ),
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
        };

        let pid = if is_running {
            let mut procs = manager.lock_processes()?;
            procs.get_mut(&server.id).map(|child| child.id())
        } else {
            None
        };

        let is_stopping = manager.is_stopping_checked(&server.id)?;
        let is_starting = manager.is_starting_checked(&server.id)?;

        let status = if is_stopping {
            ServerStatus::Stopping
        } else if is_running && is_starting {
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

        let is_starting = manager.is_starting_checked(&server.id)?;
        if snapshot.running && snapshot.pid.is_some() && is_starting {
            manager.clear_starting(&server.id);
        }

        if !snapshot.running {
            control::clear_runtime_flags(manager, &server.id);
        }

        let is_stopping = snapshot.running && manager.is_stopping_checked(&server.id)?;
        let status = local_helper::helper_runtime_status(&snapshot, is_stopping);

        Ok(Some(local_helper::runtime_snapshot_from_helper(snapshot, status)))
    }
}

impl ServerRuntime for LocalServerRuntime {
    fn start(&self, _request: RuntimeStartRequest<'_>) -> Result<RuntimeStartResult, String> {
        Err(runtime_t("server.runtime.local.adapter_not_wired"))
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
        let helper_ready_timeout = Self::helper_ready_timeout(&runtime.startup_mode);
        let ready_state =
            local_helper::wait_for_helper_ready(request.server, helper_ready_timeout)?;

        if !ready_state.running {
            return Err(ready_state
                .error_message
                .unwrap_or(ready_state.detail_message));
        }

        Ok(RuntimeStartResult { process_handle: None, fallback: None })
    }

    fn send_command(&self, _server: &ServerInstance, _command: &str) -> Result<(), String> {
        Err(runtime_t("server.runtime.local.adapter_not_wired"))
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
        Err(runtime_t("server.runtime.local.adapter_not_wired"))
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

        if manager.is_stopping_checked(&server.id)? {
            return Ok(());
        }

        manager.mark_stopping(&server.id);
        let sid = server.id.clone();
        control::spawn_stop_worker(
            "server.runtime.local",
            sid.clone(),
            format!("[Sea Lantern] {}", runtime_t("server.runtime.local.stop_failed_short")),
            move || {
                let manager = crate::services::global::server_manager();
                manager.stop_server(&sid)
            },
        );

        Ok(())
    }

    fn stop(&self, _server: &ServerInstance) -> Result<(), String> {
        Err(runtime_t("server.runtime.local.adapter_not_wired"))
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
                &format!("[Sea Lantern] {}", runtime_t("server.runtime.local.not_running_log")),
            );
            server_log_pipeline::shutdown_writer(&server.id);
            return Ok(());
        }

        let _ = server_log_pipeline::append_sealantern_log(
            &server.id,
            &format!(
                "[Sea Lantern] {}",
                runtime_t("server.runtime.local.requesting_helper_stop_log")
            ),
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
                        &format!("[Sea Lantern] {}", runtime_t("server.runtime.local.stopped_log")),
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
            &format!(
                "[Sea Lantern] {}",
                runtime_t("server.runtime.local.stop_timeout_force_killed_log")
            ),
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
        Err(runtime_t("server.runtime.local.adapter_not_wired"))
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

        if let Err(err) = local_helper::force_stop(server) {
            control::clear_runtime_flags(manager, &server.id);
            server_log_pipeline::shutdown_writer(&server.id);
            return Err(err);
        }
        control::clear_runtime_flags(manager, &server.id);
        let _ = server_log_pipeline::append_sealantern_log(
            &server.id,
            &format!(
                "[Sea Lantern] {}",
                runtime_t("server.runtime.local.force_stopped_by_confirmation_log")
            ),
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
    use super::{
        LocalServerRuntime, DEFAULT_HELPER_READY_TIMEOUT, STARTER_HELPER_READY_TIMEOUT,
    };
    use crate::models::server::{
        LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig, ServerStatus,
    };
    use crate::services::server::manager::ServerManager;
    use crate::services::server::runtime::i18n::{runtime_t, runtime_t1};
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
                cpu_policy: crate::models::server::CpuPolicyConfig::default(),
                jvm_preset: crate::models::server::JvmPresetConfig::default(),
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
        assert!(!manager
            .is_starting_checked("local-alpha")
            .expect("starting flag should read"));
        assert!(!manager
            .is_stopping_checked("local-alpha")
            .expect("stopping flag should read"));
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
        assert!(snapshot.error_message.as_deref().is_some_and(
            |message| message == runtime_t1("server.runtime.local.exit_abnormal", "7")
        ));
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

        assert!(err.contains(&runtime_t("server.runtime.local.stdin_unavailable_prefix")));

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
            child_pid: None,
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
            child_pid: None,
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

    #[test]
    fn local_status_with_manager_clears_stopping_for_stale_helper_state() {
        let manager = ServerManager::new();
        manager.mark_stopping("local-alpha");
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = local_server_at(temp_dir.path().to_string_lossy().to_string());
        let state = LocalRuntimeState {
            server_id: server.id.clone(),
            helper_pid: u32::MAX,
            child_pid: Some(u32::MAX),
            control_port: Some(25570),
            auth_token: "token".to_string(),
            running: true,
            exit_code: None,
            detail_message: "runtime=local running=true source=helper".to_string(),
            error_message: None,
            updated_at: 123,
        };
        write_state_fixture(&server, &state);

        let snapshot = LocalServerRuntime
            .status_with_manager(&manager, &server)
            .expect("status should succeed");

        assert_eq!(snapshot.status, ServerStatus::Stopped);
        assert_eq!(snapshot.pid, None);
        assert!(!manager
            .is_stopping_checked("local-alpha")
            .expect("stopping flag should read"));
    }

    #[test]
    fn local_status_with_manager_surfaces_process_lock_failures() {
        let manager = std::sync::Arc::new(ServerManager::new());
        let poison_manager = manager.clone();
        let poison_thread = std::thread::spawn(move || {
            let _guard = poison_manager
                .processes
                .lock()
                .expect("process map lock should be acquired for poisoning");
            panic!("poison process lock");
        });
        assert!(poison_thread.join().is_err(), "poison thread should panic");

        let error = match LocalServerRuntime.status_with_manager(&manager, &local_server()) {
            Err(error) => error,
            Ok(snapshot) => panic!(
                "lock failure should not be downgraded to stopped status, got: {}",
                snapshot.status.as_str()
            ),
        };

        assert_eq!(error, "processes lock poisoned");
    }

    #[test]
    fn helper_ready_timeout_uses_named_starter_budget() {
        assert_eq!(
            LocalServerRuntime::helper_ready_timeout("starter"),
            STARTER_HELPER_READY_TIMEOUT
        );
        assert_eq!(LocalServerRuntime::helper_ready_timeout("jar"), DEFAULT_HELPER_READY_TIMEOUT);
    }
}
