use crate::services::global;
use crate::utils::constants::{DATA_FILE, RUN_PATH_MAP_FILE, SETTINGS_FILE};
use crate::utils::logger::log_info_ctx;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const LOCATOR_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataDirLocatorRecord {
    version: u32,
    data_dir: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataDirStatus {
    pub current_data_dir: String,
    pub default_data_dir: String,
    pub locator_path: String,
    pub resolution_source: String,
    pub locator_exists: bool,
    pub needs_initial_selection: bool,
    pub recommended_data_dir: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataDirChangeResult {
    pub status: DataDirStatus,
    pub migrated_entries: Vec<String>,
}

fn log_data_dir_info(function: &str, message: &str) {
    log_info_ctx("services.data_dir", function, message);
}

pub fn locator_path() -> PathBuf {
    crate::utils::path::get_app_data_locator_path()
}

pub fn default_data_dir() -> PathBuf {
    crate::utils::path::default_data_dir_base()
}

pub fn current_data_dir() -> PathBuf {
    crate::utils::path::get_app_data_dir()
}

pub fn current_status() -> DataDirStatus {
    let resolution = crate::utils::path::describe_app_data_resolution();
    let locator = locator_path();
    let default_dir = default_data_dir();

    DataDirStatus {
        current_data_dir: resolution.path.to_string_lossy().to_string(),
        default_data_dir: default_dir.to_string_lossy().to_string(),
        locator_path: locator.to_string_lossy().to_string(),
        resolution_source: resolution.source.clone(),
        locator_exists: locator.exists(),
        needs_initial_selection: resolution.source == "default" && !locator.exists(),
        recommended_data_dir: default_dir.to_string_lossy().to_string(),
    }
}

pub fn persist_locator(target_dir: &Path) -> Result<(), String> {
    let locator = locator_path();
    if let Some(parent) = locator.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!("Failed to create locator directory '{}': {}", parent.display(), e)
        })?;
    }

    let record = DataDirLocatorRecord {
        version: LOCATOR_VERSION,
        data_dir: target_dir.to_string_lossy().to_string(),
    };
    let json = serde_json::to_string_pretty(&record)
        .map_err(|e| format!("Failed to serialize data directory locator: {}", e))?;
    std::fs::write(&locator, json).map_err(|e| {
        format!("Failed to write data directory locator '{}': {}", locator.display(), e)
    })
}

fn validate_target_dir(target_dir: &Path) -> Result<(), String> {
    if target_dir.as_os_str().is_empty() {
        return Err("Data directory cannot be empty".to_string());
    }

    if target_dir.exists() && !target_dir.is_dir() {
        return Err(format!(
            "Data directory '{}' exists but is not a directory",
            target_dir.display()
        ));
    }

    std::fs::create_dir_all(target_dir).map_err(|e| {
        format!("Failed to create data directory '{}': {}", target_dir.display(), e)
    })?;
    Ok(())
}

fn copy_entry_if_exists(
    source_root: &Path,
    target_root: &Path,
    relative_name: &str,
    migrated_entries: &mut Vec<String>,
) -> Result<(), String> {
    let source = source_root.join(relative_name);
    if !source.exists() {
        return Ok(());
    }

    let target = target_root.join(relative_name);
    if source.is_dir() {
        copy_dir_recursive(&source, &target)?;
    } else {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("Failed to create target directory '{}': {}", parent.display(), e)
            })?;
        }
        std::fs::copy(&source, &target).map_err(|e| {
            format!("Failed to copy '{}' to '{}': {}", source.display(), target.display(), e)
        })?;
    }

    migrated_entries.push(relative_name.to_string());
    Ok(())
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
    std::fs::create_dir_all(target)
        .map_err(|e| format!("Failed to create directory '{}': {}", target.display(), e))?;

    for entry in std::fs::read_dir(source)
        .map_err(|e| format!("Failed to read directory '{}': {}", source.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create directory '{}': {}", parent.display(), e)
                })?;
            }
            std::fs::copy(&source_path, &target_path).map_err(|e| {
                format!(
                    "Failed to copy '{}' to '{}': {}",
                    source_path.display(),
                    target_path.display(),
                    e
                )
            })?;
        }
    }

    Ok(())
}

fn reload_plugin_manager(_target_dir: &Path) -> Result<(), String> {
    let manager = global::plugin_manager();
    let mut plugin_manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    let plugins_dir = crate::services::plugin_dir::current_plugin_dir();
    let plugin_data_dir = crate::services::plugin_dir::current_plugin_data_dir();
    plugin_manager.reload_roots(plugins_dir, plugin_data_dir)?;
    plugin_manager.scan_plugins()?;
    plugin_manager.auto_enable_plugins_checked()?;
    Ok(())
}

