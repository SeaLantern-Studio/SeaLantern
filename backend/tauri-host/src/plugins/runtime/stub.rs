use crate::plugins::api::PluginApiRegistry;
use crate::services::events::{ServerEventEnvelope, ServerEventSubscription};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

fn plugin_runtime_disabled_message() -> String {
    "Local plugin runtime is disabled by feature 'plugin-local-runtime'".to_string()
}

#[allow(dead_code)]
pub struct ProcessEntry;

pub fn new_process_registry() -> Arc<Mutex<HashMap<u32, ProcessEntry>>> {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn kill_all_processes(registry: &Arc<Mutex<HashMap<u32, ProcessEntry>>>) {
    registry.lock().unwrap_or_else(|e| e.into_inner()).clear();
}

pub struct PluginRuntime {
    pub(crate) process_registry: Arc<Mutex<HashMap<u32, ProcessEntry>>>,
    #[allow(dead_code)]
    plugin_id: String,
    #[allow(dead_code)]
    plugin_dir: PathBuf,
    #[allow(dead_code)]
    data_dir: PathBuf,
    #[allow(dead_code)]
    server_dir: PathBuf,
    #[allow(dead_code)]
    global_dir: PathBuf,
    #[allow(dead_code)]
    api_registry: PluginApiRegistry,
    #[allow(dead_code)]
    permissions: Vec<String>,
    #[allow(dead_code)]
    allowed_programs: Vec<String>,
    #[allow(dead_code)]
    server_event_subscriptions: HashMap<String, ServerEventSubscription>,
}

impl PluginRuntime {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        plugin_id: &str,
        plugin_dir: &Path,
        data_dir: &Path,
        server_dir: &Path,
        global_dir: &Path,
        api_registry: PluginApiRegistry,
        permissions: Vec<String>,
        allowed_programs: Vec<String>,
        server_event_subscriptions: HashMap<String, ServerEventSubscription>,
    ) -> Result<Self, String> {
        let _ = (
            plugin_id,
            plugin_dir,
            data_dir,
            server_dir,
            global_dir,
            api_registry,
            permissions,
            allowed_programs,
            server_event_subscriptions,
        );
        Err(plugin_runtime_disabled_message())
    }

    #[allow(dead_code)]
    pub fn is_loaded(&self) -> bool {
        false
    }

    pub fn load_file(&self, _path: &Path) -> Result<(), String> {
        Err(plugin_runtime_disabled_message())
    }

    pub fn call_lifecycle(&self, _event: &str) -> Result<(), String> {
        Err(plugin_runtime_disabled_message())
    }

    pub fn call_lifecycle_with_arg(&self, _event: &str, _arg: &str) -> Result<(), String> {
        Err(plugin_runtime_disabled_message())
    }

    pub fn notify_server_event(&self, _event: &ServerEventEnvelope) -> Result<(), String> {
        Err(plugin_runtime_disabled_message())
    }

    pub fn cleanup(&self) {}

    pub fn call_registered_api(
        &self,
        _api_name: &str,
        _args: Vec<JsonValue>,
    ) -> Result<JsonValue, String> {
        Err(plugin_runtime_disabled_message())
    }

    pub fn call_context_menu_hide_callback(&self) -> Result<(), String> {
        Err(plugin_runtime_disabled_message())
    }

    pub fn call_context_menu_show_callback(
        &self,
        _context: &str,
        _target_data: JsonValue,
        _x: f64,
        _y: f64,
    ) -> Result<Vec<JsonValue>, String> {
        Err(plugin_runtime_disabled_message())
    }

    pub fn call_context_menu_callback(
        &self,
        _context: &str,
        _item_id: &str,
        _target_data: JsonValue,
    ) -> Result<(), String> {
        Err(plugin_runtime_disabled_message())
    }
}
