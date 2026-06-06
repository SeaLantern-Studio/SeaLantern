//! Starter 安装器链接入口
//!
//! 这里只保留对外查询函数，缓存、解析和版本匹配逻辑拆到子模块里

use std::path::PathBuf;

///此处常量见 utils/constants.rs
use crate::utils::constants::STARTER_INSTALLER_LINKS_FILE;
use crate::utils::constants::STARTER_INSTALLER_LINKS_URL;

/// 按核心类型和 MC 版本查找 Starter 安装器下载地址
///
/// # Parameters
///
/// - `core_type_key`: 核心类型键，例如 `forge`、`fabric`
/// - `mc_version`: 目标 Minecraft 版本
///
/// # Returns
///
/// 返回安装器下载地址，以及一个预留的附加字段
pub fn fetch_starter_installer_url(
    core_type_key: &str,
    mc_version: &str,
) -> Result<(String, Option<String>), String> {
    let data_dir = PathBuf::from(
        crate::utils::path::get_or_create_app_data_dir_checked()
            .map_err(|e| format!("获取软件目录失败: {}", e))?,
    );
    std::fs::create_dir_all(&data_dir).map_err(|e| format!("创建软件目录失败: {}", e))?;
    let links_file_path = data_dir.join(STARTER_INSTALLER_LINKS_FILE);
    sea_lantern_starter_links_core::load_or_refresh_starter_links_json(
        &links_file_path,
        STARTER_INSTALLER_LINKS_URL,
    )?;
    sea_lantern_starter_links_core::resolve_installer_url_from_cache_file(
        &links_file_path,
        core_type_key,
        mc_version,
    )
}

#[cfg(test)]
mod tests {
    use super::fetch_starter_installer_url;
    use crate::test_support::{lock_env, EnvGuard};
    use std::fs;

    #[test]
    fn fetch_starter_installer_url_surfaces_app_data_dir_creation_failures() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let blocked_root = temp_dir.path().join("blocked-root");
        fs::write(&blocked_root, b"not a directory").expect("file-backed app data root should exist");
        let blocked_path = blocked_root.join("nested");
        let _lock = lock_env();
        let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &blocked_path.to_string_lossy());

        let error = fetch_starter_installer_url("forge", "1.20.1")
            .expect_err("app data dir creation failure should not be silently downgraded");

        assert!(error.contains("获取软件目录失败"), "unexpected error: {}", error);
        assert!(error.contains("blocked-root"), "unexpected error: {}", error);
    }
}
