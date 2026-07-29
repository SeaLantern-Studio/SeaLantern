use super::super::super::{PluginManager, PluginState};
use crate::plugins::api::emit_log_event;
use crate::plugins::runtime::{kill_all_processes, PluginRuntime};
use std::path::PathBuf;
use std::sync::Arc;

pub(in crate::plugins::manager::lifecycle) fn enable_plugin(
    manager: &mut PluginManager,
    plugin_id: &str,
) -> Result<(), String> {
    println!("[PluginManager] 正在启用插件: {}", plugin_id);

    let plugin_info = manager
        .plugins
        .get(plugin_id)
        .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?
        .clone();

    if matches!(plugin_info.state, PluginState::Enabled) {
        println!("[PluginManager] 插件 '{}' 已经启用，跳过", plugin_id);
        return Ok(());
    }

    let missing_deps =
        super::super::check_dependencies(manager, &plugin_info.manifest.dependencies);
    if !missing_deps.is_empty() {
        return Err(format!(
            "无法启用插件 '{}'：缺少必须依赖：{}",
            plugin_info.manifest.name,
            missing_deps.join(", ")
        ));
    }

    let missing_optional =
        super::super::check_dependencies(manager, &plugin_info.manifest.optional_dependencies);
    if !missing_optional.is_empty() {
        println!(
            "[PluginManager] 插件 '{}' 的可选依赖未满足：{}（部分功能可能受限）",
            plugin_info.manifest.name,
            missing_optional.join(", ")
        );
    }

    let plugin_dir = PathBuf::from(&plugin_info.path);
    let plugin_data_dir = manager.data_dir.join(plugin_id);
    println!("[PluginManager] 插件数据目录: {}", plugin_data_dir.display());

    if !plugin_info.manifest.include.is_empty() {
        println!("[PluginManager] 正在复制包含的资源: {:?}", plugin_info.manifest.include);
        PluginManager::copy_included_resources(
            &plugin_dir,
            &plugin_data_dir,
            &plugin_info.manifest.include,
        )?;
    }

    let permissions = plugin_info.manifest.permissions.clone();
    println!("[PluginManager] 插件权限: {:?}", permissions);

    let app_data_dir = std::path::PathBuf::from(crate::utils::path::get_or_create_app_data_dir());
    let server_dir = app_data_dir.join("servers");
    let global_dir = app_data_dir;

    println!("[PluginManager] 正在创建运行时...");
    let runtime = PluginRuntime::new(
        plugin_id,
        &plugin_dir,
        &plugin_data_dir,
        &server_dir,
        &global_dir,
        Arc::clone(&manager.api_registry),
        permissions,
    )?;

    let main_file = plugin_dir.join(&plugin_info.manifest.main);
    println!("[PluginManager] 正在加载主文件: {}", main_file.display());
    runtime.load_file(&main_file)?;

    println!("[PluginManager] 正在调用 onLoad 生命周期...");
    runtime.call_lifecycle("onLoad")?;

    {
        let mut runtimes = manager.runtimes.write().unwrap_or_else(|e| {
            eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
            e.into_inner()
        });
        println!("[PluginManager] 运行时已插入到管理器");
        runtimes.insert(plugin_id.to_string(), runtime);
    }

    let enable_result = {
        let runtimes = manager.runtimes.read().unwrap_or_else(|e| {
            eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
            e.into_inner()
        });

        if let Some(runtime) = runtimes.get(plugin_id) {
            println!("[PluginManager] 正在调用 onEnable 生命周期...");
            runtime.call_lifecycle("onEnable")
        } else {
            Err("Runtime not found after insertion".to_string())
        }
    };

    if let Err(e) = enable_result {
        let error_msg = format!("Failed to call onEnable: {}", e);
        let _ = emit_log_event(plugin_id, "error", &error_msg);

        {
            let mut runtimes = manager.runtimes.write().unwrap_or_else(|e| {
                eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
                e.into_inner()
            });
            if let Some(runtime) = runtimes.get_mut(plugin_id) {
                let _ = runtime.call_lifecycle("onDisable");
                let _ = runtime.call_lifecycle("onUnload");
                runtime.cleanup();
                kill_all_processes(&runtime.process_registry);
            }
            runtimes.remove(plugin_id);
        }
        return Err(format!("Failed to enable plugin: {}", e));
    }

    if let Some(info) = manager.plugins.get_mut(plugin_id) {
        info.state = PluginState::Enabled;
    }

    super::super::update_all_missing_dependencies(manager);
    super::super::save_enabled_plugins(manager);

    Ok(())
}
