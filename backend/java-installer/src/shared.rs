use std::fs;
use std::path::{Path, PathBuf};

pub(crate) const JAVA_DOWNLOAD_TEMP_FILE_NAME: &str = "java_download.tmp";
pub(crate) const JAVA_DOWNLOAD_TIMEOUT_SECS: u64 = 60;
pub(crate) const JAVA_DOWNLOAD_RETRY_LIMIT: usize = 3;

/// 把字节数转成 MB 文本
pub(crate) fn bytes_to_mb(bytes: u64) -> String {
    format!("{:.2}MB", bytes as f64 / 1024.0 / 1024.0)
}

/// 找出真正要移动到目标目录的安装源
pub(crate) fn resolve_install_source(temp_dir: &Path) -> Result<PathBuf, String> {
    let entries = fs::read_dir(temp_dir).map_err(|e| format!("读取安装临时目录失败：{}", e))?;
    let mut collected = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取安装临时目录项失败：{}", e))?;
        collected.push(entry.path());
    }

    if collected.len() == 1 && collected[0].is_dir() {
        return Ok(collected.remove(0));
    }

    Ok(temp_dir.to_path_buf())
}

/// 解析 Java 可执行文件路径
pub(crate) fn resolve_java_binary_path(target_dir: &Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        target_dir.join("bin").join("java.exe")
    } else {
        target_dir.join("bin").join("java")
    }
}
