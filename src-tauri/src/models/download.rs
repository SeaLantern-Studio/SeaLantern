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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseDownloadLinks {
    pub server_types: Vec<String>,
    pub links: Vec<TypeDownloadLinks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDownloadLinks {
    pub server_type: String,
    pub versions: Vec<String>,
    pub links: Vec<DownloadLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadLink {
    pub version: String,
    pub file_name: String,
    pub url: String,
}

pub struct LinkManager;
