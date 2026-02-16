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

    #[serde(default = "default_color")]
    pub color: String,

    // 颜色主题相关字段
    #[serde(default)]
    pub bg_color: String,
    #[serde(default)]
    pub bg_secondary_color: String,
    #[serde(default)]
    pub bg_tertiary_color: String,
    #[serde(default)]
    pub primary_color: String,
    #[serde(default)]
    pub secondary_color: String,
    #[serde(default)]
    pub text_primary_color: String,
    #[serde(default)]
    pub text_secondary_color: String,
    #[serde(default)]
    pub border_color: String,
    #[serde(default)]
    pub bg_dark: String,
    #[serde(default)]
    pub bg_secondary_dark: String,
    #[serde(default)]
    pub bg_tertiary_dark: String,
    #[serde(default)]
    pub primary_dark: String,
    #[serde(default)]
    pub secondary_dark: String,
    #[serde(default)]
    pub text_primary_dark: String,
    #[serde(default)]
    pub text_secondary_dark: String,
    #[serde(default)]
    pub border_dark: String,
    #[serde(default)]
    pub bg_acrylic: String,
    #[serde(default)]
    pub bg_secondary_acrylic: String,
    #[serde(default)]
    pub bg_tertiary_acrylic: String,
    #[serde(default)]
    pub primary_acrylic: String,
    #[serde(default)]
    pub secondary_acrylic: String,
    #[serde(default)]
    pub text_primary_acrylic: String,
    #[serde(default)]
    pub text_secondary_acrylic: String,
    #[serde(default)]
    pub border_acrylic: String,
    #[serde(default)]
    pub bg_dark_acrylic: String,
    #[serde(default)]
    pub bg_secondary_dark_acrylic: String,
    #[serde(default)]
    pub bg_tertiary_dark_acrylic: String,
    #[serde(default)]
    pub primary_dark_acrylic: String,
    #[serde(default)]
    pub secondary_dark_acrylic: String,
    #[serde(default)]
    pub text_primary_dark_acrylic: String,
    #[serde(default)]
    pub text_secondary_dark_acrylic: String,
    #[serde(default)]
    pub border_dark_acrylic: String,

    // 窗口状态
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

    // 语言设置
    #[serde(default = "default_language")]
    pub language: String,

    // 关闭行为: "ask", "minimize", "close"，默认 "ask"
    #[serde(default = "default_close_action")]
    pub close_action: String,
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
fn default_color() -> String {
    "default".to_string()
}

fn default_window_width() -> u32 {
    1200
}

fn default_window_height() -> u32 {
    720
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

fn default_language() -> String {
    "zh-CN".to_string()
}

fn default_close_action() -> String {
    "ask".to_string()
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
            // 浅色主题默认值
            bg_color: "#f8fafc".to_string(),
            bg_secondary_color: "#f1f5f9".to_string(),
            bg_tertiary_color: "#e2e8f0".to_string(),
            primary_color: "#0ea5e9".to_string(),
            secondary_color: "#06b6d4".to_string(),
            text_primary_color: "#0f172a".to_string(),
            text_secondary_color: "#475569".to_string(),
            border_color: "#e2e8f0".to_string(),
            // 深色主题默认值
            bg_dark: "#0f1117".to_string(),
            bg_secondary_dark: "#1a1d28".to_string(),
            bg_tertiary_dark: "#242836".to_string(),
            primary_dark: "#60a5fa".to_string(),
            secondary_dark: "#22d3ee".to_string(),
            text_primary_dark: "#e2e8f0".to_string(),
            text_secondary_dark: "#94a3b8".to_string(),
            border_dark: "rgba(255, 255, 255, 0.1)".to_string(),
            // 浅色亚克力主题默认值
            bg_acrylic: "rgba(248, 250, 252, 0.7)".to_string(),
            bg_secondary_acrylic: "rgba(241, 245, 249, 0.6)".to_string(),
            bg_tertiary_acrylic: "rgba(226, 232, 240, 0.5)".to_string(),
            primary_acrylic: "#0ea5e9".to_string(),
            secondary_acrylic: "#06b6d4".to_string(),
            text_primary_acrylic: "#0f172a".to_string(),
            text_secondary_acrylic: "#475569".to_string(),
            border_acrylic: "#e2e8f0".to_string(),
            // 深色亚克力主题默认值
            bg_dark_acrylic: "rgba(15, 17, 23, 0.7)".to_string(),
            bg_secondary_dark_acrylic: "rgba(26, 29, 40, 0.6)".to_string(),
            bg_tertiary_dark_acrylic: "rgba(36, 40, 54, 0.5)".to_string(),
            primary_dark_acrylic: "#60a5fa".to_string(),
            secondary_dark_acrylic: "#22d3ee".to_string(),
            text_primary_dark_acrylic: "#e2e8f0".to_string(),
            text_secondary_dark_acrylic: "#94a3b8".to_string(),
            border_dark_acrylic: "rgba(255, 255, 255, 0.1)".to_string(),
            window_width: 1200,
            window_height: 720,
            window_x: None,
            window_y: None,
            window_maximized: false,
            acrylic_enabled: false,
            theme: "auto".to_string(),
            font_size: 14,
            font_family: String::new(),
            color: "default".to_string(),
            language: "zh-CN".to_string(),
            close_action: "ask".to_string(),
        }
    }
}
