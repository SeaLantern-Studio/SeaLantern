//! 待更新状态持久化管理。
//!
//! 将待安装的更新信息序列化为 JSON 文件存储在缓存目录中，
//! 以便应用重启后仍能检测到未完成的安装。
//!
//! # 安全性
//!
//! `check_pending_update` 要求调用方传入当前版本号进行比对，
//! 避免使用过时的待更新记录。

use std::path::PathBuf;

use super::paths::get_pending_update_file;
use crate::update::types::PendingUpdate;
use crate::update::version::compare_versions;

pub async fn check_pending_update(current_version: &str) -> Result<Option<PendingUpdate>, String> {
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
