use super::common::CommandHandler;
use super::{config, java, player, plugin, server, settings, system, tunnel, update};
use std::collections::HashMap;

/// 对外暴露的 HTTP 命令表。
pub struct CommandRegistry {
    handlers: HashMap<String, CommandHandler>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut handlers: HashMap<String, CommandHandler> = HashMap::new();

        server::register_handlers(&mut handlers);
        java::register_handlers(&mut handlers);
        config::register_handlers(&mut handlers);
        system::register_handlers(&mut handlers);
        player::register_handlers(&mut handlers);
        plugin::register_handlers(&mut handlers);
        settings::register_handlers(&mut handlers);
        tunnel::register_handlers(&mut handlers);
        update::register_handlers(&mut handlers);
        Self { handlers }
    }

    pub fn get_handler(&self, command: &str) -> Option<&CommandHandler> {
        self.handlers.get(command)
    }

    pub fn list_commands(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
