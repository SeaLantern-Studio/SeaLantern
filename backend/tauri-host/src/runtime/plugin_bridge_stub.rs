use crate::plugins::api::PluginApiRegistry;
use crate::plugins::runtime::PluginRuntime;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub(crate) fn install_plugin_runtime_bridge(
    _app: &tauri::App,
    _shared_runtimes: Arc<RwLock<HashMap<String, PluginRuntime>>>,
    _shared_runtimes_for_server_ready: Arc<RwLock<HashMap<String, PluginRuntime>>>,
    _api_registry: PluginApiRegistry,
) {
}
