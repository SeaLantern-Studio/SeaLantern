mod helpers;

use self::helpers::*;
pub(crate) use self::helpers::{
    build_docker_launch_detail, build_docker_run_args, resolve_docker_launch_spec,
    resolve_runtime_cpuset, DockerLaunchDetail,
};
use super::{
    control, RuntimeForceStopPreparation, RuntimeStartRequest, RuntimeStartResult,
    RuntimeStatusSnapshot, ServerRuntime,
};
use crate::models::server::{
    DockerBackendKind, DockerCommandMode, DockerItzgRuntimeConfig, ServerInstance, ServerStatus,
};
use crate::services::server::log_pipeline as server_log_pipeline;
use crate::services::server::manager::ServerManager;
use crate::utils::docker_cli::{
    docker_executable_path, ensure_docker_command_success, render_docker_command_error,
};
use crate::utils::logger;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;
use tokio::net::TcpStream;

const DOCKER_ITZG_RUNTIME_KIND: &str = "docker_itzg";
const CONFIG_VALID: &str = "ok";
const CONFIG_MISMATCH: &str = "server runtime config is not docker_itzg";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockerItzgCapability {
    Start,
    SendCommand,
    RequestStop,
    Stop,
    PrepareForceStop,
    ForceStop,
    Status,
}

impl DockerItzgCapability {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::SendCommand => "send_command",
            Self::RequestStop => "request_stop",
            Self::Stop => "stop",
            Self::PrepareForceStop => "prepare_force_stop",
            Self::ForceStop => "force_stop",
            Self::Status => "status",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerItzgCapabilityError {
    capability: DockerItzgCapability,
    docker_backend_kind: String,
    command_mode: String,
    backend_adapter: String,
    command_transport: String,
    config_validation: String,
    runtime_kind: Option<String>,
}

impl DockerItzgCapabilityError {
    fn runtime_mismatch(runtime_kind: &str, capability: DockerItzgCapability) -> Self {
        Self {
            capability,
            docker_backend_kind: "unknown".to_string(),
            command_mode: "unknown".to_string(),
            backend_adapter: "runtime config mismatch prevented docker adapter selection"
                .to_string(),
            command_transport: "runtime config mismatch prevented command transport selection"
                .to_string(),
            config_validation: CONFIG_MISMATCH.to_string(),
            runtime_kind: Some(runtime_kind.to_string()),
        }
    }

    fn build(self) -> String {
        let mut fields = vec![
            format!("action={}", self.capability.as_str()),
            format!("docker_backend_kind={}", self.docker_backend_kind),
            format!("command_mode={}", self.command_mode),
            format!("backend_adapter={}", self.backend_adapter),
            format!("command_transport={}", self.command_transport),
            format!("config_validation={}", self.config_validation),
        ];
        if let Some(runtime_kind) = self.runtime_kind {
            fields.push(format!("runtime_kind={}", runtime_kind));
        }

        format!("docker_itzg capability unavailable: {}", fields.join(", "))
    }
}

pub struct DockerItzgRuntime {
    pub cli_adapter: DockerCliAdapter,
    pub engine_api_adapter: DockerEngineApiAdapter,
}

impl DockerItzgRuntime {
    pub fn new() -> Self {
        Self {
            cli_adapter: DockerCliAdapter,
            engine_api_adapter: DockerEngineApiAdapter,
        }
    }

    pub fn capability_error(
        &self,
        server: &ServerInstance,
        capability: DockerItzgCapability,
    ) -> String {
        match server.docker_itzg_runtime() {
            Some(runtime) => self.describe_capability_error(runtime, capability).build(),
            None => DockerItzgCapabilityError::runtime_mismatch(&server.runtime_kind, capability)
                .build(),
        }
    }

    fn ensure_cli_runtime<'a>(
        &self,
        server: &'a ServerInstance,
        capability: DockerItzgCapability,
    ) -> Result<&'a DockerItzgRuntimeConfig, String> {
        let runtime = server
            .docker_itzg_runtime()
            .ok_or_else(|| self.capability_error(server, capability))?;

        if runtime.docker_backend_kind != DockerBackendKind::Cli {
            return Err(self.capability_error(server, capability));
        }

        Ok(runtime)
    }

    fn describe_capability_error(
        &self,
        runtime: &DockerItzgRuntimeConfig,
        capability: DockerItzgCapability,
    ) -> DockerItzgCapabilityError {
        DockerItzgCapabilityError {
            capability,
            docker_backend_kind: runtime.docker_backend_kind.as_str().to_string(),
            command_mode: runtime.command_mode.as_str().to_string(),
            backend_adapter: self.backend_capability_error(runtime),
            command_transport: self.command_capability_error(runtime),
            config_validation: config_validation_message(runtime),
            runtime_kind: Some(DOCKER_ITZG_RUNTIME_KIND.to_string()),
        }
    }

    fn backend_capability_error(&self, runtime: &DockerItzgRuntimeConfig) -> String {
        match runtime.docker_backend_kind {
            DockerBackendKind::Cli => self.cli_adapter.capability_error(),
            DockerBackendKind::EngineApi => self.engine_api_adapter.capability_error(),
        }
    }

    fn command_capability_error(&self, runtime: &DockerItzgRuntimeConfig) -> String {
        command_transport_message(runtime).to_string()
    }
}

impl ServerRuntime for DockerItzgRuntime {
    fn start(&self, request: RuntimeStartRequest<'_>) -> Result<RuntimeStartResult, String> {
        Err(self.capability_error(request.server, DockerItzgCapability::Start))
    }

    fn start_with_manager(
        &self,
        manager: &ServerManager,
        request: RuntimeStartRequest<'_>,
    ) -> Result<RuntimeStartResult, String> {
        let runtime = self.ensure_cli_runtime(request.server, DockerItzgCapability::Start)?;
        ensure_runtime_path_ready(request.server)?;
        server_log_pipeline::init_db(Path::new(&request.server.path))?;
        self.cli_adapter.start(request.server, runtime)?;
        manager.mark_starting(&request.server.id);

        if let Err(err) = self
            .cli_adapter
            .spawn_logs_follower(&request.server.id, &runtime.container_name)
        {
            let _ = server_log_pipeline::append_sealantern_log(
                &request.server.id,
                &format!("[Sea Lantern] Docker 日志跟随启动失败: {}", err),
            );
        }

        Ok(RuntimeStartResult { process_handle: None, fallback: None })
    }

