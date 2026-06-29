use crate::hardcode_data::app_files::PLUGIN_MANIFEST_FILE_NAME;
use crate::hardcode_data::plugin_manifest::unsupported_plugin_source_message;
use crate::models::plugin::{PluginInfo, PluginInstallResult, PluginSource, PluginState};
use std::path::Path;

use super::PluginManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PluginReplacePolicy {
    ReplaceWhenExistingDisabled,
    RejectAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PluginSourceCapabilities {
    pub can_install_from_path: bool,
    pub can_delete: bool,
    pub supports_market_update: bool,
    pub replace_policy: PluginReplacePolicy,
}

pub(crate) trait PluginSourceDriver {
    fn capabilities(&self) -> PluginSourceCapabilities;

    fn scan(&self, manager: &PluginManager) -> Result<Vec<PluginInfo>, String>;

    fn install(
        &self,
        _manager: &mut PluginManager,
        _path: &Path,
    ) -> Result<PluginInstallResult, String> {
        Err(unsupported_plugin_source_message())
    }
}

pub(crate) fn apply_source_capabilities(
    mut plugin: PluginInfo,
    capabilities: PluginSourceCapabilities,
) -> PluginInfo {
    plugin.actions.can_delete = capabilities.can_delete;
    plugin.actions.can_check_update = capabilities.supports_market_update;
    plugin
}

pub(crate) fn validate_replace_target(
    manager: &PluginManager,
    source: PluginSource,
    plugin_id: &str,
) -> Result<(), String> {
    let capabilities = manager
        .source_driver_for_source(source.clone())
        .capabilities();

    if let Some(existing) = manager.plugins().get(plugin_id) {
        if matches!(existing.source, PluginSource::Builtin)
            && !matches!(source, PluginSource::Builtin)
        {
            return Err(format!(
                "Builtin plugin '{}' cannot be replaced by local install",
                plugin_id
            ));
        }

        match capabilities.replace_policy {
            PluginReplacePolicy::ReplaceWhenExistingDisabled => {
                if matches!(existing.state, PluginState::Enabled) {
                    return Err(crate::plugins::manager::i18n::plugin_t1(
                        "plugin.install.already_running_replace",
                        existing.manifest.name.clone(),
                    ));
                }
            }
            PluginReplacePolicy::RejectAll => {
                return Err(format!(
                    "Plugin '{}' cannot be replaced for source {:?}",
                    plugin_id, source
                ));
            }
        }
    }

    Ok(())
}

pub(crate) fn source_kind_for_install_path(path: &Path) -> Option<PluginSource> {
    if path.extension().is_some_and(|ext| ext == "zip")
        || path
            .file_name()
            .is_some_and(|name| name == PLUGIN_MANIFEST_FILE_NAME)
        || path.is_dir()
    {
        return Some(PluginSource::Local);
    }

    None
}
