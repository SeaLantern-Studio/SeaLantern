mod execute;
mod paths;
mod pending;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(not(target_os = "windows"))]
mod windows;

use std::sync::atomic::AtomicBool;

use super::types::PendingUpdate;

/// 安装进度标志
#[allow(dead_code)] // 发布调用
pub static INSTALL_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// 获取更新缓存目录
#[allow(dead_code)] // 发布调用
pub fn get_update_cache_dir() -> std::path::PathBuf {
    paths::get_update_cache_dir()
}

/// 获取待更新文件路径
#[allow(dead_code)] // 发布调用
pub fn get_pending_update_file() -> std::path::PathBuf {
    paths::get_pending_update_file()
}

/// 执行更新安装
#[allow(dead_code)] // 发布调用
pub async fn execute_install(file_path: String, version: String) -> Result<(), String> {
    execute::execute_install(file_path, version).await
}

/// 检查待更新状态
#[allow(dead_code)] // 发布调用
pub async fn check_pending_update() -> Result<Option<PendingUpdate>, String> {
    pending::check_pending_update().await
}

/// 清除待更新状态
#[allow(dead_code)] // 发布调用
pub async fn clear_pending_update() -> Result<(), String> {
    pending::clear_pending_update().await
}
