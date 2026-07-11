// License: GPL-3.0-only. Copyright (C) SeaLantern Studio.
//! Java runtime download and installation helpers shared by host flows.

mod archive;
mod shared;

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures::StreamExt;
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use archive::extract_downloaded_archive;
use shared::{
    bytes_to_mb, resolve_install_source, resolve_java_binary_path, JAVA_DOWNLOAD_RETRY_LIMIT,
    JAVA_DOWNLOAD_TEMP_FILE_NAME, JAVA_DOWNLOAD_TIMEOUT_SECS,
};

struct TempInstallDir(PathBuf);

impl TempInstallDir {
    fn new(path: PathBuf) -> Result<Self, String> {
        if path.exists() {
            fs::remove_dir_all(&path).map_err(|e| format!("无法清理临时目录：{}", e))?;
        }
        fs::create_dir_all(&path).map_err(|e| format!("无法创建临时目录：{}", e))?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempInstallDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Clone, Debug, Serialize)]
/// Progress snapshot emitted during Java runtime download and extraction.
pub struct JavaInstallProgress {
    pub state: String,
    pub progress: u64,
    pub total: u64,
    pub message: String,
}

impl JavaInstallProgress {
    fn downloading_started() -> Self {
        Self {
            state: "downloading".to_string(),
            progress: 0,
            total: 0,
            message: "开始下载...".to_string(),
        }
    }

    fn downloading(progress: u64, total: u64, message: String) -> Self {
        Self {
            state: "downloading".to_string(),
            progress,
            total,
            message,
        }
    }

    fn download_finished(progress: u64, total: u64) -> Self {
        Self {
            state: "downloading".to_string(),
            progress,
            total,
            message: "下载完成，准备解压...".to_string(),
        }
    }

    fn extracting_started() -> Self {
        Self {
            state: "extracting".to_string(),
            progress: 0,
            total: 100,
            message: "正在解压...".to_string(),
        }
    }

    fn finished() -> Self {
        Self {
            state: "finished".to_string(),
            progress: 100,
            total: 100,
            message: "安装完成".to_string(),
        }
    }
}

/// Downloads and installs a Java runtime into the managed runtime directory.
pub async fn download_and_install_java<F>(
    url: &str,
    version_name: &str,
    app_dir: &Path,
    cancel_flag: &AtomicBool,
    mut emit_progress: F,
) -> Result<PathBuf, String>
where
    F: FnMut(JavaInstallProgress),
{
    let runtimes_dir = app_dir.join("runtimes");
    if !runtimes_dir.exists() {
        fs::create_dir_all(&runtimes_dir).map_err(|e| format!("无法创建运行时目录：{}", e))?;
    }

    let target_dir = runtimes_dir.join(version_name);
    let java_bin = resolve_java_binary_path(&target_dir);
    if java_bin.exists() {
        return Ok(java_bin);
    }

    emit_progress(JavaInstallProgress::downloading_started());

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
        match client.get(url).send().await {
            Ok(response) => break response,
            Err(e) => {
                if attempt >= JAVA_DOWNLOAD_RETRY_LIMIT {
                    return Err(format!("下载请求失败（第 {} 次尝试）：{}", attempt, e));
                }
            }
        }
    };

    if !response.status().is_success() {
        return Err(format!("下载请求失败（HTTP {}）", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();
    let mut downloaded = 0u64;
    let mut last_emit = std::time::Instant::now();

    let temp_dir = TempInstallDir::new(runtimes_dir.join(format!("temp_{}", version_name)))?;

    let temp_file_path = temp_dir.path().join(JAVA_DOWNLOAD_TEMP_FILE_NAME);
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
            emit_progress(JavaInstallProgress::downloading(
                downloaded,
                total_size,
                format!("正在下载：{}/{}", bytes_to_mb(downloaded), bytes_to_mb(total_size)),
            ));
            last_emit = std::time::Instant::now();
        }
    }

    emit_progress(JavaInstallProgress::download_finished(downloaded, total_size));

    if cancel_flag.load(Ordering::Relaxed) {
        return Err("用户取消下载".to_string());
    }

    emit_progress(JavaInstallProgress::extracting_started());

    let mut magic = [0u8; 2];
    let mut magic_file =
        fs::File::open(&temp_file_path).map_err(|e| format!("无法打开临时下载文件：{}", e))?;
    let read_len = magic_file
        .read(&mut magic)
        .map_err(|e| format!("读取临时文件头失败：{}", e))?;
    drop(magic_file);

    extract_downloaded_archive(&temp_file_path, temp_dir.path(), read_len, magic, cancel_flag)?;

    if cancel_flag.load(Ordering::Relaxed) {
        return Err("用户取消安装".to_string());
    }

    let install_source = resolve_install_source(temp_dir.path())?;

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir).map_err(|e| format!("清理旧文件失败：{}", e))?;
    }

    if let Err(e) = fs::rename(&install_source, &target_dir) {
        return Err(format!("移动文件失败：{}", e));
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

    emit_progress(JavaInstallProgress::finished());

    Ok(java_bin)
}
