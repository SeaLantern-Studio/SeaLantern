use std::time::Duration;

use serde::Serialize;
use tauri::{async_runtime, AppHandle, Emitter};
use tokio::time::sleep;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTaskInfo {
    pub id: String,
    pub total_size: u64,
    pub downloaded: u64,
    pub progress: f64,
    pub status: TaskStatus,
    pub is_finished: bool,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum TaskStatus {
    Simple(String),
    Error { #[serde(rename = "Error")] error: String },
}

impl DownloadTaskInfo {
    fn from_snapshot(id: &str, snapshot: sealantern_infra::download::DownloadSnapshot) -> Self {
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
            id: id.to_string(),
            total_size: snapshot.total_size,
            downloaded: snapshot.downloaded,
            progress: snapshot.progress_percentage,
            status,
            is_finished: snapshot.is_finished,
        }
    }
}

#[tauri::command]
pub async fn download_file(
    app: AppHandle,
    url: String,
    save_path: String,
    thread_count: Option<usize>,
) -> Result<String, String> {
    let thread_count = thread_count.unwrap_or(32).clamp(1, 256);
    let (id, status) = sealantern_infra::download::DownloadManager::instance()
        .create_with_handle(&url, &save_path, thread_count)
        .await
        .map_err(|e| e.to_string())?;

    let task_id = id.to_string();
    let task_id_cloned = task_id.clone();
    let app_handle = app.clone();
    async_runtime::spawn(async move {
        loop {
            let snapshot = status.snapshot().await;
            let progress = DownloadTaskInfo::from_snapshot(&task_id_cloned, snapshot);
            let _ = app_handle.emit("download-task-progress", &progress);
            if progress.is_finished {
                break;
            }
            sleep(Duration::from_millis(800)).await;
        }
    });

    Ok(task_id)
}

#[tauri::command]
pub async fn poll_task(id_str: String) -> Result<DownloadTaskInfo, String> {
    let id = Uuid::parse_str(&id_str).map_err(|e| e.to_string())?;
    let snapshot = sealantern_infra::download::DownloadManager::instance()
        .get_progress(id)
        .await
        .ok_or_else(|| "task not found".to_string())?;
    Ok(DownloadTaskInfo::from_snapshot(&id_str, snapshot))
}

#[tauri::command]
pub async fn cancel_download_task(id_str: String) -> Result<(), String> {
    let id = Uuid::parse_str(&id_str).map_err(|e| e.to_string())?;
    sealantern_infra::download::DownloadManager::instance().cancel(id).await;
    Ok(())
}
