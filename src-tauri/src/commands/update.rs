use tauri::{command, AppHandle};

#[cfg(target_os = "linux")]
use crate::commands::update_arch;
use crate::commands::{
    update_download, update_github, update_install,
    update_types::{get_github_config, PendingUpdate, UpdateInfo},
};

/// 检查更新
#[command]
pub async fn check_update() -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    println!("=== 检查更新 ===");
    println!("当前版本: {}", current_version);
    println!("目标操作系统: {}", std::env::consts::OS);

    // Arch Linux 特殊处理
    #[cfg(target_os = "linux")]
    {
        println!("Linux 条件编译通过");
        let is_arch = update_arch::is_arch_linux();
        println!("is_arch_linux() 返回: {}", is_arch);

        if is_arch {
            println!("检测到 Arch Linux，使用 AUR 更新检查");
            return update_arch::check_aur_update(current_version).await;
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        println!("不是 Linux 系统，使用 GitHub 更新检查");
    }

    println!("使用 GitHub 更新检查");
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| format!("HTTP client init failed: {}", e))?;

    let config = get_github_config();
    update_github::fetch_release(&client, &config, current_version).await
}

/// 打开下载链接
#[command]
pub async fn open_download_url(url: String) -> Result<(), String> {
    opener::open(&url).map_err(|e| format!("open link failed: {}", e))
}

/// 下载更新
#[command]
#[allow(dead_code)]
pub async fn download_update(
    app: AppHandle,
    url: String,
    expected_hash: Option<String>,
) -> Result<String, String> {
    let cache_dir = update_install::get_update_cache_dir();
    update_download::download_update_file(app, url, expected_hash, cache_dir).await
}

/// 安装更新
#[command]
#[allow(dead_code)]
pub async fn install_update(file_path: String, version: String) -> Result<(), String> {
    update_install::execute_install(file_path, version).await
}

/// 检查待更新状态
#[command]
#[allow(dead_code)]
pub async fn check_pending_update() -> Result<Option<PendingUpdate>, String> {
    update_install::check_pending_update().await
}

/// 清除待更新状态
#[command]
#[allow(dead_code)]
pub async fn clear_pending_update() -> Result<(), String> {
    update_install::clear_pending_update().await
}

/// 重启并安装
#[command]
#[allow(dead_code)]
pub async fn restart_and_install(app: AppHandle) -> Result<(), String> {
    app.restart();
}

/// 从调试 URL 下载更新
#[command]
#[allow(dead_code)]
pub async fn download_update_from_debug_url(app: AppHandle, url: String) -> Result<String, String> {
    download_update(app, url, None).await
}

#[cfg(test)]
mod tests {
    use crate::commands::update_version;

    #[test]
    fn compare_versions_handles_prerelease() {
        assert!(update_version::compare_versions("1.2.3-beta.1", "1.2.3"));
        assert!(!update_version::compare_versions("1.2.3", "1.2.3-beta.1"));
        assert!(update_version::compare_versions("1.2.3-beta.1", "1.2.3-beta.2"));
        assert!(!update_version::compare_versions("1.2.3-rc.2", "1.2.3-rc.1"));
    }

    #[test]
    fn compare_versions_handles_basic_semver() {
        assert!(update_version::compare_versions("1.2.3", "1.2.4"));
        assert!(!update_version::compare_versions("1.2.4", "1.2.3"));
        assert!(update_version::compare_versions("v1.9.9", "2.0.0"));
        assert!(!update_version::compare_versions("2.0.0", "2.0.0"));
    }

    #[test]
    fn parse_version_ignores_build_metadata() {
        assert_eq!(
            update_version::parse_version("1.2.3+abc"),
            update_version::parse_version("1.2.3+def")
        );
    }

    #[test]
    fn normalize_release_tag_version_handles_prefixed_tag() {
        assert_eq!(update_version::normalize_release_tag_version("sea-lantern-v0.5.0"), "0.5.0");
    }

    #[test]
    fn normalize_release_tag_version_handles_plain_version_tag() {
        assert_eq!(update_version::normalize_release_tag_version("v0.5.0"), "0.5.0");
    }

    #[test]
    fn normalize_release_tag_version_handles_prerelease_tag() {
        assert_eq!(
            update_version::normalize_release_tag_version("SeaLantern_release-v1.2.3-rc.1"),
            "1.2.3-rc.1"
        );
    }
}
