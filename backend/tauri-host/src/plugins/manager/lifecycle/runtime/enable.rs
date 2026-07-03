use super::super::super::{PluginManager, PluginState};
use crate::plugins::api::emit_log_event;
use crate::plugins::manager::lifecycle::dependencies::{
    check_dependencies, update_all_missing_dependencies,
};
use crate::plugins::manager::lifecycle::persistence::save_enabled_plugins_checked;
use crate::plugins::runtime::{kill_all_processes, PluginRuntime};
use crate::services::events::plugin_server_event_subscriptions_map;
use crate::services::plugin_trusted_catalog::{
    evaluate_enable_requirement, grant_scope_covers, load_enable_grants, upsert_enable_grant,
};
use std::path::PathBuf;
fn success_result(
    manager: &PluginManager,
    plugin_id: &str,
) -> crate::models::plugin::PluginEnableResult {
    crate::models::plugin::PluginEnableResult {
        success: true,
        disabled_plugins: Vec::new(),
        confirmation_required: false,
        block_reason: None,
        plugin: manager.plugins().get(plugin_id).cloned(),
        grant_scope: None,
        message: None,
    }
}

pub(in crate::plugins::manager) fn enable_plugin(
    manager: &mut PluginManager,
    plugin_id: &str,
) -> Result<crate::models::plugin::PluginEnableResult, String> {
    enable_plugin_with_confirmation(manager, plugin_id, None)
}

pub(in crate::plugins::manager) fn enable_plugin_with_confirmation(
    manager: &mut PluginManager,
    plugin_id: &str,
    confirmation: Option<crate::models::plugin::PluginEnableConfirmation>,
) -> Result<crate::models::plugin::PluginEnableResult, String> {
    let plugin_info = manager
        .plugins()
        .get(plugin_id)
        .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?
        .clone();

    if matches!(plugin_info.state, PluginState::Enabled) {
        return Ok(success_result(manager, plugin_id));
    }

    let requirement =
        evaluate_enable_requirement(&plugin_info, &load_enable_grants(&manager.data_dir)?);
    if requirement.confirmation_required {
        let Some(confirmation) = confirmation else {
            return Ok(requirement);
        };

        let required_scope = requirement
            .grant_scope
            .clone()
            .unwrap_or(crate::models::plugin::PluginEnableGrantScope::Version);
        if !grant_scope_covers(confirmation.grant_scope.clone(), required_scope) {
            return Ok(requirement);
        }

        upsert_enable_grant(&manager.data_dir, &plugin_info, confirmation.grant_scope)?;
    }

    perform_enable_plugin(manager, plugin_id)?;
    Ok(success_result(manager, plugin_id))
}

fn perform_enable_plugin(manager: &mut PluginManager, plugin_id: &str) -> Result<(), String> {
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

    let missing_deps = check_dependencies(manager, &plugin_info.manifest.dependencies);
    if !missing_deps.is_empty() {
        return Err(format!(
            "无法启用插件 '{}'：缺少必须依赖：{}",
            plugin_info.manifest.name,
            missing_deps.join(", ")
        ));
    }

    let missing_optional = check_dependencies(manager, &plugin_info.manifest.optional_dependencies);
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
    let allowed_programs = plugin_info
        .manifest
        .programs
        .iter()
        .map(|program| program.path.clone())
        .collect::<Vec<_>>();
    println!("[PluginManager] 插件权限: {:?}", permissions);

    let app_data_dir = std::path::PathBuf::from(
        crate::utils::path::get_or_create_app_data_dir_checked()
            .map_err(|e| format!("Failed to resolve app data directory: {}", e))?,
    );
    let server_dir = app_data_dir.join("servers");
    let global_dir = app_data_dir;

    println!("[PluginManager] 正在创建运行时...");
    let runtime = PluginRuntime::new(
        plugin_id,
        &plugin_dir,
        &plugin_data_dir,
        &server_dir,
        &global_dir,
        manager.api_registry.clone(),
        permissions,
        allowed_programs,
        plugin_server_event_subscriptions_map(&plugin_info.manifest.server_events),
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

    update_all_missing_dependencies(manager);
    save_enabled_plugins_checked(manager)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::enable_plugin;
    use crate::plugins::manager::PluginManager;
    use crate::test_support::{lock_env, EnvGuard};

    #[test]
    fn enable_plugin_surfaces_app_data_dir_creation_failures() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = temp_dir.path().join("plugins");
        let data_dir = temp_dir.path().join("plugin-data");
        std::fs::create_dir_all(&plugins_dir).expect("plugins dir should exist");
        std::fs::create_dir_all(&data_dir).expect("plugin data dir should exist");

        let plugin_root = plugins_dir.join("example-plugin");
        std::fs::create_dir_all(&plugin_root).expect("plugin root should exist");

        let blocked_root = temp_dir.path().join("blocked-root");
        std::fs::write(&blocked_root, b"not a directory")
            .expect("file-backed app data root should exist");
        let blocked_path = blocked_root.join("nested");
        let _env_lock = lock_env();
        let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &blocked_path.to_string_lossy());

        let mut manager = PluginManager::new(plugins_dir, data_dir);
        manager.plugins.insert(
            "example-plugin".to_string(),
            super::super::test_support::example_local_plugin_info(
                &plugin_root,
                crate::plugins::manager::PluginState::Disabled,
            ),
        );

        let error = enable_plugin(&mut manager, "example-plugin")
            .expect_err("app data dir failure should not be silently downgraded");

        assert!(
            error.contains("Failed to resolve app data directory"),
            "unexpected error: {}",
            error
        );
        assert!(error.contains("blocked-root"), "unexpected error: {}", error);
    }

    #[test]
    fn enable_plugin_surfaces_enabled_plugin_persistence_failures() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = temp_dir.path().join("plugins");
        let data_dir = temp_dir.path().join("plugin-data");
        std::fs::create_dir_all(&plugins_dir).expect("plugins dir should exist");
        std::fs::create_dir_all(&data_dir).expect("plugin data dir should exist");

        let plugin_root = plugins_dir.join("example-plugin");
        std::fs::create_dir_all(&plugin_root).expect("plugin root should exist");
        std::fs::write(
            plugin_root.join("main.lua"),
            b"function onLoad() end\nfunction onEnable() end\n",
        )
        .expect("plugin main file should exist");
        std::fs::create_dir_all(data_dir.join("enabled_plugins.json"))
            .expect("directory-backed enabled plugins path should exist");

        let _env_lock = lock_env();
        let _guard = EnvGuard::set(
            "SEALANTERN_DATA_DIR",
            &temp_dir.path().join("app-data").to_string_lossy(),
        );

        let mut manager = PluginManager::new(plugins_dir, data_dir);
        manager.plugins.insert(
            "example-plugin".to_string(),
            super::super::test_support::example_local_plugin_info(
                &plugin_root,
                crate::plugins::manager::PluginState::Disabled,
            ),
        );

        let error = enable_plugin(&mut manager, "example-plugin")
            .expect_err("enabled plugin persistence failure should not be silently downgraded");

        assert!(error.contains("Failed to save enabled plugins"), "unexpected error: {}", error);
        assert!(error.contains("enabled_plugins.json"), "unexpected error: {}", error);
    }
}
