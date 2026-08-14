#![forbid(unsafe_code)]

pub mod app_plugin;
pub mod config;
pub mod download_link;
pub mod java;
pub mod mclogs;
pub mod models;
pub mod observability;
pub mod server;
pub mod update;

#[cfg(feature = "online-tunnel")]
pub mod online;

#[path = "market/lib.rs"]
pub mod market;
