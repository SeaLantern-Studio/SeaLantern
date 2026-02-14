use crate::services::java_detector::JavaInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomColorScheme {
    #[serde(default)]
    pub primary: String,
    #[serde(default)]
    pub primary_light: String,
    #[serde(default)]
    pub primary_dark: String,
    #[serde(default)]
    pub primary_bg: String,
    #[serde(default)]
    pub accent: String,
    #[serde(default)]
    pub accent_light: String,
    #[serde(default)]
    pub success: String,
    #[serde(default)]
    pub warning: String,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub info: String,
}

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

    #[serde(default = "default_window_width")]
    pub window_width: u32,
    #[serde(default = "default_window_height")]
    pub window_height: u32,
    #[serde(default)]
    pub window_x: Option<i32>,
    #[serde(default)]
    pub window_y: Option<i32>,
    #[serde(default)]
    pub window_maximized: bool,

    #[serde(default)]
    pub acrylic_enabled: bool,

    #[serde(default = "default_color_scheme")]
    pub color_scheme: String,

    #[serde(default = "default_color_mode")]
    pub color_mode: String,

    #[serde(default)]
    pub custom_color_scheme: Option<CustomColorScheme>,

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

fn default_window_width() -> u32 {
    1200
}

fn default_window_height() -> u32 {
    720
}

fn default_color_scheme() -> String {
    "blue".to_string()
}

fn default_color_mode() -> String {
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
            window_width: 1200,
            window_height: 720,
            window_x: None,
            window_y: None,
            window_maximized: false,
            acrylic_enabled: false,
            color_scheme: "blue".to_string(),
            color_mode: "auto".to_string(),
            custom_color_scheme: None,
            font_size: 14,
            font_family: String::new(),
        }
    }
}
