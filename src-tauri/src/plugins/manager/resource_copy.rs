use std::fs;
use std::path::Path;

pub(super) fn copy_included_resources(
    plugin_dir: &Path,
    data_dir: &Path,
    includes: &[String],
) -> Result<(), String> {
    fs::create_dir_all(data_dir).map_err(|e| format!("Failed to create data dir: {}", e))?;

    for pattern in includes {
        let clean = pattern.trim_end_matches('/');
        let src = plugin_dir.join(clean);
        if !src.exists() {
            eprintln!("[插件] include 资源不存在，跳过: {}", src.display());
            continue;
        }

        let dest = data_dir.join(clean);
        if src.is_dir() {
            copy_dir_recursive(&src, &dest)?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir: {}", e))?;
            }
            fs::copy(&src, &dest)
                .map_err(|e| format!("Failed to copy {}: {}", src.display(), e))?;
        }
    }

    Ok(())
}

pub(super) fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest)
        .map_err(|e| format!("Failed to create dir {}: {}", dest.display(), e))?;
    let entries =
        fs::read_dir(src).map_err(|e| format!("Failed to read dir {}: {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)
                .map_err(|e| format!("Failed to copy {}: {}", src_path.display(), e))?;
        }
    }

    Ok(())
}
