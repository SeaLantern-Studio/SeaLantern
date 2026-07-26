//! 应用更新相关的命令。

use tauri::{command, AppHandle, Emitter};
use tracing::{debug, info};

use sealantern_extra::update::{UpdateInfo, PendingUpdate, get_github_config, UPDATE_HTTP_USER_AGENT, is_arch_linux, fetch_cnb_release, fetch_github_release, get_update_cache_dir, resolve_download_candidate_by_version, check_pending_update, clear_pending_update, write_pending_update, get_pending_update_file, INSTALL_IN_PROGRESS, DownloadProgress};

use futures::StreamExt;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::Ordering;

#[cfg(target_os = "linux")]
use sealantern_extra::update::check_aur_update;

#[cfg(target_os = "windows")]
use sealantern_extra::update::spawn_elevated_windows_process;

#[cfg(all(not(debug_assertions), target_os = "linux"))]
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

/// 检查更新
#[command]
pub async fn check_update() -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    #[cfg(debug_assertions)]
    {
        info!("[Update] Dev模式已禁用版本更新检测");
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
        debug!("=== 检查更新 ===");
        debug!(current_version = %current_version, target_os = %std::env::consts::OS; "Checking for updates");

        #[cfg(target_os = "linux")]
        {
            debug!("Linux 条件编译通过");
            let is_arch = is_arch_linux();
            debug!(is_arch; "Arch Linux detection result");

            if is_arch {
                info!("检测到 Arch Linux，使用 AUR 更新检查");
                return check_aur_update(current_version).await;
            }

            // Linux 非 Arch 系统使用 CNB + GitHub 更新检查
            info!("使用 CNB + GitHub 更新检查");
            let client = reqwest::Client::builder()
                .user_agent(UPDATE_HTTP_USER_AGENT)
                .build()
                .map_err(|e| format!("HTTP client init failed: {}", e))?;

            let cnb_result = fetch_cnb_release(&client, current_version).await;

            let config = get_github_config();
            let github_result =
                fetch_github_release(&client, &config, current_version).await;

            return select_update_result(cnb_result, github_result);
        }

        #[cfg(not(target_os = "linux"))]
        {
            info!("使用 GitHub 更新检查");
            let client = reqwest::Client::builder()
                .user_agent(UPDATE_HTTP_USER_AGENT)
                .build()
                .map_err(|e| format!("HTTP client init failed: {}", e))?;

            let config = get_github_config();
            fetch_github_release(&client, &config, current_version).await
        }
    }
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
    version: Option<String>,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent(UPDATE_HTTP_USER_AGENT)
        .build()
        .map_err(|e| format!("HTTP client init failed: {}", e))?;

    let cache_dir = get_update_cache_dir();
    let mut candidates: Vec<(String, Option<String>, &'static str)> = Vec::new();

    if let Some(v) = version.as_deref() {
        if let Ok(Some((cnb_url, cnb_hash))) =
            resolve_download_candidate_by_version(&client, v).await
        {
            candidates.push((cnb_url, cnb_hash, "CNB"));
        }
    }

    candidates.push((url, expected_hash, "GitHub"));

    let mut deduped: Vec<(String, Option<String>, &'static str)> = Vec::new();
    for (candidate_url, candidate_hash, source_name) in candidates {
        if deduped
            .iter()
            .any(|(seen_url, _, _)| seen_url == &candidate_url)
        {
            continue;
        }
        deduped.push((candidate_url, candidate_hash, source_name));
    }

    let mut errors: Vec<String> = Vec::new();
    for (candidate_url, candidate_hash, source_name) in deduped {
        match download_update_file_with_progress(
            app.clone(),
            candidate_url,
            candidate_hash,
            cache_dir.clone(),
        )
        .await
        {
            Ok(path) => return Ok(path),
            Err(error) => errors.push(format!("{} 下载失败: {}", source_name, error)),
        }
    }

    Err(errors.join("; "))
}

/// 带进度的下载更新文件
async fn download_update_file_with_progress(
    app: AppHandle,
    url: String,
    expected_hash: Option<String>,
    cache_dir: PathBuf,
) -> Result<String, String> {
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;

    let file_name = url.rsplit('/').next().unwrap_or("update");
    let file_name = file_name.split('?').next().unwrap_or("update");
    let file_name = file_name.split('#').next().unwrap_or("update");
    let file_name = if file_name.trim().is_empty() {
        "update"
    } else {
        file_name
    };
    let file_path = cache_dir.join(file_name);

    let client = reqwest::Client::builder()
        .user_agent(UPDATE_HTTP_USER_AGENT)
        .build()
        .map_err(|e| format!("HTTP client init failed: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded = 0_u64;

    let mut file = File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Failed to read chunk: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write chunk: {}", e))?;

        downloaded += chunk.len() as u64;
        let percent = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        let _ = app.emit(
            "update-download-progress",
            DownloadProgress { downloaded, total: total_size, percent },
        );
    }

    file.flush()
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    let file_path_str = file_path.to_string_lossy().to_string();

    // 验证哈希值
    if let Some(hash) = expected_hash {
        let calculated_hash =
            calculate_file_sha256(&file_path).map_err(|e| format!("Failed to calculate hash: {}", e))?;

        if calculated_hash.to_lowercase() != hash.to_lowercase() {
            std::fs::remove_file(&file_path).ok();
            return Err(format!(
                "Hash verification failed. Expected: {}, Got: {}",
                hash, calculated_hash
            ));
        }
    }

    Ok(file_path_str)
}

/// 计算文件 SHA256
fn calculate_file_sha256(file_path: &PathBuf) -> Result<String, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// 安装更新
#[command]
#[allow(dead_code)]
pub async fn install_update(file_path: String, version: String) -> Result<(), String> {
    if INSTALL_IN_PROGRESS.swap(true, Ordering::SeqCst) {
        return Err("Install is already in progress".to_string());
    }

    let result = (|| -> Result<(), String> {
        let path = PathBuf::from(&file_path);
        if !path.exists() {
            return Err(format!("Update file not found: {}", file_path));
        }

        #[cfg(target_os = "linux")]
        {
            if is_arch_linux() {
                use sealantern_extra::update::get_aur_helper;
                let helper = get_aur_helper().unwrap_or_else(|| "yay".to_string());
                return Err(format!(
                    "您使用的是 Arch Linux\n\
                     请使用包管理器更新 SeaLantern：\n\
                     {} -S sealantern\n\
                     \n\
                     或使用其他 AUR 助手",
                    helper
                ));
            }
        }

        let settings = crate::services::global::settings_manager().get();
        if settings.close_servers_on_update {
            crate::services::global::server_manager().stop_all_servers();
        }

        let pending_file = get_pending_update_file();
        write_pending_update(&pending_file, &file_path, version)?;
        launch_update_installer(&path, &file_path, &pending_file)?;

        Ok(())
    })();

    if result.is_err() {
        INSTALL_IN_PROGRESS.store(false, Ordering::SeqCst);
        std::fs::remove_file(get_pending_update_file()).ok();
    }

    result
}

/// 启动更新安装器
fn launch_update_installer(
    path: &std::path::Path,
    file_path: &str,
    pending_file: &std::path::Path,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let pending_file_path = pending_file.to_string_lossy().to_string();
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_lowercase());

        match extension.as_deref() {
            Some("msi") => spawn_elevated_windows_process(
                "msiexec.exe",
                &["/i", file_path, "/passive", "/norestart"],
                Some(file_path),
                Some(pending_file_path.as_str()),
            ),
            Some("exe") => spawn_elevated_windows_process(
                file_path,
                &["/S", "/norestart"],
                Some(file_path),
                Some(pending_file_path.as_str()),
            ),
            _ => opener::open(file_path).map_err(|e| format!("Failed to open update file: {}", e)),
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = pending_file;
        let _ = path;
        opener::open(file_path).map_err(|e| format!("Failed to open update file: {}", e))
    }

    #[cfg(all(target_os = "linux", not(target_os = "windows")))]
    {
        let _ = pending_file;
        let _ = path;
        opener::open(file_path).map_err(|e| format!("Failed to open update file: {}", e))
    }
}

/// 检查待更新状态
#[command]
#[allow(dead_code)]
pub async fn check_pending_update_cmd() -> Result<Option<PendingUpdate>, String> {
    check_pending_update().await
}

/// 清除待更新状态
#[command]
#[allow(dead_code)]
pub async fn clear_pending_update_cmd() -> Result<(), String> {
    clear_pending_update().await
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
    download_update(app, url, None, None).await
}

#[cfg(test)]
mod tests {
    use sealantern_extra::update::{compare_versions, normalize_release_tag_version};

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
        use sealantern_extra::update::parse_version;
        assert_eq!(
            parse_version("1.2.3+abc"),
            parse_version("1.2.3+def")
        );
    }

    #[test]
    fn normalize_release_tag_version_handles_prefixed_tag() {
        assert_eq!(normalize_release_tag_version("sea-lantern-v0.5.0"), "0.5.0");
    }

    #[test]
    fn normalize_release_tag_version_handles_plain_version_tag() {
        assert_eq!(normalize_release_tag_version("v0.5.0"), "0.5.0");
    }

    #[test]
    fn normalize_release_tag_version_handles_prerelease_tag() {
        assert_eq!(
            normalize_release_tag_version("SeaLantern_release-v1.2.3-rc.1"),
            "1.2.3-rc.1"
        );
    }
}