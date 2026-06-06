use crate::models::settings::{AppSettings, PartialSettings};
use crate::services::global;
use font_kit::source::SystemSource;
use serde::Deserialize;
use std::collections::HashSet;
use tauri::AppHandle;
#[cfg(target_os = "macos")]
use tauri::Manager;
#[cfg(target_os = "macos")]
use window_vibrancy::{
    apply_vibrancy, clear_vibrancy, NSVisualEffectMaterial, NSVisualEffectState,
};

/// 设置更新结果
///
/// 返回新的完整设置，以及这次更新影响到的设置分组
#[derive(serde::Serialize)]
pub struct UpdateSettingsResult {
    pub settings: AppSettings,
    pub changed_groups: Vec<String>,
}

/// 插件命令名单设置
#[derive(serde::Serialize, Deserialize)]
pub struct PluginCommands {
    pub allowed: Vec<String>,
    pub blocked: Vec<String>,
}

#[tauri::command]
/// 读取当前设置
pub fn get_settings() -> AppSettings {
    global::settings_manager().get()
}

#[tauri::command]
/// 保存完整设置
///
/// # Parameters
///
/// - `settings`: 前端传入的完整设置
///
/// # Returns
///
/// 保存成功时返回 `Ok(())`
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    global::settings_manager().update(settings)
}

#[tauri::command]
/// 保存完整设置，并返回变化分组
///
/// # Parameters
///
/// - `settings`: 前端传入的完整设置
///
/// # Returns
///
/// 返回新的完整设置和变化分组
pub fn save_settings_with_diff(settings: AppSettings) -> Result<UpdateSettingsResult, String> {
    let result = global::settings_manager().update_with_diff(settings)?;
    Ok(UpdateSettingsResult {
        settings: result.settings,
        changed_groups: result
            .changed_groups
            .into_iter()
            .map(|g| format!("{:?}", g))
            .collect(),
    })
}

#[tauri::command]
/// 按局部字段更新设置
///
/// # Parameters
///
/// - `partial`: 只包含变动字段的设置片段
///
/// # Returns
///
/// 返回合并后的设置和变化分组
pub fn update_settings_partial(partial: PartialSettings) -> Result<UpdateSettingsResult, String> {
    let result = global::settings_manager().update_partial(partial)?;
    Ok(UpdateSettingsResult {
        settings: result.settings,
        changed_groups: result
            .changed_groups
            .into_iter()
            .map(|g| format!("{:?}", g))
            .collect(),
    })
}

#[tauri::command]
/// 重置设置为默认值
pub fn reset_settings() -> Result<AppSettings, String> {
    global::settings_manager().reset()
}

#[tauri::command]
/// 导出当前设置为 JSON 文本
pub fn export_settings() -> Result<String, String> {
    let s = global::settings_manager().get();
    serde_json::to_string_pretty(&s).map_err(|e| format!("Export failed: {}", e))
}

#[tauri::command]
/// 从 JSON 文本导入设置
///
/// # Parameters
///
/// - `json`: 前端传入的设置 JSON
///
/// # Returns
///
/// 导入成功时返回新的完整设置
pub fn import_settings(json: String) -> Result<AppSettings, String> {
    let s: AppSettings = serde_json::from_str(&json).map_err(|e| format!("Invalid JSON: {}", e))?;
    global::settings_manager().update(s.clone())?;
    Ok(s)
}

#[tauri::command]
/// 读取系统字体列表
pub fn get_system_fonts() -> Result<Vec<String>, String> {
    let source = SystemSource::new();
    let fonts = source
        .all_families()
        .map_err(|e| format!("Failed to get fonts: {}", e))?;

    let mut unique_fonts: HashSet<String> = HashSet::new();
    for font in fonts {
        unique_fonts.insert(font);
    }

    let mut sorted_fonts: Vec<String> = unique_fonts.into_iter().collect();
    sorted_fonts.sort_by_key(|a| a.to_lowercase());

    Ok(sorted_fonts)
}

#[tauri::command]
/// 读取插件命令名单设置
pub fn get_plugin_commands() -> PluginCommands {
    let settings = global::settings_manager().get();
    PluginCommands {
        allowed: settings.plugin_allowed_commands,
        blocked: settings.plugin_blocked_commands,
    }
}

#[tauri::command]
/// 更新插件命令名单设置
///
/// # Parameters
///
/// - `commands`: 新的允许/拦截命令名单
///
/// # Returns
///
/// 返回更新后的完整设置和变化分组
pub fn update_plugin_commands(commands: PluginCommands) -> Result<UpdateSettingsResult, String> {
    let partial = PartialSettings {
        plugin_allowed_commands: Some(commands.allowed),
        plugin_blocked_commands: Some(commands.blocked),
        ..Default::default()
    };
    let result = global::settings_manager().update_partial(partial)?;
    Ok(UpdateSettingsResult {
        settings: result.settings,
        changed_groups: result
            .changed_groups
            .into_iter()
            .map(|g| format!("{:?}", g))
            .collect(),
    })
}

#[tauri::command]
/// 应用或清除原生毛玻璃效果
///
/// # Parameters
///
/// - `enabled`: 是否启用效果
/// - `app`: 当前应用句柄
///
/// # Returns
///
/// 平台处理成功时返回 `Ok(())`
pub fn apply_acrylic(enabled: bool, app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if let Some(window) = app.get_webview_window("main") {
            if enabled {
                apply_vibrancy(
                    &window,
                    NSVisualEffectMaterial::UnderWindowBackground,
                    Some(NSVisualEffectState::Active),
                    None,
                )
                .map_err(|e| format!("Failed to apply native macOS vibrancy effect: {}", e))?;
            } else {
                clear_vibrancy(&window)
                    .map_err(|e| format!("Failed to clear native macOS vibrancy effect: {}", e))?;
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (enabled, app);
    }

    Ok(())
}
