//! 应用里反复出现的固定文件名和目录名。

pub const PLUGIN_MANIFEST_FILE_NAME: &str = "manifest.json";
#[allow(dead_code)]
pub const PLUGIN_INSTALL_METADATA_FILE_NAME: &str =
    sea_lantern_plugin_trust_core::PLUGIN_INSTALL_METADATA_FILE_NAME;
pub const PLUGIN_MARKET_TEMP_DIR_NAME: &str = "sealantern_market_downloads";
pub const SERVER_PATH_PERMISSION_TEST_FILE_NAME: &str = ".sl_permission_test";
pub const APP_DIRECTORY_NAME: &str = "SeaLantern";
pub const APP_EXECUTABLE_NAME_WINDOWS: &str = "SeaLantern.exe";
