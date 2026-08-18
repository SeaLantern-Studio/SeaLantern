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
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use sealantern_core::instance::{
    Instance, InstanceId, InstanceLifecycleState, InstanceRestartDriver, RestartPolicy,
    StartupMode, restart_instance,
};
use sealantern_core::process::{
    CommandBuildMode, CommandBuildRequest, Daemon, JavaEnvironment, Terminal, TerminalStream,
    WindowsConsoleEncoding, build_command,
};
use sealantern_extra::java::{JavaInfo, detect_java_installations};
use sealantern_interface::server::{ServerSnapshot, ServerState};
use sealantern_interface::{InstanceService, ServerService, ServerServiceError, SettingsService};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

use crate::error::ServerError;

use super::{CoreInstanceService, CoreSettingsService, LogRecorder};

/// 优雅停止时等待进程退出的最长时长。
const STOP_GRACEFUL_TIMEOUT: Duration = Duration::from_secs(10);
/// 状态轮询间隔。
const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// 一个受管服务器进程：守护进程 + 已转移的标准流终端 + 日志记录管线。
struct ManagedProcess {
    daemon: Daemon,
    terminal: Terminal,
    recorder: Option<LogRecorder>,
}

struct ServerRestartDriver<'a> {
    service: &'a CoreServerService,
    _lifecycle_guard: &'a OwnedMutexGuard<()>,
}

#[async_trait]
impl InstanceRestartDriver for ServerRestartDriver<'_> {
    type Error = ServerServiceError;

    async fn state(&self, instance: &Instance) -> Result<InstanceLifecycleState, Self::Error> {
        let snapshot = self.service.status_for_instance(instance)?;
        let state = match snapshot.state {
            ServerState::Starting => InstanceLifecycleState::Starting,
            ServerState::Running => InstanceLifecycleState::Running,
            ServerState::Stopping => InstanceLifecycleState::Stopping,
            ServerState::Stopped => InstanceLifecycleState::Stopped,
        };
        if !state.is_active() {
            self.service.clear_lifecycle_flags(instance.id.as_str());
        }
        Ok(state)
    }

    async fn request_stop(&self, instance: &Instance) -> Result<(), Self::Error> {
        self.service.request_stop_for_restart(&instance.id).await
    }

    async fn await_terminal(
        &self,
        instance: &Instance,
        timeout: Duration,
    ) -> Result<InstanceLifecycleState, Self::Error> {
        let deadline = Instant::now() + timeout;
        loop {
            let state = self.state(instance).await?;
            if !state.is_active() || Instant::now() >= deadline {
                return Ok(state);
            }
            tokio::time::sleep(
                POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now())),
            )
            .await;
        }
    }

    async fn start(&self, instance: &Instance) -> Result<(), Self::Error> {
        self.service.start_unlocked(&instance.id, instance).await
    }
}

/// 基于 `core` 进程原语的服务器进程管理服务实现。
pub struct CoreServerService {
    /// 实例记录服务（读取启动配置与更新状态）。
    instance_service: Arc<CoreInstanceService>,
    /// 设置信息服务（读取控制台等全局配置）。
    settings_service: Arc<CoreSettingsService>,
    /// 进程注册表：实例 ID → 受管进程。
    processes: Mutex<HashMap<String, ManagedProcess>>,
    /// 启动中的实例集合。
    starting: Mutex<HashSet<String>>,
    /// 停止中的实例集合。
    stopping: Mutex<HashSet<String>>,
    /// 每个实例独立的生命周期操作锁。
    lifecycle_locks: Mutex<HashMap<String, Weak<AsyncMutex<()>>>>,
    /// 探测到并已按主版本降序缓存的本机 Java 安装列表；首次探测到结果（含“无可用
    /// Java”的空列表）时填充，之后复用，避免每次启动都扫描文件系统。`spawn_blocking`
    /// 任务被取消或 panic（探测任务本身失败）时不写入缓存，下次启动可重试。
    java_installations: Mutex<Option<Vec<JavaInfo>>>,
}

