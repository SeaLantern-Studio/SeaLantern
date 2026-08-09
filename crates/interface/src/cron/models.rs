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
pub struct CronTaskDraft {
    pub name: String,
    pub server_id: String,
    pub cron_expression: String,
    pub action: CronTaskAction,
    pub enabled: bool,
}

/// 宿主可见的定时任务快照。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
pub struct CronTaskRun {
    pub task_id: String,
    pub server_id: String,
    pub action: CronTaskAction,
    pub succeeded: bool,
    pub error: Option<String>,
}
