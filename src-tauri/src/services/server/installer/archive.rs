use flate2::read::GzDecoder;
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;
use zip::ZipArchive;

pub fn extract_modpack_archive(archive_path: &Path, target_dir: &Path) -> Result<(), String> {
    let lower_name = archive_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_ascii_lowercase())
        .unwrap_or_default();

    if lower_name.ends_with(".zip") {
        let file =
            std::fs::File::open(archive_path).map_err(|e| format!("无法打开压缩包文件: {}", e))?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| format!("无法解析 ZIP 压缩包: {}", e))?;
        return extract_zip_archive(&mut archive, target_dir);
    }

    if lower_name.ends_with(".tar.gz") || lower_name.ends_with(".tgz") {
        let file =
            std::fs::File::open(archive_path).map_err(|e| format!("无法打开压缩包文件: {}", e))?;
        let decoder = GzDecoder::new(file);
        return extract_tar_archive(decoder, target_dir);
    }

    if lower_name.ends_with(".tar") {
        let file =
            std::fs::File::open(archive_path).map_err(|e| format!("无法打开压缩包文件: {}", e))?;
        return extract_tar_archive(file, target_dir);
    }

    Err("暂不支持该压缩包格式，仅支持 .zip、.tar、.tar.gz、.tgz".to_string())
}

pub fn resolve_extracted_root(extract_dir: &Path) -> PathBuf {
    let entries = match std::fs::read_dir(extract_dir) {
        Ok(entries) => entries,
        Err(_) => return extract_dir.to_path_buf(),
    };

    let mut directories = Vec::new();
    let mut file_count = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            directories.push(path);
        } else {
            file_count += 1;
        }
    }

    if file_count == 0 && directories.len() == 1 {
        return directories.remove(0);
    }

    extract_dir.to_path_buf()
}

pub fn find_server_jar(modpack_path: &Path) -> Result<String, String> {
    let patterns = vec![
        "server.jar",
        "forge.jar",
        "fabric-server.jar",
        "minecraft_server.jar",
        "paper.jar",
        "spigot.jar",
        "purpur.jar",
    ];

    for pattern in &patterns {
        let jar_path = modpack_path.join(pattern);
        if jar_path.exists() {
            return Ok(jar_path.to_string_lossy().to_string());
        }
    }

    let entries = std::fs::read_dir(modpack_path).map_err(|e| format!("无法读取文件夹: {}", e))?;
    let mut jar_files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "jar" {
                    jar_files.push(path);
                }
            }
        }
    }

    if jar_files.is_empty() {
        return Err("整合包文件夹中未找到JAR文件".to_string());
    }

    if jar_files.len() == 1 {
        return Ok(jar_files[0].to_string_lossy().to_string());
    }

    for jar in &jar_files {
        if let Some(name) = jar.file_name() {
            let name_lower = name.to_string_lossy().to_lowercase();
            if name_lower.contains("server")
                || name_lower.contains("forge")
                || name_lower.contains("fabric")
                || name_lower.contains("mohist")
                || name_lower.contains("paper")
                || name_lower.contains("spigot")
                || name_lower.contains("purpur")
                || name_lower.contains("bukkit")
                || name_lower.contains("catserver")
                || name_lower.contains("arclight")
            {
                return Ok(jar.to_string_lossy().to_string());
            }
        }
    }

    Ok(jar_files[0].to_string_lossy().to_string())
}

pub(super) fn is_script_file(path: &Path) -> bool {
    path.extension()
        .map(|e| {
            let ext = e.to_string_lossy().to_lowercase();
            ext == "sh" || ext == "bat" || ext == "ps1"
        })
        .unwrap_or(false)
}

pub(super) fn find_server_jar_in_dir(dir: &Path) -> Option<String> {
    let entries = std::fs::read_dir(dir).ok()?;
    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.extension()? == "jar" {
                path.file_name().map(|name| name.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .next()
}

fn extract_zip_archive(
    archive: &mut ZipArchive<std::fs::File>,
    target_dir: &Path,
) -> Result<(), String> {
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let enclosed_path = file
            .enclosed_name()
            .ok_or_else(|| "ZIP 条目包含非法路径".to_string())?;
        let out_path = target_dir.join(enclosed_path);

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&out_path).map_err(|e| format!("创建目录失败: {}", e))?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        let mut out_file =
            std::fs::File::create(&out_path).map_err(|e| format!("创建文件失败: {}", e))?;
        std::io::copy(&mut file, &mut out_file).map_err(|e| format!("写入文件失败: {}", e))?;
    }

    Ok(())
}

fn extract_tar_archive<R: Read>(reader: R, target_dir: &Path) -> Result<(), String> {
    let mut archive = Archive::new(reader);
    let entries = archive
        .entries()
        .map_err(|e| format!("读取 TAR 条目失败: {}", e))?;

    for entry in entries {
        let mut entry = entry.map_err(|e| format!("解析 TAR 条目失败: {}", e))?;
        entry
            .unpack_in(target_dir)
            .map_err(|e| format!("解压 TAR 条目失败: {}", e))?;
    }

    Ok(())
}
