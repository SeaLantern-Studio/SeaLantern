//! Shared update-checking helpers used by the desktop and headless hosts.

pub mod arch;
mod asset_selector;
mod checksum;
pub mod cnb;
pub mod constants;
pub mod download;
#[cfg(not(debug_assertions))]
mod github;
pub mod install_support;
pub mod pending;
pub mod types;
mod version;
pub mod windows_install;

use types::UpdateInfo;

#[cfg(not(debug_assertions))]
use types::github_config;

#[cfg(target_os = "linux")]
#[allow(unused_imports)]
use self::arch as update_arch;
#[allow(unused_imports)]
use self::cnb as update_cnb;
#[cfg(not(debug_assertions))]
#[allow(unused_imports)]
use self::github as update_github;

#[cfg(all(not(debug_assertions), target_os = "linux"))]
/// Chooses the best Linux update result when both CNB and GitHub checks are available.
fn select_update_result(
    cnb_result: Result<UpdateInfo, String>,
    github_result: Result<UpdateInfo, String>,
) -> Result<UpdateInfo, String> {
    match (cnb_result, github_result) {
        (_, Ok(github_info)) if github_info.has_update => Ok(github_info),
        (Ok(cnb_info), _) => Ok(cnb_info),
        (Err(_), Ok(github_info)) => Ok(github_info),
        (Err(cnb_err), Err(github_err)) => {
            Err(format!("CNB 检查失败: {}; GitHub 检查失败: {}", cnb_err, github_err))
        }
    }
}

/// Checks whether a newer application version is available for the current host.
pub async fn check_update(current_version: &str) -> Result<UpdateInfo, String> {
    #[cfg(debug_assertions)]
    {
        println!("[Update] Dev模式已禁用版本更新检测");
        Ok(UpdateInfo {
            has_update: false,
            latest_version: current_version.to_string(),
            current_version: current_version.to_string(),
            download_url: None,
            release_notes: None,
            published_at: None,
            source: None,
            sha256: None,
        })
    }

    #[cfg(not(debug_assertions))]
    {
        println!("=== 检查更新 ===");
        println!("当前版本: {}", current_version);
        println!("目标操作系统: {}", std::env::consts::OS);

        #[cfg(target_os = "linux")]
        {
            println!("Linux 条件编译通过");
            let is_arch = update_arch::is_arch_linux();
            println!("is_arch_linux() 返回: {}", is_arch);

            if is_arch {
                println!("检测到 Arch Linux，使用 AUR 更新检查");
                return update_arch::check_aur_update(current_version).await;
            }

            println!("使用 CNB + GitHub 更新检查");
            let client = reqwest::Client::builder()
                .user_agent(constants::UPDATE_HTTP_USER_AGENT)
                .build()
                .map_err(|e| format!("HTTP client init failed: {}", e))?;

            let cnb_result = update_cnb::fetch_release(&client, current_version).await;

            let config = github_config();
            let github_result =
                update_github::fetch_release(&client, &config, current_version).await;

            return select_update_result(cnb_result, github_result);
        }

        #[cfg(not(target_os = "linux"))]
        {
            println!("不是 Linux 系统，使用 GitHub 更新检查");
            let client = reqwest::Client::builder()
                .user_agent(constants::UPDATE_HTTP_USER_AGENT)
                .build()
                .map_err(|e| format!("HTTP client init failed: {}", e))?;

            let config = github_config();
            update_github::fetch_release(&client, &config, current_version).await
        }
    }
}
