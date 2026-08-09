//! 应用设置模型与变更分组。

use serde::{Deserialize, Serialize};

use super::JavaInfo;

/// 当前配置版本号。
///
/// 每次配置结构变更时递增，由配置管理器据此执行数据迁移。
pub const CURRENT_CONFIG_VERSION: u32 = 2;

/// 亚克力模糊级别的默认值，旧配置缺字段时回落到这里。
pub const DEFAULT_ACRYLIC_BLUR_LEVEL: &str = "medium";

/// 完整设置不符合业务约束。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsValidationError {
    field: &'static str,
    message: &'static str,
}

impl SettingsValidationError {
    fn new(field: &'static str, message: &'static str) -> Self {
        Self { field, message }
    }

    /// 返回违反约束的设置字段。
    pub fn field(self) -> &'static str {
        self.field
    }

    /// 返回不包含用户数据的稳定校验原因。
    pub fn message(self) -> &'static str {
        self.message
    }
}

impl std::fmt::Display for SettingsValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid setting '{}': {}", self.field, self.message)
    }
}

impl std::error::Error for SettingsValidationError {}

/// 设置变更分组，用于调用方按组刷新状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsGroup {
    General,
    ServerDefaults,
    Console,
    Appearance,
    Window,
    Developer,
    PluginCommands,
}

/// 完整的应用设置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub config_version: u32,

    pub close_servers_on_exit: bool,
    pub close_servers_on_update: bool,
    pub auto_accept_eula: bool,
    pub close_action: String,

    pub default_max_memory: u32,
    pub default_min_memory: u32,
    pub default_port: u16,
    pub default_java_path: String,
    pub default_jvm_args: String,
    pub cached_java_list: Vec<JavaInfo>,

    pub console_font_size: u32,
    pub console_font_family: String,
    pub console_letter_spacing: i32,
    pub max_log_lines: u32,

    pub background_image: String,
    pub background_opacity: f32,
    pub background_blur: u32,
    pub background_brightness: f32,
    pub background_size: String,
    pub acrylic_enabled: bool,
    pub acrylic_blur_level: String,
    pub theme: String,
    pub color: String,
    pub font_size: u32,
    pub font_family: String,
    pub minimal_mode: bool,

    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub window_maximized: Option<bool>,

    pub language: String,
    pub locales_base_url: Option<String>,
    pub developer_mode: bool,
    pub last_run_path: String,
    pub agreed_to_terms: bool,

    pub plugin_allowed_commands: Vec<String>,
    pub plugin_blocked_commands: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            config_version: CURRENT_CONFIG_VERSION,
            close_servers_on_exit: true,
            close_servers_on_update: true,
            auto_accept_eula: true,
            close_action: "ask".into(),
            default_max_memory: 2048,
            default_min_memory: 512,
            default_port: 25565,
            default_java_path: String::new(),
            default_jvm_args: String::new(),
            cached_java_list: Vec::new(),
            console_font_size: 13,
            console_font_family: String::new(),
            console_letter_spacing: 0,
            max_log_lines: 5000,
            background_image: String::new(),
            background_opacity: 0.3,
            background_blur: 0,
            background_brightness: 1.0,
            background_size: "cover".into(),
            acrylic_enabled: false,
            acrylic_blur_level: DEFAULT_ACRYLIC_BLUR_LEVEL.to_string(),
            theme: "auto".into(),
            color: "default".into(),
            font_size: 14,
            font_family: String::new(),
            minimal_mode: false,
            window_width: Some(1200),
            window_height: Some(720),
            window_x: None,
            window_y: None,
            window_maximized: Some(false),
            language: "zh-CN".into(),
            locales_base_url: None,
            developer_mode: false,
            last_run_path: String::new(),
            agreed_to_terms: false,
            plugin_allowed_commands: vec![],
            plugin_blocked_commands: vec![],
        }
    }
}

impl AppSettings {
    /// 校验写入配置文件前必须满足的业务不变量。
    pub fn validate(&self) -> Result<(), SettingsValidationError> {
        if self.default_min_memory == 0 {
            return Err(SettingsValidationError::new(
                "default_min_memory",
                "must be greater than zero",
            ));
        }
        if self.default_max_memory == 0 {
            return Err(SettingsValidationError::new(
                "default_max_memory",
                "must be greater than zero",
            ));
        }
        if self.default_min_memory > self.default_max_memory {
            return Err(SettingsValidationError::new(
                "default_min_memory",
                "must not exceed default_max_memory",
            ));
        }
        if self.default_port == 0 {
            return Err(SettingsValidationError::new("default_port", "must be greater than zero"));
        }
        validate_unit_interval("background_opacity", self.background_opacity)?;
        validate_unit_interval("background_brightness", self.background_brightness)?;
        if self.window_width == Some(0) {
            return Err(SettingsValidationError::new(
                "window_width",
                "must be greater than zero when set",
            ));
        }
        if self.window_height == Some(0) {
            return Err(SettingsValidationError::new(
                "window_height",
                "must be greater than zero when set",
            ));
        }
        Ok(())
    }

