//! 定时任务的纯调度逻辑。
//!
//! 与持久化解耦：只负责草案校验、cron 表达式规范化、到期计算与「执行 +
//! 记录」的状态变换。两个存储后端共用——单文件 `CronTaskList` 外壳
//! （[`super::service::CronTaskService`]）与实例文档 `sl.json` 内嵌条目。

use std::str::FromStr;

use chrono::{DateTime, Utc};
use cron::Schedule;
use uuid::Uuid;

use super::model::{CronTask, CronTaskAction, CronTaskDraft, CronTaskRun};
use super::service::CronTaskError;
use crate::observability;

/// 校验任务草案（非空白名称/服务器标识/命令文本）。
pub fn validate_draft(draft: &CronTaskDraft) -> Result<(), CronTaskError> {
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

/// 规范化 cron 表达式：5 字段补秒位为 6 字段，并验证可解析。
pub fn normalize_cron_expression(expression: &str) -> Result<String, CronTaskError> {
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

/// 计算 `expression` 在 `now` 之后的下一次触发时间。
pub fn next_run_after(
    expression: &str,
    now: DateTime<Utc>,
) -> Result<DateTime<Utc>, CronTaskError> {
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

/// 由草案构建新任务（新 id、空执行历史、`next_run_at` 已计算）。
///
/// 调用方负责 `validate_draft`；本函数不重复校验名称/标识的非空约束。
pub fn build_task(draft: &CronTaskDraft) -> Result<CronTask, CronTaskError> {
    let cron_expression = normalize_cron_expression(&draft.cron_expression)?;
    Ok(CronTask {
        id: Uuid::new_v4().to_string(),
        name: draft.name.trim().to_owned(),
        server_id: draft.server_id.trim().to_owned(),
        cron_expression: cron_expression.clone(),
        action: draft.action.clone(),
        enabled: draft.enabled,
        last_run_at: None,
        next_run_at: Some(next_run_after(&cron_expression, Utc::now())?),
        last_error: None,
    })
}

/// 按草案更新既有任务：保留执行历史、重新计算下次触发时间。
pub fn apply_update(task: &mut CronTask, draft: &CronTaskDraft) -> Result<(), CronTaskError> {
    let cron_expression = normalize_cron_expression(&draft.cron_expression)?;
    task.name = draft.name.trim().to_owned();
    task.server_id = draft.server_id.trim().to_owned();
    task.cron_expression = cron_expression.clone();
    task.action = draft.action.clone();
    task.enabled = draft.enabled;
    task.next_run_at = Some(next_run_after(&cron_expression, Utc::now())?);
    Ok(())
}

/// 执行一次任务并把尝试结果写回任务状态（上次执行时间/下次计划/错误）。
///
/// 执行失败时：任务状态照常更新（便于调度器推进与诊断），返回
/// [`CronTaskError::Execution`]；`run` 详情由 `run_due` 类调用方从错误中重建
/// ——本函数只负责执行与回写。
pub async fn run_task<E>(
    executor: &E,
    task: &mut CronTask,
    now: DateTime<Utc>,
) -> Result<CronTaskRun, CronTaskError>
where
    E: super::service::CronTaskExecutor,
{
    let action = task.action.as_str();
    observability::server_cron_task_started(&task.id, &task.server_id, action);

    let execution_error = match &task.action {
        CronTaskAction::Restart => executor
            .restart_server(&task.server_id)
            .await
            .err()
            .map(|error| error.to_string()),
        CronTaskAction::Command { command } => executor
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

    // 尝试结果回写任务状态（包含失败——失败也要推进 next_run_at）。
    task.last_run_at = Some(now);
    task.next_run_at = Some(next_run_after(&task.cron_expression, now)?);
    task.last_error = execution_error;

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
