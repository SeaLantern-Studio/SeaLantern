//! 系统资源信息 Tauri 命令。
//!
//! 前端通过 `invoke` 调用这些命令，命令内部经应用装配层拿到
//! [`CoreSystemService`](sealantern_application::service::CoreSystemService)
//! 采集系统资源快照、进程占用与目录磁盘占用。
//!
//! 错误统一为接口契约错误 [`SystemServiceError`]，可序列化回前端，
//! 不携带底层敏感细节。

use std::sync::Arc;

use sealantern_application::port::SystemService;
use sealantern_application::service::CoreSystemService;
use sealantern_application::services::AppServices;
use sealantern_contract::SystemServiceError;
use sealantern_contract::system::{ServerResourceUsage, SystemSnapshot};

/// 获取全局系统资源信息服务句柄（惰性初始化容器）。
async fn system_service() -> Result<Arc<CoreSystemService>, SystemServiceError> {
    let services = AppServices::get()
        .await
        .map_err(|_| SystemServiceError::OperationFailed)?;
    Ok(services.system().clone())
}

/// 采集整机系统资源快照。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_system_snapshot() -> Result<SystemSnapshot, SystemServiceError> {
    let service = system_service().await?;
    service.system_snapshot().await
}

/// 获取默认运行路径。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_default_run_path() -> Result<String, SystemServiceError> {
    let service = system_service().await?;
    service.default_run_path().await
}

/// 按实例标识采集服务器资源占用。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_server_resource_usage(
    instance_id: String,
) -> Result<ServerResourceUsage, SystemServiceError> {
    let service = system_service().await?;
    service.server_resource_usage(&instance_id).await
}
