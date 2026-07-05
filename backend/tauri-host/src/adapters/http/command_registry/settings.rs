use super::common::{CommandHandler, RegistryBuilder};
use crate::commands::app::settings as settings_commands;
use crate::commands::app::settings::{ChangeDataDirRequest, ChangePluginDirRequest};
use crate::models::server::{CpuPolicyConfig, JvmPresetConfig};
use crate::models::settings::{
    AppSettings, NextHomeLayoutItem, OneBot11Settings, OneBotTarget, PartialSettings,
    TextColorOverrides,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

fn parse_wrapped_or_root<T>(params: Value, field: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let value = params.get(field).cloned().unwrap_or(params);
    serde_json::from_value(value).map_err(|e| format!("Invalid parameters: {}", e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebOneBot11Settings {
    enabled: bool,
    api_base_url: String,
    event_classes: Vec<String>,
    structured_event_kinds: Vec<String>,
    server_ids: Vec<String>,
    targets: Vec<OneBotTarget>,
    message_template: String,
    access_token_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebSettingsDto {
    close_servers_on_exit: bool,
    close_servers_on_update: bool,
    auto_accept_eula: bool,
    default_max_memory: u32,
    default_min_memory: u32,
    default_port: u16,
    default_jvm_args: Vec<String>,
    default_cpu_policy: CpuPolicyConfig,
    default_jvm_preset: JvmPresetConfig,
    console_font_size: u32,
    console_font_family: String,
    console_letter_spacing: i32,
    max_log_lines: u32,
    background_opacity: f32,
    background_blur: u32,
    background_brightness: f32,
    background_size: String,
    window_effect: String,
    theme: String,
    color: String,
    font_size: u32,
    font_family: String,
    memory_display_precision: u8,
    text_color_overrides: TextColorOverrides,
    app_display_name: String,
    language: String,
    locale_layer_order: Vec<String>,
    developer_mode: bool,
    close_action: String,
    minimal_mode: bool,
    next_home_layout: Vec<NextHomeLayoutItem>,
    agreed_to_terms: bool,
    onebot_11: WebOneBot11Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
struct WebOneBot11SettingsPatch {
    enabled: Option<bool>,
    api_base_url: Option<String>,
    event_classes: Option<Vec<String>>,
    structured_event_kinds: Option<Vec<String>>,
    server_ids: Option<Vec<String>>,
    targets: Option<Vec<OneBotTarget>>,
    message_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
struct WebSettingsPatchDto {
    close_servers_on_exit: Option<bool>,
    close_servers_on_update: Option<bool>,
    auto_accept_eula: Option<bool>,
    default_max_memory: Option<u32>,
    default_min_memory: Option<u32>,
    default_port: Option<u16>,
    default_jvm_args: Option<Vec<String>>,
    default_cpu_policy: Option<CpuPolicyConfig>,
    default_jvm_preset: Option<JvmPresetConfig>,
    console_font_size: Option<u32>,
    console_font_family: Option<String>,
    console_letter_spacing: Option<i32>,
    max_log_lines: Option<u32>,
    background_opacity: Option<f32>,
    background_blur: Option<u32>,
    background_brightness: Option<f32>,
    background_size: Option<String>,
    window_effect: Option<String>,
    theme: Option<String>,
    color: Option<String>,
    font_size: Option<u32>,
    font_family: Option<String>,
    memory_display_precision: Option<u8>,
    text_color_overrides: Option<TextColorOverrides>,
    app_display_name: Option<String>,
    language: Option<String>,
    locale_layer_order: Option<Vec<String>>,
    developer_mode: Option<bool>,
    close_action: Option<String>,
    minimal_mode: Option<bool>,
    next_home_layout: Option<Vec<NextHomeLayoutItem>>,
    agreed_to_terms: Option<bool>,
    onebot_11: Option<WebOneBot11SettingsPatch>,
}

#[derive(Debug, Clone, Serialize)]
struct WebUpdateSettingsResult {
    settings: WebSettingsDto,
    changed_groups: Vec<String>,
}

pub(super) fn register_handlers(builder: &mut RegistryBuilder) {
    builder.register("get_web_settings", handle_get_web_settings as CommandHandler);
    builder.register("get_data_dir_status", handle_get_data_dir_status as CommandHandler);
    builder.register("initialize_data_dir", handle_initialize_data_dir as CommandHandler);
    builder.register("change_data_dir", handle_change_data_dir as CommandHandler);
    builder.register("get_plugin_dir_status", handle_get_plugin_dir_status as CommandHandler);
    builder.register("change_plugin_dir", handle_change_plugin_dir as CommandHandler);
    builder.register("save_web_settings", handle_save_web_settings as CommandHandler);
    builder.register(
        "update_web_settings_partial",
        handle_update_web_settings_partial as CommandHandler,
    );
    builder.register("reset_web_settings", handle_reset_web_settings as CommandHandler);
    builder.register("export_web_settings", handle_export_web_settings as CommandHandler);
    builder.register("import_web_settings", handle_import_web_settings as CommandHandler);
    builder.register(
        "export_personalization_package",
        handle_export_personalization_package as CommandHandler,
    );
    builder.register(
        "import_personalization_package",
        handle_import_personalization_package as CommandHandler,
    );
    builder.register(
        "get_personalization_package_suggested_name",
        handle_get_personalization_package_suggested_name as CommandHandler,
    );
    builder.register("check_acrylic_support", handle_check_acrylic_support as CommandHandler);
    builder.register("apply_acrylic", handle_apply_acrylic as CommandHandler);
    builder.register("apply_window_effect", handle_apply_window_effect as CommandHandler);
    builder.register("get_system_fonts", handle_get_system_fonts as CommandHandler);
}

fn map_web_settings(settings: AppSettings) -> WebSettingsDto {
    WebSettingsDto {
        close_servers_on_exit: settings.close_servers_on_exit,
        close_servers_on_update: settings.close_servers_on_update,
        auto_accept_eula: settings.auto_accept_eula,
        default_max_memory: settings.default_max_memory,
        default_min_memory: settings.default_min_memory,
        default_port: settings.default_port,
        default_jvm_args: settings.default_jvm_args,
        default_cpu_policy: settings.default_cpu_policy,
        default_jvm_preset: settings.default_jvm_preset,
        console_font_size: settings.console_font_size,
        console_font_family: settings.console_font_family,
        console_letter_spacing: settings.console_letter_spacing,
        max_log_lines: settings.max_log_lines,
        background_opacity: settings.background_opacity,
        background_blur: settings.background_blur,
        background_brightness: settings.background_brightness,
        background_size: settings.background_size,
        window_effect: settings.window_effect,
        theme: settings.theme,
        color: settings.color,
        font_size: settings.font_size,
        font_family: settings.font_family,
        memory_display_precision: settings.memory_display_precision,
        text_color_overrides: settings.text_color_overrides,
        app_display_name: settings.app_display_name,
        language: settings.language,
        locale_layer_order: settings.locale_layer_order,
        developer_mode: settings.developer_mode,
        close_action: settings.close_action,
        minimal_mode: settings.minimal_mode,
        next_home_layout: settings.next_home_layout,
        agreed_to_terms: settings.agreed_to_terms,
        onebot_11: WebOneBot11Settings {
            enabled: settings.onebot_11.enabled,
            api_base_url: settings.onebot_11.api_base_url,
            event_classes: settings.onebot_11.event_classes,
            structured_event_kinds: settings.onebot_11.structured_event_kinds,
            server_ids: settings.onebot_11.server_ids,
            targets: settings.onebot_11.targets,
            message_template: settings.onebot_11.message_template,
            access_token_configured: !settings.onebot_11.access_token.trim().is_empty(),
        },
    }
}

fn apply_web_settings_patch(current: &AppSettings, patch: WebSettingsPatchDto) -> PartialSettings {
    let onebot_11 = patch.onebot_11.map(|web_patch| OneBot11Settings {
        enabled: web_patch.enabled.unwrap_or(current.onebot_11.enabled),
        api_base_url: web_patch
            .api_base_url
            .unwrap_or_else(|| current.onebot_11.api_base_url.clone()),
        access_token: current.onebot_11.access_token.clone(),
        event_classes: web_patch
            .event_classes
            .unwrap_or_else(|| current.onebot_11.event_classes.clone()),
        structured_event_kinds: web_patch
            .structured_event_kinds
            .unwrap_or_else(|| current.onebot_11.structured_event_kinds.clone()),
        server_ids: web_patch
            .server_ids
            .unwrap_or_else(|| current.onebot_11.server_ids.clone()),
        targets: web_patch
            .targets
            .unwrap_or_else(|| current.onebot_11.targets.clone()),
        message_template: web_patch
            .message_template
            .unwrap_or_else(|| current.onebot_11.message_template.clone()),
    });

    PartialSettings {
        close_servers_on_exit: patch.close_servers_on_exit,
        close_servers_on_update: patch.close_servers_on_update,
        auto_accept_eula: patch.auto_accept_eula,
        default_max_memory: patch.default_max_memory,
        default_min_memory: patch.default_min_memory,
        default_port: patch.default_port,
        default_jvm_args: patch.default_jvm_args,
        default_cpu_policy: patch.default_cpu_policy,
        default_jvm_preset: patch.default_jvm_preset,
        console_font_size: patch.console_font_size,
        console_font_family: patch.console_font_family,
        console_letter_spacing: patch.console_letter_spacing,
        max_log_lines: patch.max_log_lines,
        background_opacity: patch.background_opacity,
        background_blur: patch.background_blur,
        background_brightness: patch.background_brightness,
        background_size: patch.background_size,
        window_effect: patch.window_effect,
        theme: patch.theme,
        color: patch.color,
        font_size: patch.font_size,
        font_family: patch.font_family,
        memory_display_precision: patch.memory_display_precision,
        text_color_overrides: patch.text_color_overrides,
        app_display_name: patch.app_display_name,
        language: patch.language,
        locale_layer_order: patch.locale_layer_order,
        developer_mode: patch.developer_mode,
        close_action: patch.close_action,
        minimal_mode: patch.minimal_mode,
        next_home_layout: patch.next_home_layout,
        agreed_to_terms: patch.agreed_to_terms,
        onebot_11,
        ..Default::default()
    }
}

fn handle_get_web_settings(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = map_web_settings(settings_commands::get_settings());
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_get_data_dir_status(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::get_data_dir_status();
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_initialize_data_dir(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let path: String = parse_wrapped_or_root(params, "path")?;
        let result = settings_commands::initialize_data_dir(path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_change_data_dir(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let request: ChangeDataDirRequest = parse_wrapped_or_root(params, "request")?;
        let result = settings_commands::change_data_dir(request)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_get_plugin_dir_status(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::get_plugin_dir_status();
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_change_plugin_dir(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let request: ChangePluginDirRequest = parse_wrapped_or_root(params, "request")?;
        let result = settings_commands::change_plugin_dir(request)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_save_web_settings(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let settings: WebSettingsDto = parse_wrapped_or_root(params, "settings")?;
        let current = settings_commands::get_settings();
        let partial = apply_web_settings_patch(
            &current,
            WebSettingsPatchDto {
                close_servers_on_exit: Some(settings.close_servers_on_exit),
                close_servers_on_update: Some(settings.close_servers_on_update),
                auto_accept_eula: Some(settings.auto_accept_eula),
                default_max_memory: Some(settings.default_max_memory),
                default_min_memory: Some(settings.default_min_memory),
                default_port: Some(settings.default_port),
                default_jvm_args: Some(settings.default_jvm_args),
                default_cpu_policy: Some(settings.default_cpu_policy),
                default_jvm_preset: Some(settings.default_jvm_preset),
                console_font_size: Some(settings.console_font_size),
                console_font_family: Some(settings.console_font_family),
                console_letter_spacing: Some(settings.console_letter_spacing),
                max_log_lines: Some(settings.max_log_lines),
                background_opacity: Some(settings.background_opacity),
                background_blur: Some(settings.background_blur),
                background_brightness: Some(settings.background_brightness),
                background_size: Some(settings.background_size),
                window_effect: Some(settings.window_effect),
                theme: Some(settings.theme),
                color: Some(settings.color),
                font_size: Some(settings.font_size),
                font_family: Some(settings.font_family),
                memory_display_precision: Some(settings.memory_display_precision),
                text_color_overrides: Some(settings.text_color_overrides),
                app_display_name: Some(settings.app_display_name),
                language: Some(settings.language),
                locale_layer_order: Some(settings.locale_layer_order),
                developer_mode: Some(settings.developer_mode),
                close_action: Some(settings.close_action),
                minimal_mode: Some(settings.minimal_mode),
                next_home_layout: Some(settings.next_home_layout),
                agreed_to_terms: Some(settings.agreed_to_terms),
                onebot_11: Some(WebOneBot11SettingsPatch {
                    enabled: Some(settings.onebot_11.enabled),
                    api_base_url: Some(settings.onebot_11.api_base_url),
                    event_classes: Some(settings.onebot_11.event_classes),
                    structured_event_kinds: Some(settings.onebot_11.structured_event_kinds),
                    server_ids: Some(settings.onebot_11.server_ids),
                    targets: Some(settings.onebot_11.targets),
                    message_template: Some(settings.onebot_11.message_template),
                }),
            },
        );
        let result = settings_commands::update_settings_partial(partial)?;
        let payload = WebUpdateSettingsResult {
            settings: map_web_settings(result.settings),
            changed_groups: result.changed_groups,
        };
        serde_json::to_value(payload).map_err(|e| e.to_string())
    })
}

fn handle_update_web_settings_partial(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let patch: WebSettingsPatchDto = parse_wrapped_or_root(params, "partial")?;
        let current = settings_commands::get_settings();
        let result =
            settings_commands::update_settings_partial(apply_web_settings_patch(&current, patch))?;
        let payload = WebUpdateSettingsResult {
            settings: map_web_settings(result.settings),
            changed_groups: result.changed_groups,
        };
        serde_json::to_value(payload).map_err(|e| e.to_string())
    })
}

fn handle_reset_web_settings(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::reset_settings()?;
        serde_json::to_value(map_web_settings(result)).map_err(|e| e.to_string())
    })
}

fn handle_export_web_settings(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let settings = map_web_settings(settings_commands::get_settings());
        serde_json::to_string_pretty(&settings)
            .map(Value::String)
            .map_err(|e| format!("Export failed: {}", e))
    })
}

fn handle_import_web_settings(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let json: String = parse_wrapped_or_root(params, "json")?;
        let settings: WebSettingsDto =
            serde_json::from_str(&json).map_err(|e| format!("Invalid JSON: {}", e))?;
        let current = settings_commands::get_settings();
        let partial = apply_web_settings_patch(
            &current,
            WebSettingsPatchDto {
                close_servers_on_exit: Some(settings.close_servers_on_exit),
                close_servers_on_update: Some(settings.close_servers_on_update),
                auto_accept_eula: Some(settings.auto_accept_eula),
                default_max_memory: Some(settings.default_max_memory),
                default_min_memory: Some(settings.default_min_memory),
                default_port: Some(settings.default_port),
                default_jvm_args: Some(settings.default_jvm_args),
                default_cpu_policy: Some(settings.default_cpu_policy),
                default_jvm_preset: Some(settings.default_jvm_preset),
                console_font_size: Some(settings.console_font_size),
                console_font_family: Some(settings.console_font_family),
                console_letter_spacing: Some(settings.console_letter_spacing),
                max_log_lines: Some(settings.max_log_lines),
                background_opacity: Some(settings.background_opacity),
                background_blur: Some(settings.background_blur),
                background_brightness: Some(settings.background_brightness),
                background_size: Some(settings.background_size),
                window_effect: Some(settings.window_effect),
                theme: Some(settings.theme),
                color: Some(settings.color),
                font_size: Some(settings.font_size),
                font_family: Some(settings.font_family),
                memory_display_precision: Some(settings.memory_display_precision),
                text_color_overrides: Some(settings.text_color_overrides),
                app_display_name: Some(settings.app_display_name),
                language: Some(settings.language),
                locale_layer_order: Some(settings.locale_layer_order),
                developer_mode: Some(settings.developer_mode),
                close_action: Some(settings.close_action),
                minimal_mode: Some(settings.minimal_mode),
                next_home_layout: Some(settings.next_home_layout),
                agreed_to_terms: Some(settings.agreed_to_terms),
                onebot_11: Some(WebOneBot11SettingsPatch {
                    enabled: Some(settings.onebot_11.enabled),
                    api_base_url: Some(settings.onebot_11.api_base_url),
                    event_classes: Some(settings.onebot_11.event_classes),
                    structured_event_kinds: Some(settings.onebot_11.structured_event_kinds),
                    server_ids: Some(settings.onebot_11.server_ids),
                    targets: Some(settings.onebot_11.targets),
                    message_template: Some(settings.onebot_11.message_template),
                }),
            },
        );
        let result = settings_commands::update_settings_partial(partial)?;
        serde_json::to_value(map_web_settings(result.settings)).map_err(|e| e.to_string())
    })
}

fn handle_export_personalization_package(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        Err(
            "export_personalization_package is not supported in HTTP/Docker mode (requires host file access)"
                .to_string(),
        )
    })
}

fn handle_import_personalization_package(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        Err(
            "import_personalization_package is not supported in HTTP/Docker mode (requires host file access)"
                .to_string(),
        )
    })
}

