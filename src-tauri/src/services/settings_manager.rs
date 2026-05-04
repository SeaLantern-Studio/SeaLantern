use crate::models::settings::{AppSettings, PartialSettings, SettingsGroup};
use std::sync::Mutex;

///此处常量见 utils/constants.rs
use crate::utils::constants::SETTINGS_FILE;

/// 设置管理器
///
/// 负责读取、保存、重置和增量更新应用设置
pub struct SettingsManager {
    pub settings: Mutex<AppSettings>,
    pub data_dir: String,
}

/// 一次设置更新的结果
pub struct UpdateResult {
    pub settings: AppSettings,
    pub changed_groups: Vec<SettingsGroup>,
}

impl SettingsManager {
    /// 创建设置管理器
    ///
    /// # Returns
    ///
    /// 返回已经完成本地设置加载的管理器实例
    pub fn new() -> Self {
        eprintln!("[DEBUG] SettingsManager::new() called");
        let data_dir = get_data_dir();
        eprintln!("[DEBUG] SettingsManager: data_dir = {}", data_dir);
        let settings = load_settings(&data_dir);
        eprintln!(
            "[DEBUG] SettingsManager: loaded settings, agreed_to_terms = {}",
            settings.agreed_to_terms
        );
        SettingsManager { settings: Mutex::new(settings), data_dir }
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
        eprintln!(
            "[DEBUG] SettingsManager::update() called, agreed_to_terms = {}",
            new_settings.agreed_to_terms
        );
        let result = save_settings(&self.data_dir, &new_settings);
        eprintln!("[DEBUG] SettingsManager::update() save result: {:?}", result);
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
        let old_settings = self
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let changed_groups = old_settings.get_changed_groups(&new_settings);
        save_settings(&self.data_dir, &new_settings)?;
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
        eprintln!(
            "[DEBUG] SettingsManager::update_partial() called, partial.agreed_to_terms = {:?}",
            partial.agreed_to_terms
        );
        let old_settings = self
            .settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let mut new_settings = old_settings.clone();
        new_settings.merge_from(&partial);
        eprintln!(
            "[DEBUG] SettingsManager::update_partial() new_settings.agreed_to_terms = {}",
            new_settings.agreed_to_terms
        );
        let changed_groups = old_settings.get_changed_groups(&new_settings);
        save_settings(&self.data_dir, &new_settings)?;
        *self.settings.lock().unwrap_or_else(|e| e.into_inner()) = new_settings.clone();
        eprintln!("[DEBUG] SettingsManager::update_partial() saved successfully");
        Ok(UpdateResult { settings: new_settings, changed_groups })
    }

    /// 重置为默认设置
    ///
    /// # Returns
    ///
    /// 返回一份新的默认设置，并同步写回本地文件
    pub fn reset(&self) -> Result<AppSettings, String> {
        let default = AppSettings::default();
        save_settings(&self.data_dir, &default)?;
        *self.settings.lock().unwrap_or_else(|e| e.into_inner()) = default.clone();
        Ok(default)
    }
}

fn get_data_dir() -> String {
    // 使用统一的应用数据目录，确保 MSI 安装时数据存储在 %AppData%
    crate::utils::path::get_or_create_app_data_dir()
}

fn load_settings(data_dir: &str) -> AppSettings {
    let path = std::path::Path::new(data_dir).join(SETTINGS_FILE);
    eprintln!("[DEBUG] load_settings: path = {:?}", path);
    eprintln!("[DEBUG] load_settings: path.exists() = {}", path.exists());

    if !path.exists() {
        eprintln!("[DEBUG] load_settings: file not found, creating default settings");
        let default = AppSettings::default();
        let _ = save_settings(data_dir, &default);
        return default;
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            eprintln!("[DEBUG] load_settings: file content length = {} bytes", content.len());
            match serde_json::from_str::<AppSettings>(&content) {
                Ok(result) => {
                    eprintln!(
                        "[DEBUG] load_settings: parsed agreed_to_terms = {}",
                        result.agreed_to_terms
                    );
                    result
                }
                Err(error) => {
                    eprintln!("[DEBUG] load_settings: parse error: {}", error);
                    backup_corrupt_settings_file(&path);
                    AppSettings::default()
                }
            }
        }
        Err(e) => {
            eprintln!("[DEBUG] load_settings: failed to read file: {}", e);
            AppSettings::default()
        }
    }
}

fn save_settings(data_dir: &str, settings: &AppSettings) -> Result<(), String> {
    let path = std::path::Path::new(data_dir).join(SETTINGS_FILE);
    eprintln!("[DEBUG] save_settings: path = {:?}", path);
    eprintln!("[DEBUG] save_settings: agreed_to_terms = {}", settings.agreed_to_terms);

    let json = serde_json::to_string_pretty(settings).map_err(|e| {
        eprintln!("[DEBUG] save_settings: serialize error: {}", e);
        format!("Failed to serialize settings: {}", e)
    })?;

    eprintln!("[DEBUG] save_settings: json length = {} bytes", json.len());

    std::fs::write(&path, json).map_err(|e| {
        eprintln!("[DEBUG] save_settings: write error: {}", e);
        format!("Failed to save settings: {}", e)
    })?;

    eprintln!("[DEBUG] save_settings: success!");
    Ok(())
}

fn backup_corrupt_settings_file(path: &std::path::Path) {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let backup_path = path.with_extension(format!("json.bak-corrupt-{}", timestamp));

    match std::fs::copy(path, &backup_path) {
        Ok(_) => eprintln!("[DEBUG] load_settings: 已备份损坏设置到 {:?}", backup_path),
        Err(error) => eprintln!("[DEBUG] load_settings: 备份损坏设置失败: {}", error),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/services_settings_manager_tests.rs"]
mod tests;
