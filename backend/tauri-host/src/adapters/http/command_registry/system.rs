use super::common::{handle_unsupported, CommandHandler, RegistryBuilder};
use crate::commands::app::host as system_commands;
use crate::commands::app::logging as logging_commands;
use serde_json::Value;
pub(super) fn register_handlers(builder: &mut RegistryBuilder) {
    builder.register("get_system_info", handle_get_system_info as CommandHandler);
    builder.register("get_default_run_path", handle_get_default_run_path as CommandHandler);
    builder
        .register("get_server_resource_usage", handle_get_server_resource_usage as CommandHandler);
    builder.register("get_safe_mode_status", handle_get_safe_mode_status as CommandHandler);
    builder.register("test_ipv6_connectivity", handle_test_ipv6_connectivity as CommandHandler);
    builder.register("frontend_heartbeat", handle_frontend_heartbeat as CommandHandler);
    builder.register("get_logs", handle_get_logs as CommandHandler);
    builder.register("clear_logs", handle_clear_logs as CommandHandler);
    builder.register("check_developer_mode", handle_check_developer_mode as CommandHandler);
    builder.register("pick_jar_file", handle_unsupported as CommandHandler);
    builder.register("pick_startup_file", handle_unsupported as CommandHandler);
    builder.register("pick_java_file", handle_unsupported as CommandHandler);
    builder.register("pick_folder", handle_unsupported as CommandHandler);
    builder.register("pick_image_file", handle_unsupported as CommandHandler);
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
        let logs = logging_commands::get_logs(limit)?;
        serde_json::to_value(logs).map_err(|error| error.to_string())
    })
}

fn handle_clear_logs(_params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        logging_commands::clear_logs()?;
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
