use std::fmt;
use std::path::PathBuf;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sealantern_infra::fs::FsError;
use sealantern_infra::persistence::ConfigFile;

use super::engine;
use super::model::{CronTask, CronTaskDraft, CronTaskList, CronTaskRun};

/// 宿主提供的服务器操作。
#[async_trait]
pub trait CronTaskExecutor: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn restart_server(&self, server_id: &str) -> Result<(), Self::Error>;

    async fn send_server_command(&self, server_id: &str, command: &str) -> Result<(), Self::Error>;
}

/// Cron 任务服务错误。
#[derive(Debug)]
#[non_exhaustive]
pub enum CronTaskError {
    Storage(FsError),
    TaskNotFound(String),
    InvalidTask(&'static str),
    InvalidCron { expression: String, message: String },
    Execution { task_id: String, message: String },
}

impl fmt::Display for CronTaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(formatter, "cron task storage failed: {error}"),
            Self::TaskNotFound(id) => write!(formatter, "cron task not found: {id}"),
            Self::InvalidTask(reason) => write!(formatter, "invalid cron task: {reason}"),
            Self::InvalidCron { expression, message } => {
                write!(formatter, "invalid cron expression '{expression}': {message}")
            }
            Self::Execution { task_id, message } => {
                write!(formatter, "cron task execution failed for {task_id}: {message}")
            }
        }
    }
}

impl std::error::Error for CronTaskError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            _ => None,
        }
    }
}

impl From<FsError> for CronTaskError {
    fn from(error: FsError) -> Self {
        Self::Storage(error)
    }
}

/// 任务的持久化和执行调度服务（`cron_tasks.json` 单文件外壳）。
///
/// 调度语义集中在 [`super::engine`]；本类型只负责 `CronTaskList` 的
/// `ConfigFile` 读写与失败回滚。实例文档 `sl.json` 内嵌的 cron 条目由
/// application 层直接驱动同一 engine。
pub struct CronTaskService<E> {
    config: ConfigFile<CronTaskList>,
    executor: E,
}

impl<E: CronTaskExecutor> CronTaskService<E> {
    /// 从 JSON 文件加载任务；文件不存在时创建空列表。
    pub async fn load(path: impl Into<PathBuf>, executor: E) -> Result<Self, CronTaskError> {
        let config = ConfigFile::load_or_create(path, CronTaskList::default()).await?;
        Ok(Self { config, executor })
    }

    /// 返回当前任务列表。
    pub fn tasks(&self) -> &[CronTask] {
        &self.config.get().tasks
    }

    /// 创建任务并计算首次执行时间。
    pub async fn create(&mut self, draft: CronTaskDraft) -> Result<CronTask, CronTaskError> {
        engine::validate_draft(&draft)?;
        let task = engine::build_task(&draft)?;

        let previous = self.config.get().clone();
        self.config.update(|list| list.tasks.push(task.clone()));
        self.persist_or_restore(previous).await?;
        Ok(task)
    }

    /// 更新任务配置，保留执行历史并重新计算下次运行时间。
    pub async fn update(
        &mut self,
        id: &str,
        draft: CronTaskDraft,
    ) -> Result<CronTask, CronTaskError> {
        engine::validate_draft(&draft)?;
        let index = self
            .config
            .get()
            .tasks
            .iter()
            .position(|task| task.id == id)
            .ok_or_else(|| CronTaskError::TaskNotFound(id.to_owned()))?;

        let mut updated = self.config.get().tasks[index].clone();
        engine::apply_update(&mut updated, &draft)?;

        let previous = self.config.get().clone();
        self.config
            .update(|list| list.tasks[index] = updated.clone());
        self.persist_or_restore(previous).await?;
        Ok(updated)
    }

    /// 删除任务。
    pub async fn delete(&mut self, id: &str) -> Result<(), CronTaskError> {
        let previous = self.config.get().clone();
        self.config
            .update(|list| list.tasks.retain(|task| task.id != id));
        if self.config.get().tasks.len() == previous.tasks.len() {
            self.config.set(previous);
            return Err(CronTaskError::TaskNotFound(id.to_owned()));
        }
        self.persist_or_restore(previous).await
    }

