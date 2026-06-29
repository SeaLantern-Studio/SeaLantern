use super::common::{CommandHandler, RegistryBuilder};
use crate::commands::app::settings as settings_commands;
use crate::commands::app::settings::{ChangeDataDirRequest, ChangePluginDirRequest};
use crate::models::settings::{AppSettings, PartialSettings};
use serde_json::Value;

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
        let path: String = serde_json::from_value(params.get("path").cloned().unwrap_or(params))
            .map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = settings_commands::initialize_data_dir(path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_change_data_dir(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let request: ChangeDataDirRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
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
        let request: ChangePluginDirRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = settings_commands::change_plugin_dir(request)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_save_settings(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let settings: AppSettings =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        settings_commands::save_settings(settings)?;
        Ok(Value::Null)
    })
}

fn handle_save_settings_with_diff(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let settings: AppSettings =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = settings_commands::save_settings_with_diff(settings)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_update_settings_partial(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let partial_value = params.get("partial").cloned().unwrap_or(params);
        let partial: PartialSettings = serde_json::from_value(partial_value)
            .map_err(|e| format!("Invalid parameters: {}", e))?;
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
        let json: String =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = settings_commands::import_settings(json)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
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
