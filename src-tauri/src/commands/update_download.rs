use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use std::io::Read;
use tauri::{AppHandle, Emitter};

use crate::commands::update_types::DownloadProgress;

/// 从 URL 提取文件名
#[allow(dead_code)]
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
#[allow(dead_code)]
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

/// 下载更新文件
#[allow(dead_code)]
pub async fn download_update_file(
    app: AppHandle,
    url: String,
    expected_hash: Option<String>,
    cache_dir: PathBuf,
) -> Result<String, String> {
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;

    let file_name = file_name_from_url(&url);
    let file_path = cache_dir.join(file_name);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
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
    use futures::StreamExt;

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
            calculate_sha256(&file_path).map_err(|e| format!("Failed to calculate hash: {}", e))?;

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
