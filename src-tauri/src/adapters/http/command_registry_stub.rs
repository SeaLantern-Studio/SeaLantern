use futures::future::BoxFuture;
use serde_json::Value;

/// 非 docker 构建下的命令处理函数占位类型
pub type CommandHandler = fn(Value) -> BoxFuture<'static, Result<Value, String>>;

/// 非 docker 构建下的空命令表
pub struct CommandRegistry;

impl CommandRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn get_handler(&self, _command: &str) -> Option<&CommandHandler> {
        None
    }

    pub fn list_commands(&self) -> Vec<String> {
        Vec::new()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
