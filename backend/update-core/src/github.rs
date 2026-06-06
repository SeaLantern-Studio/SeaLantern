use crate::asset_selector::{platform_asset_preferences, select_best_asset_by_name};
use crate::checksum::resolve_asset_sha256;
use crate::types::{ReleaseAsset, ReleaseResponse, RepoConfig, UpdateInfo};
use crate::version::{compare_versions_checked, normalize_release_tag_version};

fn find_suitable_asset(assets: &[ReleaseAsset]) -> Option<&ReleaseAsset> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let (target_suffixes, arch_keywords) = platform_asset_preferences(os, arch);
    select_best_asset_by_name(assets, |asset| asset.name.as_str(), target_suffixes, arch_keywords)
}

pub(crate) async fn fetch_release(
    client: &reqwest::Client,
    config: &RepoConfig,
    current_version: &str,
) -> Result<UpdateInfo, String> {
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

    build_update_info(client, &release, current_version).await
}

fn resolve_version_state(current_version: &str, tag_name: &str) -> Result<(String, bool), String> {
    let latest_version = normalize_release_tag_version(tag_name);
    let is_newer_version = compare_versions_checked(current_version, &latest_version)?;
    Ok((latest_version, is_newer_version))
}

async fn build_update_info(
    client: &reqwest::Client,
    release: &ReleaseResponse,
    current_version: &str,
) -> Result<UpdateInfo, String> {
    let (latest_version, is_newer_version) = resolve_version_state(current_version, &release.tag_name)?;
    let selected_asset = find_suitable_asset(&release.assets);
    let download_url = selected_asset.map(|asset| asset.browser_download_url.clone());
    let sha256 = if is_newer_version {
        if let Some(asset) = selected_asset {
            resolve_asset_sha256(client, &release.assets, asset).await?
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
        release_notes: release.body.clone(),
        published_at: release.published_at.clone().or_else(|| release.created_at.clone()),
        source: Some("github".to_string()),
        sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::resolve_version_state;

    #[test]
    fn resolve_version_state_rejects_invalid_release_tag_version() {
        let error = resolve_version_state("1.0.0", "release-without-semver")
            .expect_err("invalid release tag should not be silently treated as no update");

        assert!(error.contains("版本号无效") || error.contains("不能为空"), "unexpected error: {}", error);
        assert!(error.contains("release-without-semver"), "unexpected error: {}", error);
    }
}
