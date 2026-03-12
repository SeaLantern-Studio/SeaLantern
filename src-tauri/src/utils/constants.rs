//! 后端常量定义
//! 为了便于快捷修改常量，在此处抽取了一部分后端使用的常量
//! 插件相关常量未抽取

///一个基本的User-agent
pub const USER_AGENT_EXAMPLE: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36 Edg/145.0.0.0";

/// services/settings_manager.rs
pub const SETTINGS_FILE: &str = "sea_lantern_settings.json";

/// services/java_detector.rs
#[cfg(target_os = "windows")]
pub const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
pub const JAVA_PATH_ALIASES: &[&str] = &[
    "java", "jdk", "jre", "graalvm", "corretto", "temurin", "zulu", "openjdk", "gvl", "ojdk",
    "bin", "j",
];

pub const ENV_VARS: &[&str] = &["JAVA_HOME", "JDK_HOME", "GRAALVM_HOME"];

#[cfg(target_os = "windows")]
pub const PROGRAM_FILES_JAVA_DIRS: &[&str] = &["Java", "Zulu", "Eclipse Adoptium", "BellSoft"];

#[cfg(not(target_os = "windows"))]
pub const COMMON_JAVA_DIRS: &[&str] =
    &["/usr/lib/jvm", "/usr/local/lib/jvm", "/Library/Java/JavaVirtualMachines"];

#[cfg(target_os = "windows")]
pub const MAX_SCAN_DEPTH: u32 = 5;

#[cfg(not(target_os = "windows"))]
pub const MAX_SCAN_DEPTH: u32 = 4;

/// services/i18n.rs
pub const SUPPORTED_LOCALES: &[&str] = &["zh-CN", "en-US"];

/// services/server/downloader.rs + services/download/starter_installer_links.rs
pub const DOWNLOAD_LINK_LIST_URL: &str = "https://cnb.cool/SeaLantern-studio/ServerCore-Mirror/-/releases/download/26.02.27/jar_lfs_links.json";
pub const STARTER_INSTALLER_LINKS_FILE: &str = "jar_lfs_links.json";
pub const STARTER_INSTALLER_LINKS_URL: &str = DOWNLOAD_LINK_LIST_URL;

/// services/download/java_installer.rs
pub const JAVA_DOWNLOAD_TIMEOUT_SECS: u64 = 60;
pub const JAVA_DOWNLOAD_RETRY_LIMIT: usize = 3;

/// services/server/log_pipeline.rs
pub const LATEST_LOG_DB_FILE: &str = "latest_log.db";
pub const LOG_BATCH_SIZE: usize = 128;
pub const LOG_FLUSH_INTERVAL_MS: u64 = 50;

/// services/server/manager.rs
pub const DATA_FILE: &str = "sea_lantern_servers.json";
pub const RUN_PATH_MAP_FILE: &str = "sea_lantern_run_path_map.json";

/// services/server/installer.rs
pub const STARTER_MC_VERSION_OPTIONS: [&str; 161] = [
    "26.1",
    "1.21.11",
    "1.21.10",
    "1.21.9",
    "1.21.8",
    "1.21.7",
    "1.21.6",
    "1.21.5",
    "1.21.4",
    "1.21.3",
    "1.21.2",
    "1.21.1",
    "1.21",
    "1.20.6",
    "1.20.5",
    "1.20.4",
    "1.20.3",
    "1.20.2",
    "1.20.1",
    "1.20",
    "1.19.4",
    "1.19.3",
    "1.19.2",
    "1.19.1",
    "1.19",
    "1.18.2",
    "1.18.1",
    "1.18",
    "1.17.1",
    "1.17",
    "1.16.5",
    "1.16.4",
    "1.16.3",
    "1.16.2",
    "1.16.1",
    "1.16",
    "1.15.2",
    "1.15.1",
    "1.15",
    "1.14.4",
    "1.14.3",
    "1.14.2",
    "1.14.1",
    "1.14",
    "1.13.2",
    "1.13.1",
    "1.13",
    "1.12.2",
    "1.12.1",
    "1.12",
    "1.11.2",
    "1.11.1",
    "1.11",
    "1.10.2",
    "1.10.1",
    "1.10",
    "1.9.4",
    "1.9.3",
    "1.9.2",
    "1.9.1",
    "1.9",
    "1.8.9",
    "1.8.8",
    "1.8.7",
    "1.8.6",
    "1.8.5",
    "1.8.4",
    "1.8.3",
    "1.8.2",
    "1.8.1",
    "1.8",
    "1.7.10",
    "1.7.9",
    "1.7.8",
    "1.7.7",
    "1.7.6",
    "1.7.5",
    "1.7.4",
    "1.7.3",
    "1.7.2",
    "1.7.1",
    "1.7",
    "1.6.4",
    "1.6.3",
    "1.6.2",
    "1.6.1",
    "1.6",
    "1.5.2",
    "1.5.1",
    "1.5",
    "1.4.7",
    "1.4.6",
    "1.4.5",
    "1.4.4",
    "1.4.3",
    "1.4.2",
    "1.4.1",
    "1.4",
    "1.3.2",
    "1.3.1",
    "1.3",
    "1.2.5",
    "1.2.4",
    "1.2.3",
    "1.2.2",
    "1.2.1",
    "1.1.9",
    "1.1",
    "1.0.10",
    "1.0",
    "1.13-pre7",
    "1.21.10-rc1",
    "1.21.11-pre1",
    "1.21.11-pre2",
    "1.21.11-pre3",
    "1.21.11-pre4",
    "1.21.11-pre5",
    "1.21.11-rc1",
    "1.21.11-rc2",
    "1.21.11-rc3",
    "1.21.9-pre1",
    "1.21.9-pre2",
    "1.21.9-pre3",
    "1.21.9-pre4",
    "1.21.9-rc1",
    "26.1-snapshot-1",
    "26.1-snapshot-2",
    "26.1-snapshot-3",
    "26.1-snapshot-4",
    "26.1-snapshot-5",
    "26.1-snapshot-6",
    "26.1-snapshot-7",
    "26.1-snapshot-8",
    "26.1-snapshot-9",
    "1.21.0",
    "1.4.0",
    "1.0.25",
    "25w46a",
    "25w45a",
    "25w44a",
    "25w43a",
    "25w42a",
    "25w41a",
    "25w37a",
    "25w36b",
    "25w35a",
    "25w33a",
    "25w32a",
    "25w21a",
    "25w20a",
    "25w19a",
    "25w18a",
    "25w17a",
    "25w10a",
    "25w09b",
    "25w08a",
    "25w07a",
    "25w05a",
    "25w04a",
    "25w03a",
    "25w02a",
];
