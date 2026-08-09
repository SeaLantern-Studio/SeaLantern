//! 设置信息 Tauri 命令。
//!
//! 提供 [`get_settings_overview`] 命令，经 [`AppServices`] 访问
//! [`CoreSettingsService`] 并向前端返回设置分组、设置项列表等信息。

use tauri::command;

use sealantern_interface::settings::SettingsOverview;
use sealantern_interface::SettingsService;

use sealantern_application::services::AppServices;

/// 获取设置概览（所有分组及其设置项列表）。
#[command]
pub async fn get_settings_overview() -> Result<SettingsOverview, String> {
    let services = AppServices::get().await.map_err(|e| e.to_string())?;
    services
        .settings()
        .settings_overview()
        .await
        .map_err(|e| e.to_string())
}
