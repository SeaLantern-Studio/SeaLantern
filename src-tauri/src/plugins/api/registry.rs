use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ApiRegistry = Arc<Mutex<HashMap<String, HashMap<String, String>>>>;

pub fn new_api_registry() -> ApiRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

pub trait ApiRegistryOps {
    fn register_api(&self, plugin_id: &str, api_name: &str, lua_fn_name: &str);

    fn has_api(&self, plugin_id: &str, api_name: &str) -> bool;

    fn list_apis(&self, plugin_id: &str) -> Vec<String>;

    fn get_api_fn_name(&self, plugin_id: &str, api_name: &str) -> Option<String>;

    fn clear_plugin_apis(&self, plugin_id: &str);
}

impl ApiRegistryOps for ApiRegistry {
    fn register_api(&self, plugin_id: &str, api_name: &str, lua_fn_name: &str) {
        let mut registry = self.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Mutex poisoned, recovering: {}", e);
            e.into_inner()
        });
        let plugin_apis = registry.entry(plugin_id.to_string()).or_default();
        plugin_apis.insert(api_name.to_string(), lua_fn_name.to_string());
    }

    fn has_api(&self, plugin_id: &str, api_name: &str) -> bool {
        let registry = self.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Mutex poisoned, recovering: {}", e);
            e.into_inner()
        });
        registry
            .get(plugin_id)
            .map(|apis| apis.contains_key(api_name))
            .unwrap_or(false)
    }

    fn list_apis(&self, plugin_id: &str) -> Vec<String> {
        let registry = self.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Mutex poisoned, recovering: {}", e);
            e.into_inner()
        });
        registry
            .get(plugin_id)
            .map(|apis| apis.keys().cloned().collect())
            .unwrap_or_default()
    }

    fn get_api_fn_name(&self, plugin_id: &str, api_name: &str) -> Option<String> {
        let registry = self.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Mutex poisoned, recovering: {}", e);
            e.into_inner()
        });
        registry
            .get(plugin_id)
            .and_then(|apis| apis.get(api_name).cloned())
    }

    fn clear_plugin_apis(&self, plugin_id: &str) {
        let mut registry = self.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Mutex poisoned, recovering: {}", e);
            e.into_inner()
        });
        registry.remove(plugin_id);
    }
}
