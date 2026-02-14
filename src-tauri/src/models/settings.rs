use crate::services::java_detector::JavaInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_true")]
    pub close_servers_on_exit: bool,

    #[serde(default = "default_true")]
    pub auto_accept_eula: bool,

    #[serde(default = "default_max_memory")]
    pub default_max_memory: u32,

    #[serde(default = "default_min_memory")]
    pub default_min_memory: u32,

    #[serde(default = "default_port")]
    pub default_port: u16,

    #[serde(default)]
    pub default_java_path: String,

    #[serde(default)]
    pub default_jvm_args: String,

    #[serde(default = "default_console_font")]
    pub console_font_size: u32,

    #[serde(default = "default_log_lines")]
    pub max_log_lines: u32,

    #[serde(default)]
    pub cached_java_list: Vec<JavaInfo>,

    // 外观设置
    #[serde(default)]
    pub background_image: String,

    #[serde(default = "default_bg_opacity")]
    pub background_opacity: f32,

    #[serde(default = "default_bg_blur")]
    pub background_blur: u32,

    #[serde(default = "default_bg_brightness")]
    pub background_brightness: f32,

    #[serde(default = "default_bg_size")]
    pub background_size: String,

    // 亚克力/毛玻璃效果 (Windows 专属，默认关闭)
    #[serde(default)]
    pub acrylic_enabled: bool,

    // 主题: "auto"、"light" 或 "dark"，默认 "auto" (跟随系统)
    #[serde(default = "default_theme")]
    pub theme: String,

    // 文本大小: 12-24，默认 14
    #[serde(default = "default_font_size")]
    pub font_size: u32,

    #[serde(default = "default_font_family")]
    pub font_family: String,
}

fn default_true() -> bool {
    true
}
fn default_max_memory() -> u32 {
    2048
}
fn default_min_memory() -> u32 {
    512
}
fn default_port() -> u16 {
    25565
}
fn default_console_font() -> u32 {
    13
}
fn default_log_lines() -> u32 {
    5000
}
fn default_bg_opacity() -> f32 {
    0.3
}
fn default_bg_blur() -> u32 {
    0
}
fn default_bg_brightness() -> f32 {
    1.0
}
fn default_bg_size() -> String {
    "cover".to_string()
}

fn default_theme() -> String {
    "auto".to_string()
}
fn default_font_size() -> u32 {
    14
}
fn default_font_family() -> String {
    String::new()
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            close_servers_on_exit: true,
            auto_accept_eula: true,
            default_max_memory: 2048,
            default_min_memory: 512,
            default_port: 25565,
            default_java_path: String::new(),
            default_jvm_args: String::new(),
            console_font_size: 13,
            max_log_lines: 5000,
            cached_java_list: Vec::new(),
            background_image: String::new(),
            background_opacity: 0.3,
            background_blur: 0,
            background_brightness: 1.0,
            background_size: "cover".to_string(),
            acrylic_enabled: false,
            theme: "auto".to_string(),
            font_size: 14,
            font_family: String::new(),
        }
    }
}
