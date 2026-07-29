//! 更新模块的公共类型定义。
//!
//! 包含更新信息、下载进度、发布响应、资源文件等数据结构的定义，
//! 以及 GitHub 仓库配置的获取函数。

use serde::{Deserialize, Serialize};

use super::constants::{UPDATE_GITHUB_API_BASE, UPDATE_GITHUB_OWNER, UPDATE_GITHUB_REPO};

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
        owner: UPDATE_GITHUB_OWNER,
        repo: UPDATE_GITHUB_REPO,
        api_base: UPDATE_GITHUB_API_BASE,
    }
}
