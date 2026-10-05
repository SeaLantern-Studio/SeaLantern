//! 服务器定时任务服务实现。
//!
//! 任务定义与执行状态存放在所属实例的 `sl.json`（`doc.cron`）——任务随实例
//! 目录迁移，无独立的全局 `cron_tasks.json`。调度语义复用
//! [`sealantern_feature::server::cron_task::engine`] 纯逻辑；执行经注入的
//! [`crate::port::ServerService`]，实例定位经 [`CoreInstanceService`]。宿主
//! 仅依赖 `contract` DTO。

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use sealantern_contract::cron::{CronTask, CronTaskAction, CronTaskDraft, CronTaskRun};
use sealantern_contract::{CronTaskServiceError, ServerServiceError};
use sealantern_core::instance::InstanceId;
use sealantern_feature::config::instance::{DocumentStore, InstanceCronEntry};
use sealantern_feature::server::cron_task::{
    CronTask as FeatureCronTask, CronTaskAction as FeatureCronTaskAction,
    CronTaskDraft as FeatureCronTaskDraft, CronTaskError as FeatureCronTaskError,
    CronTaskExecutor as FeatureCronTaskExecutor, CronTaskRun as FeatureCronTaskRun, engine,
};

use crate::error::CronTaskError;
use crate::port::{CronTaskService, InstanceService, ServerService};

use super::{CoreInstanceService, CoreServerService};

/// 自动调度检查间隔；Cron 表达式支持秒级粒度。
const SCHEDULER_TICK_INTERVAL: Duration = Duration::from_secs(1);
/// 存储等系统错误发生后的退避时间，避免持续刷日志和磁盘。
const SCHEDULER_ERROR_RETRY_INTERVAL: Duration = Duration::from_secs(30);

struct CronSchedulerHandle {
    shutdown: tokio::sync::watch::Sender<bool>,
    task: tokio::task::JoinHandle<()>,
}

struct ServerCronTaskExecutor<S> {
    server: Arc<S>,
}

impl<S> Clone for ServerCronTaskExecutor<S> {
    fn clone(&self) -> Self {
        Self { server: self.server.clone() }
    }
}

#[async_trait]
impl<S> FeatureCronTaskExecutor for ServerCronTaskExecutor<S>
where
    S: ServerService + 'static,
{
    type Error = ServerServiceError;

    async fn restart_server(&self, server_id: &str) -> Result<(), Self::Error> {
        let id = parse_instance_id(server_id)?;
        self.server.restart(&id).await
    }

    async fn send_server_command(&self, server_id: &str, command: &str) -> Result<(), Self::Error> {
        let id = parse_instance_id(server_id)?;
        self.server.send_command(&id, command).await
    }
}

/// 基于实例文档（`sl.json`）存储 + `engine` 调度的定时任务服务。
///
/// 任务的归属实例由 `server_id` 指向受信任的实例；`sl.json` 的写入经
/// `DocumentStore::update` 的锁内写回，不另设服务级互斥。
pub struct CoreCronTaskService<S = CoreServerService>
where
    S: ServerService + 'static,
{
    instance: Arc<CoreInstanceService>,
    executor: ServerCronTaskExecutor<S>,
    scheduler: tokio::sync::Mutex<Option<CronSchedulerHandle>>,
}

impl CoreCronTaskService<CoreServerService> {
    /// 以共享的实例服务与服务器进程服务构造（`AppServices::from_inner` 装配）。
    pub fn new(server: Arc<CoreServerService>, instance: Arc<CoreInstanceService>) -> Self {
        Self::with_parts(server, instance)
    }
}

