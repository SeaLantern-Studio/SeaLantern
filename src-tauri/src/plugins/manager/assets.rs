use super::{PluginManager, PluginState};
use std::path::PathBuf;

pub(super) fn get_plugin_settings(
    manager: &PluginManager,
    plugin_id: &str,
) -> Result<serde_json::Value, String> {
    let plugin_info = manager
        .plugins
        .get(plugin_id)
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

    let plugin_path = PathBuf::from(&plugin_info.path);
    let settings_path = plugin_path.join("settings.json");

    if !settings_path.exists() {
        return Ok(serde_json::json!({}));
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings file: {}", e))?;

    Ok(settings)
}

pub(super) fn set_plugin_settings(
    manager: &PluginManager,
    plugin_id: &str,
    settings: serde_json::Value,
) -> Result<(), String> {
    let plugin_info = manager
        .plugins
        .get(plugin_id)
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

    let plugin_path = PathBuf::from(&plugin_info.path);
    let settings_path = plugin_path.join("settings.json");

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    Ok(())
}

pub(super) fn get_plugin_icon(manager: &PluginManager, plugin_id: &str) -> Result<String, String> {
    let plugin_info = manager
        .plugins
        .get(plugin_id)
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

    let plugin_path = PathBuf::from(&plugin_info.path);

    let icon_filename = plugin_info.manifest.icon.as_deref().unwrap_or("icon.png");

    if icon_filename.contains("..") || std::path::Path::new(icon_filename).is_absolute() {
        return Err(format!("Plugin icon path '{}' is not safe", icon_filename));
    }

    let icon_path = plugin_path.join(icon_filename);

    if !icon_path.exists() {
        return Ok(String::new());
    }

    let content =
        std::fs::read(&icon_path).map_err(|e| format!("Failed to read icon file: {}", e))?;

    let extension = icon_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let mime_type = match extension {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        _ => "image/png",
    };

    if extension != "svg" && extension != "gif" {
        match image::load_from_memory(&content) {
            Ok(img) => {
                let width = img.width();
                let height = img.height();

                if width != height {
                    return Err(format!("Icon must be square, got {}x{}", width, height));
                }

                if width > 2048 || height > 2048 {
                    return Err(format!(
                        "Icon size must not exceed 2048x2048, got {}x{}",
                        width, height
                    ));
                }
            }
            Err(e) => {
                return Err(format!("Failed to decode icon image: {}", e));
            }
        }
    }

    use base64::Engine;
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&content);

    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}

pub(super) fn get_plugin_css(manager: &PluginManager, plugin_id: &str) -> Result<String, String> {
    let plugin_info = manager
        .plugins
        .get(plugin_id)
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

    let plugin_path = PathBuf::from(&plugin_info.path);
    let css_path = plugin_path.join("style.css");

    if !css_path.exists() {
        return Ok(String::new());
    }

    std::fs::read_to_string(&css_path).map_err(|e| format!("Failed to read CSS file: {}", e))
}

pub(super) fn get_all_plugin_css(manager: &PluginManager) -> Result<Vec<(String, String)>, String> {
    let mut result = Vec::new();

    for (plugin_id, plugin_info) in &manager.plugins {
        if matches!(plugin_info.state, PluginState::Enabled) {
            let plugin_path = PathBuf::from(&plugin_info.path);
            let css_path = plugin_path.join("style.css");
            if css_path.exists() {
                if let Ok(css_content) = std::fs::read_to_string(&css_path) {
                    if !css_content.is_empty() {
                        result.push((plugin_id.clone(), css_content));
                    }
                }
            }
        }
    }

    Ok(result)
}
