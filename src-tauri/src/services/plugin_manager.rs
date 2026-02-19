use crate::models::plugin::*;

use std::fs;
use std::io::Read;
use std::path::Path;
use trash;

pub struct PluginManager;

impl PluginManager {
    pub fn new() -> Self {
        PluginManager
    }

    pub fn get_plugins(&self, server_path: &str) -> Result<Vec<PluginInfo>, String> {
        let plugins_dir = Path::new(server_path).join("plugins");

        if !plugins_dir.exists() {
            return Ok(Vec::new());
        }

        let mut plugins = Vec::new();

        for entry in fs::read_dir(&plugins_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let is_jar = file_name.ends_with(".jar") || file_name.ends_with(".jar.disabled");

            if !is_jar {
                continue;
            }

            let is_disabled = file_name.ends_with(".jar.disabled");
            let enabled = !is_disabled;

            let jar_path = if is_disabled {
                path.with_extension("jar")
            } else {
                path.clone()
            };

            if let Ok(mut plugin) = self.parse_plugin_jar(&jar_path) {
                plugin.enabled = enabled;
                plugin.file_name = file_name.to_string();

                let plugin_config_dir = plugins_dir.join(&plugin.id);
                plugin.has_config_folder = plugin_config_dir.exists();

                if plugin.has_config_folder {
                    plugin.config_files = self.scan_plugin_config_files(&plugin_config_dir)?;
                }

                plugins.push(plugin);
            }
        }

        Ok(plugins)
    }

    fn scan_plugin_config_files(&self, config_dir: &Path) -> Result<Vec<PluginConfigFile>, String> {
        let mut configs = Vec::new();

        if !config_dir.exists() {
            return Ok(configs);
        }

        for entry in fs::read_dir(config_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext == "yml"
                        || ext == "yaml"
                        || ext == "json"
                        || ext == "txt"
                        || ext == "properties"
                        || ext == "conf"
                        || ext == "cfg"
                    {
                        let file_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        configs.push(PluginConfigFile {
                            file_name: file_name.clone(),
                            content: String::new(),
                            file_type: ext.to_string(),
                            file_path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }

        Ok(configs)
    }

    fn parse_plugin_jar(&self, jar_path: &Path) -> Result<PluginInfo, String> {
        use zip::ZipArchive;

        let file = fs::File::open(jar_path).map_err(|e| e.to_string())?;
        let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

        let plugin_yml_content = archive
            .by_name("plugin.yml")
            .map_err(|_e| "plugin.yml not found".to_string())?;

        let mut content = String::new();
        plugin_yml_content
            .take(1024 * 1024)
            .read_to_string(&mut content)
            .map_err(|e| e.to_string())?;

        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse plugin.yml: {}", e))?;

        let file_name = jar_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.jar")
            .to_string();

        let file_size = fs::metadata(jar_path).map(|m| m.len()).unwrap_or(0);

        Ok(PluginInfo {
            id: yaml["name"].as_str().unwrap_or("unknown").to_string(),
            name: yaml["name"].as_str().unwrap_or("unknown").to_string(),
            version: yaml["version"].as_str().unwrap_or("unknown").to_string(),
            description: yaml["description"].as_str().unwrap_or("").to_string(),
            author: yaml["author"]
                .as_str()
                .or_else(|| yaml["authors"].as_sequence().and_then(|a| a[0].as_str()))
                .unwrap_or("unknown")
                .to_string(),
            file_name,
            file_size,
            enabled: true,
            main_class: yaml["main"].as_str().unwrap_or("").to_string(),
            has_config_folder: false,
            config_files: Vec::new(),
        })
    }

    pub fn toggle_plugin(
        &self,
        server_path: &str,
        file_name: &str,
        enabled: bool,
    ) -> Result<(), String> {
        let plugins_dir = Path::new(server_path).join("plugins");

        if enabled {
            // 启用插件：将 .jar.disabled 重命名为 .jar
            let disabled_path = plugins_dir.join(format!("{}.jar.disabled", file_name));
            let enabled_path = plugins_dir.join(file_name);

            if disabled_path.exists() {
                fs::rename(&disabled_path, &enabled_path)
                    .map_err(|e| format!("Failed to enable plugin: {}", e))?;
            } else {
                return Err("Disabled plugin file not found".to_string());
            }
        } else {
            // 禁用插件：将 .jar 重命名为 .jar.disabled
            let enabled_path = plugins_dir.join(file_name);
            let disabled_path = plugins_dir.join(format!("{}.jar.disabled", file_name));

            if enabled_path.exists() {
                fs::rename(&enabled_path, &disabled_path)
                    .map_err(|e| format!("Failed to disable plugin: {}", e))?;
            } else {
                return Err("Plugin file not found".to_string());
            }
        }

        Ok(())
    }

    pub fn delete_plugin(&self, server_path: &str, file_name: &str) -> Result<(), String> {
        let plugins_dir = Path::new(server_path).join("plugins");

        // 删除 .jar 和 .jar.disabled 文件到回收站
        let enabled_path = plugins_dir.join(file_name);
        let disabled_path = plugins_dir.join(format!("{}.jar.disabled", file_name));

        if enabled_path.exists() {
            trash::delete(&enabled_path).map_err(|e| format!("Failed to delete plugin: {}", e))?;
        }

        if disabled_path.exists() {
            trash::delete(&disabled_path).map_err(|e| format!("Failed to delete plugin: {}", e))?;
        }

        Ok(())
    }

    pub async fn install_plugin(
        &self,
        server_path: &str,
        file_data: Vec<u8>,
        file_name: &str,
    ) -> Result<(), String> {
        let plugins_dir = Path::new(server_path).join("plugins");
        fs::create_dir_all(&plugins_dir)
            .map_err(|e| format!("Failed to create plugins directory: {}", e))?;

        let target_path = plugins_dir.join(file_name);

        fs::write(&target_path, file_data)
            .map_err(|e| format!("Failed to write plugin file: {}", e))?;

        Ok(())
    }
}
