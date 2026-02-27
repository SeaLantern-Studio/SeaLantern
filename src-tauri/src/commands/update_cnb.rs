use crate::commands::update_types::UpdateInfo;
use crate::commands::update_version::{compare_versions, normalize_release_tag_version};
use serde::Deserialize;

const CNB_BASE_URL: &str = "https://cnb.cool";
const CNB_RELEASES_URL: &str = "https://cnb.cool/_next/data/f79eab91170dd314/zh/SeaLantern-studio/SeaLantern/-/releases.json?slug=SeaLantern-studio&slug=SeaLantern&slug=-&slug=releases";

#[derive(Debug, Deserialize)]
struct CnbResponse {
    #[serde(rename = "pageProps")]
    page_props: CnbPageProps,
}

#[derive(Debug, Deserialize)]
struct CnbPageProps {
    #[serde(rename = "initialState")]
    initial_state: CnbInitialState,
}

#[derive(Debug, Deserialize)]
struct CnbInitialState {
    slug: CnbSlug,
}

#[derive(Debug, Deserialize)]
struct CnbSlug {
    repo: CnbRepo,
}

#[derive(Debug, Deserialize)]
struct CnbRepo {
    releases: CnbReleases,
}

#[derive(Debug, Deserialize)]
struct CnbReleases {
    list: CnbReleaseList,
}

#[derive(Debug, Deserialize)]
struct CnbReleaseList {
    data: Option<CnbReleaseListData>,
}

#[derive(Debug, Deserialize)]
struct CnbReleaseListData {
    #[serde(default)]
    releases: Vec<CnbRelease>,
}

#[derive(Debug, Deserialize, Clone)]
struct CnbRelease {
    #[serde(rename = "tagRef")]
    tag_ref: String,
    title: Option<String>,
    body: Option<String>,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: Option<String>,
    #[serde(default)]
    assets: Vec<CnbAsset>,
}

#[derive(Debug, Deserialize, Clone)]
struct CnbAsset {
    name: String,
    path: String,
    #[serde(rename = "hashAlgo")]
    hash_algo: Option<String>,
    #[serde(rename = "hashValue")]
    hash_value: Option<String>,
}

fn normalize_tag_ref(tag_ref: &str) -> String {
    let tag = tag_ref.rsplit('/').next().unwrap_or(tag_ref);
    normalize_release_tag_version(tag)
}

fn release_time_key(release: &CnbRelease) -> String {
    release
        .published_at
        .clone()
        .or_else(|| release.created_at.clone())
        .unwrap_or_default()
}

fn find_suitable_asset(assets: &[CnbAsset]) -> Option<&CnbAsset> {
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

fn to_absolute_download_url(path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }

    format!("{}{}", CNB_BASE_URL, path)
}

fn asset_sha256(asset: &CnbAsset) -> Option<String> {
    let algo = asset.hash_algo.as_deref()?;
    let hash = asset.hash_value.as_deref()?.trim();
    if !algo.eq_ignore_ascii_case("sha256") || hash.is_empty() {
        return None;
    }

    Some(hash.to_string())
}

async fn fetch_releases(client: &reqwest::Client) -> Result<Vec<CnbRelease>, String> {
    let resp = client
        .get(CNB_RELEASES_URL)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("CNB request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("CNB API status: {}", resp.status()));
    }

    let payload: CnbResponse = resp
        .json()
        .await
        .map_err(|e| format!("CNB response parse failed: {}", e))?;

    Ok(payload
        .page_props
        .initial_state
        .slug
        .repo
        .releases
        .list
        .data
        .map(|v| v.releases)
        .unwrap_or_default())
}

pub async fn fetch_release(
    client: &reqwest::Client,
    current_version: &str,
) -> Result<UpdateInfo, String> {
    let releases = fetch_releases(client).await?;
    let latest_release = releases
        .iter()
        .max_by(|a, b| release_time_key(a).cmp(&release_time_key(b)))
        .ok_or_else(|| "CNB release list is empty".to_string())?;

    let latest_version = normalize_tag_ref(&latest_release.tag_ref);
    let has_newer_version = compare_versions(current_version, &latest_version);

    let selected_asset = find_suitable_asset(&latest_release.assets);
    let download_url = selected_asset.map(|asset| to_absolute_download_url(&asset.path));
    let has_update = has_newer_version && download_url.is_some();

    Ok(UpdateInfo {
        has_update,
        latest_version,
        current_version: current_version.to_string(),
        download_url,
        release_notes: latest_release
            .body
            .clone()
            .or_else(|| latest_release.title.clone()),
        published_at: latest_release
            .published_at
            .clone()
            .or_else(|| latest_release.created_at.clone()),
        source: Some("cnb".to_string()),
        sha256: selected_asset.and_then(asset_sha256),
    })
}

pub async fn resolve_download_candidate_by_version(
    client: &reqwest::Client,
    version: &str,
) -> Result<Option<(String, Option<String>)>, String> {
    let releases = fetch_releases(client).await?;
    let target_version = normalize_release_tag_version(version);

    let release = releases
        .iter()
        .find(|r| normalize_tag_ref(&r.tag_ref) == target_version);

    let Some(release) = release else {
        return Ok(None);
    };

    let Some(asset) = find_suitable_asset(&release.assets) else {
        return Ok(None);
    };

    Ok(Some((to_absolute_download_url(&asset.path), asset_sha256(asset))))
}
