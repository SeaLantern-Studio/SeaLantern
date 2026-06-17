use super::common::{CommandHandler, RegistryBuilder};
use crate::commands::update as update_commands;
use crate::utils::app_version;
use serde_json::Value;

pub(super) fn register_handlers(builder: &mut RegistryBuilder) {
    builder.register("check_update", handle_check_update as CommandHandler);
    builder.register("open_download_url", handle_open_download_url as CommandHandler);
}

fn handle_check_update(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = sea_lantern_update_core::check_update(app_version::base_version()).await?;
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
