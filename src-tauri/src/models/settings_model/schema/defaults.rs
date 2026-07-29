use super::AppSettings;

/// 创建默认设置
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            close_servers_on_exit: true,
            close_servers_on_update: true,
            auto_accept_eula: true,
            default_max_memory: 2048,
            default_min_memory: 512,
            default_port: 25565,
            default_java_path: String::new(),
            default_jvm_args: String::new(),
            console_font_size: 13,
            console_font_family: String::new(),
            console_letter_spacing: 0,
            max_log_lines: 5000,
            cached_java_list: Vec::new(),
            background_image: String::new(),
            background_opacity: 0.3,
            background_blur: 0,
            background_brightness: 1.0,
            background_size: "cover".to_string(),
            window_width: 1200,
            window_height: 720,
            window_x: None,
            window_y: None,
            window_maximized: false,
            acrylic_enabled: false,
            theme: "auto".to_string(),
            color: "default".to_string(),
            font_size: 14,
            font_family: String::new(),
            language: "zh-CN".to_string(),
            developer_mode: false,
            close_action: "ask".to_string(),
            last_run_path: String::new(),
            minimal_mode: false,
            plugin_allowed_commands: default_allowed_commands(),
            plugin_blocked_commands: default_blocked_commands(),
            agreed_to_terms: false,
        }
    }
}

pub(super) fn default_true() -> bool {
    true
}

pub(super) fn default_false() -> bool {
    false
}

pub(super) fn default_max_memory() -> u32 {
    2048
}

pub(super) fn default_min_memory() -> u32 {
    512
}

pub(super) fn default_port() -> u16 {
    25565
}

pub(super) fn default_console_font() -> u32 {
    13
}

pub(super) fn default_console_font_family() -> String {
    String::new()
}

pub(super) fn default_console_letter_spacing() -> i32 {
    0
}

pub(super) fn default_log_lines() -> u32 {
    5000
}

pub(super) fn default_bg_opacity() -> f32 {
    0.3
}

pub(super) fn default_bg_blur() -> u32 {
    0
}

pub(super) fn default_bg_brightness() -> f32 {
    1.0
}

pub(super) fn default_bg_size() -> String {
    "cover".to_string()
}

pub(super) fn default_window_width() -> u32 {
    1200
}

pub(super) fn default_window_height() -> u32 {
    720
}

pub(super) fn default_theme() -> String {
    "auto".to_string()
}

pub(super) fn default_color() -> String {
    "default".to_string()
}

pub(super) fn default_font_size() -> u32 {
    14
}

pub(super) fn default_font_family() -> String {
    String::new()
}

pub(super) fn default_language() -> String {
    "zh-CN".to_string()
}

pub(super) fn default_close_action() -> String {
    "ask".to_string()
}

pub(super) fn default_allowed_commands() -> Vec<String> {
    vec![
        "tell".to_string(),
        "msg".to_string(),
        "w".to_string(),
        "say".to_string(),
        "teammsg".to_string(),
        "me".to_string(),
        "give".to_string(),
        "clear".to_string(),
        "xp".to_string(),
        "experience".to_string(),
        "kick".to_string(),
        "ban".to_string(),
        "pardon".to_string(),
        "banlist".to_string(),
        "whitelist".to_string(),
        "op".to_string(),
        "deop".to_string(),
        "effect".to_string(),
        "enchant".to_string(),
        "time".to_string(),
        "weather".to_string(),
        "gamerule".to_string(),
        "difficulty".to_string(),
        "gamemode".to_string(),
        "spawnpoint".to_string(),
        "tp".to_string(),
        "teleport".to_string(),
        "spreadplayers".to_string(),
        "particle".to_string(),
        "playsound".to_string(),
        "title".to_string(),
    ]
}

pub(super) fn default_blocked_commands() -> Vec<String> {
    vec![
        "stop".to_string(),
        "reload".to_string(),
        "restart".to_string(),
        "plugins".to_string(),
        "plugin".to_string(),
        "version".to_string(),
        "debug".to_string(),
        "save-all".to_string(),
        "save-off".to_string(),
        "save-on".to_string(),
        "timings".to_string(),
        "perworldinventory".to_string(),
        "pwi".to_string(),
    ]
}
