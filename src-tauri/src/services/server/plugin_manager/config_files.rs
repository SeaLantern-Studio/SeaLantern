use std::fs;
use std::path::Path;

use crate::models::mcs_plugin::m_PluginConfigFile;

use super::common::plugins_dir;

pub(super) fn get_plugin_config_files(
    server_path: &str,
    plugin_name: &str,
) -> Result<Vec<m_PluginConfigFile>, String> {
    let config_folder_path = plugins_dir(server_path).join(plugin_name);

    if config_folder_path.exists() {
        Ok(scan_plugin_config_files(&config_folder_path))
    } else {
        Ok(Vec::new())
    }
}

fn scan_plugin_config_files(plugin_dir: &Path) -> Vec<m_PluginConfigFile> {
    let mut config_files = Vec::new();

    if !plugin_dir.exists() {
        return config_files;
    }

    if let Ok(entries) = fs::read_dir(plugin_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let file_type = path
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                if ["yml", "yaml", "json", "properties"].contains(&file_type.as_str()) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        config_files.push(m_PluginConfigFile {
                            file_name,
                            content,
                            file_type,
                            file_path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            } else if path.is_dir() {
                config_files.extend(scan_plugin_config_files(&path));
            }
        }
    }

    config_files
}
