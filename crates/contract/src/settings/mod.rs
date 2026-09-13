//! 设置相关契约模型。

mod app;
mod app_update;
mod models;

pub use app::{
    AppSettings, CURRENT_CONFIG_VERSION, DEFAULT_ACRYLIC_BLUR_LEVEL, DEFAULT_TUNNEL_JOIN_PORT,
    DEFAULT_TUNNEL_LINK_LIFETIME, SettingsGroup, SettingsValidationError, TUNNEL_LINK_LIFETIMES,
};
pub use app_update::{NullablePatch, PartialAppSettings, UpdateResult};
pub use models::*;
