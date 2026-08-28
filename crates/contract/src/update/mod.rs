//! 应用更新检查与安装契约模型。

mod models;
mod pending;

pub use models::{UpdateInfo, UpdateSource};
pub use pending::PendingUpdate;