pub fn switch_data_dir(
    target_dir: &Path,
    migrate_existing: bool,
) -> Result<DataDirChangeResult, String> {
    validate_target_dir(target_dir)?;

    let source_dir = current_data_dir();
    let mut migrated_entries = Vec::new();

    if migrate_existing && source_dir != target_dir {
        for entry_name in [
            SETTINGS_FILE,
            DATA_FILE,
            RUN_PATH_MAP_FILE,
            "plugins",
            "plugin_data",
            "personalization",
            "online",
            "runtimes",
            "jar_lfs_links.json",
        ] {
            copy_entry_if_exists(&source_dir, target_dir, entry_name, &mut migrated_entries)?;
        }
    }

    persist_locator(target_dir)?;
    global::settings_manager().reload_from_data_dir(target_dir.to_string_lossy().as_ref())?;
    global::server_manager().reload_from_data_dir(target_dir.to_string_lossy().as_ref())?;
    reload_plugin_manager(target_dir)?;

    let status = current_status();
    log_data_dir_info(
        "switch_data_dir",
        &format!(
            "data dir switched target={} migrated={}",
            target_dir.display(),
            migrated_entries.join(",")
        ),
    );
    Ok(DataDirChangeResult { status, migrated_entries })
}

pub fn initialize_data_dir_selection(target_dir: &Path) -> Result<DataDirChangeResult, String> {
    switch_data_dir(target_dir, true)
}

#[allow(dead_code)]
pub fn ensure_locator_for_current_data_dir() -> Result<(), String> {
    let current = current_data_dir();
    if locator_path().exists() {
        return Ok(());
    }
    persist_locator(&current)
}

#[cfg(test)]
mod tests {
    use super::{copy_dir_recursive, copy_entry_if_exists};

    #[test]
    fn copy_entry_if_exists_copies_files_and_records_migration() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let source_root = temp_dir.path().join("source");
        let target_root = temp_dir.path().join("target");
        std::fs::create_dir_all(&source_root).expect("source root should exist");
        std::fs::create_dir_all(&target_root).expect("target root should exist");
        std::fs::write(source_root.join("sea_lantern_servers.json"), b"[]")
            .expect("source file should exist");

        let mut migrated_entries = Vec::new();
        copy_entry_if_exists(
            &source_root,
            &target_root,
            "sea_lantern_servers.json",
            &mut migrated_entries,
        )
        .expect("file copy should succeed");

        assert_eq!(migrated_entries, vec!["sea_lantern_servers.json".to_string()]);
        assert_eq!(
            std::fs::read_to_string(target_root.join("sea_lantern_servers.json"))
                .expect("target file should be readable"),
            "[]"
        );
    }

    #[test]
    fn copy_entry_if_exists_copies_directories_recursively() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let source_root = temp_dir.path().join("source");
        let target_root = temp_dir.path().join("target");
        let nested_dir = source_root.join("plugin_data").join("demo-plugin");
        std::fs::create_dir_all(&nested_dir).expect("nested source dir should exist");
        std::fs::create_dir_all(&target_root).expect("target root should exist");
        std::fs::write(nested_dir.join("storage.json"), b"{\"ok\":true}")
            .expect("nested source file should exist");

        let mut migrated_entries = Vec::new();
        copy_entry_if_exists(&source_root, &target_root, "plugin_data", &mut migrated_entries)
            .expect("directory copy should succeed");

        assert_eq!(migrated_entries, vec!["plugin_data".to_string()]);
        assert_eq!(
            std::fs::read_to_string(
                target_root
                    .join("plugin_data")
                    .join("demo-plugin")
                    .join("storage.json")
            )
            .expect("nested target file should be readable"),
            "{\"ok\":true}"
        );
    }

    #[test]
    fn copy_dir_recursive_preserves_nested_tree() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target");
        let nested_dir = source_dir.join("level1").join("level2");
        std::fs::create_dir_all(&nested_dir).expect("nested source dir should exist");
        std::fs::write(source_dir.join("root.txt"), b"root").expect("root file should exist");
        std::fs::write(nested_dir.join("leaf.txt"), b"leaf").expect("leaf file should exist");

        copy_dir_recursive(&source_dir, &target_dir).expect("recursive copy should succeed");

        assert_eq!(
            std::fs::read_to_string(target_dir.join("root.txt")).expect("root file should copy"),
            "root"
        );
        assert_eq!(
            std::fs::read_to_string(target_dir.join("level1").join("level2").join("leaf.txt"))
                .expect("leaf file should copy"),
            "leaf"
        );
    }
}
