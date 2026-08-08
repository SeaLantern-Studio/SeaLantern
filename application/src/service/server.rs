//! 服务器进程管理服务实现。
//!
//! 实现 [`sealantern_interface::ServerService`] 能力端口，管理实例对应的
//! 服务器进程生命周期（启动/停止/强制停止/状态/控制台命令）。
//!
//! 进程管理基于 `core` 的 `process` 原语（[`Daemon`]、[`Terminal`]、
//! [`build_command`]）；启动配置来自实例记录（[`CoreInstanceService`]）。
//!
//! 错误分层：内部以应用层主错误 [`ServerError`] 为源头，暴露
//! [`ServerService`] 时统一转为接口契约错误 [`ServerServiceError`]。

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use sealantern_core::instance::{Instance, InstanceId, StartupMode};
use sealantern_core::process::{
    build_command, CommandBuildMode, CommandBuildRequest, Daemon, Terminal, TerminalOutput,
    TerminalStream,
};
use sealantern_interface::server::{ServerSnapshot, ServerState};
use sealantern_interface::{InstanceService, ServerService, ServerServiceError};

use crate::error::ServerError;

use super::CoreInstanceService;

/// 优雅停止时等待进程退出的最长时长。
const STOP_GRACEFUL_TIMEOUT: Duration = Duration::from_secs(10);
/// 状态轮询间隔。
const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// 一个受管服务器进程：守护进程 + 已转移的标准流终端。
struct ManagedProcess {
    daemon: Daemon,
    terminal: Terminal,
}

/// 基于 `core` 进程原语的服务器进程管理服务实现。
pub struct CoreServerService {
    /// 实例记录服务（读取启动配置与更新状态）。
    instance_service: Arc<CoreInstanceService>,
    /// 进程注册表：实例 ID → 受管进程。
    processes: Mutex<HashMap<String, ManagedProcess>>,
    /// 启动中的实例集合。
    starting: Mutex<HashSet<String>>,
    /// 停止中的实例集合。
    stopping: Mutex<HashSet<String>>,
}

impl CoreServerService {
    /// 构造服务器进程管理服务。
    pub fn new(instance_service: Arc<CoreInstanceService>) -> Self {
        Self {
            instance_service,
            processes: Mutex::new(HashMap::new()),
            starting: Mutex::new(HashSet::new()),
            stopping: Mutex::new(HashSet::new()),
        }
    }

    /// 按实例 ID 查找实例记录。
    async fn find_instance(&self, id: &InstanceId) -> Result<Instance, ServerError> {
        self.instance_service
            .find(id)
            .await
            .map_err(|_| ServerError::OperationFailed {
                source: Box::new(std::io::Error::other("instance lookup failed")),
            })?
            .ok_or(ServerError::InstanceNotFound)
    }

    /// 由实例启动配置构建进程命令。
    fn build_process_command(
        &self,
        instance: &Instance,
    ) -> Result<std::process::Command, ServerError> {
        let launch = &instance.launch;
        let mode = match launch.startup_mode {
            StartupMode::Jar | StartupMode::Starter => CommandBuildMode::DirectJar,
            StartupMode::Batch => CommandBuildMode::Batch,
            StartupMode::Shell => CommandBuildMode::Shell,
            StartupMode::PowerShell => CommandBuildMode::PowerShell,
            StartupMode::Custom => CommandBuildMode::Custom,
        };

        let request = CommandBuildRequest {
            mode,
            working_directory: Path::new(&instance.directory),
            java_executable: launch.java_executable.as_deref(),
            java_environment: None,
            jvm_arguments: &[],
            entry_path: launch.startup_target.as_deref(),
            custom_command: launch.custom_command.as_deref(),
            custom_executable: launch.custom_executable.as_deref(),
            custom_arguments: &[],
            installer_url: None,
            windows_console_encoding: sealantern_core::process::WindowsConsoleEncoding::Utf8,
        };

        build_command(&request).map_err(|e| ServerError::OperationFailed { source: Box::new(e) })
    }

