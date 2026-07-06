use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeComponentEntry {
    pub id: String,
    pub component_type: String,
}

pub type LocaleChangeCallback = Arc<dyn Fn(&str, &str) + Send + Sync>;

pub trait RuntimeI18nApi: Send + Sync {
    fn t(&self, key: &str) -> String;
    fn t_with_options(&self, key: &str, options: &HashMap<String, String>) -> String;
    fn get_locale(&self) -> String;
    fn has_translation(&self, key: &str) -> bool;
    fn has_translation_for_locale(&self, locale: &str, key: &str) -> bool;
    fn get_all_translations(&self) -> HashMap<String, String>;
    fn get_translations_for_locale(&self, locale: &str) -> HashMap<String, String>;
    fn get_available_locales(&self) -> Vec<String>;
    fn register_locale(&self, plugin_id: &str, locale: &str, display_name: &str);
    fn add_plugin_translations(
        &self,
        plugin_id: &str,
        locale: &str,
        entries: HashMap<String, String>,
    );
    fn plugin_translation_entry_count(&self, plugin_id: &str) -> usize;
    fn remove_plugin_translations(&self, plugin_id: &str);
    fn on_locale_change(&self, callback: LocaleChangeCallback) -> usize;
    fn remove_locale_callback(&self, token_id: usize);
}

pub trait RuntimePluginApi: Send + Sync {
    fn call_api(
        &self,
        source_plugin_id: &str,
        target_plugin_id: &str,
        api_name: &str,
        args: Vec<JsonValue>,
    ) -> Result<JsonValue, String>;
    fn emit_ui_event(
        &self,
        plugin_id: &str,
        action: &str,
        target: &str,
        payload: &str,
    ) -> Result<(), String>;
    fn emit_component_event(&self, plugin_id: &str, payload: &str) -> Result<(), String>;
    fn emit_context_menu_event(
        &self,
        plugin_id: &str,
        action: &str,
        context: &str,
        payload: &str,
    ) -> Result<(), String>;
    fn emit_sidebar_event(
        &self,
        plugin_id: &str,
        action: &str,
        label: &str,
        icon: &str,
    ) -> Result<(), String>;
    fn emit_i18n_event(
        &self,
        plugin_id: &str,
        action: &str,
        locale: &str,
        payload: &str,
    ) -> Result<(), String>;
    fn emit_log_event(&self, plugin_id: &str, level: &str, message: &str) -> Result<(), String>;
    fn emit_permission_log(
        &self,
        plugin_id: &str,
        category: &str,
        api_name: &str,
        resource: &str,
    ) -> Result<(), String>;
    fn component_mirror_list(&self, page_filter: Option<&str>) -> Vec<RuntimeComponentEntry>;
    fn element_response_create(&self) -> (u64, std::sync::mpsc::Receiver<String>);
}

pub trait RuntimeHostApi: Send + Sync {
    fn i18n(&self) -> &dyn RuntimeI18nApi;
    fn plugin(&self) -> &dyn RuntimePluginApi;
}

static RUNTIME_HOST_API: OnceLock<Arc<dyn RuntimeHostApi>> = OnceLock::new();

pub fn install_runtime_host_api(
    host: Arc<dyn RuntimeHostApi>,
) -> Result<(), Arc<dyn RuntimeHostApi>> {
    RUNTIME_HOST_API.set(host)
}

pub fn runtime_host_api() -> &'static dyn RuntimeHostApi {
    RUNTIME_HOST_API
        .get()
        .map(Arc::as_ref)
        .expect("runtime host api not installed")
}
