use crate::models::download::{DownloadLink, LinkManager, TaskProgressResponse};
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
    manager.cancel_task(id).await?;
    Ok(())
}

/* 服务器核心下载 */

#[tauri::command]
pub async fn get_server_types() -> Result<Vec<String>, String> {
    LinkManager::get_server_types().await
}

#[tauri::command]
pub async fn get_versions_by_type(server_type: String) -> Result<Vec<String>, String> {
    LinkManager::get_versions_by_type(&server_type).await
}

#[tauri::command]
pub async fn get_download_info(
    server_type: String,
    version: String,
) -> Result<DownloadLink, String> {
    let type_group = LinkManager::get_type_by_name(&server_type).await?;
    type_group
        .get_link_by_version(&version)
        .cloned()
        .ok_or_else(|| format!("Version {} not found", version))
}
