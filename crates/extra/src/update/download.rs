//! 更新文件下载模块。
//!
//! 提供更新文件的 HTTP 下载以及下载后的 SHA256 校验功能。
//! 主要入口为 [`download_update_file_without_events`]。
//! 哈希计算委托给 `sealantern_infra::fs::hash`。
//!
//! # 错误处理
//!
//! 所有失败路径均返回 `Err(String)`，并同步通过 `observability` 模块记录结构化日志。

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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

    // 获取当前全局客户端，与全局代理设置保持一致。
    let client = sealantern_infra::net::global_client().map_err(|e| {
        let msg = format!("HTTP client init failed: {}", e);
        observability::update_download_failed(&url, &msg);
        msg
    })?;

    let response = client
        .get(&url)
        .map_err(|e| {
            let msg = format!("HTTP client init failed: {}", e);
            observability::update_download_failed(&url, &msg);
            msg
        })?
        .send()
        .await
        .map_err(|e| {
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
        let calculated_hash = sealantern_infra::fs::sha256_file(&file_path)
            .await
            .map_err(|e| format!("Failed to calculate hash: {}", e))?;
        let calculated_hex = sealantern_infra::fs::sha256_hex(calculated_hash);

        if calculated_hex.to_lowercase() != hash.to_lowercase() {
            std::fs::remove_file(&file_path).ok();
            let msg =
                format!("Hash verification failed. Expected: {}, Got: {}", hash, calculated_hex);
            observability::update_hash_mismatch(&file_path_str, &hash, &calculated_hex);
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
