//! 下载任务进度模型。

use sealantern_infra::download::DownloadSnapshot;
use serde::{Deserialize, Serialize};

/// 下载任务进度响应。
///
/// 契约字段统一使用 snake_case（`total_size` / `is_finished`），
/// 与其余后端契约保持一致。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskProgressResponse {
    pub id: String,
    pub total_size: u64,
    pub downloaded: u64,
    pub progress: f64,
    pub status: TaskStatus,
    pub is_finished: bool,
}

/// 下载任务状态。
///
/// `Error` 变体通过字段重命名序列化为 `{"Error":"message"}`，
/// 普通状态保持为字符串。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskStatus {
    Simple(String),
    Error {
        #[serde(rename = "Error", alias = "error")]
        error: String,
    },
}

impl TaskProgressResponse {
    /// 将基础设施层快照和其所属任务 ID 组合为对外响应。
    pub fn from_snapshot(id: impl Into<String>, snapshot: DownloadSnapshot) -> Self {
        let status = if let Some(error) = snapshot.error {
            TaskStatus::Error { error }
        } else if snapshot.is_finished {
            TaskStatus::Simple("Completed".to_string())
        } else if snapshot.downloaded > 0 {
            TaskStatus::Simple("Downloading".to_string())
        } else {
            TaskStatus::Simple("Pending".to_string())
        };

        Self {
            id: id.into(),
            total_size: snapshot.total_size,
            downloaded: snapshot.downloaded,
            progress: snapshot.progress_percentage,
            status,
            is_finished: snapshot.is_finished,
        }
    }
}

impl From<(String, DownloadSnapshot)> for TaskProgressResponse {
    fn from((id, snapshot): (String, DownloadSnapshot)) -> Self {
        Self::from_snapshot(id, snapshot)
    }
}

#[cfg(test)]
mod tests {
    use sealantern_infra::download::DownloadSnapshot;

    use super::{TaskProgressResponse, TaskStatus};

    #[test]
    fn task_progress_serializes_snake_case_fields() {
        let response = TaskProgressResponse::from_snapshot(
            "task-42",
            DownloadSnapshot {
                downloaded: 512,
                total_size: 1024,
                progress_percentage: 50.0,
                is_finished: false,
                error: None,
            },
        );

        assert_eq!(response.id, "task-42");
        assert_eq!(response.status, TaskStatus::Simple("Downloading".to_string()));

        let value = serde_json::to_value(response).expect("task progress should serialize");
        assert_eq!(value["total_size"], 1024);
        assert_eq!(value["is_finished"], false);
        assert!(value.get("totalSize").is_none());
        assert!(value.get("isFinished").is_none());
    }

    #[test]
    fn task_error_matches_the_external_status_shape() {
        let response = TaskProgressResponse::from_snapshot(
            "task-error",
            DownloadSnapshot {
                downloaded: 0,
                total_size: 1024,
                progress_percentage: 0.0,
                is_finished: true,
                error: Some("connection reset".to_string()),
            },
        );

        let value = serde_json::to_value(response).expect("task error should serialize");
        assert_eq!(value["status"]["Error"], "connection reset");
        assert!(value["status"].get("error").is_none());
    }

    #[test]
    fn task_progress_accepts_snake_case_fields() {
        let response: TaskProgressResponse = serde_json::from_str(
            r#"{
                "id":"task-42",
                "total_size":1024,
                "downloaded":0,
                "progress":0.0,
                "status":{"Error":"connection reset"},
                "is_finished":true
            }"#,
        )
        .expect("task progress should deserialize");

        assert_eq!(response.id, "task-42");
        assert_eq!(response.total_size, 1024);
        assert!(response.is_finished);
        assert_eq!(response.status, TaskStatus::Error { error: "connection reset".to_string() });
    }
}
