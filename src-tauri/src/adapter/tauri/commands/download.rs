//! 下载任务管理（后端语义 API，保留参考）。
//!
//! 前端命令名已由 `compat` 兼容层接管（`download_file`/`poll_task`/...），
//! 本文件保留领域语义命名的服务调用函数作为后端语义 API，不注册为 Tauri
//! 命令，供未来前端重构时直接使用。
//!
//! 参数/响应遵循「前端驼峰、后端蛇拼」：结构体 DTO 用 `rename_all = "camelCase"`
//! 输出 camelCase（匹配前端规范），Rust 字段保持 snake_case。
#![allow(dead_code)]

use std::sync::Arc;

use serde::Deserialize;

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

/// 创建下载任务请求。
///
/// 前端传 camelCase（`savePath`/`threadCount`），内部映射为 snake_case。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDownloadRequest {
    /// 下载 URL。
    pub url: String,
    /// 本地保存路径。
    pub save_path: String,
    /// 下载线程数。
    pub thread_count: usize,
}

/// 创建下载任务，返回任务信息。
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
pub async fn download_query(id: String) -> Result<DownloadTaskInfo, DownloadServiceError> {
    let service = download_service().await?;
    service
        .poll(&id)
        .await?
        .ok_or(DownloadServiceError::TaskNotFound)
}

/// 取消下载任务。
pub async fn download_cancel(id: String) -> Result<(), DownloadServiceError> {
    let service = download_service().await?;
    service.cancel(&id).await
}