impl CoreServerService {
    /// 构造服务器进程管理服务。
    pub fn new(
        instance_service: Arc<CoreInstanceService>,
        settings_service: Arc<CoreSettingsService>,
    ) -> Self {
        Self {
            instance_service,
            settings_service,
            processes: Mutex::new(HashMap::new()),
            starting: Mutex::new(HashSet::new()),
            stopping: Mutex::new(HashSet::new()),
            lifecycle_locks: Mutex::new(HashMap::new()),
            java_installations: Mutex::new(None),
        }
    }

    async fn lock_lifecycle(&self, id: &InstanceId) -> Result<OwnedMutexGuard<()>, ServerError> {
        let lock = {
            let mut locks = self
                .lifecycle_locks
                .lock()
                .map_err(|_| ServerError::Internal {
                    source: Box::new(std::io::Error::other("lifecycle locks poisoned")),
                })?;
            // 失效弱引用只在对应实例再次访问时覆盖，避免每次获取都扫描整张表。
            if let Some(lock) = locks.get(id.as_str()).and_then(Weak::upgrade) {
                lock
            } else {
                let lock = Arc::new(AsyncMutex::new(()));
                locks.insert(id.as_str().to_owned(), Arc::downgrade(&lock));
                lock
            }
        };
        Ok(lock.lock_owned().await)
    }

    /// 按实例 ID 查找实例记录。
    async fn find_instance(&self, id: &InstanceId) -> Result<Instance, ServerError> {
        self.instance_service
            .find(id)
            .await
            .map_err(|e| ServerError::OperationFailed {
                source: Box::new(std::io::Error::other(format!("instance lookup failed: {e}"))),
            })?
            .ok_or(ServerError::InstanceNotFound)
    }

    /// 从进程表中移除指定实例的受管进程并取出日志管线（不检查状态）。
    ///
    /// 供强制终止等"无论进程状态都移除"的路径使用；锁只取一次。
    fn take_recorder_unconditional(
        &self,
        id: &str,
    ) -> Result<Option<LogRecorder>, ServerServiceError> {
        let mut processes = self.processes_lock()?;
        Ok(processes
            .remove(id)
            .and_then(|mut managed| managed.recorder.take()))
    }

    /// 若实例进程已退出，移除受管进程并取出日志管线；否则返回 `None`。
    ///
    /// 供轮询确认退出后的清理路径使用；进程仍在运行时不移除，锁只取一次。
    fn take_recorder_after_exit(
        &self,
        id: &str,
    ) -> Result<Option<LogRecorder>, ServerServiceError> {
        let mut processes = self.processes_lock()?;
        let Some(managed) = processes.get_mut(id) else {
            return Ok(None);
        };
        if managed
            .daemon
            .poll()
            .map(|status| status.is_some())
            .unwrap_or(true)
        {
            Ok(processes
                .remove(id)
                .and_then(|mut managed| managed.recorder.take()))
        } else {
            Ok(None)
        }
    }

    /// 解析实例实际使用的 Java 可执行文件，缺失时探测本机 Java 并缓存结果。
    ///
    /// 优先使用实例已配置的可执行文件；为 `None` 时回退到本机探测到的最高版本
    /// Java（`detect_java_installations` 结果已按主版本降序排列）。探测结果在
    /// 进程内缓存，避免每次启动都扫描文件系统；探测失败（任务被取消 / 无可用
    /// Java）不写入缓存，下次启动可重试。
    async fn resolve_java_executable(&self, configured: &Option<PathBuf>) -> Option<PathBuf> {
        if let Some(configured) = configured {
            return Some(configured.clone());
        }
        // 命中缓存直接返回，跳过文件系统扫描。
        if let Ok(cache) = self.java_installations.lock()
            && let Some(installations) = cache.as_ref()
        {
            return installations.first().map(|info| PathBuf::from(&info.path));
        }
        // 缓存未命中：探测本机 Java 安装（阻塞扫描，调度到阻塞线程池）。
        let detected = match tokio::task::spawn_blocking(detect_java_installations).await {
            Ok(installations) => installations,
            Err(error) => {
                tracing::warn!("java detection task failed: {error}");
                return None;
            }
        };
        // 探测成功（含空列表）即写入缓存，跳过下一次启动的文件系统扫描；
        // 任务被取消 / panic 的分支已在上方提前返回，不会写入缓存，下次可重试。
        if let Ok(mut cache) = self.java_installations.lock() {
            *cache = Some(detected.clone());
        }
        detected.first().map(|info| PathBuf::from(&info.path))
    }

