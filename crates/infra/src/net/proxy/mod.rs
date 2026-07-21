//! Proxy policy for outbound network clients.
//!
//! This module owns proxy settings and their in-memory resolution. It does not
//! read application configuration files or detect operating-system changes.
//! Application configuration and platform adapters provide those inputs.

mod config;
mod monitor;
mod policy;
mod system;

pub use config::{ProxyConfigError, ProxyMode, ProxySettings};
pub use monitor::ProxyMonitor;
pub use policy::{EffectiveProxy, ProxyController, ProxyUpdate};
pub use system::{read_system_proxy, ProxyRoutes, SystemProxyProvider, SystemProxySnapshot};
