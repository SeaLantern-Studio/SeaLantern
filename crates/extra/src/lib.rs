#![forbid(unsafe_code)]

pub mod config;
pub mod observability;

#[path = "market/lib.rs"]
pub mod market;