fn handle_get_personalization_package_suggested_name(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::get_personalization_package_suggested_name();
        Ok(Value::String(result))
    })
}

fn handle_check_acrylic_support(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        Err(
            "check_acrylic_support is not supported in HTTP/Docker mode (requires Window handle)"
                .to_string(),
        )
    })
}

fn handle_apply_acrylic(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        Err("apply_acrylic is not supported in HTTP/Docker mode (requires Window handle)"
            .to_string())
    })
}

fn handle_apply_window_effect(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        Err(
            "apply_window_effect is not supported in HTTP/Docker mode (requires Window handle)"
                .to_string(),
        )
    })
}

fn handle_get_system_fonts(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::get_system_fonts();
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_wrapped_or_root_accepts_request_wrappers_for_directory_commands() {
        let wrapped: ChangeDataDirRequest = parse_wrapped_or_root(
            json!({
                "request": {
                    "path": "E:/data",
                    "migrate_existing": true
                }
            }),
            "request",
        )
        .expect("wrapped request payload should deserialize");

        assert_eq!(wrapped.path, "E:/data");
        assert!(wrapped.migrate_existing);

        let legacy_root: ChangePluginDirRequest = parse_wrapped_or_root(
            json!({
                "path": "E:/plugins",
                "migrate_existing": false
            }),
            "request",
        )
        .expect("root request payload should remain supported");

        assert_eq!(legacy_root.path, "E:/plugins");
        assert!(!legacy_root.migrate_existing);
    }

    #[test]
    fn parse_wrapped_or_root_accepts_settings_and_json_wrappers() {
        let settings: WebSettingsPatchDto = parse_wrapped_or_root(
            json!({
                "partial": {
                    "theme": "dark"
                }
            }),
            "partial",
        )
        .expect("wrapped partial payload should deserialize");

        assert_eq!(settings.theme.as_deref(), Some("dark"));

        let imported_json: String = parse_wrapped_or_root(
            json!({
                "json": "{\"theme\":\"dark\"}"
            }),
            "json",
        )
        .expect("wrapped json payload should deserialize");

        assert_eq!(imported_json, "{\"theme\":\"dark\"}");
    }

    #[test]
    fn web_settings_mapping_redacts_sensitive_and_local_fields() {
        let settings = AppSettings {
            default_java_path: "C:/java/bin/java.exe".to_string(),
            background_image: "C:/secret/background.png".to_string(),
            last_run_path: "C:/secret".to_string(),
            onebot_11: OneBot11Settings {
                enabled: true,
                api_base_url: "https://onebot.example".to_string(),
                access_token: "secret-token".to_string(),
                event_classes: vec!["server".to_string()],
                structured_event_kinds: vec!["started".to_string()],
                server_ids: vec!["alpha".to_string()],
                targets: Vec::new(),
                message_template: "hello".to_string(),
            },
            ..AppSettings::default()
        };

        let mapped = map_web_settings(settings);
        let value = serde_json::to_value(mapped).expect("serialize web settings");

        assert!(value.get("default_java_path").is_none());
        assert!(value.get("background_image").is_none());
        assert!(value.get("last_run_path").is_none());
        assert!(value
            .get("onebot_11")
            .and_then(|entry| entry.get("access_token"))
            .is_none());
        assert_eq!(
            value
                .get("onebot_11")
                .and_then(|entry| entry.get("access_token_configured"))
                .and_then(|entry| entry.as_bool()),
            Some(true)
        );
    }
}
