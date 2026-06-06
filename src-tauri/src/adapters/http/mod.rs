//! HTTP 适配层入口

#[cfg(feature = "docker")]
pub mod command_registry;
#[cfg(not(feature = "docker"))]
#[path = "command_registry_stub.rs"]
pub mod command_registry;

#[cfg(feature = "docker")]
pub mod server;
#[cfg(not(feature = "docker"))]
#[path = "server_stub.rs"]
pub mod server;

pub use server::run_http_server;
