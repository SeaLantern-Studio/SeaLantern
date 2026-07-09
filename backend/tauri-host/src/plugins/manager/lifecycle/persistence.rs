use crate::plugins::manager::lifecycle::runtime::{disable_plugin_internal, enable_plugin};
use crate::plugins::manager::PluginManager;
use std::collections::HashSet;

pub(in crate::plugins::manager) fn save_enabled_plugins_checked(
    manager: &PluginManager,
) -> Result<(), String> {
    let enabled: Vec<&str> = manager
        .plugins
        .iter()
        .filter(|(_, info)| matches!(info.state, crate::models::plugin::PluginState::Enabled))
        .map(|(id, _)| id.as_str())
        .collect();
    let path = manager.data_dir.join("enabled_plugins.json");
    let json = serde_json::to_string(&enabled)
        .map_err(|e| format!("Failed to serialize enabled plugins '{}': {}", path.display(), e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to save enabled plugins '{}': {}", path.display(), e))
}

#[allow(dead_code)]
pub(in crate::plugins::manager) fn save_enabled_plugins(manager: &PluginManager) {
    if let Err(error) = save_enabled_plugins_checked(manager) {
        eprintln!("[WARN] {}", error);
    }
}

pub(super) fn load_enabled_plugin_ids_checked(
    manager: &PluginManager,
) -> Result<Vec<String>, String> {
    let path = manager.data_dir.join("enabled_plugins.json");
    let json = match std::fs::read_to_string(&path) {
        Ok(json) => json,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(format!("Failed to read enabled plugins '{}': {}", path.display(), error));
        }
    };

    serde_json::from_str::<Vec<String>>(&json)
        .map_err(|error| format!("Failed to parse enabled plugins '{}': {}", path.display(), error))
}

#[allow(dead_code)]
pub(super) fn load_enabled_plugin_ids(manager: &PluginManager) -> Vec<String> {
    load_enabled_plugin_ids_checked(manager).unwrap_or_default()
}

pub(in crate::plugins::manager) fn auto_enable_plugins_checked(
    manager: &mut PluginManager,
) -> Result<(), String> {
    let ids = load_enabled_plugin_ids_checked(manager)?;
    if ids.is_empty() {
        return Ok(());
    }

    let mut enabled_set: HashSet<String> = HashSet::new();
    let mut remaining: Vec<String> = ids;
    let mut max_passes = remaining.len() + 1;
    while !remaining.is_empty() && max_passes > 0 {
        max_passes -= 1;
        let mut next = Vec::new();
        for id in remaining {
            let deps_ok = if let Some(info) = manager.plugins.get(&id) {
                if !manager.runtime_activation_available_for(info) {
                    eprintln!(
                        "[INFO] Auto-enable skipped '{}': runtime activation unavailable in this build",
                        id
                    );
                    continue;
                }
                info.manifest
                    .dependencies
                    .iter()
                    .all(|d| enabled_set.contains(d.id()))
            } else {
                false
            };
            if deps_ok {
                if let Err(e) = enable_plugin(manager, &id) {
                    eprintln!("[WARN] Auto-enable plugin '{}' failed: {}", id, e);
                } else {
                    enabled_set.insert(id);
                }
            } else {
                next.push(id);
            }
        }
        remaining = next;
    }

    for id in remaining {
        eprintln!("[WARN] Auto-enable skipped '{}': dependencies not met", id);
    }

    Ok(())
}

#[allow(dead_code)]
pub(in crate::plugins::manager) fn auto_enable_plugins(manager: &mut PluginManager) {
    if let Err(error) = auto_enable_plugins_checked(manager) {
        eprintln!("[WARN] Failed to auto-enable plugins: {}", error);
    }
}

pub(in crate::plugins::manager) fn disable_all_plugins_for_shutdown(manager: &mut PluginManager) {
    let enabled_ids: Vec<String> = manager
        .plugins
        .iter()
        .filter(|(_, info)| matches!(info.state, crate::models::plugin::PluginState::Enabled))
        .map(|(id, _)| id.clone())
        .collect();
    for id in enabled_ids {
        let mut visited = HashSet::new();
        if let Err(e) = disable_plugin_internal(manager, &id, &mut visited) {
            eprintln!("[WARN] Failed to disable plugin '{}' during shutdown: {}", id, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{auto_enable_plugins_checked, load_enabled_plugin_ids_checked};
    use crate::models::plugin::{
        PluginActions, PluginAuthor, PluginDistributionClass, PluginExecutionClass, PluginInfo,
        PluginIntegrityStatus, PluginManifest, PluginPermissionProfile, PluginReviewStatus,
        PluginRuntimeKind, PluginSource, PluginTrustLevelDisplay, PluginTrustedPolicySource,
    };
    use crate::plugins::manager::PluginManager;
    use crate::plugins::manager::PluginState;
    use std::collections::HashMap;

    fn example_plugin_for_unavailable_runtime(
        manager: &PluginManager,
        plugin_root: &std::path::Path,
    ) -> Option<crate::models::plugin::PluginInfo> {
        let local_plugin = PluginInfo {
            manifest: PluginManifest {
                id: "example-plugin".to_string(),
                name: "Example Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "test plugin".to_string(),
                author: PluginAuthor {
                    name: "tester".to_string(),
                    email: None,
                    url: None,
                },
                main: "main.lua".to_string(),
                license: None,
                homepage: None,
                repository: None,
                engines: None,
                permissions: Vec::new(),
                ui: None,
                events: Vec::new(),
                commands: Vec::new(),
                programs: Vec::new(),
                dependencies: Vec::new(),
                optional_dependencies: Vec::new(),
                icon: None,
                settings: None,
                sidebar: None,
                locales: None,
                include: Vec::new(),
                capabilities: Vec::new(),
                theme_var_map: HashMap::new(),
                presets: HashMap::new(),
                server_events: HashMap::new(),
            },
            state: PluginState::Disabled,
            path: plugin_root.to_string_lossy().to_string(),
            source: PluginSource::Local,
            runtime: PluginRuntimeKind::Lua,
            actions: PluginActions {
                can_toggle: true,
                can_delete: true,
                can_check_update: true,
            },
            missing_dependencies: Vec::new(),
            trust_level_display: PluginTrustLevelDisplay::StandardSandbox,
            execution_class: PluginExecutionClass::Sandboxed,
            review_status: PluginReviewStatus::Unreviewed,
            integrity_status: PluginIntegrityStatus::Unknown,
            trusted_policy_source: PluginTrustedPolicySource::None,
            permission_profile: PluginPermissionProfile::SandboxedNormal,
            publisher_id: None,
            distribution_class: PluginDistributionClass::LocalDirectory,
            trusted_catalog_matched: false,
            hash_matched: false,
            verified_hash: None,
            verified_signature: false,
            reviewed_at: None,
            revoked: false,
            exceeds_standard_sandbox: false,
            requires_explicit_consent: false,
        };

        let local_plugin = manager.normalize_plugin_info(local_plugin);
        if !manager.runtime_activation_available_for(&local_plugin) {
            return Some(local_plugin);
        }

        let builtin_plugin = crate::plugins::builtin::builtin_plugin_infos()
            .into_iter()
            .next()
            .map(|mut plugin| {
                plugin.path = plugin_root.to_string_lossy().to_string();
                manager.normalize_plugin_info(plugin)
            });

        builtin_plugin.filter(|plugin| !manager.runtime_activation_available_for(plugin))
    }

    #[test]
    fn load_enabled_plugin_ids_checked_surfaces_invalid_json() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = temp_dir.path().join("plugins");
        let data_dir = temp_dir.path().join("plugin-data");
        let manager = PluginManager::new(plugins_dir, data_dir.clone());

        std::fs::write(data_dir.join("enabled_plugins.json"), "{")
            .expect("broken enabled_plugins.json should exist");

        let error = load_enabled_plugin_ids_checked(&manager)
            .expect_err("invalid enabled plugin state should not be treated as an empty list");

        assert!(error.contains("Failed to parse enabled plugins"));
        assert!(error.contains("enabled_plugins.json"));
    }

    #[test]
    fn auto_enable_plugins_checked_surfaces_invalid_json() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = temp_dir.path().join("plugins");
        let data_dir = temp_dir.path().join("plugin-data");
        let mut manager = PluginManager::new(plugins_dir, data_dir.clone());

        std::fs::write(data_dir.join("enabled_plugins.json"), "{")
            .expect("broken enabled_plugins.json should exist");

        let error = auto_enable_plugins_checked(&mut manager)
            .expect_err("invalid enabled plugin state should abort auto-enable");

        assert!(error.contains("Failed to parse enabled plugins"));
        assert!(error.contains("enabled_plugins.json"));
    }

    #[test]
    fn auto_enable_plugins_checked_noops_when_runtime_activation_is_unavailable() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = temp_dir.path().join("plugins");
        let data_dir = temp_dir.path().join("plugin-data");
        let mut manager = PluginManager::new_checked(plugins_dir.clone(), data_dir.clone())
            .expect("plugin manager should initialize");

        let Some(plugin) = example_plugin_for_unavailable_runtime(
            &manager,
            &plugins_dir.join("unavailable-runtime-plugin"),
        ) else {
            return;
        };

        let plugin_id = plugin.manifest.id.clone();
        manager.plugins.insert(plugin_id.clone(), plugin);

        std::fs::write(data_dir.join("enabled_plugins.json"), format!("[\"{}\"]", plugin_id))
            .expect("enabled plugins file should exist");

        auto_enable_plugins_checked(&mut manager).expect("auto-enable should not fail");

        assert!(manager
            .plugins()
            .get(&plugin_id)
            .is_some_and(|info| matches!(
                info.state,
                crate::models::plugin::PluginState::Disabled
            )));
    }
}
