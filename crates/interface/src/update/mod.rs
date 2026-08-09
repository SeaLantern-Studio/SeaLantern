//! 应用更新检查契约。

pub mod models;
pub mod service;

pub use models::{UpdateInfo, UpdateSource};
pub use service::UpdateCheckService;
