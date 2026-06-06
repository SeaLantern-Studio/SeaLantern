use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::path::validate_file_name_only;

pub(crate) fn ensure_plugins_dir(server_path: &str) -> Result<PathBuf, String> {
    let plugins_dir = Path::new(server_path).join("plugins");
    if !plugins_dir.exists() {
        fs::create_dir_all(&plugins_dir)
            .map_err(|e| format!("Failed to create plugins directory: {}", e))?;
    }
    Ok(plugins_dir)
}

pub(crate) fn validate_plugin_file_name(file_name: &str) -> Result<String, String> {
    let normalized = if file_name.ends_with(".jar.disabled") {
        file_name.replace(".disabled", "")
    } else if file_name.ends_with(".jar") {
        file_name.to_string()
    } else {
        format!("{}.jar", file_name)
    };
    let safe_name = validate_file_name_only(&normalized)?;
    Ok(safe_name.to_string())
}
