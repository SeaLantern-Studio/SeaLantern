use crate::models::plugin::PluginInfo;
use crate::plugins::api::emit_log_event;
use crate::services::events::ServerEventEnvelope;
use crate::utils::logger::{log_error_ctx, log_warn_ctx};
use std::path::PathBuf;

use super::driver::{PluginDriver, PluginRuntimeCapabilities};
use super::PluginManager;

pub(crate) struct LuaLocalPluginDriver;

impl LuaLocalPluginDriver {
    fn plugin_runtime<'a>(
        manager: &'a PluginManager,
        plugin_id: &str,
    ) -> Option<
        std::sync::RwLockReadGuard<
            'a,
            std::collections::HashMap<String, crate::plugins::runtime::PluginRuntime>,
        >,
    > {
        let runtimes = manager.runtimes.read().unwrap_or_else(|e| e.into_inner());
        if !runtimes.contains_key(plugin_id) {
            return None;
        }
        Some(runtimes)
    }
}

impl PluginDriver for LuaLocalPluginDriver {
    fn capabilities(&self) -> PluginRuntimeCapabilities {
        PluginRuntimeCapabilities {
            can_toggle: true,
            has_settings: true,
            has_icon: true,
            has_css: true,
            supports_context_menu: true,
            supports_page_events: true,
            supports_locale_events: true,
            supports_server_events: true,
        }
    }

    fn enable(
        &self,
        manager: &mut PluginManager,
        plugin_id: &str,
        confirmation: Option<crate::models::plugin::PluginEnableConfirmation>,
    ) -> Result<crate::models::plugin::PluginEnableResult, String> {
        super::lifecycle::runtime::enable_plugin_with_confirmation(manager, plugin_id, confirmation)
    }

    fn disable(&self, manager: &mut PluginManager, plugin_id: &str) -> Result<Vec<String>, String> {
        super::lifecycle::runtime::disable_plugin(manager, plugin_id)
    }

    fn delete(
        &self,
        manager: &mut PluginManager,
        plugin_id: &str,
        delete_data: bool,
    ) -> Result<(), String> {
        super::install::delete_plugin(manager, plugin_id, delete_data)
    }

    fn get_settings(
        &self,
        _manager: &PluginManager,
        plugin: &PluginInfo,
    ) -> Result<serde_json::Value, String> {
        let plugin_path = PathBuf::from(&plugin.path);
        let settings_path = plugin_path.join("settings.json");

        if !settings_path.exists() {
            return Ok(serde_json::json!({}));
        }

        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        let settings: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings file: {}", e))?;

        Ok(settings)
    }

    fn set_settings(
        &self,
        _manager: &PluginManager,
        plugin: &PluginInfo,
        settings: serde_json::Value,
    ) -> Result<(), String> {
        let plugin_path = PathBuf::from(&plugin.path);
        let settings_path = plugin_path.join("settings.json");

        let content = serde_json::to_string_pretty(&settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        std::fs::write(&settings_path, content)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }

    fn get_icon(&self, _manager: &PluginManager, plugin: &PluginInfo) -> Result<String, String> {
        let plugin_path = PathBuf::from(&plugin.path);

        let icon_filename = plugin.manifest.icon.as_deref().unwrap_or("icon.png");
        if icon_filename.contains("..") || std::path::Path::new(icon_filename).is_absolute() {
            return Err(format!("Plugin icon path '{}' is not safe", icon_filename));
        }

        let icon_path = plugin_path.join(icon_filename);
        if !icon_path.exists() {
            return Ok(String::new());
        }

        let content =
            std::fs::read(&icon_path).map_err(|e| format!("Failed to read icon file: {}", e))?;
        let extension = icon_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let mime_type = match extension {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "ico" => "image/x-icon",
            "bmp" => "image/bmp",
            _ => "image/png",
        };

        if extension != "svg" && extension != "gif" {
            let img = image::load_from_memory(&content)
                .map_err(|e| format!("Failed to decode icon image: {}", e))?;
            if img.width() != img.height() {
                return Err(format!("Icon must be square, got {}x{}", img.width(), img.height()));
            }
            if img.width() > 2048 || img.height() > 2048 {
                return Err(format!(
                    "Icon size must not exceed 2048x2048, got {}x{}",
                    img.width(),
                    img.height()
                ));
            }
        }

        use base64::Engine;
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&content);
        Ok(format!("data:{};base64,{}", mime_type, base64_data))
    }

