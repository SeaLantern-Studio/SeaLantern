//! Java 下载与安装

mod archive;
mod progress;
mod shared;

use std::fs;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Window;
use tokio::io::AsyncWriteExt;

use crate::hardcode_data::app_files::JAVA_DOWNLOAD_TEMP_FILE_NAME;
use crate::utils::constants::{JAVA_DOWNLOAD_RETRY_LIMIT, JAVA_DOWNLOAD_TIMEOUT_SECS};

use archive::extract_downloaded_archive;
use progress::{emit_download_finished, emit_downloading_started, emit_extracting_started, emit_finished, emit_progress};
use shared::{bytes_to_mb, resolve_install_source, resolve_java_binary_path};

/// 下载并安装 Java 运行时
pub async fn download_and_install_java<R: tauri::Runtime>(
    url: String,
    version_name: String,
    window: Window<R>,
    cancel_flag: Arc<AtomicBool>,
) -> Result<String, String> {
    let app_dir = crate::utils::path::get_app_data_dir();
    let runtimes_dir = app_dir.join("runtimes");
    if !runtimes_dir.exists() {
        fs::create_dir_all(&runtimes_dir).map_err(|e| format!("无法创建运行时目录：{}", e))?;
    }

    let target_dir = runtimes_dir.join(&version_name);
    let java_bin = resolve_java_binary_path(&target_dir);
    if java_bin.exists() {
        return Ok(java_bin.to_string_lossy().to_string());
    }

    emit_downloading_started(&window);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(JAVA_DOWNLOAD_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("创建下载客户端失败：{}", e))?;

    let mut attempt: usize = 0;
    let response = loop {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("用户取消下载".to_string());
        }

        attempt += 1;
        match client.get(&url).send().await {
            Ok(response) => break response,
            Err(e) => {
                if attempt >= JAVA_DOWNLOAD_RETRY_LIMIT {
                    return Err(format!("下载请求失败（第 {} 次尝试）：{}", attempt, e));
                }
            }
        }
    };

    let total_size = response.content_length().unwrap_or(0);
    use futures::StreamExt;
    let mut stream = response.bytes_stream();
    let mut downloaded = 0u64;
    let mut last_emit = std::time::Instant::now();

    let temp_dir = runtimes_dir.join(format!("temp_{}", version_name));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).map_err(|e| format!("无法清理临时目录：{}", e))?;
    }
    fs::create_dir_all(&temp_dir).map_err(|e| format!("无法创建临时目录：{}", e))?;

    let temp_file_path = temp_dir.join(JAVA_DOWNLOAD_TEMP_FILE_NAME);
    let mut temp_file = tokio::fs::File::create(&temp_file_path)
        .await
        .map_err(|e| format!("无法创建临时下载文件：{}", e))?;

    while let Some(chunk) = stream.next().await {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("用户取消下载".to_string());
        }
        let chunk = chunk.map_err(|e| format!("下载流错误：{}", e))?;

        temp_file
            .write_all(&chunk)
            .await
            .map_err(|e| format!("写入临时文件失败：{}", e))?;

        downloaded += chunk.len() as u64;
        if total_size > 0 && last_emit.elapsed().as_millis() > 100 {
            emit_progress(
                &window,
                downloaded,
                total_size,
                format!(
                    "正在下载：{}/{}",
                    bytes_to_mb(downloaded),
                    bytes_to_mb(total_size)
                ),
            );
            last_emit = std::time::Instant::now();
        }
    }

    emit_download_finished(&window, downloaded, total_size);

    if cancel_flag.load(Ordering::Relaxed) {
        return Err("用户取消下载".to_string());
    }

    emit_extracting_started(&window);

    let mut magic = [0u8; 2];
    let mut magic_file =
        fs::File::open(&temp_file_path).map_err(|e| format!("无法打开临时下载文件：{}", e))?;
    let read_len = magic_file
        .read(&mut magic)
        .map_err(|e| format!("读取临时文件头失败：{}", e))?;
    drop(magic_file);

    extract_downloaded_archive(&temp_file_path, &temp_dir, read_len, magic, &cancel_flag)?;

    if cancel_flag.load(Ordering::Relaxed) {
        let _ = fs::remove_dir_all(&temp_dir);
        return Err("用户取消安装".to_string());
    }

    let install_source = resolve_install_source(&temp_dir);

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir).map_err(|e| format!("清理旧文件失败：{}", e))?;
    }

    if let Err(e) = fs::rename(&install_source, &target_dir) {
        let _ = fs::remove_dir_all(&temp_dir);
        return Err(format!("移动文件失败：{}", e));
    }

    if install_source != temp_dir {
        let _ = fs::remove_dir_all(&temp_dir);
    }

    let java_bin = resolve_java_binary_path(&target_dir);
    if !java_bin.exists() {
        return Err(format!("安装失败：未找到可执行文件 {:?}", java_bin));
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(&java_bin) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&java_bin, perms);
        }
    }

    emit_finished(&window);

    Ok(java_bin.to_string_lossy().to_string())
}
