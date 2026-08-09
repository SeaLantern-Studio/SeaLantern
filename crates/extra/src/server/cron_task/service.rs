use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule;
use sealantern_infra::fs::FsError;
use sealantern_infra::persistence::ConfigFile;
use uuid::Uuid;

use crate::observability;

use super::model::{CronTask, CronTaskAction, CronTaskDraft, CronTaskList, CronTaskRun};

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

/// 任务的持久化和执行调度服务。
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
        validate_draft(&draft)?;
        let task = CronTask {
            id: Uuid::new_v4().to_string(),
            name: draft.name.trim().to_owned(),
            server_id: draft.server_id.trim().to_owned(),
            cron_expression: normalize_cron_expression(&draft.cron_expression)?,
            action: draft.action,
            enabled: draft.enabled,
            last_run_at: None,
            next_run_at: None,
            last_error: None,
        };
        let mut task = task;
        task.next_run_at = Some(next_run_after(&task.cron_expression, Utc::now())?);

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
        validate_draft(&draft)?;
        let cron_expression = normalize_cron_expression(&draft.cron_expression)?;
        let next_run_at = next_run_after(&cron_expression, Utc::now())?;
        let previous = self.config.get().clone();
        let index = self
            .config
            .get()
            .tasks
            .iter()
            .position(|task| task.id == id)
            .ok_or_else(|| CronTaskError::TaskNotFound(id.to_owned()))?;
        let task = self.config.get().tasks[index].clone();

        let updated = CronTask {
            id: task.id,
            name: draft.name.trim().to_owned(),
            server_id: draft.server_id.trim().to_owned(),
            cron_expression,
            action: draft.action,
            enabled: draft.enabled,
            last_run_at: task.last_run_at,
            next_run_at: Some(next_run_at),
            last_error: task.last_error,
        };
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
        let previous = self.config.get().clone();
        let index = self
            .config
            .get()
            .tasks
            .iter()
            .position(|task| task.id == id)
            .ok_or_else(|| CronTaskError::TaskNotFound(id.to_owned()))?;
        let task = self.config.get().tasks[index].clone();
        let mut updated = task;
        updated.enabled = enabled;
        if enabled {
            updated.next_run_at = Some(next_run_after(&updated.cron_expression, Utc::now())?);
        }
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
        let task = self
            .config
            .get()
            .tasks
            .iter()
            .find(|task| task.id == id)
            .cloned()
            .ok_or_else(|| CronTaskError::TaskNotFound(id.to_owned()))?;
        self.run_task(task, now).await
    }

    /// 执行所有已到期且启用的任务。
    pub async fn run_due(&mut self, now: DateTime<Utc>) -> Result<Vec<CronTaskRun>, CronTaskError> {
        let due_tasks = self
            .config
            .get()
            .tasks
            .iter()
            .filter(|task| task.enabled && task.next_run_at.is_some_and(|next_run| next_run <= now))
            .cloned()
            .collect::<Vec<_>>();

        let mut runs = Vec::with_capacity(due_tasks.len());
        for task in due_tasks {
            let failed_run = CronTaskRun {
                task_id: task.id.clone(),
                server_id: task.server_id.clone(),
                action: task.action.clone(),
                succeeded: false,
                error: None,
            };
            match self.run_task(task, now).await {
                Ok(run) => runs.push(run),
                Err(CronTaskError::Execution { message, .. }) => {
                    runs.push(CronTaskRun { error: Some(message), ..failed_run })
                }
                Err(error) => return Err(error),
            }
        }
        Ok(runs)
    }

    async fn run_task(
        &mut self,
        task: CronTask,
        now: DateTime<Utc>,
    ) -> Result<CronTaskRun, CronTaskError> {
        let action = task.action.as_str();
        observability::server_cron_task_started(&task.id, &task.server_id, action);

        let execution_error = match &task.action {
            CronTaskAction::Restart => self
                .executor
                .restart_server(&task.server_id)
                .await
                .err()
                .map(|error| error.to_string()),
            CronTaskAction::Command { command } => self
                .executor
                .send_server_command(&task.server_id, command)
                .await
                .err()
                .map(|error| error.to_string()),
        };

        let run = CronTaskRun {
            task_id: task.id.clone(),
            server_id: task.server_id.clone(),
            action: task.action.clone(),
            succeeded: execution_error.is_none(),
            error: execution_error.clone(),
        };
        self.record_attempt(&task.id, now, execution_error).await?;

        if let Some(error) = &run.error {
            let error = CronTaskError::Execution {
                task_id: task.id.clone(),
                message: error.clone(),
            };
            observability::server_cron_task_failed(&task.id, &task.server_id, action, &error);
            return Err(error);
        }

        observability::server_cron_task_completed(&task.id, &task.server_id, action);
        Ok(run)
    }

    async fn record_attempt(
        &mut self,
        id: &str,
        now: DateTime<Utc>,
        error: Option<String>,
    ) -> Result<(), CronTaskError> {
        let index = self
            .config
            .get()
            .tasks
            .iter()
            .position(|task| task.id == id)
            .ok_or_else(|| CronTaskError::TaskNotFound(id.to_owned()))?;
        let next_run = next_run_after(&self.config.get().tasks[index].cron_expression, now)?;
        let previous = self.config.get().clone();
        self.config.update(|list| {
            let task = &mut list.tasks[index];
            task.last_run_at = Some(now);
            task.next_run_at = Some(next_run);
            task.last_error = error;
        });
        self.persist_or_restore(previous).await
    }

    async fn persist_or_restore(&mut self, previous: CronTaskList) -> Result<(), CronTaskError> {
        if let Err(error) = self.config.save(false).await {
            self.config.set(previous);
            return Err(error.into());
        }
        Ok(())
    }
}

