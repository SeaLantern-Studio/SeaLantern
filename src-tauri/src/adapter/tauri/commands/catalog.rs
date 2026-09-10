//! 服务器类型目录 Tauri 命令。
//!
//! 前端通过 `invoke` 调用这些命令，命令内部经应用装配层拿到
//! [`ServerCatalogService`] 查询服务器核心类型、可用版本与下载链接。
//!
//! 错误统一为接口契约错误 [`ServerCatalogServiceError`]，可序列化回前端，
//! 不携带底层敏感细节。

use sealantern_application::port::ServerCatalogService;
use sealantern_application::services::AppServices;
use sealantern_contract::{DownloadLink, ServerCatalogServiceError};
use tauri::State;

/// 查询全部可用的服务器核心类型。
#[tauri::command(rename_all = "snake_case")]
pub async fn catalog_server_types(
    services: State<'_, AppServices>,
) -> Result<Vec<String>, ServerCatalogServiceError> {
    services.catalog().server_types().await
}

/// 查询指定服务器核心类型支持的全部版本。
#[tauri::command(rename_all = "snake_case")]
pub async fn catalog_versions(
    services: State<'_, AppServices>,
    server_type: String,
) -> Result<Vec<String>, ServerCatalogServiceError> {
    services.catalog().versions(server_type).await
}

/// 查询指定服务器核心类型、指定版本的下载链接。
#[tauri::command(rename_all = "snake_case")]
pub async fn catalog_details(
    services: State<'_, AppServices>,
    server_type: String,
    server_version: String,
) -> Result<DownloadLink, ServerCatalogServiceError> {
    services
        .catalog()
        .details(server_type, server_version)
        .await
}
