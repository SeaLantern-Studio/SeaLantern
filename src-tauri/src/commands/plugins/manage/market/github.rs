use super::shared::build_market_client;
use crate::hardcode_data::plugin_market::{
    GITHUB_CODELOAD_BASE_URL, GITHUB_RELEASE_API_BASE_URL, PLUGIN_MARKET_ALLOWED_DOWNLOAD_DOMAINS,
    PLUGIN_MARKET_HTTP_USER_AGENT,
};
use url::Url;

#[derive(Debug, serde::Deserialize)]
struct GitHubReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubReleaseAsset>,
    zipball_url: String,
}

pub(super) fn resolve_plugin_download_url(
    download_url: Option<String>,
    repo: Option<String>,
    download_type: Option<String>,
    release_asset: Option<String>,
    branch: Option<String>,
    version: Option<String>,
) -> Result<String, String> {
    if let Some(url) = download_url {
        return Ok(url);
    }

    let repo = match repo {
        Some(repo) => repo,
        None => return Err("No download source specified".to_string()),
    };

    if repo.starts_with("http://") || repo.starts_with("https://") {
        return Ok(repo);
    }

    let (url, _version) = resolve_github_download_url(
        &repo,
        download_type.as_deref(),
        release_asset.as_deref(),
        branch.as_deref(),
        version.as_deref(),
    )?;

    Ok(url)
}

#[allow(clippy::needless_option_as_deref)]
fn resolve_github_download_url(
    github: &str,
    download_type: Option<&str>,
    release_asset: Option<&str>,
    branch: Option<&str>,
    version: Option<&str>,
) -> Result<(String, String), String> {
    let client = build_market_client(PLUGIN_MARKET_HTTP_USER_AGENT)?;
    let download_type = download_type.unwrap_or("release");

    if download_type == "release" {
        let api_url = match version.as_deref() {
            None | Some("latest") => {
                format!("{}/{}/releases/latest", GITHUB_RELEASE_API_BASE_URL, github)
            }
            Some(tag) => {
                format!("{}/{}/releases/tags/{}", GITHUB_RELEASE_API_BASE_URL, github, tag)
            }
        };

        let response = client
            .get(&api_url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .map_err(|e| format!("Failed to fetch GitHub release: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API returned status: {}", response.status()));
        }

        let release: GitHubRelease = response
            .json()
            .map_err(|e| format!("Failed to parse GitHub release: {}", e))?;

        let detected_version = release.tag_name.clone();
        let download_url = if let Some(asset_name) = release_asset {
            let url = release
                .assets
                .iter()
                .find(|asset| asset.name == asset_name || asset.name.contains(asset_name))
                .map(|asset| asset.browser_download_url.clone())
                .ok_or_else(|| format!("Asset '{}' not found in release", asset_name))?;

            let parsed_url =
                Url::parse(&url).map_err(|e| format!("Invalid browser_download_url: {}", e))?;
            if !PLUGIN_MARKET_ALLOWED_DOWNLOAD_DOMAINS
                .contains(&parsed_url.host_str().unwrap_or(""))
            {
                return Err(format!(
                    "browser_download_url domain '{}' is not in the allowed list",
                    parsed_url.host_str().unwrap_or("")
                ));
            }

            url
        } else {
            release.zipball_url
        };

        Ok((download_url, detected_version))
    } else {
        let branch = branch.unwrap_or("main");
        let download_url =
            format!("{}/{}/zip/refs/heads/{}", GITHUB_CODELOAD_BASE_URL, github, branch);
        Ok((download_url, "source".to_string()))
    }
}
