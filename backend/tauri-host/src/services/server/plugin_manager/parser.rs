use std::fs;
use std::io::Read;
use std::path::Path;

use crate::models::mcs_plugin::m_PluginConfig;
use zip::ZipArchive;

pub(super) fn parse_plugin_jar(jar_path: &Path) -> Result<m_PluginConfig, String> {
    let file = fs::File::open(jar_path).map_err(|e| format!("Failed to open plugin jar: {}", e))?;
    let mut zip = ZipArchive::new(file).map_err(|e| format!("Failed to read plugin jar: {}", e))?;

    for index in 0..zip.len() {
        let mut file = zip
            .by_index(index)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;

        if file.name() == "plugin.yml" || file.name() == "bungee.yml" {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| format!("Failed to read config file: {}", e))?;

            let config: m_PluginConfig = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse config file: {}", e))?;
            return Ok(config);
        }
    }

    Err("No plugin.yml or bungee.yml found in jar".to_string())
}
