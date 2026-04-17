use crate::models::settings::{AppSettings, PartialSettings, SettingsGroup};
use std::sync::Mutex;

///此处常量见 utils/constants.rs
use crate::utils::constants::SETTINGS_FILE;

pub struct SettingsManager {
    pub settings: Mutex<AppSettings>,
    pub data_dir: String,
}

pub struct UpdateResult {
    pub settings: AppSettings,
    pub changed_groups: Vec<SettingsGroup>,
}

impl SettingsManager {
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

    pub fn get(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn update(&self, new_settings: AppSettings) -> Result<(), String> {
        eprintln!(
            "[DEBUG] SettingsManager::update() called, agreed_to_terms = {}",
            new_settings.agreed_to_terms
        );
        *self.settings.lock().unwrap() = new_settings.clone();
        let result = save_settings(&self.data_dir, &new_settings);
        eprintln!("[DEBUG] SettingsManager::update() save result: {:?}", result);
        result
    }

    pub fn update_with_diff(&self, new_settings: AppSettings) -> Result<UpdateResult, String> {
        let old_settings = self.settings.lock().unwrap().clone();
        let changed_groups = old_settings.get_changed_groups(&new_settings);
        *self.settings.lock().unwrap() = new_settings.clone();
        save_settings(&self.data_dir, &new_settings)?;
        Ok(UpdateResult { settings: new_settings, changed_groups })
    }

    pub fn update_partial(&self, partial: PartialSettings) -> Result<UpdateResult, String> {
        eprintln!(
            "[DEBUG] SettingsManager::update_partial() called, partial.agreed_to_terms = {:?}",
            partial.agreed_to_terms
        );
        let old_settings = self.settings.lock().unwrap().clone();
        let mut new_settings = old_settings.clone();
        new_settings.merge_from(&partial);
        eprintln!(
            "[DEBUG] SettingsManager::update_partial() new_settings.agreed_to_terms = {}",
            new_settings.agreed_to_terms
        );
        let changed_groups = old_settings.get_changed_groups(&new_settings);
        *self.settings.lock().unwrap() = new_settings.clone();
        save_settings(&self.data_dir, &new_settings)?;
        eprintln!("[DEBUG] SettingsManager::update_partial() saved successfully");
        Ok(UpdateResult { settings: new_settings, changed_groups })
    }

    pub fn reset(&self) -> Result<AppSettings, String> {
        let default = AppSettings::default();
        *self.settings.lock().unwrap() = default.clone();
        save_settings(&self.data_dir, &default)?;
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
            let result: AppSettings = serde_json::from_str(&content).unwrap_or_default();
            eprintln!("[DEBUG] load_settings: parsed agreed_to_terms = {}", result.agreed_to_terms);
            result
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
