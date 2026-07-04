#[path = "command_registry/common.rs"]
mod common;
#[path = "command_registry/config.rs"]
mod config;
#[path = "command_registry/java.rs"]
mod java;
#[path = "command_registry/player.rs"]
mod player;
#[path = "command_registry/plugin.rs"]
mod plugin;
#[path = "command_registry/registry.rs"]
mod registry;
#[path = "command_registry/requests.rs"]
mod requests;
#[path = "command_registry/server.rs"]
mod server;
#[path = "command_registry/settings.rs"]
mod settings;
#[path = "command_registry/system.rs"]
mod system;
#[path = "command_registry/tunnel.rs"]
mod tunnel;
#[path = "command_registry/update.rs"]
mod update;

pub use registry::CommandRegistry;

#[cfg(test)]
#[path = "../../../tests/unit/adapters_http_command_registry_tests.rs"]
mod tests;
