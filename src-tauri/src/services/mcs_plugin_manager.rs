use crate::models::mcs_plugin::*;
use std::fs;
use std::io::Read;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use zip::ZipArchive;

#[allow(non_camel_case_types)]
pub struct m_PluginManager {
    // Add any necessary fields here
}

impl m_PluginManager {
    pub fn new() -> Self {
        Self {
            // Initialize any fields here
        }
    }

    pub fn m_get_plugins(&self, server_path: &str) -> Result<Vec<m_PluginInfo>, String> {
        let plugins_dir = Path::new(server_path).join("plugins");

        if !plugins_dir.exists() {
            fs::create_dir_all(&plugins_dir)
                .map_err(|e| format!("Failed to create plugins directory: {}", e))?;
            return Ok(vec![]);
        }

        let mut plugins = Vec::new();

        for entry in fs::read_dir(&plugins_dir)
            .map_err(|e| format!("Failed to read plugins directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_file() {
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                if file_name.ends_with(".jar") || file_name.ends_with(".jar.disabled") {
                    let enabled = !file_name.ends_with(".jar.disabled");
                    let base_file_name = if enabled {
                        file_name.clone()
                    } else {
                        file_name.replace(".disabled", "")
                    };

                    match self.m_parse_plugin_jar(&path) {
                        Ok(plugin_config) => {
                            // 使用插件在配置中定义的名称来查找配置文件夹
                            let plugin_name = plugin_config
                                .name
                                .clone()
                                .unwrap_or_else(|| base_file_name.replace(".jar", ""));
                            let config_folder_path = plugins_dir.join(&plugin_name);
                            let has_config_folder = config_folder_path.exists();
                            let config_files = if has_config_folder {
                                self.m_scan_plugin_config_files(&config_folder_path)
                            } else {
                                vec![]
                            };

                            let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);

                            // 处理作者信息，优先使用 author 字段，然后使用 authors 数组的第一个元素
                            let author = if let Some(author) = &plugin_config.author {
                                author.clone()
                            } else if let Some(authors) = &plugin_config.authors {
                                authors.first().unwrap_or(&"Unknown".to_string()).clone()
                            } else {
                                "Unknown".to_string()
                            };

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
                                main_class: plugin_config
                                    .main
                                    .unwrap_or_else(|| "Unknown".to_string()),
                                has_config_folder,
                                config_files,
                            });
                        }
                        Err(_) => {
                            // Skip invalid plugins
                        }
                    }
                }
            }
        }

        Ok(plugins)
    }

    pub fn m_toggle_plugin(
        &self,
        server_path: &str,
        file_name: &str,
        enabled: bool,
    ) -> Result<(), String> {
        let plugins_dir = Path::new(server_path).join("plugins");

        // 规范化文件名，确保它是 .jar 文件
        let base_file_name = if file_name.ends_with(".jar.disabled") {
            file_name.replace(".disabled", "")
        } else if file_name.ends_with(".jar") {
            file_name.to_string()
        } else {
            format!("{}.jar", file_name)
        };

        let current_path = if enabled {
            plugins_dir.join(format!("{}.disabled", base_file_name))
        } else {
            plugins_dir.join(&base_file_name)
        };

        let new_path = if enabled {
            plugins_dir.join(&base_file_name)
        } else {
            plugins_dir.join(format!("{}.disabled", base_file_name))
        };

        if current_path.exists() {
            fs::rename(&current_path, &new_path)
                .map_err(|e| format!("Failed to toggle plugin: {}", e))?;
        }

        Ok(())
    }

    pub fn m_delete_plugin(&self, server_path: &str, file_name: &str) -> Result<(), String> {
        let plugins_dir = Path::new(server_path).join("plugins");

        // 规范化文件名，确保它是 .jar 文件
        let base_file_name = if file_name.ends_with(".jar.disabled") {
            file_name.replace(".disabled", "")
        } else if file_name.ends_with(".jar") {
            file_name.to_string()
        } else {
            format!("{}.jar", file_name)
        };

        let enabled_path = plugins_dir.join(&base_file_name);
        let disabled_path = plugins_dir.join(format!("{}.disabled", base_file_name));

        if enabled_path.exists() {
            trash::delete(&enabled_path).map_err(|e| format!("Failed to delete plugin: {}", e))?;
        }

        if disabled_path.exists() {
            trash::delete(&disabled_path).map_err(|e| format!("Failed to delete plugin: {}", e))?;
        }

        Ok(())
    }

    pub async fn m_install_plugin(
        &self,
        server_path: &str,
        file_data: Vec<u8>,
        file_name: &str,
    ) -> Result<(), String> {
        let plugins_dir = Path::new(server_path).join("plugins");

        if !plugins_dir.exists() {
            fs::create_dir_all(&plugins_dir)
                .map_err(|e| format!("Failed to create plugins directory: {}", e))?;
        }

        let plugin_path = plugins_dir.join(file_name);

        let mut file = tokio::fs::File::create(&plugin_path)
            .await
            .map_err(|e| format!("Failed to create plugin file: {}", e))?;

        file.write_all(&file_data)
            .await
            .map_err(|e| format!("Failed to write plugin file: {}", e))?;

        Ok(())
    }

    fn m_parse_plugin_jar(&self, jar_path: &Path) -> Result<m_PluginConfig, String> {
        let file =
            fs::File::open(jar_path).map_err(|e| format!("Failed to open plugin jar: {}", e))?;
        let mut zip =
            ZipArchive::new(file).map_err(|e| format!("Failed to read plugin jar: {}", e))?;

        // Try to find plugin.yml or bungee.yml
        for i in 0..zip.len() {
            let mut file = zip
                .by_index(i)
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

    fn m_scan_plugin_config_files(&self, plugin_dir: &Path) -> Vec<m_PluginConfigFile> {
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
                    config_files.extend(self.m_scan_plugin_config_files(&path));
                }
            }
        }

        config_files
    }
}
