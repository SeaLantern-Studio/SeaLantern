//! HTTP 适配层入口

pub mod command_registry;

pub mod server;

pub use server::run_http_server;
