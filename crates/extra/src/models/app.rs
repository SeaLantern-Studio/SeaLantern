//! 应用设置模型

use super::JavaInfo;
use serde::{Deserialize, Serialize};

/// 应用设置（与前端 AppSettings 对应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub close_servers_on_exit: bool,
    #[serde(default)]
    pub close_servers_on_update: bool,
    pub auto_accept_eula: bool,
    pub default_max_memory: u32,
    pub default_min_memory: u32,
    pub default_port: u16,
    pub default_java_path: String,
    pub default_jvm_args: String,
    pub console_font_size: u32,
    pub console_font_family: String,
    pub console_letter_spacing: i32,
    pub max_log_lines: u32,
    pub cached_java_list: Vec<JavaInfo>,
    pub background_image: String,
    pub background_opacity: f32,
    pub background_blur: u32,
    pub background_brightness: f32,
    pub background_size: String,
    #[serde(default)]
    pub window_width: Option<u32>,
    #[serde(default)]
    pub window_height: Option<u32>,
    #[serde(default)]
    pub window_x: Option<i32>,
    #[serde(default)]
    pub window_y: Option<i32>,
    #[serde(default)]
    pub window_maximized: Option<bool>,
    pub acrylic_enabled: bool,
    pub theme: String,
    pub font_size: u32,
    pub font_family: String,
    pub color: String,
    pub language: String,
    #[serde(default)]
    pub locales_base_url: Option<String>,
    pub developer_mode: bool,
    pub close_action: String,
    pub last_run_path: String,
    pub minimal_mode: bool,
    pub agreed_to_terms: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            close_servers_on_exit: true,
            close_servers_on_update: true,
            auto_accept_eula: false,
            default_max_memory: 4096,
            default_min_memory: 1024,
            default_port: 25565,
            default_java_path: String::new(),
            default_jvm_args: String::new(),
            console_font_size: 12,
            console_font_family: String::new(),
            console_letter_spacing: 0,
            max_log_lines: 1000,
            cached_java_list: Vec::new(),
            background_image: String::new(),
            background_opacity: 0.3,
            background_blur: 0,
            background_brightness: 1.0,
            background_size: "cover".to_string(),
            window_width: None,
            window_height: None,
            window_x: None,
            window_y: None,
            window_maximized: None,
            acrylic_enabled: false,
            theme: "auto".to_string(),
            font_size: 14,
            font_family: String::new(),
            color: "default".to_string(),
            language: "zh-CN".to_string(),
            locales_base_url: None,
            developer_mode: false,
            close_action: "ask".to_string(),
            last_run_path: String::new(),
            minimal_mode: false,
            agreed_to_terms: false,
        }
    }
}
