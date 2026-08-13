//! 应用更新下载与安装 Tauri 命令。
//!
//! 前端通过 `invoke` 调用这些命令，命令内部经应用装配层拿到
//! [`UpdateInstallService`] 下载更新文件、管理待安装记录并拉起安装进程。
//!
//! 错误统一为接口契约错误 [`UpdateInstallServiceError`]，可序列化回前端，
//! 不携带底层敏感细节。

use sealantern_application::services::AppServices;
use sealantern_extra::update::PendingUpdate;
use sealantern_interface::{UpdateInstallService, UpdateInstallServiceError};

/// 获取全局应用服务句柄（惰性初始化容器）。
async fn service() -> Result<AppServices, UpdateInstallServiceError> {
    AppServices::get().await.map_err(|error| {
        tracing::error!(
            target: "sealantern.tauri.update_install",
            error = %error,
            "failed to initialize application services for update installation"
        );
        UpdateInstallServiceError::OperationFailed
    })
}

/// 下载更新文件并登记为待安装，返回本地文件路径。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_download(
    url: String,
    expected_hash: Option<String>,
    version: String,
) -> Result<String, UpdateInstallServiceError> {
    service()
        .await?
        .update_install()
        .download(url, expected_hash, version)
        .await
}

/// 查询待安装的更新（下载完成但尚未安装），无则为 `None`。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_pending() -> Result<Option<PendingUpdate>, UpdateInstallServiceError> {
    service().await?.update_install().pending().await
}

/// 清除待安装的更新记录。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_clear_pending() -> Result<(), UpdateInstallServiceError> {
    service().await?.update_install().clear_pending().await
}

/// 拉起更新安装流程；Windows 下提权启动安装器，其他平台暂不支持。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_install(
    file_path: String,
    arguments: Vec<String>,
) -> Result<(), UpdateInstallServiceError> {
    service()
        .await?
        .update_install()
        .install(file_path, arguments)
        .await
}
