use super::common::{CommandHandler, RegistryBuilder};
use crate::utils::app_version;
use serde_json::Value;

pub(super) fn register_handlers(builder: &mut RegistryBuilder) {
    builder.register("check_update", handle_check_update as CommandHandler);
}

fn handle_check_update(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = sea_lantern_update_core::check_update(app_version::base_version()).await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}
