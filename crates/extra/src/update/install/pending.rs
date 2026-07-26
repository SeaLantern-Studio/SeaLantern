use std::path::PathBuf;

use super::paths::get_pending_update_file;
use crate::update::types::PendingUpdate;
use crate::update::version::compare_versions;

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

    // 注意: 此函数需要在调用时传入current_version,这里使用环境变量
    // 实际使用时应该从外部传入
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