#![forbid(unsafe_code)]

pub mod app_plugin;
pub mod observability;
pub mod process;

#[path = "instance/lib.rs"]
pub mod instance;

#[path = "provisioning/lib.rs"]
pub mod provisioning;

#[path = "server/lib.rs"]
pub mod server;
