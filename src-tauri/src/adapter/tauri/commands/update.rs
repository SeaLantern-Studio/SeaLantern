//! 应用更新检查 Tauri 命令。

use std::sync::Arc;

use sealantern_application::port::UpdateCheckService;
use sealantern_application::service::CoreUpdateCheckService;
use sealantern_application::services::AppServices;
use sealantern_contract::UpdateCheckServiceError;
use sealantern_contract::update::UpdateInfo;
use tauri::State;

fn update_service(services: &AppServices) -> Arc<CoreUpdateCheckService> {
    services.update().clone()
}

/// 检查当前平台是否存在应用更新。
#[tauri::command(rename_all = "snake_case")]
pub async fn check_update(
    services: State<'_, AppServices>,
) -> Result<UpdateInfo, UpdateCheckServiceError> {
    update_service(&services).check().await
}
