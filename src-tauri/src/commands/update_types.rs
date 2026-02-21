use serde::{Deserialize, Serialize};

/// 更新信息结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
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

/// 下载进度结构体
#[derive(Debug, Serialize, Clone)]
#[allow(dead_code)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percent: f64,
}

/// 待更新状态结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct PendingUpdate {
    pub file_path: String,
    pub version: String,
}

/// 发布响应结构体
#[derive(Debug, Deserialize)]
pub struct ReleaseResponse {
    pub tag_name: String,
    pub body: Option<String>,
    pub assets: Vec<ReleaseAsset>,
    pub published_at: Option<String>,
    pub created_at: Option<String>,
}

/// 发布资源结构体
#[derive(Debug, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

/// 仓库配置结构体
pub struct RepoConfig {
    pub owner: &'static str,
    pub repo: &'static str,
    pub api_base: &'static str,
}

impl RepoConfig {
    pub fn api_url(&self) -> String {
        format!("{}/{}/{}/releases/latest", self.api_base, self.owner, self.repo)
    }
}

/// 获取 GitHub 仓库配置
pub fn get_github_config() -> RepoConfig {
    RepoConfig {
        owner: "FPSZ",
        repo: "SeaLantern",
        api_base: "https://api.github.com/repos",
    }
}
