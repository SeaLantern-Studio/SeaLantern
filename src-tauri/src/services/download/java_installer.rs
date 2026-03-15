#[cfg(not(target_os = "windows"))]
use flate2::read::GzDecoder;
use std::fs;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
#[cfg(not(target_os = "windows"))]
use tar::Archive;
use tauri::{Emitter, Window};
use tokio::io::AsyncWriteExt;
#[cfg(target_os = "windows")]
use zip::ZipArchive;

///此处常量见 utils/constants.rs
use crate::utils::constants::{JAVA_DOWNLOAD_RETRY_LIMIT, JAVA_DOWNLOAD_TIMEOUT_SECS};

#[derive(Clone, serde::Serialize)]
struct DownloadProgress {
    state: String,
    progress: u64,
    total: u64,
    message: String,
}

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
    if target_dir.exists() {
        // Simple check: if java executable exists, assume installed
        let java_bin = if cfg!(target_os = "windows") {
            target_dir.join("bin").join("java.exe")
        } else {
            target_dir.join("bin").join("java")
        };
        if java_bin.exists() {
            return Ok(java_bin.to_string_lossy().to_string());
        }
    }

    // 1. Download
    let _ = window.emit(
        "java-install-progress",
        DownloadProgress {
            state: "downloading".to_string(),
            progress: 0,
            total: 0,
            message: "开始下载...".to_string(),
        },
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(JAVA_DOWNLOAD_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("创建下载客户端失败：{}", e))?;

    let mut attempt: usize = 0;

    let res = loop {
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

    let total_size = res.content_length().unwrap_or(0);
    // Use bytes_stream to process chunks
    use futures::StreamExt;
    let mut stream = res.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_emit = std::time::Instant::now();

    // 预先准备临时目录和临时文件，边下边写入磁盘，避免大文件全部驻留内存
    // 如果出现 error decoding response body 错误, 那不用看这里, 是reqwest的问题, 不是我们的问题
    let temp_dir = runtimes_dir.join(format!("temp_{}", version_name));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).map_err(|e| format!("无法清理临时目录：{}", e))?;
    }
    fs::create_dir_all(&temp_dir).map_err(|e| format!("无法创建临时目录：{}", e))?;

    let temp_file_path = temp_dir.join("java_download.tmp");
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
            let _ = window.emit(
                "java-install-progress",
                DownloadProgress {
                    state: "downloading".to_string(),
                    progress: downloaded,
                    total: total_size,
                    message: format!(
                        "正在下载：{}/{}",
                        bytes_to_mb(downloaded),
                        bytes_to_mb(total_size)
                    ),
                },
            );
            last_emit = std::time::Instant::now();
        }
    }

    // Final progress emit
    let _ = window.emit(
        "java-install-progress",
        DownloadProgress {
            state: "downloading".to_string(),
            progress: downloaded,
            total: total_size,
            message: "下载完成，准备解压...".to_string(),
        },
    );

    if cancel_flag.load(Ordering::Relaxed) {
        return Err("用户取消下载".to_string());
    }

    // 2. Extract
    let _ = window.emit(
        "java-install-progress",
        DownloadProgress {
            state: "extracting".to_string(),
            progress: 0,
            total: 100,
            message: "正在解压...".to_string(),
        },
    );

    // Check file signature (Magic Numbers)，根据文件头判断格式
    let mut magic = [0u8; 2];
    let mut magic_file =
        fs::File::open(&temp_file_path).map_err(|e| format!("无法打开临时下载文件：{}", e))?;
    let read_len = magic_file
        .read(&mut magic)
        .map_err(|e| format!("读取临时文件头失败：{}", e))?;
    drop(magic_file);

    #[cfg(target_os = "windows")]
    {
        if read_len >= 2 && &magic == b"PK" {
            extract_zip(&temp_file_path, &temp_dir, &cancel_flag)?;
        } else {
            return Err("下载的文件不是有效的 ZIP 格式".to_string());
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Gzip header is usually 1F 8B
        if read_len >= 2 && magic[0] == 0x1f && magic[1] == 0x8b {
            extract_tar_gz(&temp_file_path, &temp_dir, &cancel_flag)?;
        } else {
            return Err("下载的文件不是有效的 tar.gz 格式".to_string());
        }
    }

    if cancel_flag.load(Ordering::Relaxed) {
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
        return Err("用户取消安装".to_string());
    }

    // 3. Move Logic
    // Find the single inner directory if it exists
    let install_source = if let Ok(entries) = fs::read_dir(&temp_dir) {
        let entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        if entries.len() == 1 && entries[0].path().is_dir() {
            entries[0].path()
        } else {
            temp_dir.clone()
        }
    } else {
        temp_dir.clone()
    };

    // Ensure target dir is clean
    if target_dir.exists() {
        fs::remove_dir_all(&target_dir).map_err(|e| format!("清理旧文件失败：{}", e))?;
    }

    // Rename/Move
    if let Err(e) = fs::rename(&install_source, &target_dir) {
        // Fallback: Copy if rename fails (e.g. crossfs, though unlikely here)
        // For now, return error as simple rename should work in AppData
        let _ = fs::remove_dir_all(&temp_dir);
        return Err(format!("移动文件失败：{}", e));
    }

    if install_source != temp_dir {
        let _ = fs::remove_dir_all(&temp_dir);
    }

    // 4. Verify & Permissions
    let java_bin = if cfg!(target_os = "windows") {
        target_dir.join("bin").join("java.exe")
    } else {
        target_dir.join("bin").join("java")
    };

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

    let _ = window.emit(
        "java-install-progress",
        DownloadProgress {
            state: "finished".to_string(),
            progress: 100,
            total: 100,
            message: "安装完成".to_string(),
        },
    );

    Ok(java_bin.to_string_lossy().to_string())
}

#[cfg(target_os = "windows")]
fn extract_zip(zip_path: &Path, target_dir: &Path, cancel_flag: &AtomicBool) -> Result<(), String> {
    let file = fs::File::open(zip_path).map_err(|e| format!("打开 ZIP 文件失败：{}", e))?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader).map_err(|e| format!("ZIP 解析失败：{}", e))?;

    for i in 0..archive.len() {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("用户取消解压".to_string());
        }
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("读取文件失败：{}", e))?;
        let outpath = target_dir.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| format!("创建目录失败：{}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| format!("创建父目录失败：{}", e))?;
                }
            }
            let mut outfile =
                fs::File::create(&outpath).map_err(|e| format!("创建文件失败：{}", e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("写入文件失败：{}", e))?;
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn extract_tar_gz(
    archive_path: &Path,
    target_dir: &Path,
    cancel_flag: &AtomicBool,
) -> Result<(), String> {
    let file = fs::File::open(archive_path).map_err(|e| format!("打开压缩文件失败：{}", e))?;
    let buf_reader = BufReader::new(file);
    let tar = GzDecoder::new(buf_reader);
    let mut archive = Archive::new(tar);

    if cancel_flag.load(Ordering::Relaxed) {
        return Err("用户取消解压".to_string());
    }
    archive
        .unpack(target_dir)
        .map_err(|e| format!("解压失败：{}", e))?;
    Ok(())
}

fn bytes_to_mb(bytes: u64) -> String {
    format!("{:.2}MB", bytes as f64 / 1024.0 / 1024.0)
}
