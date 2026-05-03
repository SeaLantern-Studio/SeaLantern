use super::common::{handle_unsupported, CommandHandler};
use super::{config, java, player, server, settings, system, tunnel, update};
use std::collections::HashMap;

/// 对外暴露的 HTTP 命令表。
pub struct CommandRegistry {
    handlers: HashMap<String, CommandHandler>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut handlers: HashMap<String, CommandHandler> = HashMap::new();

        server::register_handlers(&mut handlers);
        java::register_handlers(&mut handlers);
        config::register_handlers(&mut handlers);
        system::register_handlers(&mut handlers);
        player::register_handlers(&mut handlers);
        settings::register_handlers(&mut handlers);
        tunnel::register_handlers(&mut handlers);
        update::register_handlers(&mut handlers);

        for cmd in [
            "list_plugins",
            "scan_plugins",
            "enable_plugin",
            "disable_plugin",
            "get_plugin_nav_items",
            "install_plugin",
            "get_plugin_icon",
            "get_plugin_settings",
            "set_plugin_settings",
            "get_plugin_css",
            "get_all_plugin_css",
            "delete_plugin",
            "delete_plugins",
            "check_plugin_update",
            "check_all_plugin_updates",
            "fetch_market_plugins",
            "fetch_market_categories",
            "fetch_market_plugin_detail",
            "install_from_market",
            "install_plugins_batch",
            "context_menu_callback",
            "context_menu_show_notify",
            "context_menu_hide_notify",
            "on_locale_changed",
            "component_mirror_register",
            "component_mirror_unregister",
            "component_mirror_clear",
            "on_page_changed",
            "get_plugin_component_snapshot",
            "get_plugin_ui_snapshot",
            "get_plugin_sidebar_snapshot",
            "get_plugin_context_menu_snapshot",
        ] {
            handlers.insert(cmd.to_string(), handle_unsupported as CommandHandler);
        }

        Self { handlers }
    }

    pub fn get_handler(&self, command: &str) -> Option<&CommandHandler> {
        self.handlers.get(command)
    }

    pub fn list_commands(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