    /// 由实例启动配置构建进程命令。
    ///
    /// 若实例未显式配置 Java 可执行文件（如导入已有服务器时未指定），则探测
    /// 本机已安装的 Java 并取版本最高者作为回退，避免「a Java executable is
    /// required」启动失败。探测为阻塞的文件系统扫描，经 `spawn_blocking`
    /// 调度到阻塞线程池，避免阻塞异步运行时核心线程。
    async fn build_process_command(
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

        // 组装 JVM 参数（Xmx/Xms/编码 + 实例自定义参数）。
        // 默认 JVM 参数（全局配置）当前为空，待配置功能恢复后接入。
        let jvm_arguments = build_jvm_arguments(instance, "");
        let custom_arguments = launch
            .custom_arguments
            .iter()
            .map(OsString::from)
            .collect::<Vec<_>>();

        // 解析实际使用的 Java 可执行文件：优先使用实例已配置的，缺失时回退到
        // 本机探测到的最高版本 Java（探测结果已按主版本降序排列，经缓存复用）。
        let effective_java = self.resolve_java_executable(&launch.java_executable).await;

        // 从 Java 可执行文件推导 JAVA_HOME / bin（供脚本与自定义模式注入环境）。
        let java_environment = effective_java
            .as_deref()
            .map(JavaEnvironment::from_java_executable)
            .transpose()
            .map_err(|e| ServerError::OperationFailed { source: Box::new(e) })?;

        let request = CommandBuildRequest {
            mode,
            working_directory: Path::new(&instance.directory),
            java_executable: effective_java.as_deref(),
            java_environment: java_environment.as_ref(),
            jvm_arguments: &jvm_arguments,
            entry_path: launch.startup_target.as_deref(),
            custom_command: launch.custom_command.as_deref(),
            custom_executable: launch.custom_executable.as_deref(),
            custom_arguments: &custom_arguments,
            installer_url: None,
            windows_console_encoding: WindowsConsoleEncoding::Utf8,
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

    fn clear_lifecycle_flags(&self, id: &str) {
        self.clear_starting(id);
        self.clear_stopping(id);
    }

    fn status_for_instance(
        &self,
        instance: &Instance,
    ) -> Result<ServerSnapshot, ServerServiceError> {
        let id_str = instance.id.as_str().to_owned();
        let started_at = instance.last_started_at_unix_secs;
        if let Some(recorder) = self.take_recorder_after_exit(&id_str)? {
            spawn_recorder_shutdown(Some(recorder));
            return Ok(ServerSnapshot {
                instance_id: id_str,
                state: ServerState::Stopped,
                pid: None,
                uptime_secs: None,
                error_message: None,
            });
        }
        let mut processes = self.processes_lock()?;
        if let Some(managed) = processes.get_mut(&id_str) {
            let state = if self.is_stopping(&id_str) {
                ServerState::Stopping
            } else if self.is_starting(&id_str) {
                ServerState::Starting
            } else {
                ServerState::Running
            };
            return Ok(ServerSnapshot {
                instance_id: id_str,
                state,
                pid: Some(managed.daemon.id()),
                uptime_secs: started_at.and_then(|time| current_timestamp_secs().checked_sub(time)),
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
}

#[async_trait]
impl ServerService for CoreServerService {
    async fn status(&self, id: &InstanceId) -> Result<ServerSnapshot, ServerServiceError> {
        let instance = self.find_instance(id).await?;
        self.status_for_instance(&instance)
    }

    async fn start(&self, id: &InstanceId) -> Result<(), ServerServiceError> {
        let _guard = self.lock_lifecycle(id).await?;
        let instance = self.find_instance(id).await?;
        self.start_unlocked(id, &instance).await
    }

    async fn restart(&self, id: &InstanceId) -> Result<(), ServerServiceError> {
        let lifecycle_guard = self.lock_lifecycle(id).await?;
        let instance = self.find_instance(id).await?;
        let driver = ServerRestartDriver {
            service: self,
            _lifecycle_guard: &lifecycle_guard,
        };
        restart_instance(&driver, &instance, RestartPolicy { stop_timeout: STOP_GRACEFUL_TIMEOUT })
            .await
            .map_err(|error| {
                tracing::error!(
                    target: "sealantern.application.server",
                    instance_id = id.as_str(),
                    error = %error,
                    "failed to restart server process"
                );
                ServerServiceError::OperationFailed
            })?;
        Ok(())
    }

    async fn stop(&self, id: &InstanceId) -> Result<(), ServerServiceError> {
        let _guard = self.lock_lifecycle(id).await?;
        self.find_instance(id).await?;
        self.stop_unlocked(id).await
    }

    async fn force_stop(&self, id: &InstanceId) -> Result<(), ServerServiceError> {
        let _guard = self.lock_lifecycle(id).await?;
        self.find_instance(id).await?;
        self.force_stop_unlocked(id)
    }

    async fn send_command(&self, id: &InstanceId, command: &str) -> Result<(), ServerServiceError> {
        self.send_command_inner(id, command).await
    }
}

impl CoreServerService {
    async fn start_unlocked(
        &self,
        id: &InstanceId,
        instance: &Instance,
    ) -> Result<(), ServerServiceError> {
        let id_str = id.as_str().to_string();

        // 清理残留进程：若旧进程仍在运行则拒绝启动，否则收敛旧日志管线。
        {
            let mut processes = self.processes_lock()?;
            if let Some(managed) = processes.get_mut(&id_str)
                && managed.daemon.poll().map(|s| s.is_none()).unwrap_or(false)
            {
                return Err(ServerError::InvalidState.into());
            }
        }
        if let Some(recorder) = self.take_recorder_after_exit(&id_str)? {
            spawn_recorder_shutdown(Some(recorder));
        }

        let mut command = match self.build_process_command(instance).await {
            Ok(command) => command,
            Err(error) => {
                tracing::error!(
                    target: "sealantern.application.server",
                    instance_id = %id_str,
                    error = %error,
                    "failed to build process command"
                );
                return Err(error.into());
            }
        };
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        let mut daemon = match Daemon::spawn(&mut command) {
            Ok(daemon) => daemon,
            Err(error) => {
                tracing::error!(
                    target: "sealantern.application.server",
                    instance_id = %id_str,
                    error = %error,
                    "failed to spawn server process"
                );
                return Err(ServerError::OperationFailed { source: Box::new(error) }.into());
            }
        };
        let terminal = Terminal::from_daemon_with_input(&mut daemon, true);

        self.mark_starting(&id_str);
        self.processes_lock()?
            .insert(id_str.clone(), ManagedProcess { daemon, terminal, recorder: None });

        // 启动元数据持久化失败时回滚进程，保证返回失败即没有运行中的新进程。
        if let Err(error) = self.instance_service.update_last_started(id).await {
            tracing::error!(
                target: "sealantern.application.server",
                instance_id = %id_str,
                error = %error,
                "failed to persist server start metadata"
            );
            if let Err(rollback_error) = self.force_stop_unlocked(id) {
                tracing::error!(
                    target: "sealantern.application.server",
                    instance_id = %id_str,
                    error = %rollback_error,
                    "failed to roll back server process after metadata persistence failure"
                );
            }
            self.clear_starting(&id_str);
            return Err(ServerError::OperationFailed {
                source: Box::new(std::io::Error::other("failed to update last started")),
            }
            .into());
        }

        // 启动日志记录管线：读取 stdout / stderr，落库并推送实时事件。
        let (stdout, stderr) = {
            let mut processes = self.processes_lock()?;
            let Some(managed) = processes.get_mut(&id_str) else {
                return Err(ServerError::InvalidState.into());
            };
            (
                managed.terminal.take_output(TerminalStream::Stdout),
                managed.terminal.take_output(TerminalStream::Stderr),
            )
        };
        // 控制台空行策略来自全局设置（读取失败时回落到默认丢弃）。
        let drop_empty_line = self
            .settings_service
            .get()
            .await
            .map(|settings| settings.console_drop_empty_line)
            .unwrap_or(true);
        let recorder = LogRecorder::start(
            id_str.clone(),
            &instance.directory,
            stdout,
            stderr,
            drop_empty_line,
        )
        .await;
        if let Some(managed) = self.processes_lock()?.get_mut(&id_str) {
            managed.recorder = Some(recorder);
        }

        // 进程已成功拉起并注册，退出 Starting 状态（Starting 仅覆盖 spawn 竞态窗口）。
        self.clear_starting(&id_str);

        Ok(())
    }

    async fn stop_unlocked(&self, id: &InstanceId) -> Result<(), ServerServiceError> {
        let id_str = id.as_str().to_string();

        // 优雅停止：向控制台发送 stop，等待退出。
        self.send_command_inner(id, "stop").await?;
        self.mark_stopping(&id_str);

        let deadline = Instant::now() + STOP_GRACEFUL_TIMEOUT;
        loop {
            // 单次锁内完成"判断退出 → 移除 → 取出日志管线"，避免重复取锁。
            let recorder = {
                let mut processes = self.processes_lock()?;
                let exited = match processes.get_mut(&id_str) {
                    Some(managed) => managed.daemon.poll().map(|s| s.is_some()).unwrap_or(true),
                    None => {
                        self.clear_stopping(&id_str);
                        return Ok(());
                    }
                };
                if exited {
                    processes
                        .remove(&id_str)
                        .and_then(|mut managed| managed.recorder.take())
                } else {
                    None // 进程仍在运行。
                }
            };
            if let Some(recorder) = recorder {
                self.clear_stopping(&id_str);
                recorder.shutdown().await;
                return Ok(());
            }
            if Instant::now() >= deadline {
                break;
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }

        // 超时：强制终止进程树；失败时保留受管进程原状并返回错误。
        {
            let mut processes = self.processes_lock()?;
            if let Some(managed) = processes.get_mut(&id_str)
                && let Err(error) = managed.daemon.terminate_tree()
            {
                return Err(ServerError::OperationFailed { source: Box::new(error) }.into());
            }
        }
        // 终止成功后统一移除受管进程并收敛日志管线。
        let recorder = self.take_recorder_unconditional(&id_str)?;
        self.clear_stopping(&id_str);
        if let Some(recorder) = recorder {
            recorder.shutdown().await;
        }
        Ok(())
    }

    fn force_stop_unlocked(&self, id: &InstanceId) -> Result<(), ServerServiceError> {
        let id_str = id.as_str().to_string();

        let mut processes = self.processes_lock()?;
        if let Some(mut managed) = processes.remove(&id_str) {
            if let Err(error) = managed.daemon.terminate_tree() {
                processes.insert(id_str.clone(), managed);
                return Err(ServerError::OperationFailed { source: Box::new(error) }.into());
            }
            let recorder = managed.recorder.take();
            spawn_recorder_shutdown(recorder);
        }
        self.clear_lifecycle_flags(&id_str);
        Ok(())
    }

    /// 为重启流程请求优雅停止，不等待退出或执行超时强杀。
    async fn request_stop_for_restart(&self, id: &InstanceId) -> Result<(), ServerServiceError> {
        self.send_command_inner(id, "stop").await?;
        self.mark_stopping(id.as_str());
        Ok(())
    }

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

/// 后台收敛日志记录管线（供同步清理路径使用）。
///
/// 进程退出后读取任务会随 EOF 结束，这里只负责 flush 剩余批次；
/// 后台执行以避免在同步调用链中引入异步等待。
fn spawn_recorder_shutdown(recorder: Option<LogRecorder>) {
    if let Some(recorder) = recorder {
        tokio::spawn(async move {
            recorder.shutdown().await;
        });
    }
}

/// 当前 Unix 时间戳（秒）。
fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 组装服务器进程的 JVM 参数。
///
/// 顺序（对齐历史版本 `build_managed_jvm_args`）：
/// 1. `-Xmx{max}M` / `-Xms{min}M`（来自实例内存配置）
/// 2. 编码参数（`-Dfile.encoding` 等，当前固定 UTF-8）
/// 3. 全局默认 JVM 参数（`default_jvm_args`，配置功能恢复前为空）
/// 4. 实例自定义参数（`launch.jvm_arguments`）
fn build_jvm_arguments(instance: &Instance, default_jvm_args: &str) -> Vec<OsString> {
    let mut args = vec![
        OsString::from(format!("-Xmx{}M", instance.max_memory_mib)),
        OsString::from(format!("-Xms{}M", instance.min_memory_mib)),
        OsString::from("-Dfile.encoding=UTF-8"),
        OsString::from("-Dsun.stdout.encoding=UTF-8"),
        OsString::from("-Dsun.stderr.encoding=UTF-8"),
    ];

    // 全局默认 JVM 参数（配置功能恢复后接入真实值）。
    for arg in default_jvm_args.split_whitespace() {
        args.push(OsString::from(arg));
    }

    args.extend(instance.launch.jvm_arguments.iter().map(OsString::from));
    args
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use sealantern_core::instance::InstanceId;

    use super::{CoreInstanceService, CoreServerService, CoreSettingsService};

    fn registry_path() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("sealantern-server-lock-{}-{nonce}", std::process::id()))
            .join("instances.json")
    }

    #[tokio::test]
    async fn lifecycle_operations_serialize_per_instance() {
        let path = registry_path();
        let instances = CoreInstanceService::with_path(&path)
            .await
            .expect("instance service");
        let service =
            CoreServerService::new(Arc::new(instances), Arc::new(CoreSettingsService::new()));
        let first = InstanceId::new("first").expect("first id");
        let second = InstanceId::new("second").expect("second id");

        let first_guard = service.lock_lifecycle(&first).await.expect("first lock");
        assert!(
            tokio::time::timeout(Duration::from_millis(20), service.lock_lifecycle(&first))
                .await
                .is_err()
        );
        let second_guard =
            tokio::time::timeout(Duration::from_millis(20), service.lock_lifecycle(&second))
                .await
                .expect("different instance must not wait")
                .expect("second lock");

        drop(first_guard);
        drop(second_guard);
        let reacquired =
            tokio::time::timeout(Duration::from_millis(20), service.lock_lifecycle(&first))
                .await
                .expect("same instance must continue after release")
                .expect("reacquired lock");

        let locks = service.lifecycle_locks.lock().expect("lifecycle locks");
        assert_eq!(locks.len(), 2);
        assert!(
            locks
                .get(second.as_str())
                .expect("second lock slot")
                .upgrade()
                .is_none()
        );
        drop(locks);
        drop(reacquired);

        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }
}
