//! `src/api/settings.ts` 对应的兼容命令。
//!
//! 前端使用旧命令名（如 `get_settings`、`save_settings`），本模块把这些
//! 旧名注册为 Tauri 命令，内部经 [`SettingsManager`] 实现配置的读写操作。

use sealantern_application::services::AppServices;
use sealantern_extra::models::{AppSettings, PartialAppSettings, UpdateResult};

/// 获取应用设置（兼容 `get_settings`）。
#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    let services = AppServices::get().await.map_err(|e| e.to_string())?;
    let manager = services.settings_manager().map_err(|e| e.to_string())?;
    let manager = manager.lock().await;
    Ok(manager.get().clone())
}

/// 保存应用设置（兼容 `save_settings`）。
#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    let services = AppServices::get().await.map_err(|e| e.to_string())?;
    let manager = services.settings_manager().map_err(|e| e.to_string())?;
    let mut manager = manager.lock().await;
    manager.update(settings).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 保存设置并返回变更分组（兼容 `save_settings_with_diff`）。
#[tauri::command]
pub async fn save_settings_with_diff(settings: AppSettings) -> Result<UpdateResult, String> {
    let services = AppServices::get().await.map_err(|e| e.to_string())?;
    let manager = services.settings_manager().map_err(|e| e.to_string())?;
    let mut manager = manager.lock().await;
    manager.update(settings).await.map_err(|e| e.to_string())
}

/// 部分更新设置（兼容 `update_settings_partial`）。
#[tauri::command]
pub async fn update_settings_partial(partial: PartialAppSettings) -> Result<UpdateResult, String> {
    let services = AppServices::get().await.map_err(|e| e.to_string())?;
    let manager = services.settings_manager().map_err(|e| e.to_string())?;
    let mut manager = manager.lock().await;
    manager
        .update_partial(partial)
        .await
        .map_err(|e| e.to_string())
}

/// 重置设置为默认值（兼容 `reset_settings`）。
#[tauri::command]
pub async fn reset_settings() -> Result<AppSettings, String> {
    let services = AppServices::get().await.map_err(|e| e.to_string())?;
    let manager = services.settings_manager().map_err(|e| e.to_string())?;
    let mut manager = manager.lock().await;
    manager.reset().await.map_err(|e| e.to_string())
}

/// 导出设置为 JSON 字符串（兼容 `export_settings`）。
#[tauri::command]
pub async fn export_settings() -> Result<String, String> {
    let services = AppServices::get().await.map_err(|e| e.to_string())?;
    let manager = services.settings_manager().map_err(|e| e.to_string())?;
    let manager = manager.lock().await;
    manager.export_json().map_err(|e| e.to_string())
}

/// 从 JSON 字符串导入设置（兼容 `import_settings`）。
#[tauri::command]
pub async fn import_settings(json: String) -> Result<AppSettings, String> {
    let services = AppServices::get().await.map_err(|e| e.to_string())?;
    let manager = services.settings_manager().map_err(|e| e.to_string())?;
    let mut manager = manager.lock().await;
    let result = manager
        .import_json(&json)
        .await
        .map_err(|e| e.to_string())?;
    Ok(result.settings)
}
