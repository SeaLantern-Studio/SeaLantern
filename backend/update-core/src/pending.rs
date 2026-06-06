use std::path::{Path, PathBuf};

use crate::{types::PendingUpdate, version::compare_versions_checked};

pub fn write_pending_update(
    pending_file: &Path,
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

pub fn check_pending_update(
    pending_file: &Path,
    current_version: &str,
) -> Result<Option<PendingUpdate>, String> {
    if !pending_file.exists() {
        return Ok(None);
    }

    let json = std::fs::read_to_string(pending_file)
        .map_err(|e| format!("Failed to read pending update file: {}", e))?;

    let pending: PendingUpdate = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse pending update: {}", e))?;

    let path = PathBuf::from(&pending.file_path);
    if !path.exists() {
        remove_stale_pending_update_file(pending_file, "missing payload")?;
        return Ok(None);
    }

    if !compare_versions_checked(current_version, &pending.version)
        .map_err(|e| format!("Failed to compare pending update version: {}", e))?
    {
        remove_stale_pending_update_file(pending_file, "version no longer pending")?;
        return Ok(None);
    }

    Ok(Some(pending))
}

pub fn clear_pending_update(pending_file: &Path) -> Result<(), String> {
    if pending_file.exists() {
        std::fs::remove_file(pending_file)
            .map_err(|e| format!("Failed to remove pending update file: {}", e))?;
    }
    Ok(())
}

pub(crate) fn remove_stale_pending_update_file(
    pending_file: &Path,
    reason: &str,
) -> Result<(), String> {
    std::fs::remove_file(pending_file).map_err(|e| {
        format!(
            "Failed to remove stale pending update file ({}): {}",
            reason, e
        )
    })
}

#[cfg(test)]
#[path = "pending_tests.rs"]
mod tests;
