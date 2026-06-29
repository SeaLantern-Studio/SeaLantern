use crate::models::plugin::{
    PluginEnableConfirmation, PluginEnableResult, PluginInfo, PluginState,
};
use crate::plugins::builtin;
use crate::services::events::ServerEventEnvelope;
use crate::utils::logger::log_trace_ctx;

use super::driver::{PluginDriver, PluginRuntimeCapabilities};
use super::PluginManager;

pub(crate) struct BuiltinRustPluginDriver;

fn log_builtin_driver_trace(function: &str, plugin_id: &str, message: &str) {
    log_trace_ctx(
        "plugins.manager.driver_builtin",
        function,
        &format!("plugin_id={} {}", plugin_id, message),
    );
}

impl PluginDriver for BuiltinRustPluginDriver {
    fn capabilities(&self) -> PluginRuntimeCapabilities {
        PluginRuntimeCapabilities {
            can_toggle: true,
            has_settings: true,
            has_icon: false,
            has_css: false,
            supports_context_menu: false,
            supports_page_events: false,
            supports_locale_events: false,
            supports_server_events: false,
        }
    }

    fn enable(
        &self,
        manager: &mut PluginManager,
        plugin_id: &str,
        _confirmation: Option<PluginEnableConfirmation>,
    ) -> Result<PluginEnableResult, String> {
        log_builtin_driver_trace("enable", plugin_id, "begin");

        let plugin_info = manager
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;

        if !self.capabilities().can_toggle || !plugin_info.actions.can_toggle {
            log_builtin_driver_trace("enable", plugin_id, "rejected can_toggle=false");
            return Err(format!("Builtin plugin '{}' cannot be toggled", plugin_id));
        }

        if matches!(plugin_info.state, PluginState::Enabled) {
            log_builtin_driver_trace("enable", plugin_id, "skip already_enabled");
            return Ok(PluginEnableResult {
                success: true,
                disabled_plugins: Vec::new(),
                confirmation_required: false,
                block_reason: None,
                plugin: manager.plugins.get(plugin_id).cloned(),
                grant_scope: None,
                message: None,
            });
        }

        if let Some(info) = manager.plugins.get_mut(plugin_id) {
            info.state = PluginState::Enabled;
        }

        super::lifecycle::dependencies::update_all_missing_dependencies(manager);
        super::lifecycle::persistence::save_enabled_plugins_checked(manager)?;
        log_builtin_driver_trace("enable", plugin_id, "completed state=enabled");
        Ok(PluginEnableResult {
            success: true,
            disabled_plugins: Vec::new(),
            confirmation_required: false,
            block_reason: None,
            plugin: manager.plugins.get(plugin_id).cloned(),
            grant_scope: None,
            message: None,
        })
    }

    fn disable(&self, manager: &mut PluginManager, plugin_id: &str) -> Result<Vec<String>, String> {
        log_builtin_driver_trace("disable", plugin_id, "begin");

        let plugin_info = manager
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;

        if !matches!(plugin_info.state, PluginState::Enabled) {
            log_builtin_driver_trace("disable", plugin_id, "skip already_disabled");
            return Ok(Vec::new());
        }

        if let Some(info) = manager.plugins.get_mut(plugin_id) {
            info.state = PluginState::Disabled;
        }

        super::lifecycle::dependencies::update_all_missing_dependencies(manager);
        super::lifecycle::persistence::save_enabled_plugins_checked(manager)?;
        log_builtin_driver_trace("disable", plugin_id, "completed state=disabled");
        Ok(Vec::new())
    }

    fn delete(
        &self,
        _manager: &mut PluginManager,
        plugin_id: &str,
        _delete_data: bool,
    ) -> Result<(), String> {
        Err(format!("Builtin plugin '{}' cannot be deleted", plugin_id))
    }

    fn get_settings(
        &self,
        manager: &PluginManager,
        plugin: &PluginInfo,
    ) -> Result<serde_json::Value, String> {
        let settings_path = builtin::builtin_settings_dir(&manager.data_dir)
            .join(&plugin.manifest.id)
            .join("settings.json");
        if !settings_path.exists() {
            return Ok(builtin::default_settings(&plugin.manifest.id)
                .unwrap_or_else(|| serde_json::json!({})));
        }

        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings file: {}", e))
    }

    fn set_settings(
        &self,
        manager: &PluginManager,
        plugin: &PluginInfo,
        settings: serde_json::Value,
    ) -> Result<(), String> {
        let settings_dir =
            builtin::builtin_settings_dir(&manager.data_dir).join(&plugin.manifest.id);
        std::fs::create_dir_all(&settings_dir)
            .map_err(|e| format!("Failed to create builtin settings dir: {}", e))?;
        let settings_path = settings_dir.join("settings.json");
        let content = serde_json::to_string_pretty(&settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        std::fs::write(&settings_path, content)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;
        Ok(())
    }

    fn get_icon(&self, _manager: &PluginManager, _plugin: &PluginInfo) -> Result<String, String> {
        Ok(String::new())
    }

    fn get_css(&self, _manager: &PluginManager, _plugin: &PluginInfo) -> Result<String, String> {
        Ok(String::new())
    }

    fn collect_update_version(&self, _plugin: &PluginInfo) -> Option<(String, String)> {
        None
    }

    fn notify_page_changed(&self, _manager: &PluginManager, _plugin_id: &str, _path: &str) {}

    fn notify_locale_changed(&self, _manager: &PluginManager, _plugin_id: &str, _locale: &str) {}

    fn notify_server_event(
        &self,
        _manager: &PluginManager,
        _plugin_id: &str,
        _event: &ServerEventEnvelope,
    ) {
    }

    fn notify_context_menu_show(
        &self,
        _manager: &PluginManager,
        _plugin_id: &str,
        _context: &str,
        _target_data: &serde_json::Value,
        _x: f64,
        _y: f64,
    ) {
    }

    fn notify_context_menu_hide(&self, _manager: &PluginManager, _plugin_id: &str) {}

    fn dispatch_context_menu_callback(
        &self,
        _manager: &PluginManager,
        plugin_id: &str,
        _context: &str,
        _item_id: &str,
        _target_data: serde_json::Value,
    ) -> Result<(), String> {
        Err(format!(
            "Builtin plugin '{}' does not support context menu callbacks",
            plugin_id
        ))
    }
}
