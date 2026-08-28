//! 下载任务管理 Tauri 命令。
//!
//! 参数/响应统一遵循 snake_case：命令参数与结构体 DTO 均显式声明
//! `rename_all = "snake_case"`，与后端契约模型保持一致。
use std::sync::Arc;

use serde::Deserialize;

use sealantern_application::port::DownloadService;
use sealantern_application::service::CoreDownloadService;
use sealantern_application::services::AppServices;
use sealantern_contract::DownloadServiceError;
use sealantern_contract::download::{DownloadRequest, DownloadTaskInfo};

/// 获取全局下载任务管理服务句柄（惰性初始化容器）。
async fn download_service() -> Result<Arc<CoreDownloadService>, DownloadServiceError> {
    let services = AppServices::get()
        .await
        .map_err(|_| DownloadServiceError::OperationFailed)?;
    Ok(services.download().clone())
}

/// 创建下载任务请求。
///
/// 前端以 snake_case 传参（`save_path`/`thread_count`），与命令层契约直接匹配。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateDownloadRequest {
    /// 下载 URL。
    pub url: String,
    /// 本地保存路径。
    pub save_path: String,
    /// 下载线程数。
    pub thread_count: usize,
}

/// 创建下载任务，返回任务信息。
#[tauri::command(rename_all = "snake_case")]
pub async fn download_create(
    request: CreateDownloadRequest,
) -> Result<DownloadTaskInfo, DownloadServiceError> {
    let service = download_service().await?;
    let id = service
        .create(DownloadRequest {
            url: request.url,
            save_path: request.save_path,
            thread_count: request.thread_count,
        })
        .await?;
    service
        .poll(&id)
        .await?
        .ok_or(DownloadServiceError::TaskNotFound)
}

/// 查询下载任务进度。
#[tauri::command(rename_all = "snake_case")]
pub async fn download_query(id: String) -> Result<DownloadTaskInfo, DownloadServiceError> {
    let service = download_service().await?;
    service
        .poll(&id)
        .await?
        .ok_or(DownloadServiceError::TaskNotFound)
}

/// 取消下载任务。
#[tauri::command(rename_all = "snake_case")]
pub async fn download_cancel(id: String) -> Result<(), DownloadServiceError> {
    let service = download_service().await?;
    service.cancel(&id).await
}
