use crate::models::download::TaskProgressResponse;
use crate::services::download_manager::DownloadManager;
use tauri::State;
use uuid::Uuid;

/// 启动下载任务
#[tauri::command]
pub async fn download_file(
    url: String,
    save_path: String,           // 对应前端 savePath
    thread_count: Option<usize>, // 对应前端 threadCount
    manager: State<'_, DownloadManager>,
) -> Result<String, String> {
    let id = manager
        .create_task(&url, &save_path, thread_count.unwrap_or(8))
        .await;
    Ok(id.to_string())
}

/// 轮询进度
#[tauri::command]
pub async fn poll_task(
    id_str: String,
    manager: State<'_, DownloadManager>,
) -> Result<TaskProgressResponse, String> {
    let id = Uuid::parse_str(&id_str).map_err(|_| "Invalid ID")?;
    manager
        .get_progress_and_remove(id)
        .await
        .ok_or_else(|| "Task not found".to_string())
}

/// 批量轮询进度
#[tauri::command]
pub async fn poll_all_downloads(
    manager: State<'_, DownloadManager>,
) -> Result<Vec<TaskProgressResponse>, String> {
    Ok(manager.get_all_progress().await)
}

/// 单个任务手动清理
#[tauri::command]
pub async fn cancel_download_task(
    id_str: String,
    manager: State<'_, DownloadManager>,
) -> Result<(), String> {
    let id = Uuid::parse_str(&id_str).map_err(|e| e.to_string())?;
    // manager.remove_task(id).await;
    // Ok(())
    manager.cancel_task(id).await?;
    Ok(())
}
