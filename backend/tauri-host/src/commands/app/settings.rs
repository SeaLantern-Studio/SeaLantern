use crate::models::settings::{AppSettings, PartialSettings, TextColorOverrides};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::models::settings::{WINDOW_EFFECT_AUTO, WINDOW_EFFECT_OFF};
use crate::services::global;
use font_kit::source::SystemSource;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use tauri::window::Color;
use tauri::AppHandle;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use tauri::Manager;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use tauri::WebviewWindow;
#[cfg(target_os = "windows")]
use window_vibrancy::{
    apply_acrylic as apply_windows_acrylic, apply_blur, apply_mica,
    clear_acrylic as clear_windows_acrylic, clear_blur, clear_mica,
};
#[cfg(target_os = "macos")]
use window_vibrancy::{
    apply_vibrancy, clear_vibrancy, NSVisualEffectMaterial, NSVisualEffectState,
};

const PERSONALIZATION_PACKAGE_VERSION: u32 = 1;
const PERSONALIZATION_PACKAGE_FILE_STEM: &str = "sea-lantern-personalization";
const PERSONALIZATION_SETTINGS_ENTRY: &str = "personalization/settings.json";
const PERSONALIZATION_BACKGROUND_ENTRY: &str = "personalization/background";
#[cfg(target_os = "windows")]
const WINDOWS_NATIVE_WINDOW_EFFECTS_DISABLED: bool = true;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersonalizationSettings {
    background_image: String,
    background_opacity: f32,
    background_blur: u32,
    background_brightness: f32,
    background_size: String,
    window_effect: String,
    theme: String,
    color: String,
    font_size: u32,
    font_family: String,
    memory_display_precision: u8,
    text_color_overrides: TextColorOverrides,
    app_display_name: String,
    minimal_mode: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PluginPersonalizationSettings {
    plugin_id: String,
    capabilities: Vec<String>,
    settings: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersonalizationPackageManifest {
    version: u32,
    exported_at: String,
    settings: PersonalizationSettings,
    background_image_entry: Option<String>,
    plugins: Vec<PluginPersonalizationSettings>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportPersonalizationResult {
    pub settings: AppSettings,
    pub changed_groups: Vec<String>,
    pub imported_plugins: Vec<String>,
    pub skipped_plugins: Vec<String>,
}

fn sanitize_personalization_stem(file_name: &str) -> String {
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        return PERSONALIZATION_PACKAGE_FILE_STEM.to_string();
    }

    let mut sanitized = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            sanitized.push(ch);
        } else if ch.is_whitespace() {
            sanitized.push('-');
        }
    }

    let sanitized = sanitized.trim_matches('-').trim_matches('_');
    if sanitized.is_empty() {
        PERSONALIZATION_PACKAGE_FILE_STEM.to_string()
    } else {
        sanitized.to_string()
    }
}

fn export_settings_from_app(settings: &AppSettings) -> PersonalizationSettings {
    PersonalizationSettings {
        background_image: settings.background_image.clone(),
        background_opacity: settings.background_opacity,
        background_blur: settings.background_blur,
        background_brightness: settings.background_brightness,
        background_size: settings.background_size.clone(),
        window_effect: settings.window_effect.clone(),
        theme: settings.theme.clone(),
        color: settings.color.clone(),
        font_size: settings.font_size,
        font_family: settings.font_family.clone(),
        memory_display_precision: settings.memory_display_precision,
        text_color_overrides: settings.text_color_overrides.clone(),
        app_display_name: settings.app_display_name.clone(),
        minimal_mode: settings.minimal_mode,
    }
}

fn apply_personalization_settings(
    settings: &mut AppSettings,
    personalization: PersonalizationSettings,
) {
    settings.background_image = personalization.background_image;
    settings.background_opacity = personalization.background_opacity;
    settings.background_blur = personalization.background_blur;
    settings.background_brightness = personalization.background_brightness;
    settings.background_size = personalization.background_size;
    settings.window_effect = personalization.window_effect;
    settings.theme = personalization.theme;
    settings.color = personalization.color;
    settings.font_size = personalization.font_size;
    settings.font_family = personalization.font_family;
    settings.memory_display_precision = personalization.memory_display_precision;
    settings.text_color_overrides = personalization.text_color_overrides;
    settings.app_display_name = personalization.app_display_name;
    settings.minimal_mode = personalization.minimal_mode;
}

