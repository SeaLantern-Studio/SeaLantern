use futures::future::BoxFuture;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

/// HTTP 命令处理器统一签名。
pub type CommandHandler = fn(Value) -> BoxFuture<'static, Result<Value, String>>;

pub struct RegistryBuilder {
    handlers: HashMap<String, CommandHandler>,
}

impl RegistryBuilder {
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    pub fn register(&mut self, command: impl Into<String>, handler: CommandHandler) {
        self.handlers.insert(command.into(), handler);
    }

    pub fn build(self) -> CommandRegistry {
        CommandRegistry::new(self.handlers)
    }
}

impl Default for RegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP 命令注册表的轻量共享容器。
#[derive(Default)]
pub struct CommandRegistry {
    handlers: HashMap<String, CommandHandler>,
}

impl CommandRegistry {
    pub fn new(handlers: HashMap<String, CommandHandler>) -> Self {
        Self { handlers }
    }

    pub fn get_handler(&self, command: &str) -> Option<&CommandHandler> {
        self.handlers.get(command)
    }

    pub fn list_commands(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

pub fn parse_params<T: DeserializeOwned>(params: Value) -> Result<T, String> {
    serde_json::from_value(params).map_err(|error| format!("Invalid parameters: {}", error))
}

pub fn handle_unsupported(_params: Value) -> BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move { Err("This command is not supported in HTTP/Docker mode".to_string()) })
}

pub fn is_supported_http_command(command: &str) -> bool {
    !command.starts_with("plugin/") || !command.contains("unsupported")
}

pub enum DispatchResult {
    Success(Value),
    NotFound(String),
    Failure(String),
}

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
        Err(error) => DispatchResult::Failure(error),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CommandRegistry, DispatchResult, CommandHandler, RegistryBuilder, dispatch_http_command,
        handle_unsupported, is_supported_http_command,
    };
    use serde_json::Value;
    use std::collections::HashMap;

    fn ok_handler(_params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
        Box::pin(async { Ok(serde_json::json!({ "ok": true })) })
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