impl<S> CoreCronTaskService<S>
where
    S: ServerService + 'static,
{
    /// 以指定的 server/instance 服务构造（测试注入用）。
    pub fn with_parts(server: Arc<S>, instance: Arc<CoreInstanceService>) -> Self {
        Self {
            instance,
            executor: ServerCronTaskExecutor { server },
            scheduler: tokio::sync::Mutex::new(None),
        }
    }

    /// 启动此服务的唯一后台调度器；已运行时返回 `false`。
    pub async fn start_scheduler(self: &Arc<Self>) -> bool {
        self.start_scheduler_with_intervals(SCHEDULER_TICK_INTERVAL, SCHEDULER_ERROR_RETRY_INTERVAL)
            .await
    }

    /// 停止后台调度器并等待任务退出；未运行时返回 `false`。
    pub async fn stop_scheduler(&self) -> bool {
        let handle = self.scheduler.lock().await.take();
        let Some(handle) = handle else {
            return false;
        };

        let _ = handle.shutdown.send(true);
        if let Err(error) = handle.task.await {
            tracing::error!(
                target: "sealantern.application.cron_task",
                error = %error,
                "cron scheduler task failed while stopping"
            );
        }
        true
    }

    async fn start_scheduler_with_intervals(
        self: &Arc<Self>,
        tick_interval: Duration,
        error_retry_interval: Duration,
    ) -> bool {
        let mut scheduler = self.scheduler.lock().await;
        if scheduler
            .as_ref()
            .is_some_and(|handle| !handle.task.is_finished())
        {
            return false;
        }

        let (shutdown, mut shutdown_rx) = tokio::sync::watch::channel(false);
        let service = Arc::downgrade(self);
        let task = tokio::spawn(async move {
            let mut delay = tick_interval;
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(delay) => {}
                    result = shutdown_rx.changed() => {
                        if result.is_err() || *shutdown_rx.borrow() {
                            break;
                        }
                        continue;
                    }
                }

                let Some(service) = service.upgrade() else {
                    break;
                };
                delay = match service.run_due().await {
                    Ok(_) => tick_interval,
                    Err(_) => error_retry_interval,
                };
            }
        });

        *scheduler = Some(CronSchedulerHandle { shutdown, task });
        true
    }

    /// 执行当前所有到期任务，供后台调度器周期调用。
    ///
    /// 遍历受信任实例的 `sl.json`：对到期且启用的条目就地执行并写回。
    /// 单个实例的读取失败按该实例跳过并记日志，不阻断其它实例的任务。
    pub async fn run_due(&self) -> Result<Vec<CronTaskRun>, CronTaskServiceError> {
        let dirs = self
            .instance
            .trusted_instance_dirs()
            .await
            .map_err(|error| operation_failed(error.to_string()))?;
        let now = Utc::now();
        let mut runs = Vec::new();

        for dir in dirs {
            let Ok(mut store) = DocumentStore::load(DocumentStore::path_for(&dir)).await else {
                tracing::warn!(
                    target: "sealantern.application.cron_task",
                    dir = %dir.display(),
                    "skipping instance whose document could not be loaded for due cron tasks"
                );
                continue;
            };
            let server_id = store.get().id.as_str().to_string();
            // 本轮到期的任务 id 快照（条目可能并发增删，按 id 回写定位）。
            let due: Vec<String> = store
                .get()
                .cron
                .iter()
                .filter(|entry| entry.enabled && entry.next_run_at.is_some_and(|next| next <= now))
                .map(|entry| entry.id.clone())
                .collect();

            for task_id in due {
                // 每条执行前后以锁内 update 读写最新文档，避免覆盖并发修改。
                let mut task = match store.get().cron.iter().find(|entry| entry.id == task_id) {
                    Some(entry) => entry_to_feature(entry, &server_id),
                    None => continue,
                };
                let result = engine::run_task(&self.executor, &mut task, now).await;
                // 只合并执行负责的状态字段（下次计划 / 上次执行 / 错误），
                // 保留执行期间对条目其它字段（名称、表达式、启用位等）的并发
                // 修改——否则整条替换会覆盖掉它们。
                let update_result = store
                    .update(|document| {
                        match document.cron.iter_mut().find(|entry| entry.id == task.id) {
                            Some(entry) => {
                                entry.last_run_at = task.last_run_at;
                                entry.next_run_at = task.next_run_at;
                                entry.last_error = task.last_error.clone();
                                Ok(true)
                            }
                            // 执行期间被删除：不回写（条目已不存在）。
                            None => Ok(false),
                        }
                    })
                    .await;
                if let Err(error) = update_result {
                    return Err(operation_failed(error.to_string()));
                }
                match result {
                    Ok(run) => runs.push(run_to_contract(run)),
                    Err(FeatureCronTaskError::Execution { task_id, message }) => {
                        runs.push(CronTaskRun {
                            task_id,
                            server_id: server_id.clone(),
                            action: action_to_contract(task.action),
                            succeeded: false,
                            error: Some(message),
                        });
                    }
                    Err(error) => return Err(contract_error(error)),
                }
            }
        }
        Ok(runs)
    }

    /// 定位任务所属实例：返回该实例的 `DocumentStore`。
    ///
    /// 任务的 `server_id` 存在文档中；按 id 全局查找须遍历受信任实例的文档。
    async fn locate(&self, task_id: &str) -> Result<(DocumentStore, String), CronTaskServiceError> {
        let dirs = self
            .instance
            .trusted_instance_dirs()
            .await
            .map_err(|error| operation_failed(error.to_string()))?;
        for dir in dirs {
            let Ok(store) = DocumentStore::load(DocumentStore::path_for(&dir)).await else {
                continue;
            };
            let server_id = store.get().id.as_str().to_string();
            if store.get().cron.iter().any(|entry| entry.id == task_id) {
                return Ok((store, server_id));
            }
        }
        Err(CronTaskServiceError::TaskNotFound)
    }

    /// 打开 `server_id` 对应受信任实例的 `DocumentStore`。
    async fn owner_document(&self, server_id: &str) -> Result<DocumentStore, CronTaskServiceError> {
        let id = parse_instance_id(server_id).map_err(|_| CronTaskServiceError::InvalidInput)?;
        let instance = self
            .instance
            .find(&id)
            .await
            .map_err(|error| operation_failed(error.to_string()))?
            .ok_or(CronTaskServiceError::InvalidInput)?;
        DocumentStore::load(DocumentStore::path_for(&instance.directory))
            .await
            .map_err(|error| operation_failed(error.to_string()))
    }
}

