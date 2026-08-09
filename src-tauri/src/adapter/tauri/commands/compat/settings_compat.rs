//! `src/api/settings.ts` 对应的兼容命令。
//!
//! 前端使用旧命令名（如 `get_settings`、`save_settings`），本模块把这些
//! 旧名注册为 Tauri 命令，并把历史返回形状适配到统一设置服务。

use std::sync::Arc;

use sealantern_application::service::CoreSettingsService;
use sealantern_application::services::AppServices;
use sealantern_extra::models::{AppSettings, PartialAppSettings, UpdateResult};
use sealantern_interface::{SettingsService, SettingsServiceError};

/// 获取统一设置服务，并将容器初始化失败收敛为契约错误。
async fn settings_service() -> Result<Arc<CoreSettingsService>, String> {
    AppServices::settings_service().await.map_err(|error| {
        tracing::error!(
            target: "sealantern.tauri.settings_compat",
            error = %error,
            "failed to acquire settings service"
        );
        SettingsServiceError::Unavailable.to_string()
    })
}

/// 获取应用设置（兼容 `get_settings`）。
#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    settings_service()
        .await?
        .get()
        .await
        .map_err(|e| e.to_string())
}

/// 保存应用设置（兼容 `save_settings`）。
#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    settings_service()
        .await?
        .update(settings)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// 保存设置并返回变更分组（兼容 `save_settings_with_diff`）。
#[tauri::command]
pub async fn save_settings_with_diff(settings: AppSettings) -> Result<UpdateResult, String> {
    settings_service()
        .await?
        .update(settings)
        .await
        .map_err(|e| e.to_string())
}

/// 部分更新设置（兼容 `update_settings_partial`）。
#[tauri::command]
pub async fn update_settings_partial(partial: PartialAppSettings) -> Result<UpdateResult, String> {
    settings_service()
        .await?
        .update_partial(partial)
        .await
        .map_err(|e| e.to_string())
}

/// 重置设置为默认值（兼容 `reset_settings`）。
#[tauri::command]
pub async fn reset_settings() -> Result<AppSettings, String> {
    settings_service()
        .await?
        .reset()
        .await
        .map_err(|e| e.to_string())
}

/// 导出设置为 JSON 字符串（兼容 `export_settings`）。
#[tauri::command]
pub async fn export_settings() -> Result<String, String> {
    settings_service()
        .await?
        .export_json()
        .await
        .map_err(|e| e.to_string())
}

/// 从 JSON 字符串导入设置（兼容 `import_settings`）。
#[tauri::command]
pub async fn import_settings(json: String) -> Result<AppSettings, String> {
    let result = settings_service()
        .await?
        .import_json(&json)
        .await
        .map_err(|e| e.to_string())?;
    Ok(result.settings)
}
