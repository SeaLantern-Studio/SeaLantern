use super::{PluginManager, PluginState};

pub(super) fn get_all_plugin_css(manager: &PluginManager) -> Result<Vec<(String, String)>, String> {
    let mut result = Vec::new();

    for (plugin_id, plugin_info) in &manager.plugins {
        if matches!(plugin_info.state, PluginState::Enabled) {
            if let Ok(css_content) = manager
                .metadata_driver_for(plugin_info)
                .get_css(manager, plugin_info)
            {
                if !css_content.is_empty() {
                    result.push((plugin_id.clone(), css_content));
                }
            }
        }
    }

    Ok(result)
}