    fn processes_lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, ManagedProcess>>, ServerError> {
        self.processes.lock().map_err(|_| ServerError::Internal {
            source: Box::new(std::io::Error::other("processes lock poisoned")),
        })
    }

    fn is_starting(&self, id: &str) -> bool {
        self.starting
            .lock()
            .map(|s| s.contains(id))
            .unwrap_or(false)
    }

    fn is_stopping(&self, id: &str) -> bool {
        self.stopping
            .lock()
            .map(|s| s.contains(id))
            .unwrap_or(false)
    }

    fn mark_starting(&self, id: &str) {
        if let Ok(mut s) = self.starting.lock() {
            s.insert(id.to_string());
        }
    }

    fn clear_starting(&self, id: &str) {
        if let Ok(mut s) = self.starting.lock() {
            s.remove(id);
        }
    }

    fn mark_stopping(&self, id: &str) {
        if let Ok(mut s) = self.stopping.lock() {
            s.insert(id.to_string());
        }
    }

    fn clear_stopping(&self, id: &str) {
        if let Ok(mut s) = self.stopping.lock() {
            s.remove(id);
        }
    }
}

#[async_trait]
impl ServerService for CoreServerService {
    async fn status(&self, id: &InstanceId) -> Result<ServerSnapshot, ServerServiceError> {
        // 确认实例存在，并复用实例信息避免重复查询与状态不一致。
        let instance = self.find_instance(id).await?;
        let id_str = id.as_str().to_string();
        let started_at = instance.last_started_at_unix_secs;

        let mut processes = self.processes_lock()?;
        if let Some(managed) = processes.get_mut(&id_str) {
            let is_running = managed
                .daemon
                .poll()
                .map(|status| status.is_none())
                .unwrap_or(false);

            let state = if self.is_stopping(&id_str) {
                ServerState::Stopping
            } else if is_running && self.is_starting(&id_str) {
                ServerState::Starting
            } else if is_running {
                ServerState::Running
            } else {
                ServerState::Stopped
            };

            return Ok(ServerSnapshot {
                instance_id: id_str,
                state,
                pid: if is_running {
                    Some(managed.daemon.id())
                } else {
                    None
                },
                uptime_secs: started_at.and_then(|t| current_timestamp_secs().checked_sub(t)),
                error_message: None,
            });
        }

        Ok(ServerSnapshot {
            instance_id: id_str,
            state: ServerState::Stopped,
            pid: None,
            uptime_secs: None,
            error_message: None,
        })
    }

    async fn start(&self, id: &InstanceId) -> Result<(), ServerServiceError> {
        let instance = self.find_instance(id).await?;
        let id_str = id.as_str().to_string();

        {
            let mut processes = self.processes_lock()?;
            if let Some(managed) = processes.get_mut(&id_str) {
                if managed.daemon.poll().map(|s| s.is_none()).unwrap_or(false) {
                    return Err(ServerError::InvalidState.into());
                }
                processes.remove(&id_str);
            }
        }

        let mut command = self.build_process_command(&instance)?;
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        let mut daemon = Daemon::spawn(&mut command)
            .map_err(|e| ServerError::OperationFailed { source: Box::new(e) })?;
        let terminal = Terminal::from_daemon_with_input(&mut daemon, true);

        self.mark_starting(&id_str);
        self.processes_lock()?
            .insert(id_str.clone(), ManagedProcess { daemon, terminal });

        // 更新实例的最后启动时间。
        let _ = self
            .instance_service
            .update_last_started(id)
            .await
            .map_err(|_| ServerError::OperationFailed {
                source: Box::new(std::io::Error::other("failed to update last started")),
            })?;

        // 后台读取进程输出，避免管道阻塞（当前先消费，后续接入日志管线）。
        if let Some(managed) = self.processes_lock()?.get_mut(&id_str) {
            let stdout = managed.terminal.take_output(TerminalStream::Stdout);
            let stderr = managed.terminal.take_output(TerminalStream::Stderr);
            spawn_output_reader(id_str.clone(), stdout, stderr);
        }

        // 进程已成功拉起并注册，退出 Starting 状态（Starting 仅覆盖 spawn 竞态窗口）。
        self.clear_starting(&id_str);

        Ok(())
    }

