//! 应用设置的部分更新模型。

use sealantern_infra::net::proxy::ProxySettings;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{AppSettings, JavaInfo, SettingsGroup};

/// 可空设置字段的部分更新值。
///
/// `Unchanged` 表示请求未包含该字段，`Set(None)` 表示显式清空，
/// `Set(Some(value))` 表示写入新值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NullablePatch<T> {
    #[default]
    Unchanged,
    Set(Option<T>),
}

impl<T> NullablePatch<T> {
    pub fn set(value: T) -> Self {
        Self::Set(Some(value))
    }

    pub fn clear() -> Self {
        Self::Set(None)
    }

    pub fn is_unchanged(&self) -> bool {
        matches!(self, Self::Unchanged)
    }
}

impl<T: Serialize> Serialize for NullablePatch<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Unchanged => serializer.serialize_none(),
            Self::Set(value) => value.serialize(serializer),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for NullablePatch<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(deserializer).map(Self::Set)
    }
}

/// 部分更新请求，只合并请求中明确包含的字段。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartialAppSettings {
    pub close_servers_on_exit: Option<bool>,
    pub close_servers_on_update: Option<bool>,
    pub auto_accept_eula: Option<bool>,
    pub close_action: Option<String>,

    pub proxy: Option<ProxySettings>,

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
    pub acrylic_blur_level: Option<String>,
    pub theme: Option<String>,
    pub color: Option<String>,
    pub font_size: Option<u32>,
    pub font_family: Option<String>,
    pub minimal_mode: Option<bool>,

    #[serde(default, skip_serializing_if = "NullablePatch::is_unchanged")]
    pub window_width: NullablePatch<u32>,
    #[serde(default, skip_serializing_if = "NullablePatch::is_unchanged")]
    pub window_height: NullablePatch<u32>,
    #[serde(default, skip_serializing_if = "NullablePatch::is_unchanged")]
    pub window_x: NullablePatch<i32>,
    #[serde(default, skip_serializing_if = "NullablePatch::is_unchanged")]
    pub window_y: NullablePatch<i32>,
    #[serde(default, skip_serializing_if = "NullablePatch::is_unchanged")]
    pub window_maximized: NullablePatch<bool>,

    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "NullablePatch::is_unchanged")]
    pub locales_base_url: NullablePatch<String>,
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
        if let Some(value) = &self.proxy {
            target.proxy.clone_from(value);
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
        if let Some(value) = &self.acrylic_blur_level {
            target.acrylic_blur_level.clone_from(value);
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
        if let NullablePatch::Set(value) = self.window_width {
            target.window_width = value;
        }
        if let NullablePatch::Set(value) = self.window_height {
            target.window_height = value;
        }
        if let NullablePatch::Set(value) = self.window_x {
            target.window_x = value;
        }
        if let NullablePatch::Set(value) = self.window_y {
            target.window_y = value;
        }
        if let NullablePatch::Set(value) = self.window_maximized {
            target.window_maximized = value;
        }
        if let Some(value) = &self.language {
            target.language.clone_from(value);
        }
        if let NullablePatch::Set(value) = &self.locales_base_url {
            target.locales_base_url.clone_from(value);
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

#[cfg(test)]
mod tests {
    use sealantern_infra::net::proxy::{ProxyMode, ProxySettings};

    use super::{AppSettings, NullablePatch, PartialAppSettings};

    #[test]
    fn nullable_patch_distinguishes_missing_null_and_value() {
        let missing: PartialAppSettings =
            serde_json::from_str("{}").expect("empty partial settings should deserialize");
        assert_eq!(missing.window_x, NullablePatch::Unchanged);
        assert_eq!(missing.locales_base_url, NullablePatch::Unchanged);

        let clear: PartialAppSettings =
            serde_json::from_str(r#"{"window_x":null,"locales_base_url":null}"#)
                .expect("nullable fields should accept null");
        assert_eq!(clear.window_x, NullablePatch::Set(None));
        assert_eq!(clear.locales_base_url, NullablePatch::Set(None));

        let set: PartialAppSettings = serde_json::from_str(
            r#"{"window_x":120,"locales_base_url":"https://example.invalid/locales"}"#,
        )
        .expect("nullable fields should accept values");
        assert_eq!(set.window_x, NullablePatch::Set(Some(120)));
        assert_eq!(
            set.locales_base_url,
            NullablePatch::Set(Some("https://example.invalid/locales".to_string()))
        );
    }

    #[test]
    fn nullable_patch_can_clear_existing_values() {
        let mut settings = AppSettings {
            window_x: Some(120),
            locales_base_url: Some("https://example.invalid/locales".to_string()),
            ..AppSettings::default()
        };
        let partial: PartialAppSettings =
            serde_json::from_str(r#"{"window_x":null,"locales_base_url":null}"#)
                .expect("nullable fields should accept null");

        partial.merge_into(&mut settings);

        assert_eq!(settings.window_x, None);
        assert_eq!(settings.locales_base_url, None);
    }

    #[test]
    fn unchanged_nullable_fields_are_omitted_when_serialized() {
        let value = serde_json::to_value(PartialAppSettings::default())
            .expect("partial settings should serialize");

        assert!(value.get("window_x").is_none());
        assert!(value.get("locales_base_url").is_none());
    }

    #[test]
    fn partial_settings_can_replace_proxy_strategy() {
        let mut settings = AppSettings::default();
        let partial = PartialAppSettings {
            proxy: Some(ProxySettings {
                mode: ProxyMode::Manual {
                    proxy_url: "http://127.0.0.1:7890".into(),
                },
            }),
            ..PartialAppSettings::default()
        };

        partial.merge_into(&mut settings);

        assert_eq!(
            settings.proxy,
            ProxySettings {
                mode: ProxyMode::Manual {
                    proxy_url: "http://127.0.0.1:7890".into()
                }
            }
        );
    }
}
