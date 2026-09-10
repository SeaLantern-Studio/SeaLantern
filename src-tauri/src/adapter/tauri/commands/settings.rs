//! 设置管理 Tauri 命令。

use std::sync::Arc;

use sealantern_application::port::SettingsService;
use sealantern_application::service::CoreSettingsService;
use sealantern_application::services::AppServices;
use sealantern_contract::SettingsServiceError;
use sealantern_contract::settings::{
    AppSettings, PartialAppSettings, SettingsOverview, UpdateResult,
};
use tauri::{AppHandle, Manager, State};

use crate::desktop::{AutoLightweightState, tray};

/// 获取宿主注入的设置管理服务句柄。
fn settings_service(services: &AppServices) -> Arc<CoreSettingsService> {
    services.settings().clone()
}

/// 获取设置概览（所有分组及其设置项列表）。
#[tauri::command(rename_all = "snake_case")]
pub async fn settings_overview(
    services: State<'_, AppServices>,
) -> Result<SettingsOverview, SettingsServiceError> {
    settings_service(&services).settings_overview().await
}

/// 获取当前完整设置。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_settings(
    services: State<'_, AppServices>,
) -> Result<AppSettings, SettingsServiceError> {
    settings_service(&services).get().await
}

/// 全量替换当前设置。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_settings(
    app: AppHandle,
    services: State<'_, AppServices>,
    settings: AppSettings,
) -> Result<UpdateResult, SettingsServiceError> {
    let result = settings_service(&services).update(settings).await?;
    sync_auto_lightweight(&app, &result.settings);
    Ok(result)
}

/// 部分更新当前设置。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_settings_partial(
    app: AppHandle,
    services: State<'_, AppServices>,
    partial: PartialAppSettings,
) -> Result<UpdateResult, SettingsServiceError> {
    let result = settings_service(&services).update_partial(partial).await?;
    sync_auto_lightweight(&app, &result.settings);
    Ok(result)
}

/// 恢复默认设置。
#[tauri::command(rename_all = "snake_case")]
pub async fn reset_settings(
    app: AppHandle,
    services: State<'_, AppServices>,
) -> Result<AppSettings, SettingsServiceError> {
    let settings = settings_service(&services).reset().await?;
    sync_auto_lightweight(&app, &settings);
    Ok(settings)
}

/// 导出当前设置为 JSON 字符串。
#[tauri::command(rename_all = "snake_case")]
pub async fn export_settings(
    services: State<'_, AppServices>,
) -> Result<String, SettingsServiceError> {
    settings_service(&services).export_json().await
}

/// 从 JSON 字符串导入设置。
#[tauri::command(rename_all = "snake_case")]
pub async fn import_settings(
    app: AppHandle,
    services: State<'_, AppServices>,
    json: String,
) -> Result<UpdateResult, SettingsServiceError> {
    let result = settings_service(&services).import_json(&json).await?;
    sync_auto_lightweight(&app, &result.settings);
    Ok(result)
}

fn sync_auto_lightweight(app: &AppHandle, settings: &AppSettings) {
    app.state::<AutoLightweightState>()
        .configure(settings.auto_lightweight_minutes);
    tray::schedule_auto_lightweight(app);
}
