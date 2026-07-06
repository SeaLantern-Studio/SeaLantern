use crate::models::plugin::{PluginInfo, PluginInstallResult, PluginState};
use crate::plugins::loader::PluginLoader;
use crate::services::plugin_trusted_catalog::PluginInstallMetadata;
use std::path::Path;

use super::source::{PluginReplacePolicy, PluginSourceCapabilities, PluginSourceDriver};
use super::PluginManager;

pub(crate) struct LocalFilesystemPluginSourceDriver;

impl PluginSourceDriver for LocalFilesystemPluginSourceDriver {
    fn capabilities(&self) -> PluginSourceCapabilities {
        PluginSourceCapabilities {
            can_install_from_path: true,
            can_delete: true,
            supports_market_update: true,
            replace_policy: PluginReplacePolicy::ReplaceWhenExistingDisabled,
        }
    }

    fn scan(&self, manager: &PluginManager) -> Result<Vec<PluginInfo>, String> {
        let plugin_dirs = PluginLoader::discover_plugins(&manager.plugins_dir)?;
        println!("[PluginManager] 发现 {} 个本地插件目录", plugin_dirs.len());

        let mut plugins = Vec::new();
        for plugin_dir in &plugin_dirs {
            println!("[PluginManager] 正在加载本地插件: {}", plugin_dir.display());
            match PluginLoader::load_manifest(plugin_dir) {
                Ok(manifest) => {
                    println!(
                        "[PluginManager] 插件 '{}' (ID: {}) 版本 {}",
                        manifest.name, manifest.id, manifest.version
                    );

                    let state = match PluginLoader::validate_manifest(&manifest) {
                        Ok(()) => {
                            println!("[PluginManager] 插件 '{}' 验证通过", manifest.id);
                            PluginState::Loaded
                        }
                        Err(error) => {
                            println!("[PluginManager] 插件 '{}' 验证失败: {}", manifest.id, error);
                            PluginState::Error(error)
                        }
                    };

                    let install_metadata =
                        crate::services::plugin_trusted_catalog::read_install_metadata(plugin_dir)
                            .unwrap_or_default();
                    let distribution_class = install_metadata
                        .distribution_class
                        .clone()
                        .unwrap_or(crate::models::plugin::PluginDistributionClass::LocalDirectory);

                    let plugin_info = manager.make_local_plugin_info(
                        manifest,
                        state,
                        plugin_dir.to_string_lossy().to_string(),
                        distribution_class,
                        install_metadata.archive_sha256.as_deref(),
                        Vec::new(),
                    );

                    let plugin_info =
                        crate::services::plugin_trusted_catalog::apply_runtime_integrity_state(
                            plugin_info,
                            plugin_dir,
                            &install_metadata,
                        )?;

                    plugins.push(plugin_info);
                }
                Err(error) => {
                    println!(
                        "[PluginManager] 从 {} 加载 manifest 失败: {}",
                        plugin_dir.display(),
                        error
                    );
                }
            }
        }

        Ok(plugins)
    }

    fn install(
        &self,
        manager: &mut PluginManager,
        path: &Path,
        metadata: &PluginInstallMetadata,
    ) -> Result<PluginInstallResult, String> {
        super::install::install_local_plugin(manager, path, metadata)
    }
}
