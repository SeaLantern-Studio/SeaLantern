//! CNB.cool 更新检查模块。
//!
//! 从 CNB.cool 平台获取 SeaLantern 的版本发布信息。
//! CNB OpenAPI 的匿名 release 查询需要登录，因此从公开 release 页面提取
//! 稳定详情链接，再解析详情页中的 Next.js 结构化数据。

use std::collections::HashSet;

use super::constants::{CNB_BASE_URL, CNB_RELEASES_URL};
use super::types::UpdateInfo;
use super::version::{compare_versions, normalize_release_tag_version};
use crate::observability;
use dom_query::Document;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CnbNextData {
    props: CnbNextProps,
}

#[derive(Debug, Deserialize)]
struct CnbNextProps {
    #[serde(rename = "pageProps")]
    page_props: CnbPageProps,
}

#[derive(Debug, Deserialize)]
struct CnbPageProps {
    #[serde(rename = "releasesDetailData")]
    releases_detail_data: CnbReleaseDetailData,
}

#[derive(Debug, Deserialize)]
struct CnbReleaseDetailData {
    release: CnbRelease,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

fn find_suitable_asset(assets: &[CnbAsset]) -> Option<&CnbAsset> {
    let target_suffixes: &[&str] = if cfg!(target_os = "windows") {
        &[".msi", ".exe"]
    } else if cfg!(target_os = "macos") {
        &[".dmg", ".app", ".tar.gz"]
    } else {
        &[".appimage", ".deb", ".rpm", ".tar.gz"]
    };

    let target_arch_aliases: &[&str] = if cfg!(target_arch = "x86_64") {
        &["x86_64", "amd64", "x64"]
    } else if cfg!(target_arch = "aarch64") {
        &["aarch64", "arm64"]
    } else {
        &[]
    };

    find_suitable_asset_for(assets, target_suffixes, target_arch_aliases)
}

fn find_suitable_asset_for<'a>(
    assets: &'a [CnbAsset],
    target_suffixes: &[&str],
    target_arch_aliases: &[&str],
) -> Option<&'a CnbAsset> {
    const KNOWN_ARCH_MARKERS: &[&str] =
        &["x86_64", "amd64", "x64", "i686", "x86", "aarch64", "arm64", "armv7"];

    for suffix in target_suffixes {
        if let Some(asset) = assets.iter().find(|asset| {
            let name = asset.name.to_ascii_lowercase();
            let has_target_arch = target_arch_aliases.iter().any(|alias| name.contains(alias));
            let has_other_known_arch = KNOWN_ARCH_MARKERS
                .iter()
                .any(|marker| name.contains(marker));
            name.ends_with(suffix) && (has_target_arch || !has_other_known_arch)
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

fn parse_release_links(html: &str) -> Result<Vec<String>, String> {
    let document = Document::from(html);
    let mut seen = HashSet::new();
    let mut links = Vec::new();

    for element in document.select("a[href*='/-/releases/tag/']").iter() {
        let Some(href) = element.attr("href") else {
            continue;
        };
        let href = href.to_string();
        if seen.insert(href.clone()) {
            links.push(href);
        }
    }

    if links.is_empty() {
        return Err("CNB release page contains no release links".to_string());
    }
    Ok(links)
}

fn parse_release_detail(html: &str) -> Result<CnbRelease, String> {
    let document = Document::from(html);
    let payload = document.select("script#__NEXT_DATA__").text().to_string();
    if payload.trim().is_empty() {
        return Err("CNB release page contains no Next.js data".to_string());
    }

    serde_json::from_str::<CnbNextData>(&payload)
        .map(|data| data.props.page_props.releases_detail_data.release)
        .map_err(|error| format!("CNB release data parse failed: {error}"))
}

async fn fetch_page(
    client: &reqwest::Client,
    url: &str,
    operation: &'static str,
) -> Result<String, String> {
    let response = client
        .get(url)
        .header("Accept", "text/html")
        .send()
        .await
        .map_err(|error| {
            let message = format!("CNB request failed: {error}");
            observability::update_api_request_failed("cnb", operation, None, &message);
            message
        })?;

    if !response.status().is_success() {
        let message = format!("CNB page status: {}", response.status());
        observability::update_api_request_failed(
            "cnb",
            operation,
            Some(response.status().as_u16()),
            &message,
        );
        return Err(message);
    }

    response.text().await.map_err(|error| {
        let message = format!("CNB response read failed: {error}");
        observability::update_api_request_failed("cnb", operation, None, &message);
        message
    })
}

async fn fetch_release_links(client: &reqwest::Client) -> Result<Vec<String>, String> {
    let html = fetch_page(client, CNB_RELEASES_URL, "fetch_release_list").await?;
    parse_release_links(&html).inspect_err(|message| {
        observability::update_api_request_failed("cnb", "parse_release_list", None, message);
    })
}

async fn fetch_release_detail(client: &reqwest::Client, path: &str) -> Result<CnbRelease, String> {
    let url = to_absolute_download_url(path);
    let html = fetch_page(client, &url, "fetch_release_detail").await?;
    parse_release_detail(&html).inspect_err(|message| {
        observability::update_api_request_failed("cnb", "parse_release_detail", None, message);
    })
}

fn normalized_version_from_link(link: &str) -> Option<String> {
    let encoded_tag = link.rsplit('/').next()?;
    let tag = urlencoding::decode(encoded_tag).ok()?;
    Some(normalize_release_tag_version(&tag))
}

pub async fn fetch_release(
    client: &reqwest::Client,
    current_version: &str,
) -> Result<UpdateInfo, String> {
    observability::update_check_started("cnb", current_version);

    let release_links = fetch_release_links(client).await?;
    let latest_link = release_links
        .first()
        .ok_or_else(|| "CNB release list is empty".to_string())?;
    let latest_release = fetch_release_detail(client, latest_link).await?;

    let latest_version = normalize_tag_ref(&latest_release.tag_ref);
    let has_newer_version = compare_versions(current_version, &latest_version);

    let selected_asset = find_suitable_asset(&latest_release.assets);
    let download_url = selected_asset.map(|asset| to_absolute_download_url(&asset.path));
    let has_update = has_newer_version && download_url.is_some();

    observability::update_check_completed("cnb", has_update, Some(&latest_version));

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
    let release_links = fetch_release_links(client).await?;
    let target_version = normalize_release_tag_version(version);

    let release_link = release_links
        .iter()
        .find(|link| normalized_version_from_link(link).as_deref() == Some(&target_version));

    let Some(release_link) = release_link else {
        return Ok(None);
    };
    let release = fetch_release_detail(client, release_link).await?;

    let Some(asset) = find_suitable_asset(&release.assets) else {
        return Ok(None);
    };

    Ok(Some((to_absolute_download_url(&asset.path), asset_sha256(asset))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(name: &str) -> CnbAsset {
        CnbAsset {
            name: name.to_owned(),
            path: format!("/downloads/{name}"),
            hash_algo: None,
            hash_value: None,
        }
    }

    #[test]
    fn release_links_are_ordered_and_deduplicated() {
        let html = r#"
            <a href="/org/repo/-/releases/tag/v2.0.0">latest</a>
            <a href="/org/repo/-/releases/tag/v2.0.0">latest duplicate</a>
            <a href="/org/repo/-/releases/tag/v1.0.0">older</a>
        "#;

        let links = parse_release_links(html).expect("parse release links");

        assert_eq!(links, ["/org/repo/-/releases/tag/v2.0.0", "/org/repo/-/releases/tag/v1.0.0"]);
    }

    #[test]
    fn release_detail_uses_structured_next_data() {
        let html = r#"
            <script id="__NEXT_DATA__" type="application/json">
              {"props":{"pageProps":{"releasesDetailData":{"release":{
                "tagRef":"refs/tags/v2.0.0",
                "title":"Version 2",
                "body":null,
                "publishedAt":"2026-01-01T00:00:00Z",
                "createdAt":null,
                "assets":[]
              }}}}}
            </script>
        "#;

        let release = parse_release_detail(html).expect("parse release detail");

        assert_eq!(release.tag_ref, "refs/tags/v2.0.0");
        assert_eq!(release.title.as_deref(), Some("Version 2"));
    }

    #[test]
    fn asset_selection_rejects_another_cpu_architecture() {
        let assets = [
            asset("SeaLantern_2.0.0_aarch64.AppImage"),
            asset("SeaLantern_2.0.0_amd64.AppImage"),
        ];

        let selected =
            find_suitable_asset_for(&assets, &[".appimage"], &["x86_64", "amd64", "x64"])
                .expect("select x86_64 asset");

        assert_eq!(selected.name, "SeaLantern_2.0.0_amd64.AppImage");
    }
}
