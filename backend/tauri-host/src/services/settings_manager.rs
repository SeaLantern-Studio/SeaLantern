use crate::models::settings::{AppSettings, PartialSettings, SettingsGroup};
use crate::utils::logger::{log_debug_ctx, log_warn_ctx};
use std::sync::Mutex;

///此处常量见 utils/constants.rs
use crate::utils::constants::SETTINGS_FILE;

/// 设置管理器
///
/// 负责读取、保存、重置和增量更新应用设置
pub struct SettingsManager {
    pub settings: Mutex<AppSettings>,
    pub data_dir: Mutex<String>,
}

/// 一次设置更新的结果
pub struct UpdateResult {
    pub settings: AppSettings,
    pub changed_groups: Vec<SettingsGroup>,
}

fn log_settings_debug(function: &str, message: &str) {
    log_debug_ctx("services.settings_manager", function, message);
}

fn log_settings_warn(function: &str, message: &str) {
    log_warn_ctx("services.settings_manager", function, message);
}

impl SettingsManager {
    pub fn new_checked() -> Result<Self, String> {
        log_settings_debug("new_checked", "initializing settings manager");
        let data_dir = get_data_dir_checked()?;
        log_settings_debug("new_checked", &format!("resolved data_dir={}", data_dir));
        let settings = load_settings_for_bootstrap(&data_dir)?;
        log_settings_debug(
            "new_checked",
            &format!("loaded settings agreed_to_terms={}", settings.agreed_to_terms),
        );
        Ok(SettingsManager {
            settings: Mutex::new(settings),
            data_dir: Mutex::new(data_dir),
        })
    }

    /// 读取当前完整设置
    ///
    /// # Returns
    ///
    /// 返回当前内存里的设置快照
    pub fn get(&self) -> AppSettings {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// 整体替换当前设置并写回本地
    ///
    /// # Parameters
    ///
    /// - `new_settings`: 新的完整设置
    ///
    /// # Returns
    ///
    /// 保存成功时返回 `Ok(())`
    pub fn update(&self, new_settings: AppSettings) -> Result<(), String> {
        let mut new_settings = new_settings;
        new_settings.normalize_window_effect();
        new_settings.normalize_memory_display_precision();
        log_settings_debug(
            "update",
            &format!("saving full settings agreed_to_terms={}", new_settings.agreed_to_terms),
        );
        let data_dir = self.data_dir_value()?;
        let result = save_settings(&data_dir, &new_settings);
        log_settings_debug("update", &format!("save result={:?}", result));
        result?;
        *self.settings.lock().unwrap_or_else(|e| e.into_inner()) = new_settings;
        Ok(())
    }

    /// 整体替换设置，并返回受影响的设置分组
    ///
    /// # Parameters
    ///
    /// - `new_settings`: 新的完整设置
    ///
    /// # Returns
    ///
    /// 返回新的设置内容和变化分组
    pub fn update_with_diff(&self, new_settings: AppSettings) -> Result<UpdateResult, String> {
        let mut new_settings = new_settings;
        new_settings.normalize_window_effect();
        new_settings.normalize_memory_display_precision();
        let old_settings = self
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let changed_groups = old_settings.get_changed_groups(&new_settings);
        let data_dir = self.data_dir_value()?;
        save_settings(&data_dir, &new_settings)?;
        *self.settings.lock().unwrap_or_else(|e| e.into_inner()) = new_settings.clone();
        Ok(UpdateResult { settings: new_settings, changed_groups })
    }

    /// 按局部字段更新设置，并返回受影响的设置分组
    ///
    /// # Parameters
    ///
    /// - `partial`: 只包含变动字段的局部设置
    ///
    /// # Returns
    ///
    /// 返回合并后的设置内容和变化分组
    pub fn update_partial(&self, partial: PartialSettings) -> Result<UpdateResult, String> {
        log_settings_debug(
            "update_partial",
            &format!("partial agreed_to_terms={:?}", partial.agreed_to_terms),
        );
        let old_settings = self
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let mut new_settings = old_settings.clone();
        new_settings.merge_from(&partial);
        log_settings_debug(
            "update_partial",
            &format!("merged agreed_to_terms={}", new_settings.agreed_to_terms),
        );
        let changed_groups = old_settings.get_changed_groups(&new_settings);
        let data_dir = self.data_dir_value()?;
        save_settings(&data_dir, &new_settings)?;
        *self.settings.lock().unwrap_or_else(|e| e.into_inner()) = new_settings.clone();
        log_settings_debug("update_partial", "saved partial settings successfully");
        Ok(UpdateResult { settings: new_settings, changed_groups })
    }

    /// 重置为默认设置
    ///
    /// # Returns
    ///
    /// 返回一份新的默认设置，并同步写回本地文件
    pub fn reset(&self) -> Result<AppSettings, String> {
        let mut default = AppSettings::default();
        default.normalize_window_effect();
        default.normalize_memory_display_precision();
        let data_dir = self.data_dir_value()?;
        save_settings(&data_dir, &default)?;
        *self.settings.lock().unwrap_or_else(|e| e.into_inner()) = default.clone();
        Ok(default)
    }

    pub fn reload_from_data_dir(&self, data_dir: &str) -> Result<AppSettings, String> {
        let settings = load_settings_for_bootstrap(data_dir)?;
        {
            let mut current = self.settings.lock().unwrap_or_else(|e| e.into_inner());
            *current = settings.clone();
        }
        {
            let mut dir = self.data_dir.lock().unwrap_or_else(|e| e.into_inner());
            *dir = data_dir.to_string();
        }
        Ok(settings)
    }

    pub fn data_dir_value(&self) -> Result<String, String> {
        self.data_dir
            .lock()
            .map(|dir| dir.clone())
            .map_err(|_| "settings data_dir lock poisoned".to_string())
    }
}

#[allow(dead_code)]
fn get_data_dir() -> String {
    // 使用统一的应用数据目录，确保 MSI 安装时数据存储在 %AppData%
    crate::utils::path::get_or_create_app_data_dir()
}

fn get_data_dir_checked() -> Result<String, String> {
    crate::utils::path::get_or_create_app_data_dir_checked()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))
}