    /// 设置任务是否参与自动调度。
    pub async fn set_enabled(
        &mut self,
        id: &str,
        enabled: bool,
    ) -> Result<CronTask, CronTaskError> {
        let index = self
            .config
            .get()
            .tasks
            .iter()
            .position(|task| task.id == id)
            .ok_or_else(|| CronTaskError::TaskNotFound(id.to_owned()))?;

        let mut updated = self.config.get().tasks[index].clone();
        updated.enabled = enabled;
        if enabled {
            updated.next_run_at =
                Some(engine::next_run_after(&updated.cron_expression, Utc::now())?);
        }

        let previous = self.config.get().clone();
        self.config
            .update(|list| list.tasks[index] = updated.clone());
        self.persist_or_restore(previous).await?;
        Ok(updated)
    }

    /// 执行指定任务，并记录本次尝试及下一次计划时间。
    pub async fn run_now(
        &mut self,
        id: &str,
        now: DateTime<Utc>,
    ) -> Result<CronTaskRun, CronTaskError> {
        let index = self
            .config
            .get()
            .tasks
            .iter()
            .position(|task| task.id == id)
            .ok_or_else(|| CronTaskError::TaskNotFound(id.to_owned()))?;
        let mut task = self.config.get().tasks[index].clone();
        self.run_and_record(index, &mut task, now).await
    }

    /// 执行所有已到期且启用的任务。
    pub async fn run_due(&mut self, now: DateTime<Utc>) -> Result<Vec<CronTaskRun>, CronTaskError> {
        let due_indices = self
            .config
            .get()
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| {
                task.enabled && task.next_run_at.is_some_and(|next_run| next_run <= now)
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        let mut runs = Vec::with_capacity(due_indices.len());
        for index in due_indices {
            let mut task = self.config.get().tasks[index].clone();
            match self.run_and_record(index, &mut task, now).await {
                Ok(run) => runs.push(run),
                Err(CronTaskError::Execution { task_id, message }) => {
                    runs.push(CronTaskRun {
                        task_id,
                        server_id: task.server_id.clone(),
                        action: task.action.clone(),
                        succeeded: false,
                        error: Some(message),
                    });
                }
                Err(error) => return Err(error),
            }
        }
        Ok(runs)
    }

    /// 执行单个任务并把尝试结果持久化回列表。
    ///
    /// `engine::run_task` 已把尝试结果写入 `task`；本方法负责把该状态写回
    /// `config` 并持久化（执行失败的尝试也会回写——与旧实现一致）。
    async fn run_and_record(
        &mut self,
        index: usize,
        task: &mut CronTask,
        now: DateTime<Utc>,
    ) -> Result<CronTaskRun, CronTaskError> {
        let result = engine::run_task(&self.executor, task, now).await;
        let previous = self.config.get().clone();
        self.config.update(|list| list.tasks[index] = task.clone());
        self.persist_or_restore(previous).await?;
        result
    }

    async fn persist_or_restore(&mut self, previous: CronTaskList) -> Result<(), CronTaskError> {
        if let Err(error) = self.config.save(false).await {
            self.config.set(previous);
            return Err(error.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::{Arc, Mutex};

    use chrono::Duration;
    use tempfile::tempdir;

    use super::super::model::CronTaskAction;
    use super::*;

    #[derive(Clone, Default)]
    struct TestExecutor {
        calls: Arc<Mutex<Vec<String>>>,
        fail: bool,
    }

    #[async_trait]
    impl CronTaskExecutor for TestExecutor {
        type Error = io::Error;

        async fn restart_server(&self, server_id: &str) -> Result<(), Self::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("restart:{server_id}"));
            if self.fail {
                return Err(io::Error::other("restart failed"));
            }
            Ok(())
        }

        async fn send_server_command(
            &self,
            server_id: &str,
            command: &str,
        ) -> Result<(), Self::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("command:{server_id}:{command}"));
            if self.fail {
                return Err(io::Error::other("command failed"));
            }
            Ok(())
        }
    }

