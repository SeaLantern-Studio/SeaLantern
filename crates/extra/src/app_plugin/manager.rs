use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::engine::{Lifecycle, PluginEngine};
use super::{AppPluginError, PluginLoader, PluginManifest};
use crate::observability;

/// 插件管理器需要的宿主无关目录配置。
#[derive(Debug, Clone)]
pub struct PluginManagerConfig {
    /// 已安装插件的根目录。
    pub plugins_dir: PathBuf,
    /// 插件私有持久化数据的根目录。
    pub data_dir: PathBuf,
}

impl PluginManagerConfig {
    /// 以插件根目录和数据根目录创建配置。
    pub fn new(plugins_dir: impl Into<PathBuf>, data_dir: impl Into<PathBuf>) -> Self {
        Self {
            plugins_dir: plugins_dir.into(),
            data_dir: data_dir.into(),
        }
    }
}

/// 已加载插件的可观察状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// 主脚本及 \`on_load\` 已执行，尚未启用。
    Loaded,
    /// \`on_enable\` 已执行。
    Enabled,
}

/// 不暴露 Lua 引擎的插件状态快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInfo {
    /// 通过 v2 校验的插件清单。
    pub manifest: PluginManifest,
    /// 插件安装目录。
    pub path: PathBuf,
    /// 当前生命周期状态。
    pub state: PluginState,
}

struct ManagedPlugin {
    info: PluginInfo,
    engine: PluginEngine,
}

/// 管理 v2 插件发现与生命周期，不承担安装、依赖解析或宿主能力适配。
pub struct PluginManager {
    config: PluginManagerConfig,
    plugins: HashMap<String, ManagedPlugin>,
}

impl PluginManager {
    /// 创建空插件管理器。
    pub fn new(config: PluginManagerConfig) -> Self {
        Self { config, plugins: HashMap::new() }
    }

    /// 发现插件目录但不读取、加载或执行脚本。
    pub fn discover(&self) -> Result<Vec<PathBuf>, AppPluginError> {
        PluginLoader::discover_plugins(&self.config.plugins_dir)
    }

    /// 读取、校验并加载一个已发现插件。
    ///
    /// 清单检查在创建数据目录和 Lua 引擎前完成，因此旧 API 插件不会产生副作用。
    pub fn load(&mut self, plugin_dir: &Path) -> Result<PluginInfo, AppPluginError> {
        let plugin_dir = self.validate_plugin_dir(plugin_dir)?;
        let manifest = match PluginLoader::load_manifest(&plugin_dir) {
            Ok(manifest) => manifest,
            Err(error @ AppPluginError::ApiVersionTooOld) => {
                let plugin_id = plugin_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown");
                observability::app_plugin_api_too_old(plugin_id, None);
                return Err(error);
            }
            Err(error) => return Err(error),
        };
        if self.plugins.contains_key(&manifest.id) {
            return Err(AppPluginError::Engine(format!(
                "plugin '{}' is already loaded",
                manifest.id
            )));
        }

        let data_dir = self.config.data_dir.join(&manifest.id);
        fs::create_dir_all(&data_dir).map_err(|error| AppPluginError::Io {
            path: data_dir.clone(),
            message: error.to_string(),
        })?;

        let engine = PluginEngine::new(&manifest, &plugin_dir, &data_dir)?;
        engine.load()?;
        if let Err(error) = engine.call_lifecycle(Lifecycle::Load) {
            observability::app_plugin_lifecycle_failed(
                &manifest.id,
                Lifecycle::Load.as_str(),
                &error,
            );
            return Err(error);
        }

        let info = PluginInfo {
            manifest: manifest.clone(),
            path: plugin_dir,
            state: PluginState::Loaded,
        };
        self.plugins
            .insert(manifest.id.clone(), ManagedPlugin { info: info.clone(), engine });
        observability::app_plugin_loaded(&manifest.id);
        Ok(info)
    }