    /// 返回与 `other` 相比发生变更的分组列表。
    pub fn changed_groups(&self, other: &Self) -> Vec<SettingsGroup> {
        let mut groups = Vec::new();

        if self.close_servers_on_exit != other.close_servers_on_exit
            || self.close_servers_on_update != other.close_servers_on_update
            || self.auto_accept_eula != other.auto_accept_eula
            || self.close_action != other.close_action
        {
            groups.push(SettingsGroup::General);
        }

        if self.default_max_memory != other.default_max_memory
            || self.default_min_memory != other.default_min_memory
            || self.default_port != other.default_port
            || self.default_java_path != other.default_java_path
            || self.default_jvm_args != other.default_jvm_args
            || self.cached_java_list != other.cached_java_list
        {
            groups.push(SettingsGroup::ServerDefaults);
        }

        if self.console_font_size != other.console_font_size
            || self.console_font_family != other.console_font_family
            || self.console_letter_spacing != other.console_letter_spacing
            || self.max_log_lines != other.max_log_lines
        {
            groups.push(SettingsGroup::Console);
        }

        if self.background_image != other.background_image
            || self.background_opacity != other.background_opacity
            || self.background_blur != other.background_blur
            || self.background_brightness != other.background_brightness
            || self.background_size != other.background_size
            || self.acrylic_enabled != other.acrylic_enabled
            || self.acrylic_blur_level != other.acrylic_blur_level
            || self.theme != other.theme
            || self.color != other.color
            || self.font_size != other.font_size
            || self.font_family != other.font_family
            || self.minimal_mode != other.minimal_mode
        {
            groups.push(SettingsGroup::Appearance);
        }

        if self.window_width != other.window_width
            || self.window_height != other.window_height
            || self.window_x != other.window_x
            || self.window_y != other.window_y
            || self.window_maximized != other.window_maximized
        {
            groups.push(SettingsGroup::Window);
        }

        if self.language != other.language
            || self.locales_base_url != other.locales_base_url
            || self.developer_mode != other.developer_mode
            || self.last_run_path != other.last_run_path
            || self.agreed_to_terms != other.agreed_to_terms
        {
            groups.push(SettingsGroup::Developer);
        }

        if self.plugin_allowed_commands != other.plugin_allowed_commands
            || self.plugin_blocked_commands != other.plugin_blocked_commands
        {
            groups.push(SettingsGroup::PluginCommands);
        }

        groups
    }
}

fn validate_unit_interval(field: &'static str, value: f32) -> Result<(), SettingsValidationError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(SettingsValidationError::new(
            field,
            "must be a finite value between zero and one",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{AppSettings, SettingsGroup, DEFAULT_ACRYLIC_BLUR_LEVEL};

    #[test]
    fn legacy_settings_default_to_medium_acrylic_blur() {
        let settings: AppSettings =
            serde_json::from_str("{}").expect("legacy settings should load");

        assert_eq!(settings.acrylic_blur_level, DEFAULT_ACRYLIC_BLUR_LEVEL);
    }

    #[test]
    fn acrylic_blur_change_marks_appearance_group() {
        let current = AppSettings::default();
        let mut changed = current.clone();
        changed.acrylic_blur_level = "high".into();

        assert_eq!(current.changed_groups(&changed), vec![SettingsGroup::Appearance]);
    }

    #[test]
    fn default_settings_satisfy_validation_rules() {
        AppSettings::default()
            .validate()
            .expect("default settings should always remain valid");
    }

    #[test]
    fn validation_rejects_invalid_memory_and_port_values() {
        let settings = AppSettings {
            default_min_memory: 0,
            ..AppSettings::default()
        };
        assert_eq!(
            settings
                .validate()
                .expect_err("zero minimum memory should fail")
                .field(),
            "default_min_memory"
        );

        let settings = AppSettings {
            default_max_memory: 0,
            ..AppSettings::default()
        };
        assert_eq!(
            settings
                .validate()
                .expect_err("zero maximum memory should fail")
                .field(),
            "default_max_memory"
        );

        let settings = AppSettings {
            default_min_memory: AppSettings::default().default_max_memory + 1,
            ..AppSettings::default()
        };
        assert_eq!(
            settings
                .validate()
                .expect_err("minimum memory above maximum should fail")
                .field(),
            "default_min_memory"
        );

        let settings = AppSettings {
            default_port: 0,
            ..AppSettings::default()
        };
        assert_eq!(
            settings
                .validate()
                .expect_err("zero port should fail")
                .field(),
            "default_port"
        );
    }

    #[test]
    fn validation_rejects_invalid_visual_numeric_values() {
        for invalid in [-0.1, 1.1, f32::NAN, f32::INFINITY] {
            let settings = AppSettings {
                background_opacity: invalid,
                ..AppSettings::default()
            };
            assert_eq!(
                settings
                    .validate()
                    .expect_err("invalid opacity should fail")
                    .field(),
                "background_opacity"
            );
        }

        let settings = AppSettings {
            background_brightness: f32::NEG_INFINITY,
            ..AppSettings::default()
        };
        assert_eq!(
            settings
                .validate()
                .expect_err("invalid brightness should fail")
                .field(),
            "background_brightness"
        );

        let settings = AppSettings {
            window_width: Some(0),
            ..AppSettings::default()
        };
        assert_eq!(
            settings
                .validate()
                .expect_err("zero window width should fail")
                .field(),
            "window_width"
        );

        let settings = AppSettings {
            window_height: Some(0),
            ..AppSettings::default()
        };
        assert_eq!(
            settings
                .validate()
                .expect_err("zero window height should fail")
                .field(),
            "window_height"
        );
    }
}
