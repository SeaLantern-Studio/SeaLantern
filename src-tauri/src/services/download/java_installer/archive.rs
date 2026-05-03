#[cfg(not(target_os = "windows"))]
use flate2::read::GzDecoder;
use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
#[cfg(not(target_os = "windows"))]
use tar::Archive;
#[cfg(target_os = "windows")]
use zip::ZipArchive;

/// 按文件头判断格式并解压下载结果
pub(super) fn extract_downloaded_archive(
    temp_file_path: &Path,
    temp_dir: &Path,
    read_len: usize,
    magic: [u8; 2],
    cancel_flag: &Arc<AtomicBool>,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if read_len >= 2 && &magic == b"PK" {
            return extract_zip(temp_file_path, temp_dir, cancel_flag);
        }
        Err("下载的文件不是有效的 ZIP 格式".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        if read_len >= 2 && magic[0] == 0x1f && magic[1] == 0x8b {
            return extract_tar_gz(temp_file_path, temp_dir, cancel_flag);
        }
        Err("下载的文件不是有效的 tar.gz 格式".to_string())
    }
}

#[cfg(target_os = "windows")]
fn extract_zip(
    zip_path: &Path,
    target_dir: &Path,
    cancel_flag: &AtomicBool,
) -> Result<(), String> {
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