fn normalize_zip_entry_name(name: &str) -> String {
    name.replace('\\', "/")
}

fn resolve_background_entry_name(source_path: &Path) -> String {
    let extension = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.trim().to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
        .unwrap_or_else(|| "bin".to_string());
    format!("{}.{}", PERSONALIZATION_BACKGROUND_ENTRY, extension)
}

fn validate_zip_entry_path(name: &str) -> Result<(), String> {
    let normalized = normalize_zip_entry_name(name);
    let path = Path::new(&normalized);
    if path.is_absolute() {
        return Err(format!("ZIP entry '{}' is not allowed", name));
    }
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(format!("ZIP entry '{}' is not allowed", name));
        }
    }
    Ok(())
}

fn collect_personalization_plugins() -> Vec<PluginPersonalizationSettings> {
    let manager = global::plugin_manager();
    let manager = manager.lock().unwrap_or_else(|e| e.into_inner());

    manager
        .plugins()
        .values()
        .filter(|plugin| {
            !plugin.manifest.capabilities.is_empty()
                && plugin.manifest.capabilities.iter().any(|capability| {
                    capability == "theme-provider"
                        || capability == "theme-widgets-provider"
                        || capability == "personalization-provider"
                })
        })
        .filter_map(|plugin| {
            manager
                .get_plugin_settings(&plugin.manifest.id)
                .ok()
                .map(|settings| PluginPersonalizationSettings {
                    plugin_id: plugin.manifest.id.clone(),
                    capabilities: plugin.manifest.capabilities.clone(),
                    settings,
                })
        })
        .collect()
}

fn import_personalization_plugins(
    plugins: Vec<PluginPersonalizationSettings>,
) -> (Vec<String>, Vec<String>) {
    let manager = global::plugin_manager();
    let manager = manager.lock().unwrap_or_else(|e| e.into_inner());

    let mut imported = Vec::new();
    let mut skipped = Vec::new();

    for plugin in plugins {
        let Some(existing) = manager.plugins().get(&plugin.plugin_id) else {
            skipped.push(plugin.plugin_id);
            continue;
        };

        if !existing.manifest.capabilities.iter().any(|capability| {
            capability == "theme-provider"
                || capability == "theme-widgets-provider"
                || capability == "personalization-provider"
        }) {
            skipped.push(plugin.plugin_id);
            continue;
        }

        match manager.set_plugin_settings(&plugin.plugin_id, plugin.settings) {
            Ok(()) => imported.push(plugin.plugin_id),
            Err(_) => skipped.push(plugin.plugin_id),
        }
    }

    (imported, skipped)
}

fn persist_personalization_background(source_path: &Path) -> Result<String, String> {
    if !source_path.exists() {
        return Err(format!("Background image not found: {}", source_path.display()));
    }

    let app_data_dir = PathBuf::from(
        crate::utils::path::get_or_create_app_data_dir_checked()
            .map_err(|e| format!("Failed to resolve app data directory: {}", e))?,
    );
    let target_dir = app_data_dir.join("personalization").join("backgrounds");
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create personalization background directory: {}", e))?;

    let extension = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
        .unwrap_or_else(|| "bin".to_string());
    let file_name =
        format!("background-{}.{}", chrono::Local::now().format("%Y%m%d-%H%M%S"), extension);
    let target_path = target_dir.join(file_name);

    std::fs::copy(source_path, &target_path)
        .map_err(|e| format!("Failed to copy background image into app data: {}", e))?;

    Ok(target_path.to_string_lossy().to_string())
}

#[cfg(test)]
pub(crate) fn persist_personalization_background_for_test(
    source_path: &Path,
) -> Result<String, String> {
    persist_personalization_background(source_path)
}

#[cfg(target_os = "windows")]
fn tint_from_dark(dark: Option<bool>, alpha: u8) -> Option<window_vibrancy::Color> {
    dark.map(|is_dark| {
        if is_dark {
            (18, 18, 18, alpha)
        } else {
            (245, 245, 245, alpha)
        }
    })
}

#[cfg(target_os = "windows")]
fn clear_windows_effects(window: &WebviewWindow) {
    let _ = clear_mica(window);
    let _ = clear_windows_acrylic(window);
    let _ = clear_blur(window);
}

