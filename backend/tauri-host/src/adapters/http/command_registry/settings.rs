use super::common::{CommandHandler, RegistryBuilder};
use crate::commands::app::settings as settings_commands;
use crate::commands::app::settings::{ChangeDataDirRequest, ChangePluginDirRequest};
use crate::models::settings::{AppSettings, PartialSettings};
use serde::de::DeserializeOwned;
use serde_json::Value;

fn parse_wrapped_or_root<T>(params: Value, field: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let value = params.get(field).cloned().unwrap_or(params);
    serde_json::from_value(value).map_err(|e| format!("Invalid parameters: {}", e))
}

pub(super) fn register_handlers(builder: &mut RegistryBuilder) {
    builder.register("get_settings", handle_get_settings as CommandHandler);
    builder.register("get_data_dir_status", handle_get_data_dir_status as CommandHandler);
    builder.register("initialize_data_dir", handle_initialize_data_dir as CommandHandler);
    builder.register("change_data_dir", handle_change_data_dir as CommandHandler);
    builder.register("get_plugin_dir_status", handle_get_plugin_dir_status as CommandHandler);
    builder.register("change_plugin_dir", handle_change_plugin_dir as CommandHandler);
    builder.register("save_settings", handle_save_settings as CommandHandler);
    builder.register("save_settings_with_diff", handle_save_settings_with_diff as CommandHandler);
    builder.register("update_settings_partial", handle_update_settings_partial as CommandHandler);
    builder.register("reset_settings", handle_reset_settings as CommandHandler);
    builder.register("export_settings", handle_export_settings as CommandHandler);
    builder.register("import_settings", handle_import_settings as CommandHandler);
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

fn handle_get_settings(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::get_settings();
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

fn handle_save_settings(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let settings: AppSettings = parse_wrapped_or_root(params, "settings")?;
        settings_commands::save_settings(settings)?;
        Ok(Value::Null)
    })
}

fn handle_save_settings_with_diff(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let settings: AppSettings = parse_wrapped_or_root(params, "settings")?;
        let result = settings_commands::save_settings_with_diff(settings)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_update_settings_partial(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let partial: PartialSettings = parse_wrapped_or_root(params, "partial")?;
        let result = settings_commands::update_settings_partial(partial)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_reset_settings(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::reset_settings()?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_export_settings(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::export_settings()?;
        Ok(Value::String(result))
    })
}

fn handle_import_settings(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let json: String = parse_wrapped_or_root(params, "json")?;
        let result = settings_commands::import_settings(json)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestSettingsPayload {
        theme: String,
    }

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
        let settings: TestSettingsPayload = parse_wrapped_or_root(
            json!({
                "settings": {
                    "theme": "dark"
                }
            }),
            "settings",
        )
        .expect("wrapped settings payload should deserialize");

        assert_eq!(settings, TestSettingsPayload { theme: "dark".into() });

        let imported_json: String = parse_wrapped_or_root(
            json!({
                "json": "{\"theme\":\"dark\"}"
            }),
            "json",
        )
        .expect("wrapped json payload should deserialize");

        assert_eq!(imported_json, "{\"theme\":\"dark\"}");
    }
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
