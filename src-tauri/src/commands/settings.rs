use crate::models::settings::{AppSettings, PartialSettings};
use crate::services::global;
use font_kit::source::SystemSource;
use std::collections::HashSet;
#[cfg(target_os = "windows")]
use window_vibrancy;

#[derive(serde::Serialize)]
pub struct UpdateSettingsResult {
    pub settings: AppSettings,
    pub changed_groups: Vec<String>,
}

#[tauri::command]
pub fn get_settings() -> AppSettings {
    global::settings_manager().get()
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    global::settings_manager().update(settings)
}

#[tauri::command]
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
pub fn reset_settings() -> Result<AppSettings, String> {
    global::settings_manager().reset()
}

#[tauri::command]
pub fn export_settings() -> Result<String, String> {
    let s = global::settings_manager().get();
    serde_json::to_string_pretty(&s).map_err(|e| format!("Export failed: {}", e))
}

#[tauri::command]
pub fn import_settings(json: String) -> Result<AppSettings, String> {
    let s: AppSettings = serde_json::from_str(&json).map_err(|e| format!("Invalid JSON: {}", e))?;
    global::settings_manager().update(s.clone())?;
    Ok(s)
}

#[tauri::command]
pub fn check_acrylic_support() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(true)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(false)
    }
}

#[tauri::command]
pub fn apply_acrylic(window: tauri::Window, enabled: bool, dark_mode: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if enabled {
            // 根据主题选择不同的亚克力颜色
            // 格式: (R, G, B, A) - A 是透明度 (0-255)
            let color = if dark_mode {
                // 暗色主题: 深色半透明背景
                Some((15, 17, 23, 200))
            } else {
                // 浅色主题: 浅色半透明背景
                Some((248, 250, 252, 200))
            };
            window_vibrancy::apply_acrylic(&window, color)
                .map_err(|e| format!("Failed to apply acrylic: {}", e))?;
        } else {
            window_vibrancy::clear_acrylic(&window)
                .map_err(|e| format!("Failed to clear acrylic: {}", e))?;
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (window, enabled, dark_mode);
    }
    Ok(())
}

#[tauri::command]
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
