//! 设置管理 Tauri 命令。

use std::sync::Arc;

use sealantern_application::service::CoreSettingsService;
use sealantern_application::services::AppServices;
use sealantern_extra::models::{AppSettings, PartialAppSettings, UpdateResult};
use sealantern_interface::settings::SettingsOverview;
use sealantern_interface::{SettingsService, SettingsServiceError};

/// 获取全局设置管理服务句柄（惰性初始化容器）。
async fn settings_service() -> Result<Arc<CoreSettingsService>, SettingsServiceError> {
    AppServices::settings_service()
        .await
        .map_err(|_| SettingsServiceError::Unavailable)
}

/// 获取设置概览（所有分组及其设置项列表）。
#[tauri::command(rename_all = "snake_case")]
pub async fn settings_overview() -> Result<SettingsOverview, SettingsServiceError> {
    settings_service().await?.settings_overview().await
}

/// 获取当前完整设置。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_settings() -> Result<AppSettings, SettingsServiceError> {
    settings_service().await?.get().await
}

/// 全量替换当前设置。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_settings(settings: AppSettings) -> Result<UpdateResult, SettingsServiceError> {
    settings_service().await?.update(settings).await
}

/// 部分更新当前设置。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_settings_partial(
    partial: PartialAppSettings,
) -> Result<UpdateResult, SettingsServiceError> {
    settings_service().await?.update_partial(partial).await
}

/// 恢复默认设置。
#[tauri::command(rename_all = "snake_case")]
pub async fn reset_settings() -> Result<AppSettings, SettingsServiceError> {
    settings_service().await?.reset().await
}

/// 导出当前设置为 JSON 字符串。
#[tauri::command(rename_all = "snake_case")]
pub async fn export_settings() -> Result<String, SettingsServiceError> {
    settings_service().await?.export_json().await
}

/// 从 JSON 字符串导入设置。
#[tauri::command(rename_all = "snake_case")]
pub async fn import_settings(json: String) -> Result<UpdateResult, SettingsServiceError> {
    settings_service().await?.import_json(&json).await
}
