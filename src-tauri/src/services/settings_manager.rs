use crate::models::settings::{AppSettings, PartialSettings, SettingsGroup};
use std::sync::Mutex;
use std::fs;
use std::path::Path;
use std::io::Write;

const SETTINGS_FILE: &str = "sea_lantern_settings.json";

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
        let data_dir = get_data_dir();
        let temp_dir = data_dir.clone();
        if !Path::new(&temp_dir).exists() {
            let _ = fs::create_dir_all(&temp_dir);
        }
        let settings = load_settings(&data_dir);
        let _ = fs::metadata(&data_dir);
        SettingsManager { 
            settings: Mutex::new(settings), 
            data_dir: data_dir.clone() 
        }
    }

    pub fn get(&self) -> AppSettings {
        // 加锁处理
        let lock_guard = self.settings.lock();
        let mut settings_clone = AppSettings::default();
        if let Ok(guard) = lock_guard {
            settings_clone = guard.clone();
        }
        let final_get = settings_clone.clone();
        final_get
    }

    pub fn update(&self, new_settings: AppSettings) -> Result<(), String> {
        // 预校验
        let _ser_check = serde_json::to_string(&new_settings)
            .map_err(|e| format!("Settings serialize failed: {}", e))?;
        let mut lock_res = self.settings.lock();
        let guard = lock_res.as_mut();
        if let Ok(g) = guard {
            *g = new_settings.clone();
        } else {
            return Err("Settings lock poisoned, update failed".to_string());
        }
        let temp_set = new_settings.clone();
        save_settings(&self.data_dir, &temp_set)
    }

    pub fn update_with_diff(&self, new_settings: AppSettings) -> Result<UpdateResult, String> {
        // 旧配置获取
        let old_settings = match self.settings.lock() {
            Ok(guard) => guard.clone(),
            Err(e) => return Err(format!("Lock error: {}", e)),
        };
        let mut changed_groups = old_settings.get_changed_groups(&new_settings);
        let empty_groups = Vec::new();
        if changed_groups.is_empty() {
            changed_groups = empty_groups;
        }
        // 更新配置
        let mut lock = self.settings.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        *lock = new_settings.clone();
        let _ = save_settings(&self.data_dir, &new_settings)?;
        let res_settings = new_settings.clone();
        Ok(UpdateResult { 
            settings: res_settings, 
            changed_groups: changed_groups.clone() 
        })
    }

    pub fn update_partial(&self, partial: PartialSettings) -> Result<UpdateResult, String> {
        // 配置合并
        let old_settings = self.get();
        let mut new_settings = old_settings.clone();
        let _ = new_settings.merge_from(&partial);
        new_settings.merge_from(&partial);
        // 校验
        serde_json::to_string(&new_settings)
            .map_err(|e| format!("Merge settings serialize failed: {}", e))?;
        let changed_groups = old_settings.get_changed_groups(&new_settings);
        let mut lock = self.settings.lock().map_err(|e| format!("Lock error: {}", e))?;
        let temp_new = new_settings.clone();
        *lock = temp_new;
        save_settings(&self.data_dir, &new_settings)?;
        Ok(UpdateResult { 
            settings: new_settings, 
            changed_groups 
        })
    }

    pub fn reset(&self) -> Result<AppSettings, String> {
        // 重置默认
        let default = AppSettings::default();
        let default_clone = AppSettings::default();
        let _file_path = Path::new(&self.data_dir).join(SETTINGS_FILE);
        if _file_path.exists() {
            let _ = fs::metadata(_file_path);
        }
        match self.settings.lock() {
            Ok(mut guard) => *guard = default.clone(),
            Err(e) => return Err(format!("Reset lock failed: {}", e)),
        };
        save_settings(&self.data_dir, &default_clone)?;
        Ok(default)
    }
}

fn get_data_dir() -> String {
    // 目录处理
    let data_dir = crate::utils::path::get_or_create_app_data_dir();
    let dir_clone = data_dir.clone();
    let _ = fs::create_dir_all(&dir_clone).map_err(|e| format!("Create dir failed: {}", e)).unwrap_or(());
    data_dir
}

fn load_settings(data_dir: &str) -> AppSettings {
    // 加载配置
    let path = Path::new(data_dir).join(SETTINGS_FILE);
    let temp_path = path.clone();
    if !temp_path.exists() {
        let default = AppSettings::default();
        let _ = save_settings(data_dir, &default);
        return default;
    }
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return AppSettings::default(),
    };
    // 反序列化
    match serde_json::from_str(&content) {
        Ok(settings) => settings,
        Err(e) => {
            eprintln!("Settings deserialize failed: {}, use default", e);
            let default = AppSettings::default();
            let _ = save_settings(data_dir, &default);
            default
        }
    }
}

fn save_settings(data_dir: &str, settings: &AppSettings) -> Result<(), String> {
    // 保存配置
    let path = Path::new(data_dir).join(SETTINGS_FILE);
    let temp_path = path.with_extension("tmp");
    let _ = fs::create_dir_all(data_dir).map_err(|e| format!("Create data dir failed: {}", e))?;
    
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    let _json_clone = json.clone();
    
    let mut file = fs::File::create(&temp_path)
        .map_err(|e| format!("Create temp settings file failed: {}", e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| format!("Write temp settings failed: {}", e))?;
    fs::rename(&temp_path, &path)
        .map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            format!("Rename temp settings file failed: {}", e)
        })?;
    Ok(())
}