#[cfg(target_os = "windows")]
fn sync_windows_window_effect_fallback(
    window: &WebviewWindow,
    dark: Option<bool>,
) -> Result<(), String> {
    clear_windows_effects(window);

    let color = match dark {
        Some(true) => Color(15, 17, 23, 255),
        Some(false) => Color(248, 250, 252, 255),
        None => Color(248, 250, 252, 255),
    };

    window
        .set_background_color(Some(color))
        .map_err(|e| format!("Failed to set fallback Windows window background: {}", e))
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn sync_window_background(window: &WebviewWindow, effect: &str) -> Result<(), String> {
    let use_transparent_background = !effect.trim().eq_ignore_ascii_case(WINDOW_EFFECT_OFF);
    let color = if use_transparent_background {
        Some(Color(0, 0, 0, 0))
    } else {
        None
    };

    window
        .set_background_color(color)
        .map_err(|e| format!("Failed to set window background color: {}", e))
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn sync_native_window_effect(
    window: &WebviewWindow,
    effect: &str,
    dark: Option<bool>,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let normalized = effect.trim().to_ascii_lowercase();
        sync_window_background(window, &normalized)?;
        if normalized == WINDOW_EFFECT_OFF {
            clear_vibrancy(window)
                .map_err(|e| format!("Failed to clear native macOS vibrancy effect: {}", e))?;
            return Ok(());
        }

        if normalized == WINDOW_EFFECT_AUTO || normalized == "vibrancy" {
            apply_vibrancy(
                window,
                NSVisualEffectMaterial::UnderWindowBackground,
                Some(NSVisualEffectState::Active),
                None,
            )
            .map_err(|e| format!("Failed to apply native macOS vibrancy effect: {}", e))?;
            return Ok(());
        }

        clear_vibrancy(window)
            .map_err(|e| format!("Failed to clear native macOS vibrancy effect: {}", e))?;
        let _ = dark;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        if WINDOWS_NATIVE_WINDOW_EFFECTS_DISABLED {
            let _ = effect;
            return sync_windows_window_effect_fallback(window, dark);
        }

        let normalized = effect.trim().to_ascii_lowercase();
        sync_window_background(window, &normalized)?;
        clear_windows_effects(window);

        let mica_dark = dark;
        let blur_tint = tint_from_dark(dark, 120);
        let acrylic_tint = tint_from_dark(dark, 145);

        match normalized.as_str() {
            WINDOW_EFFECT_OFF => Ok(()),
            "mica" => apply_mica(window, mica_dark)
                .map_err(|e| format!("Failed to apply Windows Mica effect: {}", e)),
            "blur" => apply_blur(window, blur_tint)
                .or_else(|_| apply_mica(window, mica_dark))
                .or_else(|_| apply_windows_acrylic(window, acrylic_tint))
                .map_err(|e| format!("Failed to apply Windows Blur window effect: {}", e)),
            "acrylic" => apply_windows_acrylic(window, acrylic_tint)
                .map_err(|e| format!("Failed to apply Windows Acrylic effect: {}", e)),
            WINDOW_EFFECT_AUTO => apply_mica(window, mica_dark)
                .or_else(|_| apply_windows_acrylic(window, acrylic_tint))
                .map_err(|e| format!("Failed to apply automatic Windows window effect: {}", e)),
            _ => Ok(()),
        }
    }
}

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
pub fn export_personalization_package(path: String) -> Result<(), String> {
    let target_path = PathBuf::from(path.trim());
    if target_path.as_os_str().is_empty() {
        return Err("Export path cannot be empty".to_string());
    }

    if let Some(parent) = target_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create export directory: {}", e))?;
        }
    }

    let settings = global::settings_manager().get();
    let background_path = settings.background_image.trim();
    let mut background_entry_name = None;

    let file = File::create(&target_path)
        .map_err(|e| format!("Failed to create personalization package: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let file_options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    if !background_path.is_empty() {
        let source_path = PathBuf::from(background_path);
        if source_path.exists() {
            let entry_name = resolve_background_entry_name(&source_path);
            background_entry_name = Some(entry_name.clone());
            zip.start_file(&entry_name, file_options)
                .map_err(|e| format!("Failed to start background entry: {}", e))?;

            let mut source = File::open(&source_path)
                .map_err(|e| format!("Failed to open background image: {}", e))?;
            std::io::copy(&mut source, &mut zip)
                .map_err(|e| format!("Failed to write background image into package: {}", e))?;
        }
    }

    let manifest = PersonalizationPackageManifest {
        version: PERSONALIZATION_PACKAGE_VERSION,
        exported_at: chrono::Local::now().to_rfc3339(),
        settings: export_settings_from_app(&settings),
        background_image_entry: background_entry_name,
        plugins: collect_personalization_plugins(),
    };

    zip.start_file(PERSONALIZATION_SETTINGS_ENTRY, file_options)
        .map_err(|e| format!("Failed to start settings entry: {}", e))?;
    let manifest_json = serde_json::to_vec_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize personalization package: {}", e))?;
    zip.write_all(&manifest_json)
        .map_err(|e| format!("Failed to write personalization settings entry: {}", e))?;
    zip.finish()
        .map_err(|e| format!("Failed to finalize personalization package: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn import_personalization_package(path: String) -> Result<ImportPersonalizationResult, String> {
    let source_path = PathBuf::from(path.trim());
    if !source_path.exists() {
        return Err(format!("Personalization package not found: {}", source_path.display()));
    }

    let file = File::open(&source_path)
        .map_err(|e| format!("Failed to open personalization package: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read personalization package: {}", e))?;

    let manifest = {
        let mut settings_entry = archive
            .by_name(PERSONALIZATION_SETTINGS_ENTRY)
            .map_err(|e| format!("Failed to open personalization settings entry: {}", e))?;
        let mut json = String::new();
        settings_entry
            .read_to_string(&mut json)
            .map_err(|e| format!("Failed to read personalization settings entry: {}", e))?;
        serde_json::from_str::<PersonalizationPackageManifest>(&json)
            .map_err(|e| format!("Failed to parse personalization package: {}", e))?
    };

    let previous_settings = global::settings_manager().get();
    let mut next_settings = previous_settings.clone();
    apply_personalization_settings(&mut next_settings, manifest.settings);

    if let Some(background_entry) = manifest.background_image_entry.as_deref() {
        validate_zip_entry_path(background_entry)?;
        let mut background_file = archive
            .by_name(background_entry)
            .map_err(|e| format!("Failed to open packaged background image: {}", e))?;
        let temp_dir = std::env::temp_dir().join("sea-lantern-personalization-import");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temporary import directory: {}", e))?;
        let extension = Path::new(background_entry)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");
        let temp_file_path = temp_dir.join(format!(
            "background-{}.{}",
            chrono::Local::now().format("%Y%m%d-%H%M%S"),
            extension
        ));
        let mut temp_file = File::create(&temp_file_path)
            .map_err(|e| format!("Failed to create temporary background image: {}", e))?;
        std::io::copy(&mut background_file, &mut temp_file)
            .map_err(|e| format!("Failed to extract background image: {}", e))?;
        next_settings.background_image = persist_personalization_background(&temp_file_path)?;
        let _ = std::fs::remove_file(&temp_file_path);
    } else {
        next_settings.background_image.clear();
    }

    let result = global::settings_manager().update_with_diff(next_settings)?;
    let (imported_plugins, skipped_plugins) = import_personalization_plugins(manifest.plugins);

    Ok(ImportPersonalizationResult {
        settings: result.settings,
        changed_groups: result
            .changed_groups
            .into_iter()
            .map(|group| format!("{:?}", group))
            .collect(),
        imported_plugins,
        skipped_plugins,
    })
}

#[tauri::command]
pub fn get_personalization_package_suggested_name() -> String {
    let settings = global::settings_manager().get();
    let preferred = settings.app_display_name.trim();
    let stem = if preferred.is_empty() {
        PERSONALIZATION_PACKAGE_FILE_STEM.to_string()
    } else {
        sanitize_personalization_stem(preferred)
    };
    format!("{}.zip", stem)
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
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        if let Some(window) = app.get_webview_window("main") {
            sync_native_window_effect(
                &window,
                if enabled {
                    WINDOW_EFFECT_AUTO
                } else {
                    WINDOW_EFFECT_OFF
                },
                None,
            )?;
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = (enabled, app);
    }

    Ok(())
}

#[tauri::command]
pub fn apply_window_effect(
    effect: String,
    dark: Option<bool>,
    app: AppHandle,
) -> Result<(), String> {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        if let Some(window) = app.get_webview_window("main") {
            sync_native_window_effect(&window, &effect, dark)?;
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = (effect, dark, app);
    }

    Ok(())
}
