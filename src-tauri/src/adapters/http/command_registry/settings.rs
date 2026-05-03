use super::common::CommandHandler;
use crate::commands::app::settings as settings_commands;
use crate::models::settings::{AppSettings, PartialSettings};
use serde_json::Value;
use std::collections::HashMap;

pub(super) fn register_handlers(handlers: &mut HashMap<String, CommandHandler>) {
    handlers.insert("get_settings".to_string(), handle_get_settings as CommandHandler);
    handlers.insert("save_settings".to_string(), handle_save_settings as CommandHandler);
    handlers.insert(
        "save_settings_with_diff".to_string(),
        handle_save_settings_with_diff as CommandHandler,
    );
    handlers.insert(
        "update_settings_partial".to_string(),
        handle_update_settings_partial as CommandHandler,
    );
    handlers.insert("reset_settings".to_string(), handle_reset_settings as CommandHandler);
    handlers.insert(
        "export_settings".to_string(),
        handle_export_settings as CommandHandler,
    );
    handlers.insert(
        "import_settings".to_string(),
        handle_import_settings as CommandHandler,
    );
    handlers.insert(
        "check_acrylic_support".to_string(),
        handle_check_acrylic_support as CommandHandler,
    );
    handlers.insert("apply_acrylic".to_string(), handle_apply_acrylic as CommandHandler);
    handlers.insert(
        "get_system_fonts".to_string(),
        handle_get_system_fonts as CommandHandler,
    );
}

fn handle_get_settings(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::get_settings();
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

fn handle_get_system_fonts(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::get_system_fonts();
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}
