use crate::services::global;
use crate::utils::logger::log_info_ctx;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const PLUGIN_DIR_LOCATOR_VERSION: u32 = 2;
const PLUGIN_DIR_LOCATOR_FILE_NAME: &str = "plugin_dir.json";
const PLUGIN_DATA_DIR_NAME: &str = ".plugin_data";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginDirLocatorRecord {
    version: u32,
    plugin_dir: String,
    #[serde(default)]
    plugin_data_dir: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginDirStatus {
    pub current_plugin_dir: String,
    pub current_plugin_data_dir: String,
    pub default_plugin_dir: String,
    pub default_plugin_data_dir: String,
    pub locator_path: String,
    pub resolution_source: String,
    pub locator_exists: bool,
    pub recommended_plugin_dir: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginDirChangeResult {
    pub status: PluginDirStatus,
    pub migrated_entries: Vec<String>,
}

fn log_plugin_dir_info(function: &str, message: &str) {
    log_info_ctx("services.plugin_dir", function, message);
}

pub fn locator_path() -> PathBuf {
    crate::utils::path::get_app_data_locator_path().with_file_name(PLUGIN_DIR_LOCATOR_FILE_NAME)
}

pub fn default_plugin_dir() -> PathBuf {
    crate::utils::path::get_app_data_dir().join("plugins")
}

pub fn default_plugin_data_dir() -> PathBuf {
    crate::utils::path::get_app_data_dir().join("plugin_data")
}

pub fn derived_plugin_data_dir(plugin_dir: &Path) -> PathBuf {
    plugin_dir.join(PLUGIN_DATA_DIR_NAME)
}

fn load_locator_record() -> Option<PluginDirLocatorRecord> {
    let locator = locator_path();
    let content = std::fs::read_to_string(locator).ok()?;
    serde_json::from_str::<PluginDirLocatorRecord>(&content).ok()
}

pub fn current_plugin_dir() -> PathBuf {
    if let Some(record) = load_locator_record() {
        let trimmed = record.plugin_dir.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    default_plugin_dir()
}

pub fn current_plugin_data_dir() -> PathBuf {
    if let Some(record) = load_locator_record() {
        let trimmed = record.plugin_data_dir.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    default_plugin_data_dir()
}

pub fn current_status() -> PluginDirStatus {
    let locator = locator_path();
    let default_dir = default_plugin_dir();
    let default_data_dir = default_plugin_data_dir();
    let locator_exists = locator.exists();
    let current_dir = current_plugin_dir();
    let current_data_dir = current_plugin_data_dir();

    let resolution_source = if locator_exists { "locator" } else { "default" };

    PluginDirStatus {
        current_plugin_dir: current_dir.to_string_lossy().to_string(),
        current_plugin_data_dir: current_data_dir.to_string_lossy().to_string(),
        default_plugin_dir: default_dir.to_string_lossy().to_string(),
        default_plugin_data_dir: default_data_dir.to_string_lossy().to_string(),
        locator_path: locator.to_string_lossy().to_string(),
        resolution_source: resolution_source.to_string(),
        locator_exists,
        recommended_plugin_dir: default_dir.to_string_lossy().to_string(),
    }
}

pub fn persist_locator(target_dir: &Path, plugin_data_dir: &Path) -> Result<(), String> {
    let locator = locator_path();
    if let Some(parent) = locator.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!("Failed to create plugin locator directory '{}': {}", parent.display(), e)
        })?;
    }

    let record = PluginDirLocatorRecord {
        version: PLUGIN_DIR_LOCATOR_VERSION,
        plugin_dir: target_dir.to_string_lossy().to_string(),
        plugin_data_dir: plugin_data_dir.to_string_lossy().to_string(),
    };
    let json = serde_json::to_string_pretty(&record)
        .map_err(|e| format!("Failed to serialize plugin directory locator: {}", e))?;
    std::fs::write(&locator, json).map_err(|e| {
        format!("Failed to write plugin directory locator '{}': {}", locator.display(), e)
    })
}

fn validate_target_dir(target_dir: &Path) -> Result<(), String> {
    if target_dir.as_os_str().is_empty() {
        return Err("Plugin directory cannot be empty".to_string());
    }

    if target_dir.exists() && !target_dir.is_dir() {
        return Err(format!(
            "Plugin directory '{}' exists but is not a directory",
            target_dir.display()
        ));
    }

    std::fs::create_dir_all(target_dir).map_err(|e| {
        format!("Failed to create plugin directory '{}': {}", target_dir.display(), e)
    })?;
    Ok(())
}

fn validate_target_plugin_data_dir(target_dir: &Path) -> Result<(), String> {
    if target_dir.as_os_str().is_empty() {
        return Err("Plugin data directory cannot be empty".to_string());
    }

    if target_dir.exists() && !target_dir.is_dir() {
        return Err(format!(
            "Plugin data directory '{}' exists but is not a directory",
            target_dir.display()
        ));
    }

    std::fs::create_dir_all(target_dir).map_err(|e| {
        format!("Failed to create plugin data directory '{}': {}", target_dir.display(), e)
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

fn reload_plugin_manager(target_dir: &Path, plugin_data_dir: &Path) -> Result<(), String> {
    let manager = global::plugin_manager();
    let mut plugin_manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    plugin_manager.reload_roots(target_dir.to_path_buf(), plugin_data_dir.to_path_buf())?;
    plugin_manager.scan_plugins()?;

    #[cfg(feature = "plugin-local-runtime")]
    plugin_manager.auto_enable_plugins_checked()?;

    Ok(())
}

pub fn switch_plugin_dir(
    target_dir: &Path,
    migrate_existing: bool,
) -> Result<PluginDirChangeResult, String> {
    validate_target_dir(target_dir)?;
    let target_plugin_data_dir = derived_plugin_data_dir(target_dir);
    validate_target_plugin_data_dir(&target_plugin_data_dir)?;

    let source_dir = current_plugin_dir();
    let source_plugin_data_dir = current_plugin_data_dir();
    let mut migrated_entries = Vec::new();

    if migrate_existing && source_dir != target_dir {
        for entry in std::fs::read_dir(&source_dir).map_err(|e| {
            format!("Failed to read plugin directory '{}': {}", source_dir.display(), e)
        })? {
            let entry =
                entry.map_err(|e| format!("Failed to read plugin directory entry: {}", e))?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name == PLUGIN_DATA_DIR_NAME {
                continue;
            }
            copy_entry_if_exists(&source_dir, target_dir, &file_name, &mut migrated_entries)?;
        }
    }

    if migrate_existing
        && source_plugin_data_dir != target_plugin_data_dir
        && source_plugin_data_dir.exists()
    {
        for entry in std::fs::read_dir(&source_plugin_data_dir).map_err(|e| {
            format!(
                "Failed to read plugin data directory '{}': {}",
                source_plugin_data_dir.display(),
                e
            )
        })? {
            let entry =
                entry.map_err(|e| format!("Failed to read plugin data directory entry: {}", e))?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            copy_entry_if_exists(
                &source_plugin_data_dir,
                &target_plugin_data_dir,
                &file_name,
                &mut migrated_entries,
            )?;
        }
    }

    persist_locator(target_dir, &target_plugin_data_dir)?;
    reload_plugin_manager(target_dir, &target_plugin_data_dir)?;

    let status = current_status();
    log_plugin_dir_info(
        "switch_plugin_dir",
        &format!(
            "plugin dir switched target={} plugin_data_target={} migrated={}",
            target_dir.display(),
            target_plugin_data_dir.display(),
            migrated_entries.join(",")
        ),
    );
    Ok(PluginDirChangeResult { status, migrated_entries })
}

#[cfg(test)]
mod tests {
    use super::{
        copy_entry_if_exists, current_plugin_data_dir, derived_plugin_data_dir, locator_path,
    };
    use crate::test_support::{lock_env, EnvGuard};

    #[test]
    fn derived_plugin_data_dir_is_hidden_child_of_plugin_dir() {
        let plugin_dir = std::path::Path::new("D:/SeaLantern/plugins");
        assert_eq!(derived_plugin_data_dir(plugin_dir), plugin_dir.join(".plugin_data"));
    }

    #[test]
    fn current_plugin_data_dir_falls_back_when_locator_lacks_data_dir() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let data_dir = temp_dir.path().join("app-data");
        std::fs::create_dir_all(&data_dir).expect("app data dir should exist");

        let _lock = lock_env();
        let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &data_dir.to_string_lossy());

        let locator = locator_path();
        if let Some(parent) = locator.parent() {
            std::fs::create_dir_all(parent).expect("locator parent should exist");
        }
        std::fs::write(
            &locator,
            r#"{
  "version": 1,
  "plugin_dir": "D:/CustomPlugins"
}"#,
        )
        .expect("locator should be written");

        assert_eq!(current_plugin_data_dir(), data_dir.join("plugin_data"));
    }

    #[test]
    fn copy_entry_if_exists_copies_plugin_data_tree() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let source_root = temp_dir.path().join("source-data");
        let target_root = temp_dir.path().join("target-data");
        let nested_dir = source_root.join("demo-plugin");
        std::fs::create_dir_all(&nested_dir).expect("nested source dir should exist");
        std::fs::create_dir_all(&target_root).expect("target root should exist");
        std::fs::write(source_root.join("enabled_plugins.json"), b"[\"demo-plugin\"]")
            .expect("enabled plugins file should exist");
        std::fs::write(nested_dir.join("storage.json"), b"{\"ok\":true}")
            .expect("nested source file should exist");

        let mut migrated_entries = Vec::new();
        copy_entry_if_exists(
            &source_root,
            &target_root,
            "enabled_plugins.json",
            &mut migrated_entries,
        )
        .expect("enabled plugins file should copy");
        copy_entry_if_exists(&source_root, &target_root, "demo-plugin", &mut migrated_entries)
            .expect("plugin data directory should copy");

        assert_eq!(
            std::fs::read_to_string(target_root.join("enabled_plugins.json"))
                .expect("enabled plugins file should be readable"),
            "[\"demo-plugin\"]"
        );
        assert_eq!(
            std::fs::read_to_string(target_root.join("demo-plugin").join("storage.json"))
                .expect("plugin storage file should be readable"),
            "{\"ok\":true}"
        );
        assert_eq!(
            migrated_entries,
            vec!["enabled_plugins.json".to_string(), "demo-plugin".to_string()]
        );
    }
}
