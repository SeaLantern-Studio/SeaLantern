//! 应用更新下载与安装 Tauri 命令。
//!
//! 前端通过 `invoke` 调用这些命令，命令内部经应用装配层拿到
//! [`UpdateInstallService`] 下载更新文件、管理待安装记录并拉起安装进程。
//!
//! 错误统一为接口契约错误 [`UpdateInstallServiceError`]，可序列化回前端，
//! 不携带底层敏感细节。

use sealantern_application::port::UpdateInstallService;
use sealantern_application::services::AppServices;
use sealantern_contract::UpdateInstallServiceError;
use sealantern_contract::update::PendingUpdate;
use tauri::State;

/// 下载更新文件并登记为待安装，返回本地文件路径。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_download(
    services: State<'_, AppServices>,
    url: String,
    expected_hash: Option<String>,
    version: String,
) -> Result<String, UpdateInstallServiceError> {
    services
        .update_install()
        .download(url, expected_hash, version)
        .await
}

/// 查询待安装的更新（下载完成但尚未安装），无则为 `None`。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_pending(
    services: State<'_, AppServices>,
) -> Result<Option<PendingUpdate>, UpdateInstallServiceError> {
    services.update_install().pending().await
}

/// 清除待安装的更新记录。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_clear_pending(
    services: State<'_, AppServices>,
) -> Result<(), UpdateInstallServiceError> {
    services.update_install().clear_pending().await
}

/// 拉起更新安装流程；Windows 下提权启动安装器，其他平台暂不支持。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_install(
    services: State<'_, AppServices>,
    file_path: String,
    arguments: Vec<String>,
) -> Result<(), UpdateInstallServiceError> {
    services
        .update_install()
        .install(file_path, arguments)
        .await
}
