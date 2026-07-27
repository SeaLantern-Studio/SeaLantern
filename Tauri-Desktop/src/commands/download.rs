//! 下载相关命令

use sealantern_extra::models::{DownloadLink, LinkManager, TaskProgressResponse};
use sealantern_infra::download::DownloadManager;
use uuid::Uuid;

/// 启动下载任务
#[tauri::command]
pub async fn download_file(
    url: String,
    save_path: String,
    thread_count: Option<usize>,
) -> Result<String, String> {
    let manager = DownloadManager::instance();
    let id = manager
        .create(&url, &save_path, thread_count.unwrap_or(8))
        .await
        .map_err(|e| e.to_string())?;
    Ok(id.to_string())
}

/// 轮询进度
#[tauri::command]
pub async fn poll_task(id_str: String) -> Result<TaskProgressResponse, String> {
    let manager = DownloadManager::instance();
    let id = Uuid::parse_str(&id_str).map_err(|_| "Invalid ID".to_string())?;
    let snap = manager
        .get_progress(id)
        .await
        .ok_or_else(|| "Task not found".to_string())?;

    let mut response = TaskProgressResponse::from(snap);
    response.id = id_str;
    Ok(response)
}

/// 批量轮询进度
#[tauri::command]
pub async fn poll_all_downloads() -> Result<Vec<TaskProgressResponse>, String> {
    let manager = DownloadManager::instance();
    let all_progress = manager.get_all_progress().await;

    Ok(all_progress
        .into_iter()
        .map(|(id, snap)| {
            let mut response = TaskProgressResponse::from(snap);
            response.id = id.to_string();
            response
        })
        .collect())
}

/// 取消下载任务
#[tauri::command]
pub async fn cancel_download_task(id_str: String) -> Result<(), String> {
    let manager = DownloadManager::instance();
    let id = Uuid::parse_str(&id_str).map_err(|e| e.to_string())?;
    manager.cancel(id).await;
    Ok(())
}

/* 服务器核心下载 */

/// 获取可用的服务器类型列表
#[tauri::command]
pub async fn get_server_types() -> Result<Vec<String>, String> {
    LinkManager::get_server_types().await
}

/// 获取指定类型的服务器版本列表
#[tauri::command]
pub async fn get_versions_by_type(server_type: String) -> Result<Vec<String>, String> {
    LinkManager::get_versions_by_type(&server_type).await
}

/// 获取指定类型和版本的下载信息
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