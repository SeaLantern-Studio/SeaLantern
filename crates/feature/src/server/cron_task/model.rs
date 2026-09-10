use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 定时任务的实际服务器动作。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CronTaskAction {
    Restart,
    Command { command: String },
}

impl CronTaskAction {
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::Restart => "restart",
            Self::Command { .. } => "command",
        }
    }
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

/// 已持久化的服务器定时任务。
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

/// 定时任务持久化文件的根对象。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CronTaskList {
    pub tasks: Vec<CronTask>,
}

/// 一次定时任务执行的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronTaskRun {
    pub task_id: String,
    pub server_id: String,
    pub action: CronTaskAction,
    pub succeeded: bool,
    pub error: Option<String>,
}
