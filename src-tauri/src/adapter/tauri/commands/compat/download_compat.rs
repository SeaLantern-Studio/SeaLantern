//! `src/api/downloader.ts` 对应的兼容命令。
//!
//! 前端使用 `download_file` / `poll_task` / `cancel_download_task`，本模块把
//! 这些命令名注册为 Tauri 命令，内部经 [`CoreDownloadService`] 管理下载任务。
//!
//! `rename_all = "camelCase"` 使前端键名对齐 `src/api/downloader.ts` 的调用形状。

use std::sync::Arc;

use sealantern_application::service::CoreDownloadService;
use sealantern_application::services::AppServices;
use sealantern_interface::download::{DownloadRequest, DownloadTaskInfo};
use sealantern_interface::{DownloadService, DownloadServiceError};

/// 获取全局下载任务管理服务句柄（惰性初始化容器）。
async fn download_service() -> Result<Arc<CoreDownloadService>, DownloadServiceError> {
    let services = AppServices::get()
        .await
        .map_err(|_| DownloadServiceError::OperationFailed)?;
    Ok(services.download().clone())
}

/// 创建下载任务（兼容 `download_file`），返回任务 ID。
#[tauri::command(rename_all = "camelCase")]
pub async fn download_file(
    url: String,
    save_path: String,
    thread_count: usize,
) -> Result<String, DownloadServiceError> {
    let service = download_service().await?;
    service
        .create(DownloadRequest { url, save_path, thread_count })
        .await
}

/// 查询下载任务进度（兼容 `poll_task`）。
#[tauri::command(rename_all = "camelCase")]
pub async fn poll_task(id_str: String) -> Result<DownloadTaskInfo, DownloadServiceError> {
    let service = download_service().await?;
    service
        .poll(&id_str)
        .await?
        .ok_or(DownloadServiceError::TaskNotFound)
}

/// 取消下载任务（兼容 `cancel_download_task`）。
#[tauri::command(rename_all = "camelCase")]
pub async fn cancel_download_task(id_str: String) -> Result<(), DownloadServiceError> {
    let service = download_service().await?;
    service.cancel(&id_str).await
}
