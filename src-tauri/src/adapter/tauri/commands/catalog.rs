//! 服务器类型目录 Tauri 命令。
//!
//! 前端通过 `invoke` 调用这些命令，命令内部经应用装配层拿到
//! [`ServerCatalogService`] 查询服务器核心类型、可用版本与下载链接。
//!
//! 错误统一为接口契约错误 [`ServerCatalogServiceError`]，可序列化回前端，
//! 不携带底层敏感细节。

use sealantern_application::services::AppServices;
use sealantern_extra::download_link::TypeDownloadLinks;
use sealantern_interface::{ServerCatalogService, ServerCatalogServiceError};

/// 查询全部可用的服务器核心类型。
#[tauri::command]
pub async fn catalog_server_types() -> Result<Vec<String>, ServerCatalogServiceError> {
    AppServices::get()
        .await
        .map_err(|_| ServerCatalogServiceError::OperationFailed)?
        .catalog()
        .server_types()
        .await
}

/// 查询指定服务器核心类型支持的全部版本。
#[tauri::command]
pub async fn catalog_versions(
    server_type: String,
) -> Result<Vec<String>, ServerCatalogServiceError> {
    AppServices::get()
        .await
        .map_err(|_| ServerCatalogServiceError::OperationFailed)?
        .catalog()
        .versions(server_type)
        .await
}

/// 查询指定服务器核心类型的版本列表与各版本下载链接。
#[tauri::command]
pub async fn catalog_details(
    server_type: String,
) -> Result<TypeDownloadLinks, ServerCatalogServiceError> {
    AppServices::get()
        .await
        .map_err(|_| ServerCatalogServiceError::OperationFailed)?
        .catalog()
        .details(server_type)
        .await
}
