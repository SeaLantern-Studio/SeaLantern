#[cfg(all(target_os = "linux", not(debug_assertions)))]
use crate::constants::{AUR_PACKAGE_INFO_URL, AUR_PACKAGE_PAGE_URL, PLUGIN_MARKET_HTTP_USER_AGENT};
#[cfg(all(target_os = "linux", not(debug_assertions)))]
use crate::types::UpdateInfo;
#[cfg(all(target_os = "linux", not(debug_assertions)))]
use crate::version::compare_versions;

#[cfg(target_os = "linux")]
pub fn is_arch_linux() -> bool {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        content.contains("ID=arch")
            || content.contains("ID_LIKE=arch")
            || content.contains("ID=archlinux")
    } else {
        false
    }
}

#[cfg(target_os = "linux")]
pub fn get_aur_helper() -> Option<String> {
    get_aur_helper_with(|helper| {
        std::process::Command::new("which")
            .arg(helper)
            .output()
            .ok()
            .map(|output| output.status.success())
    })
}

#[cfg(target_os = "linux")]
fn get_aur_helper_with<F>(mut probe: F) -> Option<String>
where
    F: FnMut(&str) -> Option<bool>,
{
    let helpers = ["yay", "paru", "pamac", "trizen", "pacaur"];

    for helper in helpers {
        if probe(helper).unwrap_or(false) {
            return Some(helper.to_string());
        }
    }

    None
}

#[cfg(all(target_os = "linux", not(debug_assertions)))]
pub(crate) async fn check_aur_update(current_version: &str) -> Result<UpdateInfo, String> {
    let client = reqwest::Client::new();
    let url = AUR_PACKAGE_INFO_URL;

    let response = client
        .get(url)
        .header("User-Agent", PLUGIN_MARKET_HTTP_USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("AUR查询失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("AUR API返回错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析AUR响应失败: {}", e))?;

    let resultcount = json["resultcount"].as_u64().unwrap_or(0);
    if resultcount == 0 {
        return Err("AUR中未找到sealantern包".to_string());
    }

    let aur_version = json["results"][0]["Version"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let aur_clean = aur_version.split('-').next().unwrap_or(&aur_version);
    let current_clean = current_version.split('-').next().unwrap_or(current_version);

    let has_update = compare_versions(current_clean, aur_clean);

    let aur_helper = get_aur_helper().unwrap_or_else(|| "yay".to_string());
    let update_command = format!("{} -Syu sealantern", aur_helper);

    let release_notes = if has_update {
        format!(
            "AUR 有可用更新\n\n当前版本: {}\n最新版本: {}\n\n使用以下命令更新:\n{}\n\n或使用其他 AUR 助手",
            current_version, aur_version, update_command
        )
    } else {
        format!("已是最新版本 (Arch Linux)\n当前版本: {}", current_version)
    };

    println!("=== AUR 检查结果 ===");
    println!("has_update: {}", has_update);
    println!("source: arch-aur");
    println!("latest_version: {}", aur_version);

    Ok(UpdateInfo {
        has_update,
        latest_version: aur_version.clone(),
        current_version: current_version.to_string(),
        download_url: Some(AUR_PACKAGE_PAGE_URL.to_string()),
        release_notes: Some(release_notes),
        published_at: None,
        source: Some("arch-aur".to_string()),
        sha256: None,
    })
}

#[cfg(not(target_os = "linux"))]
pub fn is_arch_linux() -> bool {
    false
}

#[cfg(not(target_os = "linux"))]
pub fn get_aur_helper() -> Option<String> {
    None
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::get_aur_helper_with;

    #[test]
    fn aur_helper_probe_skips_failed_command_and_continues() {
        let mut calls = Vec::new();

        let helper = get_aur_helper_with(|name| {
            calls.push(name.to_string());
            match name {
                "yay" => None,
                "paru" => Some(true),
                _ => Some(false),
            }
        });

        assert_eq!(helper.as_deref(), Some("paru"));
        assert_eq!(calls, vec!["yay".to_string(), "paru".to_string()]);
    }

    #[test]
    fn aur_helper_probe_returns_none_when_all_helpers_unavailable() {
        let helper = get_aur_helper_with(|_| Some(false));

        assert_eq!(helper, None);
    }
}
