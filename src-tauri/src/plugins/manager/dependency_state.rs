use crate::models::plugin::MissingDependency;

use super::PluginManager;

pub(super) fn get_missing_dependencies(
    manager: &PluginManager,
    manifest: &crate::models::plugin::PluginManifest,
) -> Vec<MissingDependency> {
    let mut missing = Vec::new();

    for dep in &manifest.dependencies {
        append_missing_dependency(manager, &mut missing, dep, true);
    }

    for dep in &manifest.optional_dependencies {
        append_missing_dependency(manager, &mut missing, dep, false);
    }

    missing
}

fn append_missing_dependency(
    manager: &PluginManager,
    missing: &mut Vec<MissingDependency>,
    dep: &crate::models::plugin::PluginDependency,
    required: bool,
) {
    let dep_id = dep.id();
    let is_missing = match manager.plugins.get(dep_id) {
        Some(dep_info) => {
            !dep.is_satisfied_by(&dep_info.manifest.version)
                || !matches!(dep_info.state, crate::models::plugin::PluginState::Enabled)
        }
        None => true,
    };

    if is_missing {
        missing.push(MissingDependency {
            id: dep_id.to_string(),
            version_requirement: dep.version_requirement().map(|value| value.to_string()),
            required,
        });
    }
}
