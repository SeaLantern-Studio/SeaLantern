//! Arch Linux AUR 更新检查模块。
//!
//! 通过 AUR RPC 接口查询 `sealantern` 包的最新版本，
//! 并提供 AUR 助手检测功能以辅助用户执行更新命令。
//!
//! # 平台可用性
//!
//! - Linux: 完整实现，通过 `/etc/os-release` 检测是否为 Arch Linux
//! - 非 Linux: 桩实现，`is_arch_linux()` 始终返回 `false`

#[cfg(target_os = "linux")]
mod imp {
    use super::super::constants::{AUR_PACKAGE_INFO_URL, AUR_PACKAGE_PAGE_URL};
    use super::super::types::UpdateInfo;
    use super::super::version::compare_versions;
    use crate::observability;

    /// 检查是否为 Arch Linux 系统
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
    pub async fn check_aur_update(current_version: &str) -> Result<UpdateInfo, String> {
        // 获取当前全局客户端，与全局代理设置保持一致。
        let client = sealantern_infra::net::global_client()
            .map_err(|error| format!("创建 AUR 更新客户端失败: {error}"))?;
        check_aur_update_with_client(client.get_reqwest_client(), current_version).await
    }

    /// 使用调用方提供的网络策略检查 AUR 更新。
    pub(crate) async fn check_aur_update_with_client(
        client: &reqwest::Client,
        current_version: &str,
    ) -> Result<UpdateInfo, String> {
        observability::update_check_started("arch-aur", current_version);

        let url = AUR_PACKAGE_INFO_URL;

        let response = client.get(url).send().await.map_err(|e| {
            let msg = format!("AUR查询失败: {}", e);
            observability::update_api_request_failed("arch-aur", "info_request", None, &msg);
            msg
        })?;

        if !response.status().is_success() {
            let msg = format!("AUR API返回错误: {}", response.status());
            observability::update_api_request_failed(
                "arch-aur",
                "info_request",
                Some(response.status().as_u16()),
                &msg,
            );
            return Err(msg);
        }

        let json: serde_json::Value = response.json().await.map_err(|e| {
            let msg = format!("解析AUR响应失败: {}", e);
            observability::update_api_request_failed("arch-aur", "parse_response", None, &msg);
            msg
        })?;

        let resultcount = json["resultcount"].as_u64().unwrap_or(0);
        if resultcount == 0 {
            let msg = "AUR中未找到sealantern包".to_string();
            observability::update_api_request_failed("arch-aur", "check_package", None, &msg);
            return Err(msg);
        }

        let aur_version = json["results"][0]["Version"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // 比较版本(忽略pkgrel部分)
        let aur_clean = aur_version.split('-').next().unwrap_or(&aur_version);
        let current_clean = current_version.split('-').next().unwrap_or(current_version);

        let has_update = compare_versions(current_clean, aur_clean);

        let aur_helper = get_aur_helper().unwrap_or_else(|| "yay".to_string());
        let update_command = format!("{} -Syu sealantern", aur_helper);

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

        observability::update_check_completed("arch-aur", has_update, Some(&aur_version));

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
}

#[cfg(not(target_os = "linux"))]
mod imp {
    /// 非 Linux 桩实现
    pub fn is_arch_linux() -> bool {
        false
    }

    pub fn get_aur_helper() -> Option<String> {
        None
    }
}

pub use imp::{get_aur_helper, is_arch_linux};

#[cfg(target_os = "linux")]
pub use imp::check_aur_update;
#[cfg(all(target_os = "linux", not(debug_assertions)))]
pub(crate) use imp::check_aur_update_with_client;
