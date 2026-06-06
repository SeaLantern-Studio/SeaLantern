use flate2::read::GzDecoder;
use std::cmp::Ordering;
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;
use zip::ZipArchive;

const PREFERRED_SERVER_JAR_PATTERNS: &[&str] = &[
    "server.jar",
    "forge.jar",
    "fabric-server.jar",
    "minecraft_server.jar",
    "paper.jar",
    "spigot.jar",
    "purpur.jar",
];

const INDICATIVE_SERVER_JAR_KEYWORDS: &[&str] = &[
    "server",
    "forge",
    "fabric",
    "mohist",
    "paper",
    "spigot",
    "purpur",
    "bukkit",
    "catserver",
    "arclight",
];

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
    resolve_extracted_root_checked(extract_dir).unwrap_or_else(|_| extract_dir.to_path_buf())
}

pub fn resolve_extracted_root_checked(extract_dir: &Path) -> Result<PathBuf, String> {
    let entries = match std::fs::read_dir(extract_dir) {
        Ok(entries) => entries,
        Err(error) => return Err(format!("读取解压目录失败: {}", error)),
    };

    let mut directories = Vec::new();
    let mut file_count = 0;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取解压目录项失败: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            directories.push(path);
        } else {
            file_count += 1;
        }
    }

    if file_count == 0 && directories.len() == 1 {
        return Ok(directories.remove(0));
    }

    Ok(extract_dir.to_path_buf())
}

pub fn find_server_jar(modpack_path: &Path) -> Result<String, String> {
    find_server_jar_checked(modpack_path)
}

pub fn find_server_jar_checked(modpack_path: &Path) -> Result<String, String> {
    for pattern in PREFERRED_SERVER_JAR_PATTERNS {
        let jar_path = modpack_path.join(pattern);
        if jar_path.is_file() {
            return Ok(jar_path.to_string_lossy().to_string());
        }
    }

    let entries = std::fs::read_dir(modpack_path).map_err(|e| format!("无法读取文件夹: {}", e))?;
    let mut jar_files = Vec::new();
    let mut invalid_jar_dirs = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取文件夹目录项失败: {}", e))?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "jar" {
                if path.is_file() {
                    jar_files.push(path);
                } else if path.is_dir() {
                    invalid_jar_dirs.push(path);
                }
            }
        }
    }

    let selected = if let Some(path) = select_best_server_jar_path(jar_files) {
        path
    } else if let Some(path) = invalid_jar_dirs.into_iter().next() {
        return Err(format!(
            "检测到目录伪装成 JAR 文件: {}",
            path.to_string_lossy()
        ));
    } else {
        return Err("整合包文件夹中未找到JAR文件".to_string());
    };

    Ok(selected.to_string_lossy().to_string())
}

pub(crate) fn is_script_file(path: &Path) -> bool {
    path.extension()
        .map(|e| {
            let ext = e.to_string_lossy().to_lowercase();
            ext == "sh" || ext == "bat" || ext == "cmd" || ext == "ps1"
        })
        .unwrap_or(false)
}

pub(crate) fn find_server_jar_in_dir_checked(dir: &Path) -> Result<Option<String>, String> {
    let path = match find_server_jar_checked(dir) {
        Ok(path) => path,
        Err(error) if error == "整合包文件夹中未找到JAR文件" => return Ok(None),
        Err(error) => return Err(error),
    };

    Ok(Path::new(&path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string()))
}

fn select_best_server_jar_path(mut jar_files: Vec<PathBuf>) -> Option<PathBuf> {
    if jar_files.is_empty() {
        return None;
    }

    jar_files.sort_by(|left, right| compare_server_jar_candidates(left, right));
    jar_files.into_iter().next()
}

fn compare_server_jar_candidates(left: &Path, right: &Path) -> Ordering {
    let left_name = left
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let right_name = right
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    server_jar_candidate_rank(&left_name)
        .cmp(&server_jar_candidate_rank(&right_name))
        .then_with(|| left_name.cmp(&right_name))
}

fn server_jar_candidate_rank(file_name: &str) -> u8 {
    if INDICATIVE_SERVER_JAR_KEYWORDS
        .iter()
        .any(|keyword| file_name.contains(keyword))
    {
        0
    } else {
        1
    }
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