    fn draft(action: CronTaskAction) -> CronTaskDraft {
        CronTaskDraft {
            name: "Nightly task".to_owned(),
            server_id: "server-a".to_owned(),
            cron_expression: "* * * * *".to_owned(),
            action,
            enabled: true,
        }
    }

    async fn service(executor: TestExecutor) -> (tempfile::TempDir, CronTaskService<TestExecutor>) {
        let directory = tempdir().unwrap();
        let path = directory.path().join("cron_tasks.json");
        let service = CronTaskService::load(path, executor).await.unwrap();
        (directory, service)
    }

    #[tokio::test]
    async fn creates_and_executes_a_command_task() {
        let executor = TestExecutor::default();
        let calls = Arc::clone(&executor.calls);
        let (_directory, mut service) = service(executor).await;
        let task = service
            .create(draft(CronTaskAction::Command { command: "say scheduled".to_owned() }))
            .await
            .unwrap();

        let run = service.run_now(&task.id, Utc::now()).await.unwrap();

        assert!(run.succeeded);
        assert_eq!(*calls.lock().unwrap(), ["command:server-a:say scheduled"]);
        assert!(service.tasks()[0].last_run_at.is_some());
        assert!(service.tasks()[0].last_error.is_none());
    }

    #[tokio::test]
    async fn failed_execution_is_recorded_and_rescheduled() {
        let (_directory, mut service) =
            service(TestExecutor { fail: true, ..Default::default() }).await;
        let task = service
            .create(draft(CronTaskAction::Restart))
            .await
            .unwrap();
        let now = Utc::now();

        let result = service.run_now(&task.id, now).await;

        assert!(matches!(result, Err(CronTaskError::Execution { .. })));
        let task = &service.tasks()[0];
        assert_eq!(task.last_run_at, Some(now));
        assert!(task.next_run_at.is_some_and(|next| next > now));
        assert_eq!(task.last_error.as_deref(), Some("restart failed"));
    }

    #[tokio::test]
    async fn due_tasks_execute_once_and_advance_the_schedule() {
        let executor = TestExecutor::default();
        let calls = Arc::clone(&executor.calls);
        let (_directory, mut service) = service(executor).await;
        let task = service
            .create(draft(CronTaskAction::Restart))
            .await
            .unwrap();
        let now = Utc::now();
        service.config.update(|list| {
            list.tasks[0].next_run_at = Some(now - Duration::seconds(1));
        });

        let runs = service.run_due(now).await.unwrap();

        assert_eq!(runs.len(), 1);
        assert!(runs[0].succeeded);
        assert_eq!(*calls.lock().unwrap(), ["restart:server-a"]);
        assert!(
            service.tasks()[0]
                .next_run_at
                .is_some_and(|next| next > now)
        );
        assert_eq!(service.tasks()[0].id, task.id);
    }

    #[tokio::test]
    async fn due_task_failure_does_not_stop_later_tasks() {
        let executor = TestExecutor { fail: true, ..Default::default() };
        let (_directory, mut service) = service(executor).await;
        let first = service
            .create(draft(CronTaskAction::Restart))
            .await
            .unwrap();
        let second = service
            .create(CronTaskDraft {
                name: "Second task".to_owned(),
                ..draft(CronTaskAction::Command { command: "say still runs".to_owned() })
            })
            .await
            .unwrap();
        let now = Utc::now();
        service.config.update(|list| {
            for task in &mut list.tasks {
                task.next_run_at = Some(now - Duration::seconds(1));
            }
        });

        let runs = service.run_due(now).await.unwrap();

        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].task_id, first.id);
        assert_eq!(runs[1].task_id, second.id);
        assert!(runs.iter().all(|run| !run.succeeded));
    }

    #[test]
    fn accepts_five_or_six_field_cron_expressions() {
        assert_eq!(engine::normalize_cron_expression("0 4 * * *").unwrap(), "0 0 4 * * *");
        assert_eq!(engine::normalize_cron_expression("0 0 4 * * *").unwrap(), "0 0 4 * * *");
    }
}