#[async_trait]
impl<S> CronTaskService for CoreCronTaskService<S>
where
    S: ServerService + 'static,
{
    /// 汇总所有受信任实例的任务（`server_id` 由所属实例补充）。
    async fn list(&self) -> Result<Vec<CronTask>, CronTaskServiceError> {
        let dirs = self
            .instance
            .trusted_instance_dirs()
            .await
            .map_err(|error| operation_failed(error.to_string()))?;
        let mut tasks = Vec::new();
        for dir in dirs {
            let Ok(store) = DocumentStore::load(DocumentStore::path_for(&dir)).await else {
                tracing::warn!(
                    target: "sealantern.application.cron_task",
                    dir = %dir.display(),
                    "skipping instance whose document could not be loaded for cron listing"
                );
                continue;
            };
            let server_id = store.get().id.as_str().to_string();
            tasks.extend(
                store
                    .get()
                    .cron
                    .iter()
                    .map(|entry| task_to_contract(entry_to_feature(entry, &server_id))),
            );
        }
        Ok(tasks)
    }

    /// 在 `draft.server_id` 指向的实例文档中创建任务。
    async fn create(&self, draft: CronTaskDraft) -> Result<CronTask, CronTaskServiceError> {
        let feature_draft = draft_to_feature(draft);
        engine::validate_draft(&feature_draft).map_err(contract_error)?;
        let mut store = self.owner_document(&feature_draft.server_id).await?;

        let task = engine::build_task(&feature_draft).map_err(contract_error)?;
        let entry = entry_from_feature(&task);
        store
            .update(|document| {
                document.cron.push(entry.clone());
                Ok(true)
            })
            .await
            .map_err(|error| operation_failed(error.to_string()))?;
        Ok(task_to_contract(task))
    }

    /// 更新任务配置，保留执行历史并重新计算下次运行时间。
    ///
    /// `draft.server_id` 必须与任务当前归属实例一致（不支持迁移归属）。
    /// 新状态在锁外计算，锁内按任务 id 原子替换。
    async fn update(
        &self,
        id: &str,
        draft: CronTaskDraft,
    ) -> Result<CronTask, CronTaskServiceError> {
        let feature_draft = draft_to_feature(draft);
        engine::validate_draft(&feature_draft).map_err(contract_error)?;
        let (mut store, server_id) = self.locate(id).await?;
        if feature_draft.server_id.trim() != server_id {
            return Err(CronTaskServiceError::InvalidInput);
        }

        let Some(entry) = store.get().cron.iter().find(|entry| entry.id == id) else {
            return Err(CronTaskServiceError::TaskNotFound);
        };
        let mut task = entry_to_feature(entry, &server_id);
        engine::apply_update(&mut task, &feature_draft).map_err(contract_error)?;
        let replacement = entry_from_feature(&task);

        let mut found = false;
        store
            .update(|document| {
                if let Some(entry) = document.cron.iter_mut().find(|entry| entry.id == id) {
                    *entry = replacement.clone();
                    found = true;
                    Ok(true)
                } else {
                    Ok(false)
                }
            })
            .await
            .map_err(|error| operation_failed(error.to_string()))?;
        if !found {
            return Err(CronTaskServiceError::TaskNotFound);
        }
        Ok(task_to_contract(task))
    }

    async fn delete(&self, id: &str) -> Result<(), CronTaskServiceError> {
        let (mut store, _) = self.locate(id).await?;
        let mut removed = false;
        store
            .update(|document| {
                let len = document.cron.len();
                document.cron.retain(|entry| entry.id != id);
                removed = document.cron.len() != len;
                Ok(removed)
            })
            .await
            .map_err(|error| operation_failed(error.to_string()))?;
        if !removed {
            return Err(CronTaskServiceError::TaskNotFound);
        }
        Ok(())
    }

    async fn set_enabled(&self, id: &str, enabled: bool) -> Result<CronTask, CronTaskServiceError> {
        let (mut store, server_id) = self.locate(id).await?;
        let Some(entry) = store.get().cron.iter().find(|entry| entry.id == id) else {
            return Err(CronTaskServiceError::TaskNotFound);
        };
        let mut task = entry_to_feature(entry, &server_id);
        task.enabled = enabled;
        if enabled {
            task.next_run_at = Some(
                engine::next_run_after(&task.cron_expression, Utc::now())
                    .map_err(contract_error)?,
            );
        }
        let replacement = entry_from_feature(&task);

        let mut found = false;
        store
            .update(|document| {
                if let Some(entry) = document.cron.iter_mut().find(|entry| entry.id == id) {
                    *entry = replacement.clone();
                    found = true;
                    Ok(true)
                } else {
                    Ok(false)
                }
            })
            .await
            .map_err(|error| operation_failed(error.to_string()))?;
        if !found {
            return Err(CronTaskServiceError::TaskNotFound);
        }
        Ok(task_to_contract(task))
    }

    /// 立即执行指定任务并把尝试结果写回 `sl.json`。
    async fn run_now(&self, id: &str) -> Result<CronTaskRun, CronTaskServiceError> {
        let (mut store, server_id) = self.locate(id).await?;
        let Some(entry) = store.get().cron.iter().find(|entry| entry.id == id) else {
            return Err(CronTaskServiceError::TaskNotFound);
        };
        let mut task = entry_to_feature(entry, &server_id);
        let now = Utc::now();
        let result = engine::run_task(&self.executor, &mut task, now).await;
        store
            .update(|document| match document.cron.iter_mut().find(|entry| entry.id == id) {
                Some(entry) => {
                    *entry = entry_from_feature(&task);
                    Ok(true)
                }
                None => Ok(false),
            })
            .await
            .map_err(|error| operation_failed(error.to_string()))?;
        result.map(run_to_contract).map_err(contract_error)
    }
}

