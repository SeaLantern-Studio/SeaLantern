#[path = "schema/defaults.rs"]
mod defaults;

use crate::models::server::{CpuPolicyConfig, JvmPresetConfig};
use crate::services::java_detector::JavaInfo;
use serde::{Deserialize, Deserializer, Serialize};

use defaults::{
    default_allowed_commands, default_bg_blur, default_bg_brightness, default_bg_opacity,
    default_bg_size, default_blocked_commands, default_close_action, default_color,
    default_console_font, default_console_font_family, default_console_letter_spacing,
    default_false, default_font_family, default_font_size, default_language, default_log_lines,
    default_max_memory, default_min_memory, default_port, default_theme, default_true,
    default_window_effect, default_window_height, default_window_width,
};

pub const WINDOW_EFFECT_OFF: &str = "off";
pub const WINDOW_EFFECT_AUTO: &str = "auto";
pub const WINDOW_EFFECT_BLUR: &str = "blur";
pub const WINDOW_EFFECT_ACRYLIC: &str = "acrylic";
pub const WINDOW_EFFECT_MICA: &str = "mica";
pub const WINDOW_EFFECT_VIBRANCY: &str = "vibrancy";

fn deserialize_jvm_args<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum JvmArgsRepr {
        String(String),
        Array(Vec<String>),
    }

    let value = Option::<JvmArgsRepr>::deserialize(deserializer)?;
    Ok(match value {
        Some(JvmArgsRepr::String(raw)) => {
            raw.split_whitespace().map(|arg| arg.to_string()).collect()
        }
        Some(JvmArgsRepr::Array(values)) => values,
        None => Vec::new(),
    })
}

fn deserialize_optional_jvm_args<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum JvmArgsRepr {
        String(String),
        Array(Vec<String>),
    }

    let value = Option::<JvmArgsRepr>::deserialize(deserializer)?;
    Ok(value.map(|repr| match repr {
        JvmArgsRepr::String(raw) => raw.split_whitespace().map(|arg| arg.to_string()).collect(),
        JvmArgsRepr::Array(values) => values,
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TextColorOverrides {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub description: String,
}

/// 设置变更分组
///
/// 用来标记一次设置更新影响了哪一块功能
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsGroup {
    General,
    ServerDefaults,
    Console,
    Appearance,
    Window,
    Developer,
    PluginCommands,
}

/// 完整应用设置
///
/// 这是保存到本地配置文件里的主设置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_true")]
    pub close_servers_on_exit: bool,

    #[serde(default = "default_true")]
    pub close_servers_on_update: bool,

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

    #[serde(default, deserialize_with = "deserialize_jvm_args")]
    pub default_jvm_args: Vec<String>,

    #[serde(default)]
    pub default_cpu_policy: CpuPolicyConfig,

    #[serde(default)]
    pub default_jvm_preset: JvmPresetConfig,

    #[serde(default = "default_console_font")]
    pub console_font_size: u32,

    #[serde(default = "default_console_font_family")]
    pub console_font_family: String,

    #[serde(default = "default_console_letter_spacing")]
    pub console_letter_spacing: i32,

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

    #[serde(default = "default_window_effect")]
    pub window_effect: String,

    #[serde(default, skip_serializing)]
    pub acrylic_enabled: bool,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_color")]
    pub color: String,

    #[serde(default = "default_font_size")]
    pub font_size: u32,

    #[serde(default = "default_font_family")]
    pub font_family: String,

    #[serde(default)]
    pub text_color_overrides: TextColorOverrides,

    #[serde(default)]
    pub app_display_name: String,

    #[serde(default = "default_language")]
    pub language: String,

    #[serde(default = "default_false")]
    pub developer_mode: bool,

    #[serde(default = "default_close_action")]
    pub close_action: String,

    #[serde(default)]
    pub last_run_path: String,

    #[serde(default)]
    pub minimal_mode: bool,

    #[serde(default = "default_allowed_commands")]
    pub plugin_allowed_commands: Vec<String>,

    #[serde(default = "default_blocked_commands")]
    pub plugin_blocked_commands: Vec<String>,

    #[serde(default = "default_false")]
    pub agreed_to_terms: bool,
}

/// 局部设置更新结构
///
/// 前端做增量保存时，只需要传入变动字段
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_servers_on_exit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_servers_on_update: Option<bool>,
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
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_jvm_args"
    )]
    pub default_jvm_args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_cpu_policy: Option<CpuPolicyConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_jvm_preset: Option<JvmPresetConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub console_font_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub console_font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub console_letter_spacing: Option<i32>,
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
    pub window_effect: Option<String>,
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
    pub text_color_overrides: Option<TextColorOverrides>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_display_name: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_allowed_commands: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_blocked_commands: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agreed_to_terms: Option<bool>,
}
