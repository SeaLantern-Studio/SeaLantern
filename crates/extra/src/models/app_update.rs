//! 应用设置的部分更新模型。

use serde::{Deserialize, Serialize};

use super::{AppSettings, JavaInfo, SettingsGroup};

/// 部分更新请求，只合并值为 `Some` 的字段。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartialAppSettings {
    pub close_servers_on_exit: Option<bool>,
    pub close_servers_on_update: Option<bool>,
    pub auto_accept_eula: Option<bool>,
    pub close_action: Option<String>,

    pub default_max_memory: Option<u32>,
    pub default_min_memory: Option<u32>,
    pub default_port: Option<u16>,
    pub default_java_path: Option<String>,
    pub default_jvm_args: Option<String>,
    pub cached_java_list: Option<Vec<JavaInfo>>,

    pub console_font_size: Option<u32>,
    pub console_font_family: Option<String>,
    pub console_letter_spacing: Option<i32>,
    pub max_log_lines: Option<u32>,

    pub background_image: Option<String>,
    pub background_opacity: Option<f32>,
    pub background_blur: Option<u32>,
    pub background_brightness: Option<f32>,
    pub background_size: Option<String>,
    pub acrylic_enabled: Option<bool>,
    pub theme: Option<String>,
    pub color: Option<String>,
    pub font_size: Option<u32>,
    pub font_family: Option<String>,
    pub minimal_mode: Option<bool>,

    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub window_maximized: Option<bool>,

    pub language: Option<String>,
    pub locales_base_url: Option<String>,
    pub developer_mode: Option<bool>,
    pub last_run_path: Option<String>,
    pub agreed_to_terms: Option<bool>,

    pub plugin_allowed_commands: Option<Vec<String>>,
    pub plugin_blocked_commands: Option<Vec<String>>,
}

impl PartialAppSettings {
    /// 将部分更新合并到 `target`。
    pub fn merge_into(&self, target: &mut AppSettings) {
        if let Some(value) = self.close_servers_on_exit {
            target.close_servers_on_exit = value;
        }
        if let Some(value) = self.close_servers_on_update {
            target.close_servers_on_update = value;
        }
        if let Some(value) = self.auto_accept_eula {
            target.auto_accept_eula = value;
        }
        if let Some(value) = &self.close_action {
            target.close_action.clone_from(value);
        }
        if let Some(value) = self.default_max_memory {
            target.default_max_memory = value;
        }
        if let Some(value) = self.default_min_memory {
            target.default_min_memory = value;
        }
        if let Some(value) = self.default_port {
            target.default_port = value;
        }
        if let Some(value) = &self.default_java_path {
            target.default_java_path.clone_from(value);
        }
        if let Some(value) = &self.default_jvm_args {
            target.default_jvm_args.clone_from(value);
        }
        if let Some(value) = &self.cached_java_list {
            target.cached_java_list.clone_from(value);
        }
        if let Some(value) = self.console_font_size {
            target.console_font_size = value;
        }
        if let Some(value) = &self.console_font_family {
            target.console_font_family.clone_from(value);
        }
        if let Some(value) = self.console_letter_spacing {
            target.console_letter_spacing = value;
        }
        if let Some(value) = self.max_log_lines {
            target.max_log_lines = value;
        }
        if let Some(value) = &self.background_image {
            target.background_image.clone_from(value);
        }
        if let Some(value) = self.background_opacity {
            target.background_opacity = value;
        }
        if let Some(value) = self.background_blur {
            target.background_blur = value;
        }
        if let Some(value) = self.background_brightness {
            target.background_brightness = value;
        }
        if let Some(value) = &self.background_size {
            target.background_size.clone_from(value);
        }
        if let Some(value) = self.acrylic_enabled {
            target.acrylic_enabled = value;
        }
        if let Some(value) = &self.theme {
            target.theme.clone_from(value);
        }
        if let Some(value) = &self.color {
            target.color.clone_from(value);
        }
        if let Some(value) = self.font_size {
            target.font_size = value;
        }
        if let Some(value) = &self.font_family {
            target.font_family.clone_from(value);
        }
        if let Some(value) = self.minimal_mode {
            target.minimal_mode = value;
        }
        if let Some(value) = self.window_width {
            target.window_width = Some(value);
        }
        if let Some(value) = self.window_height {
            target.window_height = Some(value);
        }
        if let Some(value) = self.window_x {
            target.window_x = Some(value);
        }
        if let Some(value) = self.window_y {
            target.window_y = Some(value);
        }
        if let Some(value) = self.window_maximized {
            target.window_maximized = Some(value);
        }
        if let Some(value) = &self.language {
            target.language.clone_from(value);
        }
        if let Some(value) = &self.locales_base_url {
            target.locales_base_url = Some(value.clone());
        }
        if let Some(value) = self.developer_mode {
            target.developer_mode = value;
        }
        if let Some(value) = &self.last_run_path {
            target.last_run_path.clone_from(value);
        }
        if let Some(value) = self.agreed_to_terms {
            target.agreed_to_terms = value;
        }
        if let Some(value) = &self.plugin_allowed_commands {
            target.plugin_allowed_commands.clone_from(value);
        }
        if let Some(value) = &self.plugin_blocked_commands {
            target.plugin_blocked_commands.clone_from(value);
        }
    }
}

/// 设置更新结果，包含更新后的设置和变更分组。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    pub settings: AppSettings,
    pub changed_groups: Vec<SettingsGroup>,
}
