//! 待更新状态持久化管理。
//!
//! 将待安装的更新信息序列化为 JSON 文件存储在缓存目录中，
//! 以便应用重启后仍能检测到未完成的安装。

use std::path::PathBuf;

use super::paths::get_pending_update_file;
use crate::update::types::PendingUpdate;
use crate::update::version::compare_versions;

/// 检查待更新状态
///
/// 当前版本通过编译时 `env!("CARGO_PKG_VERSION")` 获取，
/// 项目中通过版本管理脚本确保各 crate 版本号一致。
pub async fn check_pending_update() -> Result<Option<PendingUpdate>, String> {
    let pending_file = get_pending_update_file();

    if !pending_file.exists() {
        return Ok(None);
    }

    let json = std::fs::read_to_string(&pending_file)
        .map_err(|e| format!("Failed to read pending update file: {}", e))?;

    let pending: PendingUpdate = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse pending update: {}", e))?;

    let path = PathBuf::from(&pending.file_path);
    if !path.exists() {
        std::fs::remove_file(&pending_file).ok();
        return Ok(None);
    }

    let current_version = env!("CARGO_PKG_VERSION");
    if !compare_versions(current_version, &pending.version) {
        std::fs::remove_file(&pending_file).ok();
        return Ok(None);
    }

    Ok(Some(pending))
}

pub async fn clear_pending_update() -> Result<(), String> {
    let pending_file = get_pending_update_file();
    if pending_file.exists() {
        std::fs::remove_file(&pending_file)
            .map_err(|e| format!("Failed to remove pending update file: {}", e))?;
    }
    Ok(())
}
