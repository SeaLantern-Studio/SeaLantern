use super::common::RegistryBuilder;
use super::{config, java, player, plugin, server, settings, system, tunnel, update};
use runtime::{dispatch_http_command, CommandRegistry as SharedCommandRegistry, DispatchResult};

/// 对外暴露的 HTTP 命令表。
pub struct CommandRegistry {
    shared: SharedCommandRegistry,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut builder = RegistryBuilder::new();

        server::register_handlers(&mut builder);
        java::register_handlers(&mut builder);
        config::register_handlers(&mut builder);
        system::register_handlers(&mut builder);
        player::register_handlers(&mut builder);
        plugin::register_handlers(&mut builder);
        settings::register_handlers(&mut builder);
        tunnel::register_handlers(&mut builder);
        update::register_handlers(&mut builder);
        Self { shared: builder.build() }
    }

    pub fn list_commands(&self) -> Vec<String> {
        self.shared.list_commands()
    }

    pub async fn dispatch(&self, command: &str, params: serde_json::Value) -> DispatchResult {
        dispatch_http_command(&self.shared, command, params).await
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