    fn get_css(&self, _manager: &PluginManager, plugin: &PluginInfo) -> Result<String, String> {
        let plugin_path = PathBuf::from(&plugin.path);
        let css_path = plugin_path.join("style.css");
        if !css_path.exists() {
            return Ok(String::new());
        }
        std::fs::read_to_string(&css_path).map_err(|e| format!("Failed to read CSS file: {}", e))
    }

    fn collect_update_version(&self, plugin: &PluginInfo) -> Option<(String, String)> {
        if !plugin.actions.can_check_update {
            return None;
        }
        Some((plugin.manifest.id.clone(), plugin.manifest.version.clone()))
    }

    fn notify_page_changed(&self, manager: &PluginManager, plugin_id: &str, path: &str) {
        let Some(runtimes) = Self::plugin_runtime(manager, plugin_id) else {
            return;
        };
        let Some(runtime) = runtimes.get(plugin_id) else {
            return;
        };
        if let Err(e) = runtime.call_lifecycle_with_arg("onPageChanged", path) {
            log_warn_ctx(
                "plugins.manager.notify",
                "notify_page_changed",
                &format!("failed to call onPageChanged for plugin '{}': {}", plugin_id, e),
            );
        }
    }

    fn notify_locale_changed(&self, manager: &PluginManager, plugin_id: &str, locale: &str) {
        let Some(runtimes) = Self::plugin_runtime(manager, plugin_id) else {
            return;
        };
        let Some(runtime) = runtimes.get(plugin_id) else {
            return;
        };
        if let Err(e) = runtime.call_lifecycle_with_arg("onLocaleChanged", locale) {
            log_warn_ctx(
                "plugins.manager.notify",
                "notify_locale_changed",
                &format!("failed to call onLocaleChanged for plugin '{}': {}", plugin_id, e),
            );
            if let Err(error) = emit_log_event(
                plugin_id,
                "error",
                &format!("Failed to call onLocaleChanged: {}", e),
            ) {
                log_error_ctx(
                    "plugins.manager.notify",
                    "notify_locale_changed",
                    &format!(
                        "plugin locale change error log emit failed: plugin_id={} error={}",
                        plugin_id, error
                    ),
                );
            }
        }
    }

    fn notify_server_event(
        &self,
        manager: &PluginManager,
        plugin_id: &str,
        event: &ServerEventEnvelope,
    ) {
        let Some(runtimes) = Self::plugin_runtime(manager, plugin_id) else {
            return;
        };
        let Some(runtime) = runtimes.get(plugin_id) else {
            return;
        };
        if let Err(error) = runtime.notify_server_event(event) {
            log_warn_ctx(
                "plugins.manager.notify",
                "notify_server_event",
                &format!("failed to call onServerEvent for plugin '{}': {}", plugin_id, error),
            );
            if let Err(log_error) = emit_log_event(
                plugin_id,
                "error",
                &format!("Failed to call onServerEvent: {}", error),
            ) {
                log_error_ctx(
                    "plugins.manager.notify",
                    "notify_server_event",
                    &format!(
                        "plugin server event error log emit failed: plugin_id={} error={}",
                        plugin_id, log_error
                    ),
                );
            }
        }
    }

    fn notify_context_menu_show(
        &self,
        manager: &PluginManager,
        plugin_id: &str,
        context: &str,
        target_data: &serde_json::Value,
        x: f64,
        y: f64,
    ) {
        let Some(runtimes) = Self::plugin_runtime(manager, plugin_id) else {
            return;
        };
        let Some(runtime) = runtimes.get(plugin_id) else {
            return;
        };
        let _ = runtime.call_context_menu_show_callback(context, target_data.clone(), x, y);
    }

    fn notify_context_menu_hide(&self, manager: &PluginManager, plugin_id: &str) {
        let Some(runtimes) = Self::plugin_runtime(manager, plugin_id) else {
            return;
        };
        let Some(runtime) = runtimes.get(plugin_id) else {
            return;
        };
        let _ = runtime.call_context_menu_hide_callback();
    }

    fn dispatch_context_menu_callback(
        &self,
        manager: &PluginManager,
        plugin_id: &str,
        context: &str,
        item_id: &str,
        target_data: serde_json::Value,
    ) -> Result<(), String> {
        let Some(runtimes) = Self::plugin_runtime(manager, plugin_id) else {
            return Err(format!("插件 '{}' 的运行时不存在", plugin_id));
        };
        let Some(runtime) = runtimes.get(plugin_id) else {
            return Err(format!("插件 '{}' 的运行时不存在", plugin_id));
        };
        runtime.call_context_menu_callback(context, item_id, target_data)
    }
}
