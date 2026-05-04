use super::common::CommandHandler;
use crate::commands::update as update_commands;
use serde_json::Value;
use std::collections::HashMap;

pub(super) fn register_handlers(handlers: &mut HashMap<String, CommandHandler>) {
    handlers.insert("check_update".to_string(), handle_check_update as CommandHandler);
    handlers.insert("open_download_url".to_string(), handle_open_download_url as CommandHandler);
}

fn handle_check_update(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = update_commands::check_update().await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_open_download_url(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let url: String =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        update_commands::open_download_url(url).await?;
        Ok(Value::Null)
    })
}
