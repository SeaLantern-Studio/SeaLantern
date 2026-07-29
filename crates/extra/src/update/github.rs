//! GitHub Release 更新检查模块。
//!
//! 通过 GitHub Releases API 获取 SeaLantern 的最新版本信息，
//! 并根据当前操作系统和 CPU 架构自动匹配合适的发布资源文件。
//!
//! # 平台匹配规则
//!
//! | 系统 | 匹配后缀 |
//! |------|----------|
//! | Windows | `.msi`, `.exe` |
//! | macOS | `.dmg`, `.app`, `.tar.gz` |
//! | Linux | `.appimage`, `.deb`, `.rpm`, `.tar.gz` |

use super::checksum::resolve_asset_sha256;
use super::types::{ReleaseAsset, ReleaseResponse, RepoConfig, UpdateInfo};
use super::version::normalize_release_tag_version;
use crate::observability;

/// 查找适合当前平台的资源文件
pub fn find_suitable_asset(assets: &[ReleaseAsset]) -> Option<&ReleaseAsset> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let (target_suffixes, arch_keywords) = get_platform_info(os, arch);

    for suffix in target_suffixes {
        let matching_assets: Vec<&ReleaseAsset> = assets
            .iter()
            .filter(|a| {
                let name = a.name.to_ascii_lowercase();
                name.ends_with(suffix)
            })
            .collect();

        if matching_assets.is_empty() {
            continue;
        }

        for arch_keyword in &arch_keywords {
            if let Some(asset) = matching_assets.iter().find(|a| {
                let name = a.name.to_ascii_lowercase();
                name.contains(arch_keyword)
            }) {
                return Some(asset);
            }
        }

        return matching_assets.into_iter().next();
    }

    None
}

// 查询平台信息
// 注意, 这里的arch是指cpu架构, 不是指系统架构, 不要和arch系统混淆了
fn get_platform_info(os: &str, arch: &str) -> (Vec<&'static str>, Vec<&'static str>) {
    let target_suffixes: Vec<&'static str> = match os {
        "windows" => vec![".msi", ".exe"],
        "macos" => vec![".dmg", ".app", ".tar.gz"],
        _ => vec![".appimage", ".deb", ".rpm", ".tar.gz"],
    };

    let arch_keywords: Vec<&'static str> = match arch {
        "x86_64" | "x64" | "amd64" => vec!["x86_64", "x64", "amd64"],
        "aarch64" | "arm64" | "arm" => vec!["aarch64", "arm64", "arm"],
        _ => vec![],
    };

    (target_suffixes, arch_keywords)
}

/// 获取 GitHub 最新发布版本
pub async fn fetch_release(
    client: &reqwest::Client,
    config: &RepoConfig,
    current_version: &str,
) -> Result<UpdateInfo, String> {
    use super::version::compare_versions;

    observability::update_check_started("github", current_version);

    let url = config.api_url();

    let resp = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| {
            let msg = format!("request failed: {}", e);
            observability::update_api_request_failed("github", "fetch_release", None, &msg);
            msg
        })?;

    if !resp.status().is_success() {
        let msg = format!("API status: {}", resp.status());
        observability::update_api_request_failed(
            "github",
            "fetch_release",
            Some(resp.status().as_u16()),
            &msg,
        );
        return Err(msg);
    }

    let release: ReleaseResponse = resp.json().await.map_err(|e| {
        let msg = format!("response parse failed: {}", e);
        observability::update_api_request_failed("github", "parse_release", None, &msg);
        msg
    })?;

    let latest_version = normalize_release_tag_version(&release.tag_name);
    let is_newer_version = compare_versions(current_version, &latest_version);
    let selected_asset = find_suitable_asset(&release.assets);
    let download_url = selected_asset.map(|asset| asset.browser_download_url.clone());
    let sha256 = if is_newer_version {
        if let Some(asset) = selected_asset {
            resolve_asset_sha256(client, &release.assets, asset).await
        } else {
            None
        }
    } else {
        None
    };

    let has_update = is_newer_version && download_url.is_some();

    observability::update_check_completed("github", has_update, Some(&latest_version));

    Ok(UpdateInfo {
        has_update,
        latest_version,
        current_version: current_version.to_string(),
        download_url,
        release_notes: release.body,
        published_at: release.published_at.or(release.created_at),
        source: Some("github".to_string()),
        sha256,
    })
}
