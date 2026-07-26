#![forbid(unsafe_code)]

pub mod config;
pub mod observability;
pub mod update;

#[path = "market/lib.rs"]
pub mod market;
