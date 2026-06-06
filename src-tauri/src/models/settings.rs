//! 设置模型入口
//!
//! 这里保留对外公开的设置类型，具体实现拆到 `settings_model/` 目录下

#[path = "settings_model/groups.rs"]
mod groups;
#[path = "settings_model/merge.rs"]
mod merge;
#[path = "settings_model/schema.rs"]
mod schema;

/// 应用设置相关的公开类型
pub use schema::{AppSettings, PartialSettings, SettingsGroup};
