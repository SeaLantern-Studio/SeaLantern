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
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GiteeRelease {
    tag_name: String,
    name: String,
    body: Option<String>,
    assets: Vec<GiteeAsset>,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct GiteeAsset {
    name: String,
    browser_download_url: String,
}

/// 检查Gitee上的更新
#[command]
pub async fn check_update(
    owner: String,
    repo: String,
) -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    // 构建Gitee API URL
    let url = format!(
        "https://gitee.com/api/v5/repos/{}/{}/releases/latest",
        owner, repo
    );

    // 发送HTTP请求
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Sea-Lantern")
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API返回错误: {}", response.status()));
    }

    let release: GiteeRelease = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    // 解析版本号（去掉v前缀）
    let latest_version = release.tag_name.trim_start_matches('v');

    // 比较版本号
    let has_update = compare_versions(current_version, latest_version);

    // 查找Windows安装包
    let download_url = release.assets.iter()
        .find(|asset| {
            let name = asset.name.to_lowercase();
            name.ends_with(".exe") || name.ends_with(".msi")
        })
        .map(|asset| asset.browser_download_url.clone())
        .or_else(|| {
            // 如果没有找到安装包，返回Release页面链接
            Some(format!(
                "https://gitee.com/{}/{}/releases/tag/{}",
                owner, repo, release.tag_name
            ))
        });

    Ok(UpdateInfo {
        has_update,
        latest_version: latest_version.to_string(),
        current_version: current_version.to_string(),
        download_url,
        release_notes: release.body,
        published_at: Some(release.created_at),
    })
}

/// 简单的版本号比较（支持 x.y.z 格式）
fn compare_versions(current: &str, latest: &str) -> bool {
    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };

    let current_parts = parse_version(current);
    let latest_parts = parse_version(latest);

    for i in 0..current_parts.len().max(latest_parts.len()) {
        let c = current_parts.get(i).unwrap_or(&0);
        let l = latest_parts.get(i).unwrap_or(&0);

        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }

    false
}

/// 打开下载链接
#[command]
pub async fn open_download_url(url: String) -> Result<(), String> {
    // 注意：这个命令实际上不需要了，因为前端可以直接使用 @tauri-apps/plugin-opener
    // 但为了保持API的完整性，我们保留它
    opener::open(&url).map_err(|e| format!("打开链接失败: {}", e))
}
