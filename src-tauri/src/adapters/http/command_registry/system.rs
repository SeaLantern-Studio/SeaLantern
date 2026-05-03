use super::common::{handle_unsupported, CommandHandler};
use crate::commands::app::host as system_commands;
use serde_json::Value;
use std::collections::HashMap;

pub(super) fn register_handlers(handlers: &mut HashMap<String, CommandHandler>) {
    handlers.insert("get_system_info".to_string(), handle_get_system_info as CommandHandler);
    handlers.insert("pick_jar_file".to_string(), handle_unsupported as CommandHandler);
    handlers.insert(
        "pick_startup_file".to_string(),
        handle_unsupported as CommandHandler,
    );
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
