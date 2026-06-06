use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use futures::StreamExt;
use sha2::{Digest, Sha256};

use crate::constants::UPDATE_HTTP_USER_AGENT;
use crate::types::DownloadProgress;

pub(crate) fn file_name_from_url(url: &str) -> String {
    let candidate = url.rsplit('/').next().unwrap_or("update");
    let candidate = candidate.split('?').next().unwrap_or("update");
    let candidate = candidate.split('#').next().unwrap_or("update");
    if candidate.trim().is_empty() {
        "update".to_string()
    } else {
        candidate.to_string()
    }
}

pub(crate) fn calculate_sha256(file_path: &Path) -> Result<String, std::io::Error> {
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

fn build_hash_mismatch_error(expected_hash: &str, calculated_hash: &str) -> String {
    format!(
        "Hash verification failed. Expected: {}, Got: {}",
        expected_hash, calculated_hash
    )
}

fn remove_corrupted_download(file_path: &Path, mismatch_error: String) -> Result<(), String> {
    std::fs::remove_file(file_path).map_err(|e| {
        format!(
            "{}; failed to remove corrupted download {}: {}",
            mismatch_error,
            file_path.display(),
            e
        )
    })
}

pub async fn download_update_file<F>(
    url: String,
    expected_hash: Option<String>,
    cache_dir: PathBuf,
    mut on_progress: F,
) -> Result<String, String>
where
    F: FnMut(DownloadProgress),
{
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;

    let file_name = file_name_from_url(&url);
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

        on_progress(DownloadProgress { downloaded, total: total_size, percent });
    }

    file.flush()
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    let file_path_str = file_path.to_string_lossy().to_string();

    if let Some(hash) = expected_hash {
        let calculated_hash =
            calculate_sha256(&file_path).map_err(|e| format!("Failed to calculate hash: {}", e))?;

        if calculated_hash.to_lowercase() != hash.to_lowercase() {
            let mismatch_error = build_hash_mismatch_error(&hash, &calculated_hash);
            remove_corrupted_download(&file_path, mismatch_error.clone())?;
            return Err(mismatch_error);
        }
    }

    Ok(file_path_str)
}

#[cfg(test)]
#[path = "download_tests.rs"]
mod tests;
