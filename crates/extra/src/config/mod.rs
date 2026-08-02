pub mod data_migration;
pub mod sealantern;
pub mod server;

pub use sealantern::types::{
    AppSettings, InstanceList, JavaInfo, PartialAppSettings, SettingsGroup, UpdateResult,
};
pub use sealantern::InstanceRegistry;
pub use sealantern::SettingsManager;

/// 解析应用数据目录，优先使用环境变量 `SEALANTERN_DATA_DIR`。
pub use sealantern_infra::platform::get_app_data_dir as resolve_data_dir;