fn parse_instance_id(raw: &str) -> Result<InstanceId, ServerServiceError> {
    InstanceId::new(raw.to_owned()).map_err(|_| ServerServiceError::InvalidInput)
}

fn contract_error(error: FeatureCronTaskError) -> CronTaskServiceError {
    let error = CronTaskError::from(error);
    tracing::error!(
        target: "sealantern.application.cron_task",
        error = %error,
        "cron task operation failed"
    );
    error.into()
}

/// `sl.json` 读写与实例查找失败收敛为通用操作失败（细节由日志承载）。
fn operation_failed(message: String) -> CronTaskServiceError {
    tracing::error!(
        target: "sealantern.application.cron_task",
        error = %message,
        "cron task operation failed"
    );
    CronTaskServiceError::OperationFailed
}

fn action_to_feature(action: CronTaskAction) -> FeatureCronTaskAction {
    match action {
        CronTaskAction::Restart => FeatureCronTaskAction::Restart,
        CronTaskAction::Command { command } => FeatureCronTaskAction::Command { command },
    }
}

fn action_to_contract(action: FeatureCronTaskAction) -> CronTaskAction {
    match action {
        FeatureCronTaskAction::Restart => CronTaskAction::Restart,
        FeatureCronTaskAction::Command { command } => CronTaskAction::Command { command },
    }
}

fn draft_to_feature(draft: CronTaskDraft) -> FeatureCronTaskDraft {
    FeatureCronTaskDraft {
        name: draft.name,
        server_id: draft.server_id,
        cron_expression: draft.cron_expression,
        action: action_to_feature(draft.action),
        enabled: draft.enabled,
    }
}

