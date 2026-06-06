//! Starter 安装器链接入口
//!
//! 这里只保留对外查询函数，缓存、解析和版本匹配逻辑拆到子模块里

#[path = "starter_installer_links_core/cache.rs"]
mod cache;
#[path = "starter_installer_links_core/parser.rs"]
mod parser;
#[path = "starter_installer_links_core/version.rs"]
mod version;

use std::path::PathBuf;

use parser::{resolve_installer_url_from_nested_json, StarterLinksPayload};

///此处常量见 utils/constants.rs
use crate::utils::constants::STARTER_INSTALLER_LINKS_FILE;

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
    let data_dir = PathBuf::from(crate::utils::path::get_or_create_app_data_dir());
    std::fs::create_dir_all(&data_dir).map_err(|e| format!("创建软件目录失败: {}", e))?;
    let links_file_path = data_dir.join(STARTER_INSTALLER_LINKS_FILE);
    let body = cache::load_or_refresh_starter_links_json(&links_file_path)?;

    let payload: StarterLinksPayload =
        serde_json::from_slice(&body).map_err(|e| format!("解析 Starter 下载信息失败: {}", e))?;
    let core_key = core_type_key.trim().to_ascii_lowercase();
    let target_version = mc_version.trim().to_ascii_lowercase();
    if core_key.is_empty() || target_version.is_empty() {
        return Err("Starter 下载参数缺少核心类型或 MC 版本".to_string());
    }

    if let Some(installer_url) =
        resolve_installer_url_from_nested_json(&payload, &core_key, &target_version)
    {
        return Ok((installer_url, None));
    }

    Err(format!(
        "未在 CNB 镜像中找到匹配下载链接：core={}, version={}",
        core_type_key, mc_version
    ))
}
