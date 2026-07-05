use crate::models::plugin::PluginInfo;

use super::source::{PluginReplacePolicy, PluginSourceCapabilities, PluginSourceDriver};
use super::PluginManager;

pub(crate) struct BuiltinPluginSourceDriver;

impl PluginSourceDriver for BuiltinPluginSourceDriver {
    fn capabilities(&self) -> PluginSourceCapabilities {
        PluginSourceCapabilities {
            can_install_from_path: false,
            can_delete: false,
            supports_market_update: false,
            replace_policy: PluginReplacePolicy::RejectAll,
        }
    }

    fn scan(&self, _manager: &PluginManager) -> Result<Vec<PluginInfo>, String> {
        Ok(crate::plugins::builtin::builtin_plugin_infos())
    }
}
