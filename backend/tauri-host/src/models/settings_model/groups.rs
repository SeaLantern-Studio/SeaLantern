use super::schema::{AppSettings, SettingsGroup};

impl AppSettings {
    pub fn get_changed_groups(&self, other: &AppSettings) -> Vec<SettingsGroup> {
        let mut changed = Vec::new();

        if self.close_servers_on_exit != other.close_servers_on_exit
            || self.close_servers_on_update != other.close_servers_on_update
            || self.auto_accept_eula != other.auto_accept_eula
            || self.enable_desktop_web_ui != other.enable_desktop_web_ui
            || self.close_action != other.close_action
            || self.language != other.language
            || self.locale_layer_order != other.locale_layer_order
            || self.auto_check_update != other.auto_check_update
        {
            changed.push(SettingsGroup::General);
        }

        if self.default_max_memory != other.default_max_memory
            || self.default_min_memory != other.default_min_memory
            || self.default_port != other.default_port
            || self.default_java_path != other.default_java_path
            || self.default_jvm_args != other.default_jvm_args
            || self.default_cpu_policy != other.default_cpu_policy
            || self.default_jvm_preset != other.default_jvm_preset
            || self.cached_java_list != other.cached_java_list
        {
            changed.push(SettingsGroup::ServerDefaults);
        }

        if self.console_font_size != other.console_font_size
            || self.console_font_family != other.console_font_family
            || self.console_letter_spacing != other.console_letter_spacing
            || self.max_log_lines != other.max_log_lines
        {
            changed.push(SettingsGroup::Console);
        }

        if self.background_image != other.background_image
            || self.background_opacity != other.background_opacity
            || self.background_blur != other.background_blur
            || self.background_brightness != other.background_brightness
            || self.background_size != other.background_size
            || self.window_effect != other.window_effect
            || self.acrylic_enabled != other.acrylic_enabled
            || self.theme != other.theme
            || self.color != other.color
            || self.font_size != other.font_size
            || self.font_family != other.font_family
            || self.memory_display_precision != other.memory_display_precision
            || self.text_color_overrides != other.text_color_overrides
            || self.app_display_name != other.app_display_name
            || self.minimal_mode != other.minimal_mode
            || self.next_home_layout != other.next_home_layout
        {
            changed.push(SettingsGroup::Appearance);
        }

        if self.window_width != other.window_width
            || self.window_height != other.window_height
            || self.window_x != other.window_x
            || self.window_y != other.window_y
            || self.window_maximized != other.window_maximized
        {
            changed.push(SettingsGroup::Window);
        }

        if self.developer_mode != other.developer_mode {
            changed.push(SettingsGroup::Developer);
        }

        if self.plugin_console_allowed_commands != other.plugin_console_allowed_commands
            || self.plugin_console_blocked_commands != other.plugin_console_blocked_commands
        {
            changed.push(SettingsGroup::PluginConsoleCommands);
        }

        if self.onebot_11 != other.onebot_11 {
            changed.push(SettingsGroup::Online);
        }

        changed
    }
}
