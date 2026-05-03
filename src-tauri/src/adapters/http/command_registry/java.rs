use super::common::{parse_params, CommandHandler};
use super::requests::ValidateJavaPathRequest;
use crate::commands::downloads::java as java_commands;
use serde_json::Value;
use std::collections::HashMap;

pub(super) fn register_handlers(handlers: &mut HashMap<String, CommandHandler>) {
    handlers.insert("detect_java".to_string(), handle_detect_java as CommandHandler);
    handlers.insert(
        "validate_java_path".to_string(),
        handle_validate_java_path as CommandHandler,
    );
    handlers.insert(
        "cancel_java_install".to_string(),
        handle_cancel_java_install as CommandHandler,
    );
}

fn handle_detect_java(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = java_commands::detect_java().await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_validate_java_path(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ValidateJavaPathRequest = parse_params(params)?;
        let result = java_commands::validate_java_path(req.path).await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_cancel_java_install(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        java_commands::cancel_java_install().await?;
        Ok(Value::Null)
    })
}