/// `sl.json` 内嵌条目 → feature 任务（`server_id` 由所属实例文档补充）。
fn entry_to_feature(entry: &InstanceCronEntry, server_id: &str) -> FeatureCronTask {
    FeatureCronTask {
        id: entry.id.clone(),
        name: entry.name.clone(),
        server_id: server_id.to_owned(),
        cron_expression: entry.cron_expression.clone(),
        action: match &entry.action {
            sealantern_contract::cron::CronTaskAction::Restart => FeatureCronTaskAction::Restart,
            sealantern_contract::cron::CronTaskAction::Command { command } => {
                FeatureCronTaskAction::Command { command: command.clone() }
            }
        },
        enabled: entry.enabled,
        last_run_at: entry.last_run_at,
        next_run_at: entry.next_run_at,
        last_error: entry.last_error.clone(),
    }
}

/// feature 任务 → `sl.json` 内嵌条目（归属由文档隐含，`server_id` 丢弃）。
fn entry_from_feature(task: &FeatureCronTask) -> InstanceCronEntry {
    InstanceCronEntry {
        id: task.id.clone(),
        name: task.name.clone(),
        cron_expression: task.cron_expression.clone(),
        action: match &task.action {
            FeatureCronTaskAction::Restart => CronTaskAction::Restart,
            FeatureCronTaskAction::Command { command } => {
                CronTaskAction::Command { command: command.clone() }
            }
        },
        enabled: task.enabled,
        last_run_at: task.last_run_at,
        next_run_at: task.next_run_at,
        last_error: task.last_error.clone(),
    }
}

fn task_to_contract(task: FeatureCronTask) -> CronTask {
    CronTask {
        id: task.id,
        name: task.name,
        server_id: task.server_id,
        cron_expression: task.cron_expression,
        action: action_to_contract(task.action),
        enabled: task.enabled,
        last_run_at: task.last_run_at,
        next_run_at: task.next_run_at,
        last_error: task.last_error,
    }
}

