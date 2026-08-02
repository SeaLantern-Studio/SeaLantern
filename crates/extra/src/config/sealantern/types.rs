//! 配置模块的模型兼容导出。
//!
//! 模型定义由 `crate::models` 统一拥有；此模块保留既有配置访问路径。

pub use crate::models::{
    AppSettings, InstanceList, JavaInfo, PartialAppSettings, SettingsGroup, UpdateResult,
    CURRENT_CONFIG_VERSION,
};