    /// 执行 \`on_enable\` 并将插件置为启用状态。
    pub fn enable(&mut self, plugin_id: &str) -> Result<(), AppPluginError> {
        let mut plugin = self.remove_plugin(plugin_id)?;
        if plugin.info.state == PluginState::Enabled {
            self.plugins.insert(plugin_id.to_string(), plugin);
            return Ok(());
        }

        if let Err(error) = plugin.engine.call_lifecycle(Lifecycle::Enable) {
            self.fail_plugin(plugin, Lifecycle::Enable, &error);
            return Err(error);
        }

        plugin.info.state = PluginState::Enabled;
        self.plugins.insert(plugin_id.to_string(), plugin);
        Ok(())
    }

    /// 执行 \`on_disable\` 并保留已加载但未启用的插件。
    pub fn disable(&mut self, plugin_id: &str) -> Result<(), AppPluginError> {
        let mut plugin = self.remove_plugin(plugin_id)?;
        if plugin.info.state == PluginState::Loaded {
            self.plugins.insert(plugin_id.to_string(), plugin);
            return Ok(());
        }

        if let Err(error) = plugin.engine.call_lifecycle(Lifecycle::Disable) {
            self.fail_plugin(plugin, Lifecycle::Disable, &error);
            return Err(error);
        }

        plugin.info.state = PluginState::Loaded;
        self.plugins.insert(plugin_id.to_string(), plugin);
        Ok(())
    }

    /// 释放一个插件；启用态会先执行 \`on_disable\`，随后总会尝试 \`on_unload\`。
    pub fn unload(&mut self, plugin_id: &str) -> Result<(), AppPluginError> {
        let plugin = self.remove_plugin(plugin_id)?;
        let mut first_error = None;

        if plugin.info.state == PluginState::Enabled {
            if let Err(error) = plugin.engine.call_lifecycle(Lifecycle::Disable) {
                observability::app_plugin_lifecycle_failed(
                    plugin_id,
                    Lifecycle::Disable.as_str(),
                    &error,
                );
                first_error = Some(error);
            }
        }

        if let Err(error) = plugin.engine.call_lifecycle(Lifecycle::Unload) {
            observability::app_plugin_lifecycle_failed(
                plugin_id,
                Lifecycle::Unload.as_str(),
                &error,
            );
            if first_error.is_none() {
                first_error = Some(error);
            }
        }

        first_error.map_or(Ok(()), Err)
    }

    /// 返回一个插件的状态快照。
    pub fn plugin(&self, plugin_id: &str) -> Option<PluginInfo> {
        self.plugins
            .get(plugin_id)
            .map(|plugin| plugin.info.clone())
    }

    /// 返回当前所有已加载插件的状态快照，按 ID 排序。
    pub fn plugins(&self) -> Vec<PluginInfo> {
        let mut plugins = self
            .plugins
            .values()
            .map(|plugin| plugin.info.clone())
            .collect::<Vec<_>>();
        plugins.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
        plugins
    }

    fn remove_plugin(&mut self, plugin_id: &str) -> Result<ManagedPlugin, AppPluginError> {
        self.plugins
            .remove(plugin_id)
            .ok_or_else(|| AppPluginError::Engine(format!("plugin '{plugin_id}' is not loaded")))
    }

    fn validate_plugin_dir(&self, plugin_dir: &Path) -> Result<PathBuf, AppPluginError> {
        let plugins_root =
            self.config
                .plugins_dir
                .canonicalize()
                .map_err(|error| AppPluginError::Io {
                    path: self.config.plugins_dir.clone(),
                    message: error.to_string(),
                })?;
        let plugin_dir = plugin_dir
            .canonicalize()
            .map_err(|error| AppPluginError::Io {
                path: plugin_dir.to_path_buf(),
                message: error.to_string(),
            })?;

        if plugin_dir.parent() != Some(plugins_root.as_path()) {
            return Err(AppPluginError::InvalidPath {
                path: plugin_dir,
                message: "plugin directory must be a direct child of the configured plugins root"
                    .to_string(),
            });
        }
        Ok(plugin_dir)
    }

