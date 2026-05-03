mod common;
mod config;
mod java;
mod player;
mod registry;
mod requests;
mod server;
mod settings;
mod system;
mod tunnel;
mod update;

pub use registry::CommandRegistry;

#[cfg(test)]
#[path = "../../../tests/unit/adapters_http_command_registry_tests.rs"]
mod tests;
