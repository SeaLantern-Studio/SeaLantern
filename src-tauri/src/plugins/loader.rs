use crate::hardcode_data::app_files::PLUGIN_MANIFEST_FILE_NAME;
use crate::models::plugin::PluginManifest;
use std::fs;
use std::path::{Path, PathBuf};

/// 插件加载器
///
/// 负责发现插件目录、读取清单和做基础校验
pub struct PluginLoader;

impl PluginLoader {
    /// 扫描插件目录下的可用插件文件夹
    ///
    /// # Parameters
    ///
    /// - `plugins_dir`: 插件根目录
    ///
    /// # Returns
    ///
    /// 返回所有包含 `manifest.json` 的插件目录
    pub fn discover_plugins(plugins_dir: &Path) -> Result<Vec<PathBuf>, String> {
        if !plugins_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(plugins_dir)
            .map_err(|e| format!("Failed to read plugins directory: {}", e))?;

        let mut plugin_dirs = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() && path.join(PLUGIN_MANIFEST_FILE_NAME).exists() {
                plugin_dirs.push(path);
            }
        }

        Ok(plugin_dirs)
    }

    /// 读取单个插件目录里的清单文件
    ///
    /// # Parameters
    ///
    /// - `plugin_dir`: 插件目录
    ///
    /// # Returns
    ///
    /// 读取成功时返回解析后的插件清单
    pub fn load_manifest(plugin_dir: &Path) -> Result<PluginManifest, String> {
        let manifest_path = plugin_dir.join(PLUGIN_MANIFEST_FILE_NAME);

        let content = fs::read_to_string(&manifest_path).map_err(|e| {
            format!("Failed to read manifest at {}: {}", manifest_path.display(), e)
        })?;

        let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
            format!("Failed to parse manifest at {}: {}", manifest_path.display(), e)
        })?;

        Ok(manifest)
    }

    /// 校验插件清单的基础合法性
    ///
    /// # Parameters
    ///
    /// - `manifest`: 待校验的插件清单
    ///
    /// # Returns
    ///
    /// 校验通过时返回 `Ok(())`
    pub fn validate_manifest(manifest: &PluginManifest) -> Result<(), String> {
        if manifest.id.trim().is_empty() {
            return Err("Manifest field 'id' is required".into());
        }

        if !manifest
            .id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(format!("Plugin ID '{}' contains invalid characters. Only alphanumeric, '-', '_' and '.' are allowed.", manifest.id));
        }
        if manifest.name.trim().is_empty() {
            return Err("Manifest field 'name' is required".into());
        }
        if manifest.version.trim().is_empty() {
            return Err("Manifest field 'version' is required".into());
        }
        if manifest.description.trim().is_empty() {
            return Err("Manifest field 'description' is required".into());
        }
        if manifest.author.name.trim().is_empty() {
            return Err("Manifest field 'author.name' is required".into());
        }
        if manifest.main.trim().is_empty() {
            return Err("Manifest field 'main' is required".into());
        }

        if manifest.main.contains("..") || std::path::Path::new(&manifest.main).is_absolute() {
            return Err(format!(
                "Plugin main file '{}' must be a safe relative path without '..'",
                manifest.main
            ));
        }

        let valid_permissions = [
            "log",
            "fs",
            "fs.data",
            "fs.server",
            "fs.global",
            "api",
            "storage",
            "network",
            "system",
            "server",
            "console",
            "ui",
            "element",
            "execute_program",
            "plugin_folder_access",
            "plugins",
            "ui.component.read",
            "ui.component.write",
            "ui.component.proxy",
            "ui.component.create",
        ];
        for perm in &manifest.permissions {
            if !valid_permissions.contains(&perm.as_str()) {
                return Err(format!(
                    "Plugin '{}': unknown permission '{}' is not allowed. Valid permissions are: {:?}",
                    manifest.id, perm, valid_permissions
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/plugins_loader_tests.rs"]
mod tests;