    async fn stop(&self, id: &InstanceId) -> Result<(), ServerServiceError> {
        let id_str = id.as_str().to_string();
        self.find_instance(id).await?;

        // 优雅停止：向控制台发送 stop，等待退出。
        let _ = self.send_command_inner(id, "stop").await;
        self.mark_stopping(&id_str);

        let deadline = Instant::now() + STOP_GRACEFUL_TIMEOUT;
        loop {
            {
                let mut processes = self.processes_lock()?;
                if let Some(managed) = processes.get_mut(&id_str) {
                    if managed.daemon.poll().map(|s| s.is_some()).unwrap_or(true) {
                        processes.remove(&id_str);
                        self.clear_stopping(&id_str);
                        return Ok(());
                    }
                } else {
                    self.clear_stopping(&id_str);
                    return Ok(());
                }
            }
            if Instant::now() >= deadline {
                break;
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }

        // 超时：强制终止进程树。
        let mut processes = self.processes_lock()?;
        if let Some(mut managed) = processes.remove(&id_str) {
            managed
                .daemon
                .terminate_tree()
                .map_err(|e| ServerError::OperationFailed { source: Box::new(e) })?;
        }
        self.clear_stopping(&id_str);
        Ok(())
    }

    async fn force_stop(&self, id: &InstanceId) -> Result<(), ServerServiceError> {
        let id_str = id.as_str().to_string();
        self.find_instance(id).await?;

        let mut processes = self.processes_lock()?;
        if let Some(mut managed) = processes.remove(&id_str) {
            managed
                .daemon
                .terminate_tree()
                .map_err(|e| ServerError::OperationFailed { source: Box::new(e) })?;
        }
        self.clear_starting(&id_str);
        self.clear_stopping(&id_str);
        Ok(())
    }

    async fn send_command(&self, id: &InstanceId, command: &str) -> Result<(), ServerServiceError> {
        self.send_command_inner(id, command).await
    }
}

impl CoreServerService {
    /// 向服务器控制台发送命令（内部实现）。
    async fn send_command_inner(
        &self,
        id: &InstanceId,
        command: &str,
    ) -> Result<(), ServerServiceError> {
        let id_str = id.as_str().to_string();
        let mut processes = self.processes_lock()?;
        let Some(managed) = processes.get_mut(&id_str) else {
            return Err(ServerError::InvalidState.into());
        };
        if managed.daemon.poll().map(|s| s.is_some()).unwrap_or(true) {
            return Err(ServerError::InvalidState.into());
        }

        // 写入 stdin 是短同步 IO（单行命令），直接持有锁执行。
        managed
            .terminal
            .write_line(command)
            .map_err(|e| ServerError::OperationFailed { source: Box::new(e) })?;
        Ok(())
    }
}

/// 后台读取进程输出流，防止管道阻塞。
///
/// 当前仅消费输出；后续接入日志管线（SQLite / 流式推送）时在此扩展。
fn spawn_output_reader(
    instance_id: String,
    stdout: Option<TerminalOutput>,
    stderr: Option<TerminalOutput>,
) {
    std::thread::spawn(move || {
        let mut readers: Vec<TerminalOutput> = stdout.into_iter().chain(stderr).collect();

        while !readers.is_empty() {
            let mut next = Vec::with_capacity(readers.len());
            for mut reader in readers {
                let mut buffer = [0u8; 4096];
                match std::io::Read::read(&mut reader, &mut buffer) {
                    Ok(0) => {} // EOF：丢弃该流。
                    Ok(n) => {
                        // 当前仅消费输出以防管道阻塞；debug 级记录便于诊断进程问题。
                        let text = String::from_utf8_lossy(&buffer[..n]);
                        tracing::debug!(
                            target: "sealantern.application.server",
                            instance_id,
                            output = %text.trim_end(),
                            "server process output"
                        );
                        next.push(reader);
                    }
                    Err(_) => {} // 读取错误：丢弃该流。
                }
            }
            readers = next;
            std::thread::sleep(POLL_INTERVAL);
        }
    });
}

/// 当前 Unix 时间戳（秒）。
fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
