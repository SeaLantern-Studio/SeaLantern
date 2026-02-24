use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Downloading,
    Completed,
    Error(String),
}

/// 用于 API 返回的简易快照
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgressResponse {
    pub id: Uuid,
    pub total_size: u64,
    pub downloaded: u64,
    pub progress: f64,
    pub status: TaskStatus,
    pub is_finished: bool,
}
