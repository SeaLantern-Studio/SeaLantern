use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 定时任务执行的服务器动作。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CronTaskAction {
    /// 重启指定服务器。
    Restart,
    /// 向指定服务器控制台发送命令。
    Command { command: String },
}

/// 创建或更新定时任务时可修改的字段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CronTaskDraft {
    pub name: String,
    pub server_id: String,
    pub cron_expression: String,
    pub action: CronTaskAction,
    pub enabled: bool,
}

/// 宿主可见的定时任务快照。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CronTask {
    pub id: String,
    pub name: String,
    pub server_id: String,
    pub cron_expression: String,
    pub action: CronTaskAction,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

/// 一次定时任务执行结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CronTaskRun {
    pub task_id: String,
    pub server_id: String,
    pub action: CronTaskAction,
    pub succeeded: bool,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::CronTaskServiceError;

    use super::{CronTaskAction, CronTaskDraft};

    #[test]
    fn cron_task_contract_uses_snake_case_fields() {
        let draft = CronTaskDraft {
            name: "Nightly restart".to_owned(),
            server_id: "server-a".to_owned(),
            cron_expression: "0 0 4 * * *".to_owned(),
            action: CronTaskAction::Restart,
            enabled: true,
        };

        let value = serde_json::to_value(draft).expect("serialize cron task draft");

        assert_eq!(value["server_id"], "server-a");
        assert_eq!(value["cron_expression"], "0 0 4 * * *");
        assert_eq!(value["action"]["kind"], "restart");
        assert!(value.get("serverId").is_none());
        assert!(value.get("cronExpression").is_none());
    }

    #[test]
    fn cron_task_errors_use_snake_case_variants() {
        let value = serde_json::to_value(CronTaskServiceError::TaskNotFound)
            .expect("serialize cron task error");

        assert_eq!(value, "task_not_found");
    }
}
