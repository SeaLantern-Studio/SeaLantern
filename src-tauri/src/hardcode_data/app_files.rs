//! 应用里反复出现的固定文件名和目录名。

pub const PLUGIN_MANIFEST_FILE_NAME: &str = "manifest.json";
pub const PLUGIN_MARKET_TEMP_DIR_NAME: &str = "sealantern_market_downloads";
pub const JAVA_DOWNLOAD_TEMP_FILE_NAME: &str = "java_download.tmp";
pub const SERVER_PATH_PERMISSION_TEST_FILE_NAME: &str = ".sl_permission_test";
pub const APP_DIRECTORY_NAME: &str = "SeaLantern";
#[cfg(target_os = "linux")]
pub const APP_DIRECTORY_NAME_LOWERCASE: &str = "sea-lantern";
pub const APP_HIDDEN_DIRECTORY_NAME: &str = ".sea-lantern";
pub const APP_DOCKER_DATA_DIR: &str = "./data";
pub const APP_EXECUTABLE_NAME_WINDOWS: &str = "SeaLantern.exe";
