#![forbid(unsafe_code)]

pub mod observability;
pub mod process;

#[path = "instance/lib.rs"]
pub mod instance;

#[path = "server/lib.rs"]
pub mod server;
