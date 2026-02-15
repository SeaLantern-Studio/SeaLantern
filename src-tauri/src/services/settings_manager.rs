use crate::models::settings::AppSettings;
use std::sync::Mutex;

const SETTINGS_FILE: &str = "sea_lantern_settings.json";

pub struct SettingsManager {
    pub settings: Mutex<AppSettings>,
    pub data_dir: String,
}

impl SettingsManager {
    pub fn new() -> Self {
        let data_dir = get_data_dir();
        let settings = load_settings(&data_dir);
        SettingsManager { settings: Mutex::new(settings), data_dir }
    }

    pub fn get(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn update(&self, new_settings: AppSettings) -> Result<(), String> {
        *self.settings.lock().unwrap() = new_settings.clone();
        save_settings(&self.data_dir, &new_settings)
    }

    pub fn reset(&self) -> Result<AppSettings, String> {
        let default = AppSettings::default();
        *self.settings.lock().unwrap() = default.clone();
        save_settings(&self.data_dir, &default)?;
        Ok(default)
    }
}

fn get_data_dir() -> String {
    // Use a consistent data directory regardless of dev/prod mode
    // Try to use the user's home directory first
    if let Some(home_dir) = dirs_next::home_dir() {
        let data_dir = home_dir.join(".sea-lantern");
        // Create directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            eprintln!("Warning: Failed to create data directory: {}", e);
        }
        return data_dir.to_string_lossy().to_string();
    }

    // Fallback to current directory
    ".".to_string()
}

fn load_settings(data_dir: &str) -> AppSettings {
    let path = std::path::Path::new(data_dir).join(SETTINGS_FILE);
    if !path.exists() {
        let default = AppSettings::default();
        let _ = save_settings(data_dir, &default);
        return default;
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

fn save_settings(data_dir: &str, settings: &AppSettings) -> Result<(), String> {
    let path = std::path::Path::new(data_dir).join(SETTINGS_FILE);
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to save settings: {}", e))
}
