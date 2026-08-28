//! 应用更新检查 Tauri 命令。

use std::sync::Arc;

use sealantern_application::port::UpdateCheckService;
use sealantern_application::service::CoreUpdateCheckService;
use sealantern_application::services::AppServices;
use sealantern_contract::UpdateCheckServiceError;
use sealantern_contract::update::UpdateInfo;

async fn update_service() -> Result<Arc<CoreUpdateCheckService>, UpdateCheckServiceError> {
    let services = AppServices::get().await.map_err(|error| {
        tracing::error!(
            target: "sealantern.tauri.update",
            error = %error,
            "failed to initialize application services for update check"
        );
        UpdateCheckServiceError::CheckFailed
    })?;
    Ok(services.update().clone())
}

/// 检查当前平台是否存在应用更新。
#[tauri::command(rename_all = "snake_case")]
pub async fn check_update() -> Result<UpdateInfo, UpdateCheckServiceError> {
    update_service().await?.check().await
}
