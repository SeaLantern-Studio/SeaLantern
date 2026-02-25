use crate::services::java_detector::JavaInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsGroup {
    General,
    ServerDefaults,
    Console,
    Appearance,
    Window,
    Developer,
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

    // 颜色主题: 预设主题 ID 或 "custom"，默认 "default"
    #[serde(default = "default_color")]
    pub color: String,

    // 文本大小: 12-24，默认 14
    #[serde(default = "default_font_size")]
    pub font_size: u32,

    #[serde(default = "default_font_family")]
    pub font_family: String,

    // 语言设置
    #[serde(default = "default_language")]
    pub language: String,

    // 开发者模式
    #[serde(default = "default_false")]
    pub developer_mode: bool,
    // 关闭行为: "ask", "minimize", "close"，默认 "ask"
    #[serde(default = "default_close_action")]
    pub close_action: String,

    // 上次选择的开服路径
    #[serde(default)]
    pub last_run_path: String,

    // 极简模式：关闭所有动效和特效
    #[serde(default)]
    pub minimal_mode: bool,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
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

fn default_theme() -> String {
    "auto".to_string()
}
fn default_color() -> String {
    "default".to_string()
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

impl AppSettings {
    pub fn get_changed_groups(&self, other: &AppSettings) -> Vec<SettingsGroup> {
        let mut changed = Vec::new();

        if self.close_servers_on_exit != other.close_servers_on_exit
            || self.auto_accept_eula != other.auto_accept_eula
            || self.close_action != other.close_action
        {
            changed.push(SettingsGroup::General);
        }

        if self.default_max_memory != other.default_max_memory
            || self.default_min_memory != other.default_min_memory
            || self.default_port != other.default_port
            || self.default_java_path != other.default_java_path
            || self.default_jvm_args != other.default_jvm_args
            || self.cached_java_list != other.cached_java_list
        {
            changed.push(SettingsGroup::ServerDefaults);
        }

        if self.console_font_size != other.console_font_size
            || self.max_log_lines != other.max_log_lines
        {
            changed.push(SettingsGroup::Console);
        }

        if self.background_image != other.background_image
            || self.background_opacity != other.background_opacity
            || self.background_blur != other.background_blur
            || self.background_brightness != other.background_brightness
            || self.background_size != other.background_size
            || self.acrylic_enabled != other.acrylic_enabled
            || self.theme != other.theme
            || self.color != other.color
            || self.font_size != other.font_size
            || self.font_family != other.font_family
            || self.minimal_mode != other.minimal_mode
        {
            changed.push(SettingsGroup::Appearance);
        }

        if self.window_width != other.window_width
            || self.window_height != other.window_height
            || self.window_x != other.window_x
            || self.window_y != other.window_y
            || self.window_maximized != other.window_maximized
        {
            changed.push(SettingsGroup::Window);
        }

        if self.developer_mode != other.developer_mode {
            changed.push(SettingsGroup::Developer);
        }

        changed
    }

    pub fn merge_from(&mut self, partial: &PartialSettings) {
        if let Some(v) = partial.close_servers_on_exit {
            self.close_servers_on_exit = v;
        }
        if let Some(v) = partial.auto_accept_eula {
            self.auto_accept_eula = v;
        }
        if let Some(v) = partial.default_max_memory {
            self.default_max_memory = v;
        }
        if let Some(v) = partial.default_min_memory {
            self.default_min_memory = v;
        }
        if let Some(v) = partial.default_port {
            self.default_port = v;
        }
        if let Some(ref v) = partial.default_java_path {
            self.default_java_path = v.clone();
        }
        if let Some(ref v) = partial.default_jvm_args {
            self.default_jvm_args = v.clone();
        }
        if let Some(v) = partial.console_font_size {
            self.console_font_size = v;
        }
        if let Some(v) = partial.max_log_lines {
            self.max_log_lines = v;
        }
        if let Some(ref v) = partial.cached_java_list {
            self.cached_java_list = v.clone();
        }
        if let Some(ref v) = partial.background_image {
            self.background_image = v.clone();
        }
        if let Some(v) = partial.background_opacity {
            self.background_opacity = v;
        }
        if let Some(v) = partial.background_blur {
            self.background_blur = v;
        }
        if let Some(v) = partial.background_brightness {
            self.background_brightness = v;
        }
        if let Some(ref v) = partial.background_size {
            self.background_size = v.clone();
        }
        if let Some(v) = partial.window_width {
            self.window_width = v;
        }
        if let Some(v) = partial.window_height {
            self.window_height = v;
        }
        if let Some(ref v) = partial.window_x {
            self.window_x = *v;
        }
        if let Some(ref v) = partial.window_y {
            self.window_y = *v;
        }
        if let Some(v) = partial.window_maximized {
            self.window_maximized = v;
        }
        if let Some(v) = partial.acrylic_enabled {
            self.acrylic_enabled = v;
        }
        if let Some(ref v) = partial.theme {
            self.theme = v.clone();
        }
        if let Some(ref v) = partial.color {
            self.color = v.clone();
        }
        if let Some(v) = partial.font_size {
            self.font_size = v;
        }
        if let Some(ref v) = partial.font_family {
            self.font_family = v.clone();
        }
        if let Some(ref v) = partial.language {
            self.language = v.clone();
        }
        if let Some(v) = partial.developer_mode {
            self.developer_mode = v;
        }
        if let Some(ref v) = partial.close_action {
            self.close_action = v.clone();
        }
        if let Some(ref v) = partial.last_run_path {
            self.last_run_path = v.clone();
        }
        if let Some(v) = partial.minimal_mode {
            self.minimal_mode = v;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_servers_on_exit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_accept_eula: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_max_memory: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_min_memory: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_java_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_jvm_args: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub console_font_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_log_lines: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_java_list: Option<Vec<JavaInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_opacity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_blur: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_brightness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_x: Option<Option<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_y: Option<Option<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_maximized: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acrylic_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub developer_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimal_mode: Option<bool>,
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
            theme: "auto".to_string(),
            color: "default".to_string(),
            font_size: 14,
            font_family: String::new(),
            language: "zh-CN".to_string(),
            developer_mode: false,
            close_action: "ask".to_string(),
            last_run_path: String::new(),
            minimal_mode: false,
        }
    }
}
