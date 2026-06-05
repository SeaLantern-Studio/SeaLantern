use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::path::validate_file_name_only;

pub(super) fn plugins_dir(server_path: &str) -> PathBuf {
    Path::new(server_path).join("plugins")
}

pub(super) fn ensure_plugins_dir(server_path: &str) -> Result<PathBuf, String> {
    let plugins_dir = plugins_dir(server_path);
    if !plugins_dir.exists() {
        fs::create_dir_all(&plugins_dir)
            .map_err(|e| format!("Failed to create plugins directory: {}", e))?;
    }
    Ok(plugins_dir)
}

pub(super) fn normalize_plugin_file_name(file_name: &str) -> String {
    if file_name.ends_with(".jar.disabled") {
        file_name.replace(".disabled", "")
    } else if file_name.ends_with(".jar") {
        file_name.to_string()
    } else {
        format!("{}.jar", file_name)
    }
}

pub(super) fn validate_plugin_file_name(file_name: &str) -> Result<String, String> {
    let normalized = normalize_plugin_file_name(file_name);
    let safe_name = validate_file_name_only(&normalized)?;
    Ok(safe_name.to_string())
}