#[allow(dead_code)]
fn load_settings(data_dir: &str) -> AppSettings {
    load_settings_checked(data_dir).unwrap_or_default()
}

fn load_settings_checked(data_dir: &str) -> Result<AppSettings, String> {
    let path = std::path::Path::new(data_dir).join(SETTINGS_FILE);
    log_settings_debug("load_settings_checked", &format!("path={}", path.display()));
    log_settings_debug("load_settings_checked", &format!("path_exists={}", path.exists()));

    if !path.exists() {
        log_settings_debug("load_settings_checked", "settings file missing; creating defaults");
        let default = AppSettings::default();
        save_settings(data_dir, &default)?;
        return Ok(default);
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            log_settings_debug("load_settings_checked", &format!("read bytes={}", content.len()));
            match serde_json::from_str::<AppSettings>(&content) {
                Ok(mut result) => {
                    result.normalize_window_effect();
                    result.normalize_memory_display_precision();
                    log_settings_debug(
                        "load_settings_checked",
                        &format!("parsed agreed_to_terms={}", result.agreed_to_terms),
                    );
                    Ok(result)
                }
                Err(error) => {
                    log_settings_warn(
                        "load_settings_checked",
                        &format!("parse error path={} error={}", path.display(), error),
                    );
                    backup_corrupt_settings_file(&path);
                    Err(format!("Failed to parse settings '{}': {}", path.display(), error))
                }
            }
        }
        Err(e) => {
            log_settings_warn(
                "load_settings_checked",
                &format!("read failed path={} error={}", path.display(), e),
            );
            Err(format!("Failed to read settings '{}': {}", path.display(), e))
        }
    }
}

fn load_settings_for_bootstrap(data_dir: &str) -> Result<AppSettings, String> {
    match load_settings_checked(data_dir) {
        Ok(settings) => Ok(settings),
        Err(error) if is_corrupt_settings_error(&error) => {
            log_settings_warn(
                "load_settings_for_bootstrap",
                &format!("recoverable corruption detected: {}", error),
            );
            let path = std::path::Path::new(data_dir).join(SETTINGS_FILE);
            let repaired = try_salvage_settings_from_file(&path).unwrap_or_else(|| {
                log_settings_warn(
                    "load_settings_for_bootstrap",
                    "unable to salvage structured fields; falling back to defaults",
                );
                AppSettings::default()
            });
            save_settings(data_dir, &repaired)?;
            Ok(repaired)
        }
        Err(error) => Err(error),
    }
}

fn is_corrupt_settings_error(error: &str) -> bool {
    error.contains("Failed to parse settings")
}

fn try_salvage_settings_from_file(path: &std::path::Path) -> Option<AppSettings> {
    let content = std::fs::read_to_string(path).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&content).ok()?;
    let object = value.as_object()?;

    let cleaned_map = object
        .iter()
        .filter_map(|(key, raw_value)| {
            let candidate = serde_json::json!({ key: raw_value.clone() });
            serde_json::from_value::<PartialSettings>(candidate)
                .ok()
                .filter(partial_settings_has_updates)
                .map(|_| (key.clone(), raw_value.clone()))
        })
        .collect::<serde_json::Map<String, serde_json::Value>>();

    if cleaned_map.is_empty() {
        return None;
    }

    let partial =
        serde_json::from_value::<PartialSettings>(serde_json::Value::Object(cleaned_map)).ok()?;
    let mut settings = AppSettings::default();
    settings.merge_from(&partial);
    Some(settings)
}

fn partial_settings_has_updates(partial: &PartialSettings) -> bool {
    serde_json::to_value(partial)
        .ok()
        .and_then(|value| value.as_object().map(|object| !object.is_empty()))
        .unwrap_or(false)
}

fn save_settings(data_dir: &str, settings: &AppSettings) -> Result<(), String> {
    let path = std::path::Path::new(data_dir).join(SETTINGS_FILE);
    log_settings_debug("save_settings", &format!("path={}", path.display()));
    log_settings_debug("save_settings", &format!("agreed_to_terms={}", settings.agreed_to_terms));

    let json = serde_json::to_string_pretty(settings).map_err(|e| {
        log_settings_warn("save_settings", &format!("serialize error={}", e));
        format!("Failed to serialize settings: {}", e)
    })?;

    log_settings_debug("save_settings", &format!("json_length={}", json.len()));

    std::fs::write(&path, json).map_err(|e| {
        log_settings_warn(
            "save_settings",
            &format!("write failed path={} error={}", path.display(), e),
        );
        format!("Failed to save settings: {}", e)
    })?;

    log_settings_debug("save_settings", "settings saved successfully");
    Ok(())
}

fn backup_corrupt_settings_file(path: &std::path::Path) {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let backup_path = path.with_extension(format!("json.bak-corrupt-{}", timestamp));

    match std::fs::copy(path, &backup_path) {
        Ok(_) => log_settings_warn(
            "backup_corrupt_settings_file",
            &format!("backed up corrupt settings to {}", backup_path.display()),
        ),
        Err(error) => log_settings_warn(
            "backup_corrupt_settings_file",
            &format!("failed to back up corrupt settings: {}", error),
        ),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/services_settings_manager_tests.rs"]
mod tests;
