//! 更新文件下载模块。
//!
//! 提供更新文件的 HTTP 下载、SHA256 哈希计算与校验功能。
//! 主要入口为 [`download_update_file_without_events`]。
//!
//! # 错误处理
//!
//! 所有失败路径均返回 `Err(String)`，并同步通过 `observability` 模块记录结构化日志。

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use std::io::Read;

use super::constants::UPDATE_HTTP_USER_AGENT;
use super::types::DownloadProgress;
use crate::observability;

/// 从 URL 提取文件名
pub fn file_name_from_url(url: &str) -> String {
    let candidate = url.rsplit('/').next().unwrap_or("update");
    let candidate = candidate.split('?').next().unwrap_or("update");
    let candidate = candidate.split('#').next().unwrap_or("update");
    if candidate.trim().is_empty() {
        "update".to_string()
    } else {
        candidate.to_string()
    }
}

/// 计算文件的 SHA256 哈希值
pub fn calculate_sha256(file_path: &PathBuf) -> Result<String, std::io::Error> {
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

/// 下载更新文件 (不包含 Tauri 事件发送)
pub async fn download_update_file_without_events(
    url: String,
    expected_hash: Option<String>,
    cache_dir: PathBuf,
) -> Result<String, String> {
    observability::update_download_started(&url);

    std::fs::create_dir_all(&cache_dir).map_err(|e| {
        let msg = format!("Failed to create cache directory: {}", e);
        observability::update_download_failed(&url, &msg);
        msg
    })?;

    let file_name = file_name_from_url(&url);
    let file_path = cache_dir.join(file_name);

    let client = reqwest::Client::builder()
        .user_agent(UPDATE_HTTP_USER_AGENT)
        .build()
        .map_err(|e| {
            let msg = format!("HTTP client init failed: {}", e);
            observability::update_download_failed(&url, &msg);
            msg
        })?;

    let response = client.get(&url).send().await.map_err(|e| {
        let msg = format!("Download request failed: {}", e);
        observability::update_download_failed(&url, &msg);
        msg
    })?;

    if !response.status().is_success() {
        let msg = format!("Download failed with status: {}", response.status());
        observability::update_download_failed(&url, &msg);
        return Err(msg);
    }

    let mut file = File::create(&file_path).map_err(|e| {
        let msg = format!("Failed to create file: {}", e);
        observability::update_download_failed(&url, &msg);
        msg
    })?;

    let bytes = response.bytes().await.map_err(|e| {
        let msg = format!("Failed to read response: {}", e);
        observability::update_download_failed(&url, &msg);
        msg
    })?;

    file.write_all(&bytes).map_err(|e| {
        let msg = format!("Failed to write file: {}", e);
        observability::update_download_failed(&url, &msg);
        msg
    })?;

    file.flush().map_err(|e| {
        let msg = format!("Failed to flush file: {}", e);
        observability::update_download_failed(&url, &msg);
        msg
    })?;

    let file_path_str = file_path.to_string_lossy().to_string();

    // 验证哈希值
    if let Some(hash) = expected_hash {
        let calculated_hash =
            calculate_sha256(&file_path).map_err(|e| format!("Failed to calculate hash: {}", e))?;

        if calculated_hash.to_lowercase() != hash.to_lowercase() {
            std::fs::remove_file(&file_path).ok();
            let msg =
                format!("Hash verification failed. Expected: {}, Got: {}", hash, calculated_hash);
            observability::update_hash_mismatch(&file_path_str, &hash, &calculated_hash);
            return Err(msg);
        }
        observability::update_hash_verified(&file_path_str);
    }

    observability::update_download_completed(&file_path_str);
    Ok(file_path_str)
}

/// 计算下载进度
pub fn calculate_progress(downloaded: u64, total: u64) -> DownloadProgress {
    let percent = if total > 0 {
        (downloaded as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    DownloadProgress { downloaded, total, percent }
}
