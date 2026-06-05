use std::fs;

use crate::models::mcs_plugin::m_PluginInfo;

use super::common::ensure_plugins_dir;
use super::parser::parse_plugin_jar;

pub(super) async fn get_plugins(server_path: &str) -> Result<Vec<m_PluginInfo>, String> {
    let plugins_dir = ensure_plugins_dir(server_path)?;
    let mut plugins = Vec::new();

    let entries = fs::read_dir(&plugins_dir)
        .map_err(|e| format!("Failed to read plugins directory: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if !file_name.ends_with(".jar") && !file_name.ends_with(".jar.disabled") {
            continue;
        }

        let enabled = !file_name.ends_with(".jar.disabled");
        let base_file_name = if enabled {
            file_name.clone()
        } else {
            file_name.replace(".disabled", "")
        };

        if let Ok(plugin_config) = parse_plugin_jar(&path) {
            let plugin_name = plugin_config
                .name
                .clone()
                .unwrap_or_else(|| base_file_name.replace(".jar", ""));
            let config_folder_path = plugins_dir.join(&plugin_name);
            let has_config_folder = config_folder_path.exists();
            let file_size = path.metadata().map(|meta| meta.len()).unwrap_or(0);
            let author = resolve_author(&plugin_config);

            plugins.push(m_PluginInfo {
                m_id: format!(
                    "{}-{}",
                    plugin_config.name.as_deref().unwrap_or("unknown"),
                    plugin_config.version.as_deref().unwrap_or("unknown")
                ),
                name: plugin_config
                    .name
                    .unwrap_or_else(|| "Unknown Plugin".to_string()),
                version: plugin_config
                    .version
                    .unwrap_or_else(|| "Unknown".to_string()),
                description: plugin_config
                    .description
                    .unwrap_or_else(|| "No description".to_string()),
                author,
                file_name: base_file_name,
                file_size,
                enabled,
                main_class: plugin_config.main.unwrap_or_else(|| "Unknown".to_string()),
                has_config_folder,
                config_files: Vec::new(),
            });
        }
    }

    Ok(plugins)
}

fn resolve_author(plugin_config: &crate::models::mcs_plugin::m_PluginConfig) -> String {
    if let Some(author) = &plugin_config.author {
        author.clone()
    } else if let Some(authors) = &plugin_config.authors {
        authors.first().unwrap_or(&"Unknown".to_string()).clone()
    } else {
        "Unknown".to_string()
    }
}
