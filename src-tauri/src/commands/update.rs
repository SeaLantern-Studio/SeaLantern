use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub has_update: bool,
    pub latest_version: String,
    pub current_version: String,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum UpdateSource {
    Gitee,
    GitHub,
}

impl UpdateSource {
    fn config(self) -> RepoConfig {
        match self {
            Self::Gitee => RepoConfig {
                owner: "fps_z",
                repo: "SeaLantern",
                api_base: "https://gitee.com/api/v5/repos",
                web_base: "https://gitee.com",
                accept_header: self.accept_header(),
            },
            Self::GitHub => RepoConfig {
                owner: "FPSZ",
                repo: "SeaLantern",
                api_base: "https://api.github.com/repos",
                web_base: "https://github.com",
                accept_header: self.accept_header(),
            },
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Gitee => "gitee",
            Self::GitHub => "github",
        }
    }

    fn accept_header(self) -> &'static str {
        match self {
            Self::Gitee => "application/json",
            Self::GitHub => "application/vnd.github+json",
        }
    }
}

struct RepoConfig {
    owner: &'static str,
    repo: &'static str,
    api_base: &'static str,
    web_base: &'static str,
    accept_header: &'static str,
}

impl RepoConfig {
    fn api_url(&self) -> String {
        format!("{}/{}/{}/releases/latest", self.api_base, self.owner, self.repo)
    }

    fn release_url(&self, tag: &str) -> String {
        format!("{}/{}/{}/releases/tag/{}", self.web_base, self.owner, self.repo, tag)
    }
}

#[derive(Debug, Deserialize)]
struct ReleaseResponse {
    tag_name: String,
    html_url: Option<String>,
    body: Option<String>,
    assets: Vec<ReleaseAsset>,
    published_at: Option<String>,
    created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[command]
pub async fn check_update() -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| format!("HTTP client init failed: {}", e))?;

    let gitee_source = UpdateSource::Gitee;
    let gitee_config = gitee_source.config();

    let gitee_error =
        match fetch_release(&client, &gitee_config, current_version, gitee_source.as_str()).await {
            Ok(info) => return Ok(info),
            Err(err) => err,
        };

    let github_source = UpdateSource::GitHub;
    let github_config = github_source.config();

    fetch_release(&client, &github_config, current_version, github_source.as_str())
        .await
        .map_err(|github_error| {
            format!(
                "Both Gitee and GitHub failed. Gitee: {}; GitHub: {}",
                gitee_error, github_error
            )
        })
}

async fn fetch_release(
    client: &reqwest::Client,
    config: &RepoConfig,
    current_version: &str,
    source: &str,
) -> Result<UpdateInfo, String> {
    let url = config.api_url();

    let resp = client
        .get(&url)
        .header("Accept", config.accept_header)
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

    let latest_version = release.tag_name.trim_start_matches(['v', 'V']);
    let has_update = compare_versions(current_version, latest_version);

    let download_url = find_suitable_asset(&release.assets).or_else(|| {
        release
            .html_url
            .clone()
            .or_else(|| Some(config.release_url(&release.tag_name)))
    });

    Ok(UpdateInfo {
        has_update,
        latest_version: latest_version.to_string(),
        current_version: current_version.to_string(),
        download_url,
        release_notes: release.body,
        published_at: release.published_at.or(release.created_at),
        source: Some(source.to_string()),
    })
}

fn find_suitable_asset(assets: &[ReleaseAsset]) -> Option<String> {
    let target_suffixes: &[&str] = if cfg!(target_os = "windows") {
        &[".msi", ".exe"]
    } else if cfg!(target_os = "macos") {
        &[".dmg", ".app", ".tar.gz"]
    } else {
        &[".appimage", ".deb", ".rpm", ".tar.gz"]
    };

    for suffix in target_suffixes {
        if let Some(asset) = assets
            .iter()
            .find(|a| a.name.to_ascii_lowercase().ends_with(suffix))
        {
            return Some(asset.browser_download_url.clone());
        }
    }

    None
}

fn compare_versions(current: &str, latest: &str) -> bool {
    let current_v = parse_version(current);
    let latest_v = parse_version(latest);
    latest_v > current_v
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ParsedVersion {
    core: [u64; 3],
    pre: Option<Vec<PreIdent>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum PreIdent {
    Numeric(u64),
    AlphaNum(String),
}

impl Ord for PreIdent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match (self, other) {
            (Self::Numeric(a), Self::Numeric(b)) => a.cmp(b),
            (Self::Numeric(_), Self::AlphaNum(_)) => Ordering::Less,
            (Self::AlphaNum(_), Self::Numeric(_)) => Ordering::Greater,
            (Self::AlphaNum(a), Self::AlphaNum(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for PreIdent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ParsedVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match self.core.cmp(&other.core) {
            Ordering::Equal => {}
            ord => return ord,
        }

        match (&self.pre, &other.pre) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => {
                for i in 0..std::cmp::max(a.len(), b.len()) {
                    match (a.get(i), b.get(i)) {
                        (Some(x), Some(y)) => match x.cmp(y) {
                            Ordering::Equal => continue,
                            ord => return ord,
                        },
                        (Some(_), None) => return Ordering::Greater,
                        (None, Some(_)) => return Ordering::Less,
                        (None, None) => return Ordering::Equal,
                    }
                }

                Ordering::Equal
            }
        }
    }
}

impl PartialOrd for ParsedVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_version(input: &str) -> ParsedVersion {
    let normalized = input.trim().trim_start_matches(['v', 'V']);
    let no_build = normalized.split('+').next().unwrap_or(normalized);

    let (core_part, pre_part) = no_build
        .split_once('-')
        .map_or((no_build, None), |(core, pre)| (core, Some(pre)));

    let mut core = [0_u64; 3];
    for (idx, piece) in core_part.split('.').take(3).enumerate() {
        core[idx] = piece.trim().parse::<u64>().unwrap_or(0);
    }

    let pre = pre_part.and_then(|p| {
        let idents: Vec<PreIdent> = p
            .split('.')
            .filter(|s| !s.is_empty())
            .map(|s| match s.parse::<u64>() {
                Ok(n) => PreIdent::Numeric(n),
                Err(_) => PreIdent::AlphaNum(s.to_ascii_lowercase()),
            })
            .collect();

        if idents.is_empty() {
            None
        } else {
            Some(idents)
        }
    });

    ParsedVersion { core, pre }
}

#[command]
pub async fn open_download_url(url: String) -> Result<(), String> {
    opener::open(&url).map_err(|e| format!("open link failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_versions_handles_prerelease() {
        assert!(compare_versions("1.2.3-beta.1", "1.2.3"));
        assert!(!compare_versions("1.2.3", "1.2.3-beta.1"));
        assert!(compare_versions("1.2.3-beta.1", "1.2.3-beta.2"));
        assert!(!compare_versions("1.2.3-rc.2", "1.2.3-rc.1"));
    }

    #[test]
    fn compare_versions_handles_basic_semver() {
        assert!(compare_versions("1.2.3", "1.2.4"));
        assert!(!compare_versions("1.2.4", "1.2.3"));
        assert!(compare_versions("v1.9.9", "2.0.0"));
        assert!(!compare_versions("2.0.0", "2.0.0"));
    }

    #[test]
    fn parse_version_ignores_build_metadata() {
        assert_eq!(parse_version("1.2.3+abc"), parse_version("1.2.3+def"));
    }
}
