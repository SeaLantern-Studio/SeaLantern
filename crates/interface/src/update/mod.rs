//! 应用更新检查契约。

mod models;
mod service;

pub use models::{UpdateInfo, UpdateSource};
pub use service::{UpdateCheckService, UpdateInstallService};