    fn send_command(&self, server: &ServerInstance, _command: &str) -> Result<(), String> {
        Err(self.capability_error(server, DockerItzgCapability::SendCommand))
    }

    fn send_command_with_manager(
        &self,
        _manager: &ServerManager,
        server: &ServerInstance,
        command: &str,
    ) -> Result<(), String> {
        let runtime = self.ensure_cli_runtime(server, DockerItzgCapability::SendCommand)?;
        self.cli_adapter.send_command(server, runtime, command)
    }

    fn request_stop(&self, server: &ServerInstance) -> Result<(), String> {
        Err(self.capability_error(server, DockerItzgCapability::RequestStop))
    }

    fn request_stop_with_manager(
        &self,
        manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<(), String> {
        let runtime = self.ensure_cli_runtime(server, DockerItzgCapability::RequestStop)?;

        if manager.is_stopping(&server.id) {
            return Ok(());
        }

        manager.mark_stopping(&server.id);
        let sid = server.id.clone();
        let container_name = runtime.container_name.clone();
        control::spawn_stop_worker(
            "server.runtime.docker_itzg",
            sid.clone(),
            format!("[Sea Lantern] Docker 停止失败 (container={})", container_name),
            move || {
                let manager = crate::services::global::server_manager();
                manager.stop_server(&sid)
            },
        );

        Ok(())
    }

    fn stop(&self, server: &ServerInstance) -> Result<(), String> {
        Err(self.capability_error(server, DockerItzgCapability::Stop))
    }

    fn stop_with_manager(
        &self,
        manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<(), String> {
        let runtime = self.ensure_cli_runtime(server, DockerItzgCapability::Stop)?;
        let snapshot = self.cli_adapter.status(server, runtime)?;

        if docker_status_is_not_running(&snapshot) {
            control::clear_runtime_flags(manager, &server.id);
            let _ = server_log_pipeline::append_sealantern_log(
                &server.id,
                &format!(
                    "[Sea Lantern] Docker 容器已不在运行态: status={} detail={}",
                    snapshot.status.as_str(),
                    snapshot.detail_message.as_deref().unwrap_or("-")
                ),
            );
            return Ok(());
        }

        let _ = server_log_pipeline::append_sealantern_log(
            &server.id,
            &format!("[Sea Lantern] 正在停止 Docker 容器: {}", runtime.container_name),
        );
        let graceful_stop_accepted = match self.cli_adapter.request_graceful_stop(server, runtime) {
            Ok(accepted) => accepted,
            Err(err) => {
                logger::log_trace(&format!(
                    "[server.runtime.docker_itzg] graceful_stop_request_failed server_id={} container={} error={}",
                    server.id, runtime.container_name, err
                ));
                let _ = server_log_pipeline::append_sealantern_log(
                    &server.id,
                    &format!(
                        "[Sea Lantern] Docker 优雅停服命令发送失败，将回退到 docker stop: {}",
                        err
                    ),
                );
                false
            }
        };

        if graceful_stop_accepted {
            for _ in 0..20 {
                thread::sleep(Duration::from_millis(500));
                let snapshot = self.cli_adapter.status(server, runtime)?;
                if docker_status_is_not_running(&snapshot) {
                    control::clear_runtime_flags(manager, &server.id);
                    let _ = server_log_pipeline::append_sealantern_log(
                        &server.id,
                        "[Sea Lantern] Docker 容器已通过服务端 stop 命令退出运行态",
                    );
                    return Ok(());
                }
            }

            let _ = server_log_pipeline::append_sealantern_log(
                &server.id,
                "[Sea Lantern] Docker stop 命令已发送，但容器仍未退出；将回退到 docker stop",
            );
        }

        self.cli_adapter.stop(runtime)?;

        for _ in 0..20 {
            thread::sleep(Duration::from_millis(500));
            let snapshot = self.cli_adapter.status(server, runtime)?;
            if docker_status_is_not_running(&snapshot) {
                control::clear_runtime_flags(manager, &server.id);
                let _ = server_log_pipeline::append_sealantern_log(
                    &server.id,
                    "[Sea Lantern] Docker 容器已停止或已退出运行态",
                );
                return Ok(());
            }
        }

        let _ = server_log_pipeline::append_sealantern_log(
            &server.id,
            "[Sea Lantern] Docker 容器停止超时，将执行强制终止",
        );
        self.cli_adapter.force_stop(runtime)?;
        control::clear_runtime_flags(manager, &server.id);
        Ok(())
    }

    fn prepare_force_stop(
        &self,
        _server: &ServerInstance,
    ) -> Result<RuntimeForceStopPreparation, String> {
        Ok(RuntimeForceStopPreparation { supported: true })
    }

    fn prepare_force_stop_with_manager(
        &self,
        _manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<RuntimeForceStopPreparation, String> {
        let _ = self.ensure_cli_runtime(server, DockerItzgCapability::PrepareForceStop)?;
        Ok(RuntimeForceStopPreparation { supported: true })
    }

    fn force_stop(&self, server: &ServerInstance) -> Result<(), String> {
        Err(self.capability_error(server, DockerItzgCapability::ForceStop))
    }

    fn force_stop_with_manager(
        &self,
        manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<(), String> {
        let runtime = self.ensure_cli_runtime(server, DockerItzgCapability::ForceStop)?;
        self.cli_adapter.force_stop(runtime)?;
        control::clear_runtime_flags(manager, &server.id);
        let _ = server_log_pipeline::append_sealantern_log(
            &server.id,
            &format!("[Sea Lantern] 已强制终止 Docker 容器: {}", runtime.container_name),
        );
        Ok(())
    }

    fn status(&self, server: &ServerInstance) -> Result<RuntimeStatusSnapshot, String> {
        let runtime = self.ensure_cli_runtime(server, DockerItzgCapability::Status)?;
        self.cli_adapter.status(server, runtime)
    }

    fn status_with_manager(
        &self,
        manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<RuntimeStatusSnapshot, String> {
        let runtime = self.ensure_cli_runtime(server, DockerItzgCapability::Status)?;
        let mut snapshot = self.cli_adapter.status(server, runtime)?;

        if let Some(state) = self.cli_adapter.inspect_container_state(runtime)? {
            if container_should_clear_starting(&state) {
                manager.clear_starting(&server.id);
            }
        }

        snapshot.status = resolve_managed_status(
            snapshot.status,
            manager.is_starting(&server.id),
            manager.is_stopping(&server.id),
        );

        control::clear_runtime_flags_if_terminal(manager, &server.id, &snapshot.status);

        Ok(snapshot)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerContainerState {
    status: String,
    running: bool,
    pid: Option<u32>,
    exit_code: Option<i64>,
    health_status: Option<String>,
    error_message: Option<String>,
}

pub struct DockerCliAdapter;

impl DockerCliAdapter {
    pub fn capability_error(&self) -> String {
        "docker CLI adapter is unavailable for the requested runtime combination".to_string()
    }

    pub fn start(
        &self,
        server: &ServerInstance,
        runtime: &DockerItzgRuntimeConfig,
    ) -> Result<(), String> {
        logger::log_user_action(
            "server.runtime.docker_itzg",
            "start",
            &format!(
                "server_id={} container={} image={}:{} command_mode={}",
                server.id,
                runtime.container_name,
                runtime.image,
                runtime.image_tag,
                runtime.command_mode.as_str()
            ),
        );
        let existing = self.inspect_container_state(runtime)?;
        if let Some(state) = existing {
            if state.running {
                return Err(format!("Docker 容器已在运行: {}", runtime.container_name));
            }

            self.remove_container(runtime)?;
        }

        let launch_spec = resolve_docker_launch_spec(runtime)?;
        logger::log_trace(&format!(
            "[server.runtime.docker_itzg] docker_jvm_env_synthesized server_id={} container={} preset={} jvm_opts_args={} jvm_xx_opts_args={}",
            server.id,
            runtime.container_name,
            runtime_jvm_preset_name(&runtime.jvm_preset.preset),
            launch_spec.jvm_opts_args_count,
            launch_spec.jvm_xx_opts_args_count
        ));
        if let Some(cpuset) = launch_spec.cpuset_cpus.as_deref() {
            logger::log_trace(&format!(
                "[server.runtime.docker_itzg] docker_cpu_policy_applied server_id={} container={} mode={} cpuset={} sync_active_processor_count={}",
                server.id,
                runtime.container_name,
                runtime.cpu_policy.mode.as_str(),
                cpuset,
                runtime.cpu_policy.sync_active_processor_count
            ));
        }

        let docker_path = docker_executable_path()?;
        let mut command = Command::new(docker_path);
        for arg in build_docker_run_args(runtime, &launch_spec) {
            command.arg(arg);
        }

        let output = command
            .output()
            .map_err(|e| format!("执行 docker run 失败: {}", e))?;
        let image_ref = docker_image_ref(runtime);
        ensure_docker_command_success(
            output,
            "docker run",
            Some(&image_ref),
            Some(&runtime.container_name),
        )?;

        let _ = server_log_pipeline::append_sealantern_log(
            &server.id,
            &format!("[Sea Lantern] Docker 容器已创建并启动: {}", runtime.container_name),
        );

        Ok(())
    }

    pub fn send_command(
        &self,
        server: &ServerInstance,
        runtime: &DockerItzgRuntimeConfig,
        command_text: &str,
    ) -> Result<(), String> {
        let trimmed = command_text.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        logger::log_user_action(
            "server.runtime.docker_itzg",
            "send_command_attempt",
            &format!(
                "server_id={} container={} command_mode={} command={}",
                server.id,
                runtime.container_name,
                runtime.command_mode.as_str(),
                trimmed
            ),
        );

        let state = self.inspect_container_state(runtime)?;
        let state = state.ok_or_else(|| render_send_command_precondition_missing(runtime))?;
        if !state.running {
            return Err(render_send_command_precondition_stopped(server, runtime, &state));
        }
        if let Some(message) = render_send_command_precondition_not_ready(server, runtime, &state) {
            return Err(message);
        }

        match runtime.command_mode {
            DockerCommandMode::DockerStdio => {
                self.send_command_via_docker_exec(runtime, trimmed)?;
            }
            DockerCommandMode::Rcon => {
                self.send_command_via_rcon(runtime, trimmed)?;
            }
        }

        let _ = server_log_pipeline::append_sealantern_log(
            &server.id,
            &format!("[Sea Lantern] 已发送 Docker 控制台命令: {}", trimmed),
        );
        logger::log_user_action(
            "server.runtime.docker_itzg",
            "send_command",
            &format!(
                "server_id={} container={} command_mode={} command={}",
                server.id,
                runtime.container_name,
                runtime.command_mode.as_str(),
                trimmed
            ),
        );

        Ok(())
    }

    pub fn stop(&self, runtime: &DockerItzgRuntimeConfig) -> Result<(), String> {
        logger::log_user_action(
            "server.runtime.docker_itzg",
            "stop",
            &format!(
                "container={} stop_timeout_secs={}",
                runtime.container_name,
                requested_stop_timeout_secs(runtime)
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "default".to_string())
            ),
        );
        let docker_path = docker_executable_path()?;
        let mut command = Command::new(docker_path);
        command.arg("stop");
        if let Some(timeout_secs) = requested_stop_timeout_secs(runtime) {
            command.arg("-t").arg(timeout_secs.to_string());
        }
        let output = command
            .arg(&runtime.container_name)
            .output()
            .map_err(|e| format!("执行 docker stop 失败: {}", e))?;
        if docker_output_indicates_missing_container(&output) {
            return Ok(());
        }
        let image_ref = docker_image_ref(runtime);
        ensure_docker_command_success(
            output,
            "docker stop",
            Some(&image_ref),
            Some(&runtime.container_name),
        )
    }

    pub fn request_graceful_stop(
        &self,
        server: &ServerInstance,
        runtime: &DockerItzgRuntimeConfig,
    ) -> Result<bool, String> {
        logger::log_user_action(
            "server.runtime.docker_itzg",
            "request_graceful_stop",
            &format!(
                "server_id={} container={} command_mode={}",
                server.id,
                runtime.container_name,
                runtime.command_mode.as_str()
            ),
        );

        let state = self.inspect_container_state(runtime)?;
        let Some(state) = state else {
            return Ok(false);
        };
        if !state.running {
            return Ok(false);
        }

        match runtime.command_mode {
            DockerCommandMode::Rcon => self.send_command_via_rcon(runtime, "stop")?,
            DockerCommandMode::DockerStdio => self.send_command_via_docker_exec(runtime, "stop")?,
        }

        Ok(true)
    }

    pub fn force_stop(&self, runtime: &DockerItzgRuntimeConfig) -> Result<(), String> {
        logger::log_user_action(
            "server.runtime.docker_itzg",
            "force_stop",
            &format!("container={}", runtime.container_name),
        );
        let docker_path = docker_executable_path()?;
        let output = Command::new(docker_path)
            .arg("rm")
            .arg("-f")
            .arg(&runtime.container_name)
            .output()
            .map_err(|e| format!("执行 docker rm -f 失败: {}", e))?;
        if docker_output_indicates_missing_container(&output) {
            return Ok(());
        }
        let image_ref = docker_image_ref(runtime);
        ensure_docker_command_success(
            output,
            "docker rm -f",
            Some(&image_ref),
            Some(&runtime.container_name),
        )
    }

    pub fn status(
        &self,
        _server: &ServerInstance,
        runtime: &DockerItzgRuntimeConfig,
    ) -> Result<RuntimeStatusSnapshot, String> {
        let state = self.inspect_container_state(runtime)?;

        match state {
            Some(state) => Ok(RuntimeStatusSnapshot {
                status: map_container_status(&state),
                pid: state.pid,
                detail_message: Some(render_container_detail(runtime, &state)),
                error_message: render_container_error(runtime, &state),
            }),
            None => Ok(RuntimeStatusSnapshot {
                status: ServerStatus::Stopped,
                pid: None,
                detail_message: Some(format!(
                    "runtime=docker_itzg container={} state=missing",
                    runtime.container_name
                )),
                error_message: None,
            }),
        }
    }

    pub fn spawn_logs_follower(&self, server_id: &str, container_name: &str) -> Result<(), String> {
        logger::log_user_action(
            "server.runtime.docker_itzg",
            "logs_follow",
            &format!("server_id={} container={}", server_id, container_name),
        );
        let docker_path = docker_executable_path()?;
        let mut child = Command::new(docker_path)
            .arg("logs")
            .arg("-f")
            .arg(container_name)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("执行 docker logs -f 失败: {}", e))?;

        if let Some(stdout) = child.stdout.take() {
            server_log_pipeline::spawn_server_output_reader(server_id.to_string(), stdout);
        }
        if let Some(stderr) = child.stderr.take() {
            server_log_pipeline::spawn_server_output_reader(server_id.to_string(), stderr);
        }

        std::thread::spawn(move || {
            let _ = child.wait();
        });

        Ok(())
    }

    fn inspect_container_state(
        &self,
        runtime: &DockerItzgRuntimeConfig,
    ) -> Result<Option<DockerContainerState>, String> {
        let docker_path = docker_executable_path()?;
        let output = Command::new(docker_path)
            .arg("inspect")
            .arg(&runtime.container_name)
            .arg("--format")
            .arg("{{.State.Status}}|{{.State.Running}}|{{.State.Pid}}|{{.State.ExitCode}}|{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}|{{.State.Error}}")
            .output()
            .map_err(|e| format!("执行 docker inspect 失败: {}", e))?;

        if !output.status.success() {
            let stderr = stderr_text(&output);
            if is_container_not_found(&stderr) {
                return Ok(None);
            }
            let image_ref = docker_image_ref(runtime);
            return Err(render_docker_command_error(
                "docker inspect",
                &output,
                Some(&image_ref),
                Some(&runtime.container_name),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            return Ok(None);
        }

        let mut parts = stdout.splitn(6, '|');
        let status = parts.next().unwrap_or_default().trim().to_string();
        let running = parts
            .next()
            .unwrap_or_default()
            .trim()
            .eq_ignore_ascii_case("true");
        let pid = parts
            .next()
            .unwrap_or_default()
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|pid| *pid > 0);
        let exit_code = parts.next().unwrap_or_default().trim().parse::<i64>().ok();
        let health_status = parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("none"))
            .map(|value| value.to_string());
        let error_message = parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string());

        Ok(Some(DockerContainerState {
            status,
            running,
            pid,
            exit_code,
            health_status,
            error_message,
        }))
    }

    fn remove_container(&self, runtime: &DockerItzgRuntimeConfig) -> Result<(), String> {
        let docker_path = docker_executable_path()?;
        let output = Command::new(docker_path)
            .arg("rm")
            .arg("-f")
            .arg(&runtime.container_name)
            .output()
            .map_err(|e| format!("执行 docker rm -f 失败: {}", e))?;
        if docker_output_indicates_missing_container(&output) {
            return Ok(());
        }
        let image_ref = docker_image_ref(runtime);
        ensure_docker_command_success(
            output,
            "docker rm -f",
            Some(&image_ref),
            Some(&runtime.container_name),
        )
    }

    fn send_command_via_docker_exec(
        &self,
        runtime: &DockerItzgRuntimeConfig,
        command_text: &str,
    ) -> Result<(), String> {
        let docker_path = docker_executable_path()?;
        let mut command = Command::new(docker_path);
        command.arg("exec");

        if let Some(uid) = runtime_env_value(runtime, "UID")
            .map(str::trim)
            .filter(|uid| !uid.is_empty() && *uid != "0")
        {
            command.arg("--user").arg(uid);
        }

        let output = command
            .arg(&runtime.container_name)
            .arg("mc-send-to-console")
            .arg(command_text)
            .output()
            .map_err(|e| format!("执行 docker exec 发送命令失败: {}", e))?;
        if output.status.success() {
            return Ok(());
        }

        let stderr = stderr_text(&output);
        if docker_exec_missing_mc_send_to_console(&stderr) {
            return Err(
                "当前 Docker 镜像内未提供 mc-send-to-console，docker_stdio command mode 不可用；请确认使用 itzg/minecraft-server，或切换到 --command-mode rcon"
                    .to_string(),
            );
        }
        if docker_exec_requires_console_pipe(&stderr) {
            return Err(
                "当前 Docker 镜像要求先开启 CREATE_CONSOLE_IN_PIPE=true 才能使用 docker_stdio；SeaLantern 已按 itzg 语义注入该变量。请检查现有容器是否由旧配置创建，必要时重建容器，或切换到 --command-mode rcon"
                    .to_string(),
            );
        }
        if docker_exec_named_pipe_missing(&stderr) {
            return Err(
                "当前 Docker 容器内缺少 itzg 控制台 named pipe，docker_stdio 暂不可用；这通常意味着容器仍在启动、未按 CREATE_CONSOLE_IN_PIPE=true 创建，或镜像/入口脚本与 itzg 语义不兼容。请先查看日志，必要时重建容器，或切换到 --command-mode rcon"
                    .to_string(),
            );
        }
        if docker_exec_requires_uid(&stderr) {
            let uid_hint = runtime_env_value(runtime, "UID").unwrap_or("1000");
            return Err(format!(
                "当前 Docker 镜像要求以 UID={} 执行 mc-send-to-console；SeaLantern 已优先读取 runtime.env.UID 注入 docker exec --user。请确认容器创建时的 UID 配置一致，或切换到 --command-mode rcon",
                uid_hint
            ));
        }

        let image_ref = docker_image_ref(runtime);
        let raw = render_docker_command_error(
            "docker exec mc-send-to-console",
            &output,
            Some(&image_ref),
            Some(&runtime.container_name),
        );
        Err(format!(
            "通过 docker_stdio 发送 Docker 命令失败: container={} command_mode=docker_stdio; {}；若镜像不兼容，可切换到 --command-mode rcon",
            runtime.container_name, raw
        ))
    }

    fn send_command_via_rcon(
        &self,
        runtime: &DockerItzgRuntimeConfig,
        command_text: &str,
    ) -> Result<(), String> {
        let rcon = runtime
            .rcon
            .as_ref()
            .ok_or_else(|| "RCON 配置缺失，无法通过 RCON 发送命令".to_string())?;
        let runtime_config = runtime;
        let address = format!("{}:{}", rcon.host.trim(), rcon.port);
        let password = rcon.password.clone();
        let command = command_text.to_string();

        let tokio_runtime = tokio::runtime::Runtime::new()
            .map_err(|e| format!("创建 RCON Tokio runtime 失败: {}", e))?;

        tokio_runtime.block_on(async move {
            let mut connection = rcon::Connection::<TcpStream>::builder()
                .enable_minecraft_quirks(true)
                .connect(address.as_str(), password.as_str())
                .await
                .map_err(|e| {
                    render_rcon_connect_error(runtime_config, address.as_str(), &e.to_string())
                })?;
            connection.cmd(command.as_str()).await.map_err(|e| {
                render_rcon_command_error(runtime_config, address.as_str(), &e.to_string())
            })?;
            Ok::<(), String>(())
        })
    }
}

pub struct DockerEngineApiAdapter;

impl DockerEngineApiAdapter {
    pub fn capability_error(&self) -> String {
        backend_adapter_message(&DockerBackendKind::EngineApi).to_string()
    }
}

fn backend_adapter_message(kind: &DockerBackendKind) -> &'static str {
    match kind {
        DockerBackendKind::Cli => {
            "docker CLI adapter can handle start/stop/status; unsupported combinations still return capability errors"
        }
        DockerBackendKind::EngineApi => "docker Engine API adapter skeleton is not implemented yet",
    }
}

fn command_transport_message(runtime: &DockerItzgRuntimeConfig) -> &'static str {
    match runtime.command_mode {
        DockerCommandMode::Rcon => "docker CLI uses a real RCON client connection",
        DockerCommandMode::DockerStdio => {
            "docker CLI sends commands via mc-send-to-console inside the container"
        }
    }
}

fn config_validation_message(runtime: &DockerItzgRuntimeConfig) -> String {
    match runtime.command_mode {
        DockerCommandMode::Rcon => validate_rcon_config(runtime),
        DockerCommandMode::DockerStdio => CONFIG_VALID.to_string(),
    }
}

fn validate_rcon_config(runtime: &DockerItzgRuntimeConfig) -> String {
    let Some(rcon) = runtime.rcon.as_ref() else {
        return CONFIG_VALID.to_string();
    };

    if rcon.host.trim().is_empty() {
        return "rcon host is empty".to_string();
    }
    if rcon.port == 0 {
        return "rcon port is zero".to_string();
    }
    if rcon.password.is_empty() {
        return "rcon password is empty".to_string();
    }

    CONFIG_VALID.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::server::{RconConfig, ServerRuntimeConfig};
    use std::collections::BTreeMap;
    use std::process::Output;

    fn docker_runtime(command_mode: DockerCommandMode) -> DockerItzgRuntimeConfig {
        DockerItzgRuntimeConfig {
            image: "itzg/minecraft-server".to_string(),
            image_tag: "java21".to_string(),
            container_name: "sea-test".to_string(),
            type_value: "PAPER".to_string(),
            version: "1.20.6".to_string(),
            data_dir_mount: "/data".to_string(),
            published_game_port: 25565,
            env: BTreeMap::new(),
            extra_ports: Vec::new(),
            volume_mounts: Vec::new(),
            docker_backend_kind: DockerBackendKind::Cli,
            command_mode,
            rcon: Some(RconConfig {
                host: "127.0.0.1".to_string(),
                port: 25575,
                password: "secret".to_string(),
            }),
            jvm_args: Vec::new(),
            cpu_policy: crate::models::server::CpuPolicyConfig::default(),
            jvm_preset: crate::models::server::JvmPresetConfig::default(),
        }
    }

    fn docker_server(runtime: DockerItzgRuntimeConfig) -> ServerInstance {
        ServerInstance {
            id: "server-1".to_string(),
            name: "Server 1".to_string(),
            aliases: vec!["docker-test".to_string()],
            core_type: "paper".to_string(),
            core_version: "latest".to_string(),
            mc_version: "1.20.6".to_string(),
            path: "servers/server-1".to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: DOCKER_ITZG_RUNTIME_KIND.to_string(),
            runtime: ServerRuntimeConfig::DockerItzg(runtime),
        }
    }

    #[test]
    fn docker_capability_error_contains_stable_fields() {
        let runtime = DockerItzgRuntime::new();
        let server = docker_server(docker_runtime(DockerCommandMode::Rcon));
        let message = runtime.capability_error(&server, DockerItzgCapability::Stop);

        assert!(message.contains("docker_itzg capability unavailable:"));
        assert!(message.contains("action=stop"));
        assert!(message.contains("docker_backend_kind=cli"));
        assert!(message.contains("command_mode=rcon"));
        assert!(message.contains(
            "backend_adapter=docker CLI adapter is unavailable for the requested runtime combination"
        ));
        assert!(message.contains("command_transport=docker CLI uses a real RCON client connection"));
        assert!(message.contains("config_validation=ok"));
    }

    #[test]
    fn docker_capability_error_for_runtime_mismatch_is_stable() {
        let runtime = DockerItzgRuntime::new();
        let server = ServerInstance {
            id: "local-1".to_string(),
            name: "Local 1".to_string(),
            aliases: vec![],
            core_type: "paper".to_string(),
            core_version: "latest".to_string(),
            mc_version: "1.20.6".to_string(),
            path: "servers/local-1".to_string(),
            port: 25565,
            max_memory: 2048,
            min_memory: 1024,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(crate::models::server::LocalRuntimeConfig {
                jar_path: "server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "java".to_string(),
                jvm_args: Vec::new(),
                cpu_policy: crate::models::server::CpuPolicyConfig::default(),
                jvm_preset: crate::models::server::JvmPresetConfig::default(),
            }),
        };

        let message = runtime.capability_error(&server, DockerItzgCapability::ForceStop);

        assert!(message.contains("action=force_stop"));
        assert!(message.contains("docker_backend_kind=unknown"));
        assert!(message.contains("command_mode=unknown"));
        assert!(message.contains("config_validation=server runtime config is not docker_itzg"));
        assert!(message.contains("runtime_kind=local"));
    }

    #[test]
    fn docker_cli_status_maps_running_state() {
        let snapshot = map_container_status(&DockerContainerState {
            status: "running".to_string(),
            running: true,
            pid: Some(123),
            exit_code: Some(0),
            health_status: None,
            error_message: None,
        });

        assert_eq!(snapshot, ServerStatus::Running);
    }

    #[test]
    fn docker_cli_status_prefers_healthy_healthcheck() {
        let snapshot = map_container_status(&DockerContainerState {
            status: "running".to_string(),
            running: true,
            pid: Some(123),
            exit_code: Some(0),
            health_status: Some("healthy".to_string()),
            error_message: None,
        });

        assert_eq!(snapshot, ServerStatus::Running);
    }

    #[test]
    fn docker_cli_status_maps_healthcheck_starting_to_starting() {
        let snapshot = map_container_status(&DockerContainerState {
            status: "running".to_string(),
            running: true,
            pid: Some(123),
            exit_code: Some(0),
            health_status: Some("starting".to_string()),
            error_message: None,
        });

        assert_eq!(snapshot, ServerStatus::Starting);
    }

    #[test]
    fn docker_cli_status_maps_unhealthy_container_to_error() {
        let snapshot = map_container_status(&DockerContainerState {
            status: "running".to_string(),
            running: true,
            pid: Some(123),
            exit_code: Some(0),
            health_status: Some("unhealthy".to_string()),
            error_message: None,
        });

        assert_eq!(snapshot, ServerStatus::Error);
    }

    #[test]
    fn docker_cli_status_maps_non_zero_exit_to_error() {
        let snapshot = map_container_status(&DockerContainerState {
            status: "exited".to_string(),
            running: false,
            pid: None,
            exit_code: Some(137),
            health_status: None,
            error_message: None,
        });

        assert_eq!(snapshot, ServerStatus::Error);
    }

    #[test]
    fn docker_cli_status_treats_signal_style_graceful_exit_codes_as_stopped() {
        let interrupted = map_container_status(&DockerContainerState {
            status: "exited".to_string(),
            running: false,
            pid: None,
            exit_code: Some(130),
            health_status: None,
            error_message: None,
        });
        let terminated = map_container_status(&DockerContainerState {
            status: "exited".to_string(),
            running: false,
            pid: None,
            exit_code: Some(143),
            health_status: None,
            error_message: None,
        });

        assert_eq!(interrupted, ServerStatus::Stopped);
        assert_eq!(terminated, ServerStatus::Stopped);
    }

    #[test]
    fn render_container_error_reports_unhealthy_running_container() {
        let runtime = docker_runtime(DockerCommandMode::Rcon);
        let state = DockerContainerState {
            status: "running".to_string(),
            running: true,
            pid: Some(123),
            exit_code: Some(0),
            health_status: Some("unhealthy".to_string()),
            error_message: None,
        };

        let error = render_container_error(&runtime, &state)
            .expect("unhealthy container should produce error message");
        assert!(error.contains("健康检查失败"));
        assert!(error.contains("health=unhealthy"));
    }

    #[test]
    fn render_container_error_ignores_signal_style_graceful_exit_codes() {
        let runtime = docker_runtime(DockerCommandMode::Rcon);
        let state = DockerContainerState {
            status: "exited".to_string(),
            running: false,
            pid: None,
            exit_code: Some(130),
            health_status: None,
            error_message: None,
        };

        assert_eq!(render_container_error(&runtime, &state), None);
    }

    #[test]
    fn render_container_detail_reports_health_and_exit_code() {
        let runtime = docker_runtime(DockerCommandMode::DockerStdio);
        let detail = render_container_detail(
            &runtime,
            &DockerContainerState {
                status: "running".to_string(),
                running: true,
                pid: Some(456),
                exit_code: Some(0),
                health_status: Some("healthy".to_string()),
                error_message: None,
            },
        );

        assert!(detail.contains("runtime=docker_itzg"));
        assert!(detail.contains("container=sea-test"));
        assert!(detail.contains("health=healthy"));
        assert!(detail.contains("command_mode=docker_stdio"));
    }

    #[test]
    fn render_send_command_precondition_stopped_includes_actionable_state() {
        let runtime = docker_runtime(DockerCommandMode::Rcon);
        let server = docker_server(docker_runtime(DockerCommandMode::Rcon));
        let message = render_send_command_precondition_stopped(
            &server,
            &runtime,
            &DockerContainerState {
                status: "exited".to_string(),
                running: false,
                pid: None,
                exit_code: Some(137),
                health_status: None,
                error_message: None,
            },
        );

        assert!(message.contains("当前不可接收命令"));
        assert!(message.contains("state=exited"));
        assert!(message.contains("exit_code=137"));
        assert!(message.contains("sealantern server status server-1"));
        assert!(message.contains("sealantern server logs server-1 --lines 50"));
    }

    #[test]
    fn render_send_command_precondition_not_ready_for_health_starting_points_to_waiting() {
        let runtime = docker_runtime(DockerCommandMode::Rcon);
        let server = docker_server(docker_runtime(DockerCommandMode::Rcon));
        let message = render_send_command_precondition_not_ready(
            &server,
            &runtime,
            &DockerContainerState {
                status: "running".to_string(),
                running: true,
                pid: Some(123),
                exit_code: Some(0),
                health_status: Some("starting".to_string()),
                error_message: None,
            },
        )
        .expect("starting health should reject commands");

        assert!(message.contains("当前不可接收命令"));
        assert!(message.contains("health=starting"));
        assert!(message.contains("health=healthy 后再重试"));
        assert!(message.contains("sealantern server status server-1"));
    }

    #[test]
    fn render_send_command_precondition_not_ready_for_unhealthy_points_to_logs() {
        let runtime = docker_runtime(DockerCommandMode::DockerStdio);
        let server = docker_server(docker_runtime(DockerCommandMode::DockerStdio));
        let message = render_send_command_precondition_not_ready(
            &server,
            &runtime,
            &DockerContainerState {
                status: "running".to_string(),
                running: true,
                pid: Some(123),
                exit_code: Some(0),
                health_status: Some("unhealthy".to_string()),
                error_message: None,
            },
        )
        .expect("unhealthy health should reject commands");

        assert!(message.contains("当前不可接收命令"));
        assert!(message.contains("health=unhealthy"));
        assert!(message.contains("sealantern server logs server-1 --lines 50"));
    }

    #[test]
    fn render_send_command_precondition_not_ready_allows_healthy_or_missing_health() {
        let runtime = docker_runtime(DockerCommandMode::Rcon);
        let server = docker_server(docker_runtime(DockerCommandMode::Rcon));

        assert!(render_send_command_precondition_not_ready(
            &server,
            &runtime,
            &DockerContainerState {
                status: "running".to_string(),
                running: true,
                pid: Some(123),
                exit_code: Some(0),
                health_status: Some("healthy".to_string()),
                error_message: None,
            },
        )
        .is_none());

        assert!(render_send_command_precondition_not_ready(
            &server,
            &runtime,
            &DockerContainerState {
                status: "running".to_string(),
                running: true,
                pid: Some(123),
                exit_code: Some(0),
                health_status: None,
                error_message: None,
            },
        )
        .is_none());
    }

    #[test]
    fn render_rcon_connect_error_includes_operator_guidance() {
        let runtime = docker_runtime(DockerCommandMode::Rcon);
        let message = render_rcon_connect_error(&runtime, "127.0.0.1:25575", "timeout");

        assert!(message.contains("通过 RCON 连接 Docker 容器失败"));
        assert!(message.contains("endpoint=127.0.0.1:25575"));
        assert!(message.contains("切换到 --command-mode docker_stdio"));
    }

    #[test]
    fn container_should_clear_starting_for_running_container_without_healthcheck() {
        assert!(container_should_clear_starting(&DockerContainerState {
            status: "running".to_string(),
            running: true,
            pid: Some(123),
            exit_code: Some(0),
            health_status: None,
            error_message: None,
        }));

        assert!(!container_should_clear_starting(&DockerContainerState {
            status: "running".to_string(),
            running: true,
            pid: Some(123),
            exit_code: Some(0),
            health_status: Some("starting".to_string()),
            error_message: None,
        }));

        assert!(container_should_clear_starting(&DockerContainerState {
            status: "running".to_string(),
            running: true,
            pid: Some(123),
            exit_code: Some(0),
            health_status: Some("healthy".to_string()),
            error_message: None,
        }));
    }

    #[test]
    fn resolve_managed_status_only_keeps_starting_overlay_while_runtime_is_not_ready() {
        assert_eq!(
            resolve_managed_status(ServerStatus::Running, true, false),
            ServerStatus::Starting
        );
        assert_eq!(
            resolve_managed_status(ServerStatus::Starting, false, false),
            ServerStatus::Starting
        );
        assert_eq!(
            resolve_managed_status(ServerStatus::Running, false, false),
            ServerStatus::Running
        );
        assert_eq!(
            resolve_managed_status(ServerStatus::Running, true, true),
            ServerStatus::Stopping
        );
    }

    #[test]
    fn resolve_managed_status_terminal_result_should_clear_runtime_flags() {
        let manager = ServerManager::new();
        manager.mark_starting("docker-alpha");
        manager.mark_stopping("docker-alpha");

        control::clear_runtime_flags_if_terminal(&manager, "docker-alpha", &ServerStatus::Error);

        assert!(!manager.is_starting("docker-alpha"));
        assert!(!manager.is_stopping("docker-alpha"));
    }

    #[test]
    fn docker_status_is_not_running_treats_exited_error_as_already_stopped_for_stop_flow() {
        let stopped = RuntimeStatusSnapshot {
            status: ServerStatus::Stopped,
            pid: None,
            detail_message: Some(
                "runtime=docker_itzg container=sea-test state=missing".to_string(),
            ),
            error_message: None,
        };
        let exited_error = RuntimeStatusSnapshot {
            status: ServerStatus::Error,
            pid: None,
            detail_message: Some(
                "runtime=docker_itzg container=sea-test state=exited running=false health=none exit_code=137 backend=cli command_mode=rcon"
                    .to_string(),
            ),
            error_message: Some("Docker 容器已退出".to_string()),
        };
        let unhealthy_running = RuntimeStatusSnapshot {
            status: ServerStatus::Error,
            pid: Some(123),
            detail_message: Some(
                "runtime=docker_itzg container=sea-test state=running running=true health=unhealthy exit_code=0 backend=cli command_mode=rcon"
                    .to_string(),
            ),
            error_message: Some("Docker 容器健康检查失败".to_string()),
        };

        assert!(docker_status_is_not_running(&stopped));
        assert!(docker_status_is_not_running(&exited_error));
        assert!(!docker_status_is_not_running(&unhealthy_running));
    }

    #[test]
    fn docker_exec_missing_mc_send_to_console_detects_common_not_found_errors() {
        assert!(docker_exec_missing_mc_send_to_console(
            "exec: \"mc-send-to-console\": executable file not found in $PATH"
        ));
        assert!(docker_exec_missing_mc_send_to_console("sh: mc-send-to-console: not found"));
        assert!(!docker_exec_missing_mc_send_to_console(
            "permission denied while trying to connect to the Docker daemon socket"
        ));
    }

    #[test]
    fn docker_exec_console_pipe_related_errors_are_classified() {
        assert!(docker_exec_requires_console_pipe(
            "Console pipe needs to be enabled by setting CREATE_CONSOLE_IN_PIPE to true"
        ));
        assert!(docker_exec_named_pipe_missing(
            "Named pipe /tmp/minecraft-console-in is missing"
        ));
        assert!(docker_exec_requires_uid("Exec needs to be run with user ID 1000"));
    }

    #[test]
    fn docker_output_indicates_missing_container_detects_stdout_and_stderr_variants() {
        let stderr_only = Output {
            status: exit_status_failure(),
            stdout: Vec::new(),
            stderr: b"Error response from daemon: No such container: sea-test".to_vec(),
        };
        let stdout_only = Output {
            status: exit_status_failure(),
            stdout: b"Error: no such object: sea-test".to_vec(),
            stderr: Vec::new(),
        };

        assert!(docker_output_indicates_missing_container(&stderr_only));
        assert!(docker_output_indicates_missing_container(&stdout_only));
    }

    #[test]
    fn docker_cli_env_injects_required_runtime_values() {
        let mut runtime = docker_runtime(DockerCommandMode::DockerStdio);
        runtime.env.insert("EULA".to_string(), "FALSE".to_string());

        let (env, _) = build_effective_env(&runtime).unwrap();

        assert!(env
            .iter()
            .any(|(key, value)| key == "TYPE" && value == "PAPER"));
        assert!(env
            .iter()
            .any(|(key, value)| key == "VERSION" && value == "1.20.6"));
        assert!(env
            .iter()
            .any(|(key, value)| key == "EULA" && value == "FALSE"));
        assert!(env
            .iter()
            .any(|(key, value)| key == "CREATE_CONSOLE_IN_PIPE" && value == "true"));
    }

    #[test]
    fn build_docker_run_args_includes_cpuset_for_count_policy() {
        let mut runtime = docker_runtime(DockerCommandMode::DockerStdio);
        runtime.cpu_policy = crate::models::server::CpuPolicyConfig {
            mode: crate::models::server::CpuPolicyMode::Count,
            count: Some(4),
            explicit_set: None,
            sync_active_processor_count: true,
        };

        let launch_spec = resolve_docker_launch_spec(&runtime).unwrap();
        let args = build_docker_run_args(&runtime, &launch_spec);

        let cpuset_index = args
            .iter()
            .position(|arg| arg == "--cpuset-cpus")
            .expect("cpuset flag should exist");
        assert_eq!(args[cpuset_index + 1], "0-3");
    }

    #[test]
    fn build_docker_run_args_omits_cpuset_for_off_policy() {
        let runtime = docker_runtime(DockerCommandMode::Rcon);
        let launch_spec = resolve_docker_launch_spec(&runtime).unwrap();
        let args = build_docker_run_args(&runtime, &launch_spec);

        assert!(!args.iter().any(|arg| arg == "--cpuset-cpus"));
    }

    #[test]
    fn requested_stop_timeout_secs_reads_positive_stop_duration_env() {
        let mut runtime = docker_runtime(DockerCommandMode::Rcon);
        runtime
            .env
            .insert("STOP_DURATION".to_string(), "180".to_string());

        assert_eq!(requested_stop_timeout_secs(&runtime), Some(180));
    }

    #[test]
    fn requested_stop_timeout_secs_ignores_zero_or_invalid_values() {
        let mut zero = docker_runtime(DockerCommandMode::Rcon);
        zero.env
            .insert("STOP_DURATION".to_string(), "0".to_string());
        assert_eq!(requested_stop_timeout_secs(&zero), None);

        let mut invalid = docker_runtime(DockerCommandMode::Rcon);
        invalid
            .env
            .insert("STOP_DURATION".to_string(), "abc".to_string());
        assert_eq!(requested_stop_timeout_secs(&invalid), None);
    }

    #[test]
    fn docker_command_transport_message_matches_rcon_mode() {
        let runtime = docker_runtime(DockerCommandMode::Rcon);

        assert_eq!(
            command_transport_message(&runtime),
            "docker CLI uses a real RCON client connection"
        );
    }

    #[test]
    fn docker_command_transport_message_matches_stdio_mode() {
        let runtime = docker_runtime(DockerCommandMode::DockerStdio);

        assert_eq!(
            command_transport_message(&runtime),
            "docker CLI sends commands via mc-send-to-console inside the container"
        );
    }

    #[test]
    fn docker_capability_error_for_engine_api_remains_explicit() {
        let mut runtime = docker_runtime(DockerCommandMode::DockerStdio);
        runtime.docker_backend_kind = DockerBackendKind::EngineApi;
        let server = docker_server(runtime);
        let adapter = DockerItzgRuntime::new();

        let message = adapter.capability_error(&server, DockerItzgCapability::Start);

        assert!(message.contains("docker_backend_kind=engine_api"));
        assert!(message
            .contains("backend_adapter=docker Engine API adapter skeleton is not implemented yet"));
    }

    #[test]
    fn docker_command_error_classifies_network_pull_failure() {
        let failure = crate::utils::docker_cli::classify_docker_command_failure(
            "failed to resolve reference \"docker.io/itzg/minecraft-server:latest\": dial tcp 1.2.3.4:443: connectex: A connection attempt failed",
        );

        assert_eq!(failure, crate::utils::docker_cli::DockerCommandFailureKind::Network);
    }

    #[test]
    fn docker_command_error_classifies_manifest_unknown_as_image_not_found() {
        let failure = crate::utils::docker_cli::classify_docker_command_failure(
            "manifest unknown: manifest unknown",
        );

        assert_eq!(failure, crate::utils::docker_cli::DockerCommandFailureKind::ImageNotFound);
    }

    fn exit_status_failure() -> std::process::ExitStatus {
        #[cfg(windows)]
        {
            std::os::windows::process::ExitStatusExt::from_raw(1)
        }
        #[cfg(unix)]
        {
            std::os::unix::process::ExitStatusExt::from_raw(1)
        }
    }
}
