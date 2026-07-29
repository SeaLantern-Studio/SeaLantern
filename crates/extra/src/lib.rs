#![forbid(unsafe_code)]

pub mod backup;
pub mod app_plugin;
pub mod config;
pub mod download_link;
pub mod models;
pub mod observability;
pub mod update;

#[cfg(feature = "online-tunnel")]
pub mod online;

#[path = "market/lib.rs"]
pub mod market;
