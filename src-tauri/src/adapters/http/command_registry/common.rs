use futures::future::BoxFuture;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// HTTP 命令处理器统一签名。
pub type CommandHandler = fn(Value) -> BoxFuture<'static, Result<Value, String>>;

pub(super) fn parse_params<T: DeserializeOwned>(params: Value) -> Result<T, String> {
    serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))
}

pub(super) fn handle_unsupported(_params: Value) -> BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move { Err("This command is not supported in HTTP/Docker mode".to_string()) })
}
