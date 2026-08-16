//! 设置管理 Tauri 命令。

use std::sync::Arc;

use sealantern_application::service::CoreSettingsService;
use sealantern_application::services::AppServices;
use sealantern_extra::models::{AppSettings, PartialAppSettings, UpdateResult};
use sealantern_interface::settings::SettingsOverview;
use sealantern_interface::{SettingsService, SettingsServiceError};
use tauri::{AppHandle, Manager};

use crate::desktop::{tray, AutoLightweightState};

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
pub async fn update_settings(
    app: AppHandle,
    settings: AppSettings,
) -> Result<UpdateResult, SettingsServiceError> {
    let result = settings_service().await?.update(settings).await?;
    sync_auto_lightweight(&app, &result.settings);
    Ok(result)
}

/// 部分更新当前设置。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_settings_partial(
    app: AppHandle,
    partial: PartialAppSettings,
) -> Result<UpdateResult, SettingsServiceError> {
    let result = settings_service().await?.update_partial(partial).await?;
    sync_auto_lightweight(&app, &result.settings);
    Ok(result)
}

/// 恢复默认设置。
#[tauri::command(rename_all = "snake_case")]
pub async fn reset_settings(app: AppHandle) -> Result<AppSettings, SettingsServiceError> {
    let settings = settings_service().await?.reset().await?;
    sync_auto_lightweight(&app, &settings);
    Ok(settings)
}

/// 导出当前设置为 JSON 字符串。
#[tauri::command(rename_all = "snake_case")]
pub async fn export_settings() -> Result<String, SettingsServiceError> {
    settings_service().await?.export_json().await
}

/// 从 JSON 字符串导入设置。
#[tauri::command(rename_all = "snake_case")]
pub async fn import_settings(
    app: AppHandle,
    json: String,
) -> Result<UpdateResult, SettingsServiceError> {
    let result = settings_service().await?.import_json(&json).await?;
    sync_auto_lightweight(&app, &result.settings);
    Ok(result)
}

fn sync_auto_lightweight(app: &AppHandle, settings: &AppSettings) {
    app.state::<AutoLightweightState>()
        .configure(settings.auto_lightweight_minutes);
    tray::schedule_auto_lightweight(app);
}
