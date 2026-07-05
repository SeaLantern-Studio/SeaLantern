//! HTTP 适配层入口

#[cfg(feature = "docker")]
#[path = "command_registry.rs"]
pub mod command_registry;

#[cfg(not(feature = "docker"))]
#[path = "command_registry_stub.rs"]
pub mod command_registry;

#[cfg(feature = "docker")]
#[path = "server.rs"]
pub mod server;

#[cfg(not(feature = "docker"))]
#[path = "server_stub.rs"]
pub mod server;
