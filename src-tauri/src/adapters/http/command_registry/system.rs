use super::common::{handle_unsupported, CommandHandler};
use crate::commands::app::host as system_commands;
use crate::commands::app::logging as logging_commands;
use serde_json::Value;
use std::collections::HashMap;

pub(super) fn register_handlers(handlers: &mut HashMap<String, CommandHandler>) {
    handlers.insert("get_system_info".to_string(), handle_get_system_info as CommandHandler);
    handlers.insert(
        "get_default_run_path".to_string(),
        handle_get_default_run_path as CommandHandler,
    );
    handlers.insert(
        "get_server_resource_usage".to_string(),
        handle_get_server_resource_usage as CommandHandler,
    );
    handlers.insert(
        "get_safe_mode_status".to_string(),
        handle_get_safe_mode_status as CommandHandler,
    );
    handlers.insert(
        "test_ipv6_connectivity".to_string(),
        handle_test_ipv6_connectivity as CommandHandler,
    );
    handlers.insert("frontend_heartbeat".to_string(), handle_frontend_heartbeat as CommandHandler);
    handlers.insert("get_logs".to_string(), handle_get_logs as CommandHandler);
    handlers.insert("clear_logs".to_string(), handle_clear_logs as CommandHandler);
    handlers.insert(
        "check_developer_mode".to_string(),
        handle_check_developer_mode as CommandHandler,
    );
    handlers.insert("pick_jar_file".to_string(), handle_unsupported as CommandHandler);
    handlers.insert("pick_startup_file".to_string(), handle_unsupported as CommandHandler);
    handlers.insert("pick_java_file".to_string(), handle_unsupported as CommandHandler);
    handlers.insert("pick_folder".to_string(), handle_unsupported as CommandHandler);
    handlers.insert("pick_image_file".to_string(), handle_unsupported as CommandHandler);
}

fn handle_get_system_info(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = system_commands::get_system_info()?;
        Ok(result)
    })
}

fn handle_get_default_run_path(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = system_commands::get_default_run_path()?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_get_server_resource_usage(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let server_id = params
            .get("serverId")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "Missing serverId".to_string())?
            .to_string();

        let result = system_commands::get_server_resource_usage(server_id)?;
        Ok(result)
    })
}

fn handle_get_safe_mode_status(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = system_commands::get_safe_mode_status()?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_test_ipv6_connectivity(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move { system_commands::test_ipv6_connectivity().await })
}

fn handle_frontend_heartbeat(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        system_commands::frontend_heartbeat()?;
        Ok(Value::Null)
    })
}

fn handle_get_logs(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let limit = params
            .get("limit")
            .and_then(|value| value.as_u64())
            .map(|value| value as usize);
        serde_json::to_value(logging_commands::get_logs(limit)).map_err(|error| error.to_string())
    })
}

fn handle_clear_logs(_params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        logging_commands::clear_logs();
        Ok(Value::Null)
    })
}

fn handle_check_developer_mode(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        serde_json::to_value(logging_commands::check_developer_mode())
            .map_err(|error| error.to_string())
    })
}
