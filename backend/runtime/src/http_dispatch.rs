use futures::future::BoxFuture;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

const INVALID_REQUEST_PREFIX: &str = "__sea_lantern_http_invalid_request__:";

/// Async handler signature used by the headless HTTP command registry.
pub type CommandHandler = fn(Value) -> BoxFuture<'static, Result<Value, String>>;

/// Tags a command failure as an invalid client request so the HTTP layer can map it to 400.
pub fn invalid_request(message: impl Into<String>) -> String {
    format!("{}{}", INVALID_REQUEST_PREFIX, message.into())
}

fn strip_invalid_request_tag(message: &str) -> Option<&str> {
    message.strip_prefix(INVALID_REQUEST_PREFIX)
}

/// Builder for assembling a command registry before sharing it at runtime.
pub struct RegistryBuilder {
    handlers: HashMap<String, CommandHandler>,
}

impl RegistryBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    /// Registers a handler under the externally visible command name.
    pub fn register(&mut self, command: impl Into<String>, handler: CommandHandler) {
        self.handlers.insert(command.into(), handler);
    }

    /// Freezes the builder into an immutable registry.
    pub fn build(self) -> CommandRegistry {
        CommandRegistry::new(self.handlers)
    }
}

impl Default for RegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Lightweight shared container for HTTP command handlers.
#[derive(Default)]
pub struct CommandRegistry {
    handlers: HashMap<String, CommandHandler>,
}

impl CommandRegistry {
    /// Creates a registry from a prebuilt handler map.
    pub fn new(handlers: HashMap<String, CommandHandler>) -> Self {
        Self { handlers }
    }

    /// Returns the handler registered for `command`, if any.
    pub fn get_handler(&self, command: &str) -> Option<&CommandHandler> {
        self.handlers.get(command)
    }

    /// Lists registered command names for diagnostics and discovery endpoints.
    pub fn list_commands(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

/// Deserializes raw JSON parameters into the strongly typed request payload.
pub fn parse_params<T: DeserializeOwned>(params: Value) -> Result<T, String> {
    serde_json::from_value(params)
        .map_err(|error| invalid_request(format!("Invalid parameters: {}", error)))
}

/// Default handler used for commands that exist on desktop but are intentionally disabled in HTTP mode.
pub fn handle_unsupported(_params: Value) -> BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move { Err("This command is not supported in HTTP/Docker mode".to_string()) })
}

/// Returns whether a command should be exposed through the HTTP transport.
pub fn is_supported_http_command(command: &str) -> bool {
    !command.starts_with("plugin/") || !command.contains("unsupported")
}

/// Result of dispatching an HTTP command without coupling callers to transport-specific status codes.
pub enum DispatchResult {
    Success(Value),
    NotFound(String),
    InvalidRequest(String),
    Failure(String),
}

/// Routes an incoming command to its handler and normalizes the outcome for the HTTP layer.
pub async fn dispatch_http_command(
    registry: &CommandRegistry,
    command: &str,
    params: Value,
) -> DispatchResult {
    let Some(handler) = registry.get_handler(command) else {
        return DispatchResult::NotFound(format!(
            "Command '{}' not found. Use GET /api/list to see available commands.",
            command
        ));
    };

    match handler(params).await {
        Ok(data) => DispatchResult::Success(data),
        Err(error) => match strip_invalid_request_tag(&error) {
            Some(message) => DispatchResult::InvalidRequest(message.to_string()),
            None => DispatchResult::Failure(error),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        dispatch_http_command, handle_unsupported, invalid_request, is_supported_http_command,
        CommandHandler, CommandRegistry, DispatchResult, RegistryBuilder,
    };
    use serde_json::Value;
    use std::collections::HashMap;

    fn ok_handler(_params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
        Box::pin(async { Ok(serde_json::json!({ "ok": true })) })
    }

    fn invalid_request_handler(
        _params: Value,
    ) -> futures::future::BoxFuture<'static, Result<Value, String>> {
        Box::pin(async { Err(invalid_request("Missing path")) })
    }

    #[test]
    fn supported_http_command_filter_matches_plugin_unsupported_convention() {
        assert!(is_supported_http_command("get_logs"));
        assert!(is_supported_http_command("plugin/foo"));
        assert!(!is_supported_http_command("plugin/unsupported/foo"));
    }

    #[test]
    fn dispatch_returns_not_found_for_unknown_command() {
        let registry = CommandRegistry::default();
        let runtime = tokio::runtime::Runtime::new().expect("runtime");

        match runtime.block_on(dispatch_http_command(&registry, "missing", Value::Null)) {
            DispatchResult::NotFound(message) => assert!(message.contains("missing")),
            _ => panic!("expected not found"),
        }
    }

    #[test]
    fn dispatch_runs_registered_handler() {
        let mut handlers = HashMap::new();
        handlers.insert("ok".to_string(), ok_handler as CommandHandler);
        let registry = CommandRegistry::new(handlers);
        let runtime = tokio::runtime::Runtime::new().expect("runtime");

        match runtime.block_on(dispatch_http_command(&registry, "ok", Value::Null)) {
            DispatchResult::Success(value) => assert_eq!(value["ok"], Value::Bool(true)),
            _ => panic!("expected success"),
        }
    }

    #[test]
    fn dispatch_preserves_invalid_request_classification() {
        let mut handlers = HashMap::new();
        handlers.insert("invalid".to_string(), invalid_request_handler as CommandHandler);
        let registry = CommandRegistry::new(handlers);
        let runtime = tokio::runtime::Runtime::new().expect("runtime");

        match runtime.block_on(dispatch_http_command(&registry, "invalid", Value::Null)) {
            DispatchResult::InvalidRequest(message) => assert_eq!(message, "Missing path"),
            _ => panic!("expected invalid request"),
        }
    }

    #[test]
    fn unsupported_handler_produces_failure_dispatch() {
        let mut handlers = HashMap::new();
        handlers.insert("unsupported".to_string(), handle_unsupported as CommandHandler);
        let registry = CommandRegistry::new(handlers);
        let runtime = tokio::runtime::Runtime::new().expect("runtime");

        match runtime.block_on(dispatch_http_command(&registry, "unsupported", Value::Null)) {
            DispatchResult::Failure(message) => assert!(message.contains("not supported")),
            _ => panic!("expected failure"),
        }
    }

    #[test]
    fn registry_builder_registers_handlers() {
        let mut builder = RegistryBuilder::new();
        builder.register("ok", ok_handler as CommandHandler);
        let registry = builder.build();

        assert!(registry.get_handler("ok").is_some());
    }
}
