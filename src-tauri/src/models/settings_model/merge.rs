use super::schema::{AppSettings, PartialSettings};

impl AppSettings {
    pub fn merge_from(&mut self, partial: &PartialSettings) {
        if let Some(v) = partial.close_servers_on_exit {
            self.close_servers_on_exit = v;
        }
        if let Some(v) = partial.close_servers_on_update {
            self.close_servers_on_update = v;
        }
        if let Some(v) = partial.auto_accept_eula {
            self.auto_accept_eula = v;
        }
        if let Some(v) = partial.default_max_memory {
            self.default_max_memory = v;
        }
        if let Some(v) = partial.default_min_memory {
            self.default_min_memory = v;
        }
        if let Some(v) = partial.default_port {
            self.default_port = v;
        }
        if let Some(ref v) = partial.default_java_path {
            self.default_java_path = v.clone();
        }
        if let Some(ref v) = partial.default_jvm_args {
            self.default_jvm_args = v.clone();
        }
        if let Some(v) = partial.console_font_size {
            self.console_font_size = v;
        }
        if let Some(ref v) = partial.console_font_family {
            self.console_font_family = v.clone();
        }
        if let Some(v) = partial.console_letter_spacing {
            self.console_letter_spacing = v;
        }
        if let Some(v) = partial.max_log_lines {
            self.max_log_lines = v;
        }
        if let Some(ref v) = partial.cached_java_list {
            self.cached_java_list = v.clone();
        }
        if let Some(ref v) = partial.background_image {
            self.background_image = v.clone();
        }
        if let Some(v) = partial.background_opacity {
            self.background_opacity = v;
        }
        if let Some(v) = partial.background_blur {
            self.background_blur = v;
        }
        if let Some(v) = partial.background_brightness {
            self.background_brightness = v;
        }
        if let Some(ref v) = partial.background_size {
            self.background_size = v.clone();
        }
        if let Some(v) = partial.window_width {
            self.window_width = v;
        }
        if let Some(v) = partial.window_height {
            self.window_height = v;
        }
        if let Some(ref v) = partial.window_x {
            self.window_x = *v;
        }
        if let Some(ref v) = partial.window_y {
            self.window_y = *v;
        }
        if let Some(v) = partial.window_maximized {
            self.window_maximized = v;
        }
        if let Some(v) = partial.acrylic_enabled {
            self.acrylic_enabled = v;
        }
        if let Some(ref v) = partial.theme {
            self.theme = v.clone();
        }
        if let Some(ref v) = partial.color {
            self.color = v.clone();
        }
        if let Some(v) = partial.font_size {
            self.font_size = v;
        }
        if let Some(ref v) = partial.font_family {
            self.font_family = v.clone();
        }
        if let Some(ref v) = partial.language {
            self.language = v.clone();
        }
        if let Some(v) = partial.developer_mode {
            self.developer_mode = v;
        }
        if let Some(ref v) = partial.close_action {
            self.close_action = v.clone();
        }
        if let Some(ref v) = partial.last_run_path {
            self.last_run_path = v.clone();
        }
        if let Some(v) = partial.minimal_mode {
            self.minimal_mode = v;
        }
        if let Some(ref v) = partial.plugin_allowed_commands {
            self.plugin_allowed_commands = v.clone();
        }
        if let Some(ref v) = partial.plugin_blocked_commands {
            self.plugin_blocked_commands = v.clone();
        }
        if let Some(v) = partial.agreed_to_terms {
            self.agreed_to_terms = v;
        }
    }
}
