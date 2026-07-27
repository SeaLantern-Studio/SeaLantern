//! 下载任务进度模型

use serde::{Deserialize, Serialize};

/// 任务进度响应（与前端 DownloadTaskInfo 对应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressResponse {
    pub id: String,
    pub total_size: u64,
    pub downloaded: u64,
    pub progress: f64,
    pub status: TaskStatus,
    pub is_finished: bool,
}

/// 任务状态（与前端 TaskStatus 对应）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskStatus {
    Simple(String),
    Error {
        #[serde(rename = "Error")]
        error: String,
    },
}

impl From<sealantern_infra::download::DownloadSnapshot> for TaskProgressResponse {
    fn from(snap: sealantern_infra::download::DownloadSnapshot) -> Self {
        let status = if let Some(err) = snap.error {
            TaskStatus::Error { error: err }
        } else if snap.is_finished {
            TaskStatus::Simple("Completed".to_string())
        } else if snap.downloaded > 0 {
            TaskStatus::Simple("Downloading".to_string())
        } else {
            TaskStatus::Simple("Pending".to_string())
        };

        Self {
            id: String::new(),
            total_size: snap.total_size,
            downloaded: snap.downloaded,
            progress: snap.progress_percentage,
            status,
            is_finished: snap.is_finished,
        }
    }
}