    fn fail_plugin(&self, plugin: ManagedPlugin, lifecycle: Lifecycle, error: &AppPluginError) {
        observability::app_plugin_lifecycle_failed(
            &plugin.info.manifest.id,
            lifecycle.as_str(),
            error,
        );
        if let Err(cleanup_error) = plugin.engine.call_lifecycle(Lifecycle::Unload) {
            observability::app_plugin_lifecycle_failed(
                &plugin.info.manifest.id,
                Lifecycle::Unload.as_str(),
                &cleanup_error,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn test_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the Unix epoch")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("sealantern-extra-app-plugin-{label}-{}-{nonce}", std::process::id()))
    }

    fn write_plugin(root: &Path, api_version: u32, script: &str) -> PathBuf {
        let plugin_dir = root.join("example.plugin");
        fs::create_dir_all(&plugin_dir).expect("plugin directory should be created");
        fs::write(
            plugin_dir.join("manifest.json"),
            format!(
                r#"{{
                    "apiVersion": {api_version},
                    "id": "example.plugin",
                    "name": "Example",
                    "version": "1.0.0",
                    "main": "main.lua"
                }}"#
            ),
        )
        .expect("manifest should be written");
        fs::write(plugin_dir.join("main.lua"), script).expect("script should be written");
        plugin_dir
    }

    #[test]
    fn manager_runs_v2_lifecycle_and_removes_unloaded_plugin() {
        let root = test_root("lifecycle");
        let plugin_dir = write_plugin(
            &root,
            2,
            r#"
                function on_load() end
                function on_enable() end
                function on_disable() end
                function on_unload() end
            "#,
        );
        let data_dir = root.join("data");
        let mut manager = PluginManager::new(PluginManagerConfig::new(&root, &data_dir));

        assert_eq!(
            manager.load(&plugin_dir).expect("plugin should load").state,
            PluginState::Loaded
        );
        manager
            .enable("example.plugin")
            .expect("plugin should enable");
        assert_eq!(
            manager
                .plugin("example.plugin")
                .expect("loaded plugin should be visible")
                .state,
            PluginState::Enabled
        );
        manager
            .disable("example.plugin")
            .expect("plugin should disable");
        manager
            .unload("example.plugin")
            .expect("plugin should unload");
        assert!(manager.plugin("example.plugin").is_none());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn old_api_is_rejected_before_data_directory_creation() {
        let root = test_root("old-api");
        let plugin_dir = write_plugin(&root, 1, "return {}");
        let data_dir = root.join("data");
        let mut manager = PluginManager::new(PluginManagerConfig::new(&root, &data_dir));

        let error = manager
            .load(&plugin_dir)
            .expect_err("v1 plugins must not load");
        assert_eq!(error.to_string(), "版本过旧");
        assert!(!data_dir.join("example.plugin").exists());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn enable_failure_removes_partial_plugin_state() {
        let root = test_root("enable-failure");
        let plugin_dir = write_plugin(
            &root,
            2,
            r#"
                function on_enable()
                    error("expected enable failure")
                end
            "#,
        );
        let mut manager = PluginManager::new(PluginManagerConfig::new(&root, root.join("data")));

        manager.load(&plugin_dir).expect("plugin should load");
        assert!(manager.enable("example.plugin").is_err());
        assert!(manager.plugin("example.plugin").is_none());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn manager_rejects_plugin_directories_outside_its_root() {
        let root = test_root("root-boundary");
        let plugins_root = root.join("plugins");
        let outside_root = root.join("outside");
        fs::create_dir_all(&plugins_root).expect("plugins root should be created");
        let plugin_dir = write_plugin(&outside_root, 2, "return {}");
        let mut manager =
            PluginManager::new(PluginManagerConfig::new(&plugins_root, root.join("data")));

        assert!(matches!(manager.load(&plugin_dir), Err(AppPluginError::InvalidPath { .. })));

        let _ = fs::remove_dir_all(root);
    }
}
