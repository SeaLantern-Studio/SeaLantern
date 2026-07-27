//! 设置相关命令

use crate::models::{AppSettings, JavaInfo};
use sealantern_extra::config::get_app_data_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

/// 应用状态：设置管理
pub struct SettingsState {
    pub settings: Mutex<AppSettings>,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            settings: Mutex::new(AppSettings::default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettingsResult {
    pub settings: AppSettings,
    pub changed_groups: Vec<String>,
}

/// 获取应用设置
#[tauri::command]
pub async fn get_settings(state: State<'_, SettingsState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

/// 保存应用设置
#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    state: State<'_, SettingsState>,
) -> Result<(), String> {
    // 保存到文件
    save_settings_to_file(&settings)?;

    // 更新内存状态
    let mut current = state.settings.lock().map_err(|e| e.to_string())?;
    *current = settings;

    Ok(())
}

/// 保存设置并返回变更分组
#[tauri::command]
pub async fn save_settings_with_diff(
    settings: AppSettings,
    state: State<'_, SettingsState>,
) -> Result<UpdateSettingsResult, String> {
    let old_settings = state.settings.lock().map_err(|e| e.to_string())?.clone();

    // 计算变更的分组
    let changed_groups = compute_changed_groups(&old_settings, &settings);

    // 保存到文件
    save_settings_to_file(&settings)?;

    // 更新内存状态
    let mut current = state.settings.lock().map_err(|e| e.to_string())?;
    *current = settings.clone();

    Ok(UpdateSettingsResult {
        settings,
        changed_groups,
    })
}

/// 部分更新设置
#[tauri::command]
pub async fn update_settings_partial(
    partial: PartialSettings,
    state: State<'_, SettingsState>,
) -> Result<UpdateSettingsResult, String> {
    let old_settings = state.settings.lock().map_err(|e| e.to_string())?.clone();

    // 合并部分设置
    let mut new_settings = old_settings.clone();
    apply_partial(&mut new_settings, &partial);

    // 计算变更的分组
    let changed_groups = compute_changed_groups(&old_settings, &new_settings);

    // 保存到文件
    save_settings_to_file(&new_settings)?;

    // 更新内存状态
    let mut current = state.settings.lock().map_err(|e| e.to_string())?;
    *current = new_settings.clone();

    Ok(UpdateSettingsResult {
        settings: new_settings,
        changed_groups,
    })
}

/// 重置设置为默认值
#[tauri::command]
pub async fn reset_settings(state: State<'_, SettingsState>) -> Result<AppSettings, String> {
    let default = AppSettings::default();

    // 保存到文件
    save_settings_to_file(&default)?;

    // 更新内存状态
    let mut current = state.settings.lock().map_err(|e| e.to_string())?;
    *current = default.clone();

    Ok(default)
}

/// 导出设置为 JSON
#[tauri::command]
pub async fn export_settings(state: State<'_, SettingsState>) -> Result<String, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&*settings).map_err(|e| e.to_string())
}

/// 从 JSON 导入设置
#[tauri::command]
pub async fn import_settings(
    json: String,
    state: State<'_, SettingsState>,
) -> Result<AppSettings, String> {
    let settings: AppSettings =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse settings: {}", e))?;

    // 保存到文件
    save_settings_to_file(&settings)?;

    // 更新内存状态
    let mut current = state.settings.lock().map_err(|e| e.to_string())?;
    *current = settings.clone();

    Ok(settings)
}

// === 辅助函数 ===

/// 部分设置更新结构（与前端 PartialSettings 对应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialSettings {
    pub close_servers_on_exit: Option<bool>,
    pub close_servers_on_update: Option<bool>,
    pub auto_accept_eula: Option<bool>,
    pub default_max_memory: Option<u32>,
    pub default_min_memory: Option<u32>,
    pub default_port: Option<u16>,
    pub default_java_path: Option<String>,
    pub default_jvm_args: Option<String>,
    pub console_font_size: Option<u32>,
    pub console_font_family: Option<String>,
    pub console_letter_spacing: Option<i32>,
    pub max_log_lines: Option<u32>,
    pub cached_java_list: Option<Vec<JavaInfo>>,
    pub background_image: Option<String>,
    pub background_opacity: Option<f32>,
    pub background_blur: Option<u32>,
    pub background_brightness: Option<f32>,
    pub background_size: Option<String>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub window_maximized: Option<bool>,
    pub acrylic_enabled: Option<bool>,
    pub theme: Option<String>,
    pub font_size: Option<u32>,
    pub font_family: Option<String>,
    pub color: Option<String>,
    pub language: Option<String>,
    pub developer_mode: Option<bool>,
    pub close_action: Option<String>,
    pub last_run_path: Option<String>,
    pub minimal_mode: Option<bool>,
    pub agreed_to_terms: Option<bool>,
}

/// 应用部分设置更新
fn apply_partial(settings: &mut AppSettings, partial: &PartialSettings) {
    if let Some(v) = partial.close_servers_on_exit {
        settings.close_servers_on_exit = v;
    }
    if let Some(v) = partial.close_servers_on_update {
        settings.close_servers_on_update = v;
    }
    if let Some(v) = partial.auto_accept_eula {
        settings.auto_accept_eula = v;
    }
    if let Some(v) = partial.default_max_memory {
        settings.default_max_memory = v;
    }
    if let Some(v) = partial.default_min_memory {
        settings.default_min_memory = v;
    }
    if let Some(v) = partial.default_port {
        settings.default_port = v;
    }
    if let Some(v) = &partial.default_java_path {
        settings.default_java_path = v.clone();
    }
    if let Some(v) = &partial.default_jvm_args {
        settings.default_jvm_args = v.clone();
    }
    if let Some(v) = partial.console_font_size {
        settings.console_font_size = v;
    }
    if let Some(v) = &partial.console_font_family {
        settings.console_font_family = v.clone();
    }
    if let Some(v) = partial.console_letter_spacing {
        settings.console_letter_spacing = v;
    }
    if let Some(v) = partial.max_log_lines {
        settings.max_log_lines = v;
    }
    if let Some(v) = &partial.cached_java_list {
        settings.cached_java_list = v.clone();
    }
    if let Some(v) = &partial.background_image {
        settings.background_image = v.clone();
    }
    if let Some(v) = partial.background_opacity {
        settings.background_opacity = v;
    }
    if let Some(v) = partial.background_blur {
        settings.background_blur = v;
    }
    if let Some(v) = partial.background_brightness {
        settings.background_brightness = v;
    }
    if let Some(v) = &partial.background_size {
        settings.background_size = v.clone();
    }
    if let Some(v) = partial.window_width {
        settings.window_width = Some(v);
    }
    if let Some(v) = partial.window_height {
        settings.window_height = Some(v);
    }
    if let Some(v) = partial.window_x {
        settings.window_x = Some(v);
    }
    if let Some(v) = partial.window_y {
        settings.window_y = Some(v);
    }
    if let Some(v) = partial.window_maximized {
        settings.window_maximized = Some(v);
    }
    if let Some(v) = partial.acrylic_enabled {
        settings.acrylic_enabled = v;
    }
    if let Some(v) = &partial.theme {
        settings.theme = v.clone();
    }
    if let Some(v) = partial.font_size {
        settings.font_size = v;
    }
    if let Some(v) = &partial.font_family {
        settings.font_family = v.clone();
    }
    if let Some(v) = &partial.color {
        settings.color = v.clone();
    }
    if let Some(v) = &partial.language {
        settings.language = v.clone();
    }
    if let Some(v) = partial.developer_mode {
        settings.developer_mode = v;
    }
    if let Some(v) = &partial.close_action {
        settings.close_action = v.clone();
    }
    if let Some(v) = &partial.last_run_path {
        settings.last_run_path = v.clone();
    }
    if let Some(v) = partial.minimal_mode {
        settings.minimal_mode = v;
    }
    if let Some(v) = partial.agreed_to_terms {
        settings.agreed_to_terms = v;
    }
}

/// 计算变更的设置分组
fn compute_changed_groups(old: &AppSettings, new: &AppSettings) -> Vec<String> {
    let mut groups = Vec::new();

    // General
    if old.close_servers_on_exit != new.close_servers_on_exit
        || old.close_servers_on_update != new.close_servers_on_update
        || old.auto_accept_eula != new.auto_accept_eula
        || old.close_action != new.close_action
        || old.last_run_path != new.last_run_path
        || old.minimal_mode != new.minimal_mode
    {
        groups.push("General".to_string());
    }

    // ServerDefaults
    if old.default_max_memory != new.default_max_memory
        || old.default_min_memory != new.default_min_memory
        || old.default_port != new.default_port
        || old.default_java_path != new.default_java_path
        || old.default_jvm_args != new.default_jvm_args
    {
        groups.push("ServerDefaults".to_string());
    }

    // Console
    if old.console_font_size != new.console_font_size
        || old.console_font_family != new.console_font_family
        || old.console_letter_spacing != new.console_letter_spacing
        || old.max_log_lines != new.max_log_lines
    {
        groups.push("Console".to_string());
    }

    // Appearance
    if old.theme != new.theme
        || old.font_size != new.font_size
        || old.font_family != new.font_family
        || old.color != new.color
        || old.language != new.language
        || old.background_image != new.background_image
        || old.background_opacity != new.background_opacity
        || old.background_blur != new.background_blur
        || old.background_brightness != new.background_brightness
        || old.background_size != new.background_size
        || old.acrylic_enabled != new.acrylic_enabled
    {
        groups.push("Appearance".to_string());
    }

    // Window
    if old.window_width != new.window_width
        || old.window_height != new.window_height
        || old.window_x != new.window_x
        || old.window_y != new.window_y
        || old.window_maximized != new.window_maximized
    {
        groups.push("Window".to_string());
    }

    // Developer
    if old.developer_mode != new.developer_mode {
        groups.push("Developer".to_string());
    }

    groups
}

/// 获取设置文件路径
fn settings_file_path() -> PathBuf {
    get_app_data_dir().join("config").join("settings.json")
}

/// 保存设置到文件
fn save_settings_to_file(settings: &AppSettings) -> Result<(), String> {
    let path = settings_file_path();

    // 确保目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }

    // 写入文件
    let content =
        serde_json::to_string_pretty(settings).map_err(|e| format!("Failed to serialize: {}", e))?;

    fs::write(&path, content).map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

/// 从文件加载设置
pub fn load_settings_from_file() -> AppSettings {
    let path = settings_file_path();

    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(settings) => return settings,
                    Err(e) => {
                        tracing::warn!("Failed to parse settings file, using defaults: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read settings file, using defaults: {}", e);
            }
        }
    }

    AppSettings::default()
}