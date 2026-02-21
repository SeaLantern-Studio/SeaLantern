use crate::commands::update_checksum::resolve_asset_sha256;
use crate::commands::update_types::{ReleaseAsset, ReleaseResponse, RepoConfig, UpdateInfo};
use crate::commands::update_version::normalize_release_tag_version;

/// 查找适合当前平台的资源文件
pub fn find_suitable_asset(assets: &[ReleaseAsset]) -> Option<&ReleaseAsset> {
    let target_suffixes: &[&str] = if cfg!(target_os = "windows") {
        &[".msi", ".exe"]
    } else if cfg!(target_os = "macos") {
        &[".dmg", ".app", ".tar.gz"]
    } else {
        &[".appimage", ".deb", ".rpm", ".tar.gz"]
    };

    for suffix in target_suffixes {
        if let Some(asset) = assets.iter().find(|a| {
            let name = a.name.to_ascii_lowercase();
            name.ends_with(suffix)
        }) {
            return Some(asset);
        }
    }

    None
}

/// 获取 GitHub 最新发布版本
pub async fn fetch_release(
    client: &reqwest::Client,
    config: &RepoConfig,
    current_version: &str,
) -> Result<UpdateInfo, String> {
    use crate::commands::update_version::compare_versions;

    let url = config.api_url();

    let resp = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("API status: {}", resp.status()));
    }

    let release: ReleaseResponse = resp
        .json()
        .await
        .map_err(|e| format!("response parse failed: {}", e))?;

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
