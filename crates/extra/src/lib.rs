#![forbid(unsafe_code)]

pub mod observability;
pub mod config;

#[path = "market/lib.rs"]
pub mod market;
