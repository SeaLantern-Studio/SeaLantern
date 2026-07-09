#[cfg(not(debug_assertions))]
use crate::constants::{UPDATE_GITHUB_API_BASE, UPDATE_GITHUB_OWNER, UPDATE_GITHUB_REPO};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Normalized update payload returned to the host application.
pub struct UpdateInfo {
    pub has_update: bool,
    pub latest_version: String,
    pub current_version: String,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
    pub source: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
/// Progress snapshot emitted while a release asset is downloading.
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
/// Persisted marker for an update that was downloaded but not yet installed.
pub struct PendingUpdate {
    pub file_path: String,
    pub version: String,
}

#[cfg_attr(debug_assertions, allow(dead_code))]
#[derive(Debug, Deserialize)]
/// Minimal GitHub release payload used by the production update client.
pub(crate) struct ReleaseResponse {
    pub(crate) tag_name: String,
    pub(crate) body: Option<String>,
    pub(crate) assets: Vec<ReleaseAsset>,
    pub(crate) published_at: Option<String>,
    pub(crate) created_at: Option<String>,
}

#[cfg_attr(debug_assertions, allow(dead_code))]
#[derive(Debug, Deserialize)]
/// Minimal GitHub release asset payload used by the production update client.
pub(crate) struct ReleaseAsset {
    pub(crate) name: String,
    pub(crate) browser_download_url: String,
}

#[cfg_attr(debug_assertions, allow(dead_code))]
/// Repository coordinates for the production GitHub release API.
pub(crate) struct RepoConfig {
    pub(crate) owner: &'static str,
    pub(crate) repo: &'static str,
    pub(crate) api_base: &'static str,
}

#[cfg_attr(debug_assertions, allow(dead_code))]
impl RepoConfig {
    /// Builds the latest-release endpoint for the configured repository.
    pub(crate) fn api_url(&self) -> String {
        format!("{}/{}/{}/releases/latest", self.api_base, self.owner, self.repo)
    }
}

#[cfg(not(debug_assertions))]
/// Returns the repository configuration used by production update checks.
pub(crate) fn github_config() -> RepoConfig {
    RepoConfig {
        owner: UPDATE_GITHUB_OWNER,
        repo: UPDATE_GITHUB_REPO,
        api_base: UPDATE_GITHUB_API_BASE,
    }
}
