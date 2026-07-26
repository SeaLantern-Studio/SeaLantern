mod paths;
mod pending;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(not(target_os = "windows"))]
pub mod windows;

use std::sync::atomic::AtomicBool;

use crate::update::types::PendingUpdate;

/// 安装进度标志
pub static INSTALL_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// 获取更新缓存目录
pub fn get_update_cache_dir() -> std::path::PathBuf {
    paths::get_update_cache_dir()
}

/// 获取待更新文件路径
pub fn get_pending_update_file() -> std::path::PathBuf {
    paths::get_pending_update_file()
}

/// 检查待更新状态
pub async fn check_pending_update() -> Result<Option<PendingUpdate>, String> {
    pending::check_pending_update().await
}

/// 清除待更新状态
pub async fn clear_pending_update() -> Result<(), String> {
    pending::clear_pending_update().await
}

/// 写入待更新状态文件
pub fn write_pending_update(
    pending_file: &std::path::Path,
    file_path: &str,
    version: String,
) -> Result<(), String> {
    if let Some(parent) = pending_file.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create pending update directory: {}", e))?;
    }

    let pending = PendingUpdate {
        file_path: file_path.to_string(),
        version,
    };
    let json = serde_json::to_string(&pending)
        .map_err(|e| format!("Failed to serialize pending update: {}", e))?;

    std::fs::write(pending_file, json)
        .map_err(|e| format!("Failed to write pending update file: {}", e))?;
    Ok(())
}