use super::{PluginInfo, PluginManager, PluginState};
use crate::plugins::loader::PluginLoader;
use crate::plugins::runtime::kill_all_processes;

pub(super) fn scan_plugins(manager: &mut PluginManager) -> Result<Vec<PluginInfo>, String> {
    println!("[PluginManager] 开始扫描插件目录: {}", manager.plugins_dir.display());

    {
        let mut runtimes = manager.runtimes.write().unwrap_or_else(|e| {
            eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
            e.into_inner()
        });
        println!("[PluginManager] 清理旧的运行时，共 {} 个", runtimes.len());
        for (id, runtime) in runtimes.drain() {
            println!("[PluginManager] 停止插件 '{}' 的运行时", id);
            kill_all_processes(&runtime.process_registry);
            if let Err(e) = runtime.call_lifecycle("onDisable") {
                eprintln!(
                    "[WARN] Failed to call onDisable for plugin '{}' during rescan: {}",
                    id, e
                );
            }
            if let Err(e) = runtime.call_lifecycle("onUnload") {
                eprintln!(
                    "[WARN] Failed to call onUnload for plugin '{}' during rescan: {}",
                    id, e
                );
            }
        }
    }

    manager.plugins.clear();

    let plugin_dirs = PluginLoader::discover_plugins(&manager.plugins_dir)?;
    println!("[PluginManager] 发现 {} 个插件目录", plugin_dirs.len());

    for plugin_dir in &plugin_dirs {
        println!("[PluginManager] 正在加载插件: {}", plugin_dir.display());
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
                    Err(e) => {
                        println!("[PluginManager] 插件 '{}' 验证失败: {}", manifest.id, e);
                        PluginState::Error(e)
                    }
                };

                let plugin_info = PluginInfo {
                    manifest: manifest.clone(),
                    state,
                    path: plugin_dir.to_string_lossy().to_string(),
                    missing_dependencies: Vec::new(),
                };

                manager.plugins.insert(manifest.id.clone(), plugin_info);
                println!("[PluginManager] 插件 '{}' 已添加到管理器", manifest.id);
            }
            Err(e) => {
                println!(
                    "[PluginManager] 从 {} 加载 manifest 失败: {}",
                    plugin_dir.display(),
                    e
                );
            }
        }
    }

    super::update_all_missing_dependencies(manager);
    println!("[PluginManager] 插件扫描完成，共加载 {} 个插件", manager.plugins.len());

    Ok(manager.plugins.values().cloned().collect())
}
