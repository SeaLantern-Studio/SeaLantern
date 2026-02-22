use crate::commands::update_types::UpdateInfo;
#[cfg(target_os = "linux")]
use crate::commands::update_version::compare_versions;

/// 检查是否为 Arch Linux 系统
#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub fn is_arch_linux() -> bool {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        content.contains("ID=arch")
            || content.contains("ID_LIKE=arch")
            || content.contains("ID=archlinux")
    } else {
        false
    }
}

/// 获取可用的 AUR 助手
#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub fn get_aur_helper() -> Option<String> {
    let helpers = ["yay", "paru", "pamac", "trizen", "pacaur"];

    for helper in helpers {
        let output = std::process::Command::new("which")
            .arg(helper)
            .output()
            .ok()?;

        if output.status.success() {
            return Some(helper.to_string());
        }
    }

    None
}

/// 检查 AUR 更新
#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub async fn check_aur_update(current_version: &str) -> Result<UpdateInfo, String> {
    let client = reqwest::Client::new();
    let url = "https://aur.archlinux.org/rpc/v5/info/sealantern";

    let response = client
        .get(url)
        .header("User-Agent", "SeaLantern")
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

    // 比较版本（忽略pkgrel部分）
    let aur_clean = aur_version.split('-').next().unwrap_or(&aur_version);
    let current_clean = current_version.split('-').next().unwrap_or(current_version);

    let has_update = compare_versions(current_clean, aur_clean);

    let aur_helper = get_aur_helper().unwrap_or_else(|| "yay".to_string());
    let update_command = format!("{} -S sealantern", aur_helper);

    // 构建 release_notes 文本
    let release_notes = if has_update {
        format!(
            "AUR 有可用更新\n\n\
             当前版本: {}\n\
             最新版本: {}\n\n\
             使用以下命令更新:\n\
             {}\n\n\
             或使用其他 AUR 助手",
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
        download_url: Some("https://aur.archlinux.org/packages/sealantern".to_string()),
        release_notes: Some(release_notes),
        published_at: None,
        source: Some("arch-aur".to_string()),
        sha256: None,
    })
}

/// 非 Linux 系统的占位实现
#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
pub fn is_arch_linux() -> bool {
    false
}

/// 非 Linux 系统的占位实现
#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
pub fn get_aur_helper() -> Option<String> {
    None
}

/// 非 Linux 系统的占位实现
#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
pub async fn check_aur_update(_current_version: &str) -> Result<UpdateInfo, String> {
    Err("AUR update check is only available on Linux".to_string())
}