fn run_to_contract(run: FeatureCronTaskRun) -> CronTaskRun {
    CronTaskRun {
        task_id: run.task_id,
        server_id: run.server_id,
        action: action_to_contract(run.action),
        succeeded: run.succeeded,
        error: run.error,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Mutex;

    use sealantern_contract::server::{ServerSnapshot, ServerState};
    use sealantern_core::instance::{InstanceSpec, LocalLaunch, StartupMode};
    use sealantern_feature::config::SettingsManager;
    use tempfile::tempdir;

    use crate::port::InstanceService;
    use crate::service::CoreSettingsService;

    use super::*;

    #[derive(Default)]
    struct FakeServerService {
        calls: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl ServerService for FakeServerService {
        async fn status(&self, id: &InstanceId) -> Result<ServerSnapshot, ServerServiceError> {
            Ok(ServerSnapshot {
                instance_id: id.as_str().to_owned(),
                state: ServerState::Stopped,
                pid: None,
                uptime_secs: None,
                error_message: None,
            })
        }

        async fn start(&self, _id: &InstanceId) -> Result<(), ServerServiceError> {
            Ok(())
        }

        async fn restart(&self, id: &InstanceId) -> Result<(), ServerServiceError> {
            self.calls
                .lock()
                .expect("calls lock")
                .push(format!("restart:{}", id.as_str()));
            Ok(())
        }

        async fn stop(&self, _id: &InstanceId) -> Result<(), ServerServiceError> {
            Ok(())
        }

        async fn force_stop(&self, _id: &InstanceId) -> Result<(), ServerServiceError> {
            Ok(())
        }

        async fn send_command(
            &self,
            id: &InstanceId,
            command: &str,
        ) -> Result<(), ServerServiceError> {
            self.calls
                .lock()
                .expect("calls lock")
                .push(format!("command:{}:{command}", id.as_str()));
            Ok(())
        }
    }

    /// 实例规格：`root/instances/{id}` 下的受管目录（启动目标在目录内）。
    fn instance_spec(root: &Path, id: &str) -> InstanceSpec {
        let dir = root.join("instances").join(id);
        InstanceSpec {
            id: InstanceId::new(id).expect("valid id"),
            name: format!("server-{id}"),
            aliases: Vec::new(),
            core_type: "paper".into(),
            core_version: "1.20.4".into(),
            game_version: "1.20.4".into(),
            required_java: None,
            directory: dir.clone(),
            port: 25565,
            max_memory_mib: 2048,
            min_memory_mib: 512,
            created_at_unix_secs: 0,
            last_started_at_unix_secs: None,
            server_metadata: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(dir.join("server.jar")),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: None,
                jvm_arguments: Vec::new(),
            },
        }
    }

    /// 以临时根装配共享设置的实例服务 + cron 服务（任务存进实例 sl.json）。
    async fn test_services(
        root: &Path,
    ) -> (
        Arc<CoreCronTaskService<FakeServerService>>,
        Arc<FakeServerService>,
        Arc<CoreInstanceService>,
    ) {
        let manager = SettingsManager::load(root.join("settings.json"))
            .await
            .expect("load settings");
        let settings = Arc::new(CoreSettingsService::with_manager(manager));
        let instance = Arc::new(CoreInstanceService::new(settings));
        let server = Arc::new(FakeServerService::default());
        let service = Arc::new(CoreCronTaskService::with_parts(server.clone(), instance.clone()));
        (service, server, instance)
    }

    fn draft(action: CronTaskAction) -> CronTaskDraft {
        CronTaskDraft {
            name: "nightly task".to_owned(),
            server_id: "server-a".to_owned(),
            cron_expression: "* * * * *".to_owned(),
            action,
            enabled: true,
        }
    }

    #[tokio::test]
    async fn persists_tasks_and_executes_through_server_contract() {
        let directory = tempdir().expect("temp directory");
        let (service, server, instance) = test_services(directory.path()).await;
        instance
            .create(instance_spec(directory.path(), "server-a"))
            .await
            .expect("create instance");

        let restart = service
            .create(draft(CronTaskAction::Restart))
            .await
            .expect("create restart task");
        let command = service
            .create(draft(CronTaskAction::Command { command: "say scheduled".to_owned() }))
            .await
            .expect("create command task");

        service
            .run_now(&restart.id)
            .await
            .expect("run restart task");
        service
            .run_now(&command.id)
            .await
            .expect("run command task");

        assert_eq!(
            *server.calls.lock().expect("calls lock"),
            ["restart:server-a", "command:server-a:say scheduled"]
        );
        // 任务持久化在实例文档 sl.json 内。
        assert!(
            tokio::fs::read_to_string(
                directory
                    .path()
                    .join("instances")
                    .join("server-a")
                    .join("sl.json")
            )
            .await
            .expect("read persisted document")
            .contains("say scheduled")
        );

        // 重新装配（新设置/实例服务）后任务仍在。
        let (reloaded, _, _) = test_services(directory.path()).await;
        assert_eq!(reloaded.list().await.expect("reload tasks").len(), 2);
    }

    #[tokio::test]
    async fn rejects_invalid_server_id_before_calling_server_contract() {
        let directory = tempdir().expect("temp directory");
        let (service, server, _) = test_services(directory.path()).await;
        let task = service
            .create(CronTaskDraft {
                server_id: "   ".to_owned(),
                ..draft(CronTaskAction::Restart)
            })
            .await;

        assert_eq!(task, Err(CronTaskServiceError::InvalidInput));
        assert!(server.calls.lock().expect("calls lock").is_empty());
    }

    #[tokio::test]
    async fn scheduler_runs_due_tasks_once_and_stops_cleanly() {
        let directory = tempdir().expect("temp directory");
        let (service, server, instance) = test_services(directory.path()).await;
        instance
            .create(instance_spec(directory.path(), "server-a"))
            .await
            .expect("create instance");
        service
            .create(CronTaskDraft {
                cron_expression: "* * * * * *".to_owned(),
                ..draft(CronTaskAction::Restart)
            })
            .await
            .expect("create scheduled task");

        assert!(
            service
                .start_scheduler_with_intervals(
                    Duration::from_millis(10),
                    Duration::from_millis(20),
                )
                .await
        );
        assert!(
            !service
                .start_scheduler_with_intervals(
                    Duration::from_millis(10),
                    Duration::from_millis(20),
                )
                .await
        );

        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if !server.calls.lock().expect("calls lock").is_empty() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("scheduler executes due task");

        assert!(service.stop_scheduler().await);
        assert!(!service.stop_scheduler().await);
        assert_eq!(*server.calls.lock().expect("calls lock"), ["restart:server-a"]);
    }
}
