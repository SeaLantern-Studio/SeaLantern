#[cfg(not(target_os = "windows"))]
use flate2::read::GzDecoder;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
#[cfg(not(target_os = "windows"))]
use tar::Archive;
use tauri::{Emitter, Manager, Window};
#[cfg(target_os = "windows")]
use zip::ZipArchive;

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
    let app_handle = window.app_handle();
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {}", e))?;

    let runtimes_dir = app_dir.join("runtimes");
    if !runtimes_dir.exists() {
        fs::create_dir_all(&runtimes_dir).map_err(|e| format!("无法创建运行时目录: {}", e))?;
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

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    let total_size = res.content_length().unwrap_or(0);
    // Use bytes_stream to process chunks
    use futures::StreamExt;
    let mut stream = res.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut data = Vec::new();
    let mut last_emit = std::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("用户取消下载".to_string());
        }
        let chunk = chunk.map_err(|e| format!("下载流错误: {}", e))?;
        data.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;

        if total_size > 0 && last_emit.elapsed().as_millis() > 100 {
            let _ = window.emit(
                "java-install-progress",
                DownloadProgress {
                    state: "downloading".to_string(),
                    progress: downloaded,
                    total: total_size,
                    message: format!(
                        "正在下载: {}/{}",
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
            message: "下载完成, 准备解压...".to_string(),
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

    let temp_dir = runtimes_dir.join(format!("temp_{}", version_name));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).map_err(|e| format!("无法清理临时目录: {}", e))?;
    }
    fs::create_dir_all(&temp_dir).map_err(|e| format!("无法创建临时目录: {}", e))?;

    // Check file signature (Magic Numbers)
    #[cfg(target_os = "windows")]
    {
        if data.len() > 2 && &data[0..2] == b"PK" {
            extract_zip(&data, &temp_dir, &cancel_flag)?;
        } else {
            return Err("下载的文件不是有效的 ZIP 格式".to_string());
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Gzip header is usually 1F 8B
        if data.len() > 2 && data[0] == 0x1f && data[1] == 0x8b {
            extract_tar_gz(&data, &temp_dir, &cancel_flag)?;
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
        fs::remove_dir_all(&target_dir).map_err(|e| format!("清理旧文件失败: {}", e))?;
    }

    // Rename/Move
    if let Err(e) = fs::rename(&install_source, &target_dir) {
        // Fallback: Copy if rename fails (e.g. crossfs, though unlikely here)
        // For now, return error as simple rename should work in AppData
        let _ = fs::remove_dir_all(&temp_dir);
        return Err(format!("移动文件失败: {}", e));
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
        return Err(format!("安装失败: 未找到可执行文件 {:?}", java_bin));
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
fn extract_zip(data: &[u8], target_dir: &Path, cancel_flag: &AtomicBool) -> Result<(), String> {
    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("ZIP 解析失败: {}", e))?;

    for i in 0..archive.len() {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("用户取消解压".to_string());
        }
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        let outpath = target_dir.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| format!("创建父目录失败: {}", e))?;
                }
            }
            let mut outfile =
                fs::File::create(&outpath).map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn extract_tar_gz(data: &[u8], target_dir: &Path, cancel_flag: &AtomicBool) -> Result<(), String> {
    let cursor = Cursor::new(data);
    let tar = GzDecoder::new(cursor);
    let mut archive = Archive::new(tar);

    // tar archive unpacking is usually one-shot, but we can check if there's a way to iterate.
    // The `unpack` method is convenient but not cancellable per-entry easily without manual iteration.
    // For now, check flag before. Note: unpacking fully is usually fast on modern SSDs unless huge.
    if cancel_flag.load(Ordering::Relaxed) {
        return Err("用户取消解压".to_string());
    }
    archive
        .unpack(target_dir)
        .map_err(|e| format!("解压失败: {}", e))?;
    Ok(())
}

fn bytes_to_mb(bytes: u64) -> String {
    format!("{:.2}MB", bytes as f64 / 1024.0 / 1024.0)
}
