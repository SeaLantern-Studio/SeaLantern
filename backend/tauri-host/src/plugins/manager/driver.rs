use crate::models::plugin::{PluginEnableConfirmation, PluginEnableResult, PluginInfo};
use crate::services::events::ServerEventEnvelope;

use super::PluginManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PluginMetadataCapabilities {
    pub has_settings: bool,
    pub has_icon: bool,
    pub has_css: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PluginRuntimeCapabilities {
    pub can_toggle: bool,
    pub supports_context_menu: bool,
    pub supports_page_events: bool,
    pub supports_locale_events: bool,
    pub supports_server_events: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PluginDriverKind {
    LuaLocal,
    BuiltinRust,
}

pub(crate) trait PluginMetadataDriver {
    fn metadata_capabilities(&self) -> PluginMetadataCapabilities;

    fn delete(
        &self,
        manager: &mut PluginManager,
        plugin_id: &str,
        delete_data: bool,
    ) -> Result<(), String>;
    fn get_settings(
        &self,
        manager: &PluginManager,
        plugin: &PluginInfo,
    ) -> Result<serde_json::Value, String>;
    fn set_settings(
        &self,
        manager: &PluginManager,
        plugin: &PluginInfo,
        settings: serde_json::Value,
    ) -> Result<(), String>;
    fn get_icon(&self, manager: &PluginManager, plugin: &PluginInfo) -> Result<String, String>;
    fn get_css(&self, manager: &PluginManager, plugin: &PluginInfo) -> Result<String, String>;
    fn collect_update_version(&self, plugin: &PluginInfo) -> Option<(String, String)>;
}

pub(crate) trait PluginRuntimeDriver {
    fn runtime_capabilities(&self) -> PluginRuntimeCapabilities;

    fn enable(
        &self,
        manager: &mut PluginManager,
        plugin_id: &str,
        confirmation: Option<PluginEnableConfirmation>,
    ) -> Result<PluginEnableResult, String>;
    fn disable(&self, manager: &mut PluginManager, plugin_id: &str) -> Result<Vec<String>, String>;
    fn notify_page_changed(&self, manager: &PluginManager, plugin_id: &str, path: &str);
    fn notify_locale_changed(&self, manager: &PluginManager, plugin_id: &str, locale: &str);
    fn notify_server_event(
        &self,
        manager: &PluginManager,
        plugin_id: &str,
        event: &ServerEventEnvelope,
    );
    fn notify_context_menu_show(
        &self,
        manager: &PluginManager,
        plugin_id: &str,
        context: &str,
        target_data: &serde_json::Value,
        x: f64,
        y: f64,
    );
    fn notify_context_menu_hide(&self, manager: &PluginManager, plugin_id: &str);
    fn dispatch_context_menu_callback(
        &self,
        manager: &PluginManager,
        plugin_id: &str,
        context: &str,
        item_id: &str,
        target_data: serde_json::Value,
    ) -> Result<(), String>;
}

pub(crate) fn driver_kind_for(plugin: &PluginInfo) -> PluginDriverKind {
    match (&plugin.source, &plugin.runtime) {
        (
            crate::models::plugin::PluginSource::Builtin,
            crate::models::plugin::PluginRuntimeKind::Rust,
        ) => PluginDriverKind::BuiltinRust,
        _ => PluginDriverKind::LuaLocal,
    }
}
