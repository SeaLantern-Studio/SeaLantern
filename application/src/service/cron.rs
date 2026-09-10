//! 服务器定时任务服务实现。
//!
//! 使用 `feature` 的 Cron 调度与 JSON 持久化能力，并通过注入的
//! [`crate::port::ServerService`] 执行重启和控制台命令。宿主仅依赖 `contract` DTO。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use sealantern_contract::cron::{CronTask, CronTaskAction, CronTaskDraft, CronTaskRun};
use sealantern_contract::{CronTaskServiceError, ServerServiceError};
use sealantern_core::instance::InstanceId;
use sealantern_feature::server::cron_task::{
    CronTask as FeatureCronTask, CronTaskAction as FeatureCronTaskAction,
    CronTaskDraft as FeatureCronTaskDraft, CronTaskError as FeatureCronTaskError,
    CronTaskExecutor as FeatureCronTaskExecutor, CronTaskRun as FeatureCronTaskRun,
    CronTaskService as FeatureCronTaskService,
};
use sealantern_infra::platform::get_app_data_dir;

use crate::error::CronTaskError;
use crate::port::{CronTaskService, ServerService};

use super::CoreServerService;

/// 定时任务 JSON 文件名，置于应用数据根目录。
const CRON_TASKS_FILE: &str = "cron_tasks.json";
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

type InnerCronTaskService<S> = FeatureCronTaskService<ServerCronTaskExecutor<S>>;

/// 基于 `feature` 调度实现与服务器进程契约的定时任务服务。
pub struct CoreCronTaskService<S = CoreServerService>
where
    S: ServerService + 'static,
{
    path: PathBuf,
    executor: ServerCronTaskExecutor<S>,
    inner: tokio::sync::OnceCell<tokio::sync::Mutex<InnerCronTaskService<S>>>,
    scheduler: tokio::sync::Mutex<Option<CronSchedulerHandle>>,
}

impl CoreCronTaskService<CoreServerService> {
    /// 使用应用数据目录中的默认 JSON 文件构造服务。
    pub fn new(server: Arc<CoreServerService>) -> Self {
        Self::with_path(get_app_data_dir().join(CRON_TASKS_FILE), server)
    }
}

impl<S> CoreCronTaskService<S>
where
    S: ServerService + 'static,
{
    /// 使用指定 JSON 路径构造服务，实际加载延迟到首次调用。
    pub fn with_path(path: impl Into<PathBuf>, server: Arc<S>) -> Self {
        Self {
            path: path.into(),
            executor: ServerCronTaskExecutor { server },
            inner: tokio::sync::OnceCell::new(),
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

    /// 执行当前所有到期任务，供后续后台调度器周期调用。
    pub async fn run_due(&self) -> Result<Vec<CronTaskRun>, CronTaskServiceError> {
        let mut service = self.service().await?;
        service
            .run_due(Utc::now())
            .await
            .map(|runs| runs.into_iter().map(run_to_contract).collect())
            .map_err(contract_error)
    }

    async fn service(
        &self,
    ) -> Result<tokio::sync::MutexGuard<'_, InnerCronTaskService<S>>, CronTaskServiceError> {
        let service = self
            .inner
            .get_or_try_init(|| async {
                FeatureCronTaskService::load(self.path.clone(), self.executor.clone())
                    .await
                    .map(tokio::sync::Mutex::new)
                    .map_err(contract_error)
            })
            .await?;
        Ok(service.lock().await)
    }
}

#[async_trait]
impl<S> CronTaskService for CoreCronTaskService<S>
where
    S: ServerService + 'static,
{
    async fn list(&self) -> Result<Vec<CronTask>, CronTaskServiceError> {
        let service = self.service().await?;
        Ok(service
            .tasks()
            .iter()
            .cloned()
            .map(task_to_contract)
            .collect())
    }

    async fn create(&self, draft: CronTaskDraft) -> Result<CronTask, CronTaskServiceError> {
        let mut service = self.service().await?;
        service
            .create(draft_to_feature(draft))
            .await
            .map(task_to_contract)
            .map_err(contract_error)
    }

    async fn update(
        &self,
        id: &str,
        draft: CronTaskDraft,
    ) -> Result<CronTask, CronTaskServiceError> {
        let mut service = self.service().await?;
        service
            .update(id, draft_to_feature(draft))
            .await
            .map(task_to_contract)
            .map_err(contract_error)
    }

    async fn delete(&self, id: &str) -> Result<(), CronTaskServiceError> {
        let mut service = self.service().await?;
        service.delete(id).await.map_err(contract_error)
    }

    async fn set_enabled(&self, id: &str, enabled: bool) -> Result<CronTask, CronTaskServiceError> {
        let mut service = self.service().await?;
        service
            .set_enabled(id, enabled)
            .await
            .map(task_to_contract)
            .map_err(contract_error)
    }

    async fn run_now(&self, id: &str) -> Result<CronTaskRun, CronTaskServiceError> {
        let mut service = self.service().await?;
        service
            .run_now(id, Utc::now())
            .await
            .map(run_to_contract)
            .map_err(contract_error)
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
    use std::sync::Mutex;

    use sealantern_contract::server::{ServerSnapshot, ServerState};
    use tempfile::tempdir;

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

    fn draft(action: CronTaskAction) -> CronTaskDraft {
        CronTaskDraft {
            name: "测试任务".to_owned(),
            server_id: "server-a".to_owned(),
            cron_expression: "* * * * *".to_owned(),
            action,
            enabled: true,
        }
    }

    #[tokio::test]
    async fn persists_tasks_and_executes_through_server_contract() {
        let directory = tempdir().expect("temp directory");
        let path = directory.path().join("cron_tasks.json");
        let server = Arc::new(FakeServerService::default());
        let service = CoreCronTaskService::with_path(&path, server.clone());

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
        assert!(
            tokio::fs::read_to_string(&path)
                .await
                .expect("read persisted tasks")
                .contains("say scheduled")
        );

        let reloaded = CoreCronTaskService::with_path(path, server);
        assert_eq!(reloaded.list().await.expect("reload tasks").len(), 2);
    }

    #[tokio::test]
    async fn rejects_invalid_server_id_before_calling_server_contract() {
        let directory = tempdir().expect("temp directory");
        let server = Arc::new(FakeServerService::default());
        let service = CoreCronTaskService::with_path(
            directory.path().join("cron_tasks.json"),
            server.clone(),
        );
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
        let server = Arc::new(FakeServerService::default());
        let service = Arc::new(CoreCronTaskService::with_path(
            directory.path().join("cron_tasks.json"),
            server.clone(),
        ));
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
