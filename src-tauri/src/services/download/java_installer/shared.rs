use std::fs;
use std::path::{Path, PathBuf};

/// 把字节数转成 MB 文本
pub(super) fn bytes_to_mb(bytes: u64) -> String {
    format!("{:.2}MB", bytes as f64 / 1024.0 / 1024.0)
}

/// 找出真正要移动到目标目录的安装源
pub(super) fn resolve_install_source(temp_dir: &Path) -> PathBuf {
    if let Ok(entries) = fs::read_dir(temp_dir) {
        let entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        if entries.len() == 1 && entries[0].path().is_dir() {
            return entries[0].path();
        }
    }

    temp_dir.to_path_buf()
}

/// 解析 Java 可执行文件路径
pub(super) fn resolve_java_binary_path(target_dir: &Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        target_dir.join("bin").join("java.exe")
    } else {
        target_dir.join("bin").join("java")
    }
}
