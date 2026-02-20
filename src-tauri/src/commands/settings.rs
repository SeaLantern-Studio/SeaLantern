use crate::models::settings::AppSettings;
use crate::services::global;
use font_kit::source::SystemSource;
use std::collections::HashSet;

#[tauri::command]
pub fn get_settings() -> AppSettings {
    global::settings_manager().get()
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    global::settings_manager().update(settings)
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
    // Windows 和 macOS 都支持毛玻璃/亚克力效果
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        Ok(true)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Ok(false)
    }
}

#[tauri::command]
pub fn apply_acrylic(window: tauri::Window, enabled: bool, dark_mode: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use window_vibrancy;
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
    #[cfg(target_os = "macos")]
    {
        let _ = dark_mode; // macOS 不使用此参数
        use window_vibrancy::NSVisualEffectMaterial;
        if enabled {
            // macOS 使用 vibrancy 效果，Sidebar 材质适合侧边栏风格的应用
            window_vibrancy::apply_vibrancy(
                &window,
                NSVisualEffectMaterial::Sidebar,
                None,
                Some(10.0),
            )
            .map_err(|e| format!("Failed to apply vibrancy: {}", e))?;
        } else {
            // macOS 清除 vibrancy 效果 - 使用 WindowBackground 作为默认
            window_vibrancy::apply_vibrancy(
                &window,
                NSVisualEffectMaterial::WindowBackground,
                None,
                Some(10.0),
            )
            .map_err(|e| format!("Failed to clear vibrancy: {}", e))?;
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
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
