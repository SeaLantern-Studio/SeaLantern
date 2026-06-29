use crate::models::plugin::PluginSource;
use crate::plugins::manager::lifecycle::dependencies::update_all_missing_dependencies;
use crate::plugins::manager::{PluginInfo, PluginManager};
use crate::plugins::runtime::kill_all_processes;

pub(in crate::plugins::manager) fn scan_plugins(
    manager: &mut PluginManager,
) -> Result<Vec<PluginInfo>, String> {
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
    for source in [PluginSource::Builtin, PluginSource::Local] {
        let scanned_plugins = manager
            .source_driver_for_source(source.clone())
            .scan(manager)?;
        println!(
            "[PluginManager] 来源 {:?} 扫描完成，得到 {} 个插件",
            source,
            scanned_plugins.len()
        );

        for plugin_info in scanned_plugins {
            let plugin_id = plugin_info.manifest.id.clone();
            if let Some(existing) = manager.plugins.get(&plugin_id) {
                println!(
                    "[PluginManager] 跳过重复插件 '{}'：保留来源 {:?}，忽略来源 {:?}",
                    plugin_id, existing.source, plugin_info.source
                );
                continue;
            }

            println!("[PluginManager] 插件 '{}' 已添加到管理器", plugin_id);
            manager.plugins.insert(plugin_id, plugin_info);
        }
    }

    update_all_missing_dependencies(manager);
    println!("[PluginManager] 插件扫描完成，共加载 {} 个插件", manager.plugins.len());

    Ok(manager.plugins.values().cloned().collect())
}
