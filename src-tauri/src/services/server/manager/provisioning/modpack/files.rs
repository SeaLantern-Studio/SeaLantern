use std::path::Path;

use super::super::super::fs::{copy_dir_recursive, path_is_child_of, paths_equal};
use crate::services::server::installer;

pub(super) fn prepare_modpack_files(source_path: &Path, run_dir: &Path) -> Result<(), String> {
    let source_file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("server.jar");
    let source_extension = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .unwrap_or_default();

    if source_path.is_file() {
        std::fs::create_dir_all(run_dir).map_err(|e| format!("无法创建运行目录: {}", e))?;
        if source_extension == "jar" {
            let target_jar = run_dir.join(source_file_name);
            std::fs::copy(source_path, &target_jar)
                .map_err(|e| format!("复制 JAR 文件失败: {}", e))?;
        } else {
            installer::extract_modpack_archive(source_path, run_dir)?;
        }
        return Ok(());
    }

    if source_path.is_dir() {
        if !paths_equal(source_path, run_dir) {
            if path_is_child_of(run_dir, source_path) {
                return Err("运行目录不能位于整合包源目录内部，请选择其他目录".to_string());
            }
            std::fs::create_dir_all(run_dir).map_err(|e| format!("无法创建运行目录: {}", e))?;
            copy_dir_recursive(source_path, run_dir)
                .map_err(|e| format!("复制整合包文件失败: {}", e))?;
        }
        return Ok(());
    }

    Err("无效的整合包路径".to_string())
}