fn validate_draft(draft: &CronTaskDraft) -> Result<(), CronTaskError> {
    if draft.name.trim().is_empty() {
        return Err(CronTaskError::InvalidTask("name must not be empty"));
    }
    if draft.server_id.trim().is_empty() {
        return Err(CronTaskError::InvalidTask("server_id must not be empty"));
    }
    if matches!(&draft.action, CronTaskAction::Command { command } if command.trim().is_empty()) {
        return Err(CronTaskError::InvalidTask("command must not be empty"));
    }
    Ok(())
}

fn normalize_cron_expression(expression: &str) -> Result<String, CronTaskError> {
    let trimmed = expression.trim();
    let field_count = trimmed.split_whitespace().count();
    let normalized = match field_count {
        5 => format!("0 {trimmed}"),
        6 => trimmed.to_owned(),
        _ => {
            return Err(CronTaskError::InvalidCron {
                expression: expression.to_owned(),
                message: "expected five or six fields".to_owned(),
            });
        }
    };
    Schedule::from_str(&normalized).map_err(|error| CronTaskError::InvalidCron {
        expression: expression.to_owned(),
        message: error.to_string(),
    })?;
    Ok(normalized)
}

fn next_run_after(expression: &str, now: DateTime<Utc>) -> Result<DateTime<Utc>, CronTaskError> {
    let schedule = Schedule::from_str(expression).map_err(|error| CronTaskError::InvalidCron {
        expression: expression.to_owned(),
        message: error.to_string(),
    })?;
    schedule
        .after(&now)
        .next()
        .ok_or_else(|| CronTaskError::InvalidCron {
            expression: expression.to_owned(),
            message: "no upcoming occurrence".to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::{Arc, Mutex};

    use chrono::Duration;
    use tempfile::tempdir;

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
        assert!(service.tasks()[0]
            .next_run_at
            .is_some_and(|next| next > now));
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
        assert_eq!(normalize_cron_expression("0 4 * * *").unwrap(), "0 0 4 * * *");
        assert_eq!(normalize_cron_expression("0 0 4 * * *").unwrap(), "0 0 4 * * *");
    }
}
