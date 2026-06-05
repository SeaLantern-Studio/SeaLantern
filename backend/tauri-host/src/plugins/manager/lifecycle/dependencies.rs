use super::{PluginInfo, PluginManager, PluginState};
use crate::models::plugin::PluginDependency;

pub(super) fn update_all_missing_dependencies(manager: &mut PluginManager) {
    let plugin_manifests: Vec<(String, crate::models::plugin::PluginManifest)> = manager
        .plugins
        .iter()
        .map(|(id, info)| (id.clone(), info.manifest.clone()))
        .collect();

    for (plugin_id, manifest) in plugin_manifests {
        let missing = manager.get_missing_dependencies(&manifest);
        if let Some(info) = manager.plugins.get_mut(&plugin_id) {
            info.missing_dependencies = missing;
        }
    }
}

pub(super) fn check_dependencies(
    manager: &PluginManager,
    dependencies: &[PluginDependency],
) -> Vec<String> {
    let mut missing = Vec::new();
    for dep in dependencies {
        let dep_id = dep.id();
        match manager.plugins.get(dep_id) {
            Some(dep_info) => {
                if !matches!(dep_info.state, PluginState::Enabled) {
                    missing.push(format!("{} (未启用)", dep_id));
                } else if !dep.is_satisfied_by(&dep_info.manifest.version) {
                    let req = dep.version_requirement().unwrap_or("any");
                    missing.push(format!(
                        "{} (版本 {} 不满足要求 {})",
                        dep_id, dep_info.manifest.version, req
                    ));
                }
            }
            None => {
                if let Some(req) = dep.version_requirement() {
                    missing.push(format!("{} {} (未安装)", dep_id, req));
                } else {
                    missing.push(format!("{} (未安装)", dep_id));
                }
            }
        }
    }
    missing
}

pub(super) fn get_dependent_plugin_ids(manager: &PluginManager, plugin_id: &str) -> Vec<String> {
    let mut dependents = Vec::new();
    for (id, info) in &manager.plugins {
        if !matches!(info.state, PluginState::Enabled) {
            continue;
        }

        if info
            .manifest
            .dependencies
            .iter()
            .any(|d| d.id() == plugin_id)
        {
            dependents.push(id.clone());
        }
    }
    dependents
}

pub(super) fn get_plugin_list(manager: &PluginManager) -> Vec<PluginInfo> {
    manager.plugins.values().cloned().collect()
}
