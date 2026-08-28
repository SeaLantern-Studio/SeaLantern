//! 设置相关契约模型。

mod app;
mod app_update;
mod models;

pub use app::{
    AppSettings, CURRENT_CONFIG_VERSION, DEFAULT_ACRYLIC_BLUR_LEVEL, SettingsGroup,
    SettingsValidationError,
};
pub use app_update::{NullablePatch, PartialAppSettings, UpdateResult};
pub use models::*;
