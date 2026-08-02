//! 应用设置的模型定义与分组/变更检测类型。

use serde::{Deserialize, Serialize};

/// 当前配置版本号。
///
/// 每次配置结构变更（新增/删除/重命名字段）时递增，
/// 用于触发 `SettingsManager` 中的自动迁移。
pub const CURRENT_CONFIG_VERSION: u32 = 2;

// ---------------------------------------------------------------------------
// 设置分组
// ---------------------------------------------------------------------------

/// 设置变更分组，用于前端按组刷新 UI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsGroup {
    General,
    ServerDefaults,
    Console,
    Appearance,
    Window,
    Developer,
    PluginCommands,
}

// ---------------------------------------------------------------------------
// 应用设置（42 字段，覆盖完整旧版功能）
// ---------------------------------------------------------------------------

/// 完整的应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    /// 配置版本号，用于结构变更时触发迁移
    pub config_version: u32,

    // General
    pub close_servers_on_exit: bool,
    pub close_servers_on_update: bool,
    pub auto_accept_eula: bool,
    pub close_action: String,

    // ServerDefaults
    pub default_max_memory: u32,
    pub default_min_memory: u32,
    pub default_port: u16,
    pub default_java_path: String,
    pub default_jvm_args: String,
    pub cached_java_list: Vec<JavaInfo>,

    // Console
    pub console_font_size: u32,
    pub console_font_family: String,
    pub console_letter_spacing: i32,
    pub max_log_lines: u32,

    // Appearance
    pub background_image: String,
    pub background_opacity: f32,
    pub background_blur: u32,
    pub background_brightness: f32,
    pub background_size: String,
    pub acrylic_enabled: bool,
    pub theme: String,
    pub color: String,
    pub font_size: u32,
    pub font_family: String,
    pub minimal_mode: bool,

    // Window
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub window_maximized: Option<bool>,

    // Developer
    pub language: String,
    pub locales_base_url: Option<String>,
    pub developer_mode: bool,
    pub last_run_path: String,
    pub agreed_to_terms: bool,

    // PluginCommands
    pub plugin_allowed_commands: Vec<String>,
    pub plugin_blocked_commands: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            config_version: CURRENT_CONFIG_VERSION,
            close_servers_on_exit: true,
            close_servers_on_update: true,
            auto_accept_eula: true,
            close_action: "ask".into(),
            default_max_memory: 2048,
            default_min_memory: 512,
            default_port: 25565,
            default_java_path: String::new(),
            default_jvm_args: String::new(),
            cached_java_list: Vec::new(),
            console_font_size: 13,
            console_font_family: String::new(),
            console_letter_spacing: 0,
            max_log_lines: 5000,
            background_image: String::new(),
            background_opacity: 0.3,
            background_blur: 0,
            background_brightness: 1.0,
            background_size: "cover".into(),
            acrylic_enabled: false,
            theme: "auto".into(),
            color: "default".into(),
            font_size: 14,
            font_family: String::new(),
            minimal_mode: false,
            window_width: Some(1200),
            window_height: Some(720),
            window_x: None,
            window_y: None,
            window_maximized: Some(false),
            language: "zh-CN".into(),
            locales_base_url: None,
            developer_mode: false,
            last_run_path: String::new(),
            agreed_to_terms: false,
            plugin_allowed_commands: vec![],
            plugin_blocked_commands: vec![],
        }
    }
}

impl AppSettings {
    /// 返回与 `other` 相比发生变更的分组列表
    pub fn changed_groups(&self, other: &Self) -> Vec<SettingsGroup> {
        let mut groups = Vec::new();

        if self.close_servers_on_exit != other.close_servers_on_exit
            || self.close_servers_on_update != other.close_servers_on_update
            || self.auto_accept_eula != other.auto_accept_eula
            || self.close_action != other.close_action
        {
            groups.push(SettingsGroup::General);
        }

        if self.default_max_memory != other.default_max_memory
            || self.default_min_memory != other.default_min_memory
            || self.default_port != other.default_port
            || self.default_java_path != other.default_java_path
            || self.default_jvm_args != other.default_jvm_args
            || self.cached_java_list != other.cached_java_list
        {
            groups.push(SettingsGroup::ServerDefaults);
        }

        if self.console_font_size != other.console_font_size
            || self.console_font_family != other.console_font_family
            || self.console_letter_spacing != other.console_letter_spacing
            || self.max_log_lines != other.max_log_lines
        {
            groups.push(SettingsGroup::Console);
        }

        if self.background_image != other.background_image
            || self.background_opacity != other.background_opacity
            || self.background_blur != other.background_blur
            || self.background_brightness != other.background_brightness
            || self.background_size != other.background_size
            || self.acrylic_enabled != other.acrylic_enabled
            || self.theme != other.theme
            || self.color != other.color
            || self.font_size != other.font_size
            || self.font_family != other.font_family
            || self.minimal_mode != other.minimal_mode
        {
            groups.push(SettingsGroup::Appearance);
        }

        if self.window_width != other.window_width
            || self.window_height != other.window_height
            || self.window_x != other.window_x
            || self.window_y != other.window_y
            || self.window_maximized != other.window_maximized
        {
            groups.push(SettingsGroup::Window);
        }

        if self.language != other.language
            || self.locales_base_url != other.locales_base_url
            || self.developer_mode != other.developer_mode
            || self.last_run_path != other.last_run_path
            || self.agreed_to_terms != other.agreed_to_terms
        {
            groups.push(SettingsGroup::Developer);
        }

        if self.plugin_allowed_commands != other.plugin_allowed_commands
            || self.plugin_blocked_commands != other.plugin_blocked_commands
        {
            groups.push(SettingsGroup::PluginCommands);
        }

        groups
    }
}

// ---------------------------------------------------------------------------
// 部分更新支持
// ---------------------------------------------------------------------------

/// 部分更新请求 — 所有字段均为 `Option`，只处理 `Some` 的值
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartialAppSettings {
    // General
    pub close_servers_on_exit: Option<bool>,
    pub close_servers_on_update: Option<bool>,
    pub auto_accept_eula: Option<bool>,
    pub close_action: Option<String>,

    // ServerDefaults
    pub default_max_memory: Option<u32>,
    pub default_min_memory: Option<u32>,
    pub default_port: Option<u16>,
    pub default_java_path: Option<String>,
    pub default_jvm_args: Option<String>,
    pub cached_java_list: Option<Vec<JavaInfo>>,

    // Console
    pub console_font_size: Option<u32>,
    pub console_font_family: Option<String>,
    pub console_letter_spacing: Option<i32>,
    pub max_log_lines: Option<u32>,

    // Appearance
    pub background_image: Option<String>,
    pub background_opacity: Option<f32>,
    pub background_blur: Option<u32>,
    pub background_brightness: Option<f32>,
    pub background_size: Option<String>,
    pub acrylic_enabled: Option<bool>,
    pub theme: Option<String>,
    pub color: Option<String>,
    pub font_size: Option<u32>,
    pub font_family: Option<String>,
    pub minimal_mode: Option<bool>,

    // Window
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub window_maximized: Option<bool>,

    // Developer
    pub language: Option<String>,
    pub locales_base_url: Option<String>,
    pub developer_mode: Option<bool>,
    pub last_run_path: Option<String>,
    pub agreed_to_terms: Option<bool>,

    // PluginCommands
    pub plugin_allowed_commands: Option<Vec<String>>,
    pub plugin_blocked_commands: Option<Vec<String>>,
}

impl PartialAppSettings {
    /// 将部分更新合并到 `AppSettings` 中
    pub fn merge_into(&self, target: &mut AppSettings) {
        if let Some(v) = &self.close_servers_on_exit {
            target.close_servers_on_exit = *v;
        }
        if let Some(v) = &self.close_servers_on_update {
            target.close_servers_on_update = *v;
        }
        if let Some(v) = &self.auto_accept_eula {
            target.auto_accept_eula = *v;
        }
        if let Some(v) = &self.close_action {
            target.close_action = v.clone();
        }
        if let Some(v) = &self.default_max_memory {
            target.default_max_memory = *v;
        }
        if let Some(v) = &self.default_min_memory {
            target.default_min_memory = *v;
        }
        if let Some(v) = &self.default_port {
            target.default_port = *v;
        }
        if let Some(v) = &self.default_java_path {
            target.default_java_path = v.clone();
        }
        if let Some(v) = &self.default_jvm_args {
            target.default_jvm_args = v.clone();
        }
        if let Some(v) = &self.cached_java_list {
            target.cached_java_list = v.clone();
        }
        if let Some(v) = &self.console_font_size {
            target.console_font_size = *v;
        }
        if let Some(v) = &self.console_font_family {
            target.console_font_family = v.clone();
        }
        if let Some(v) = &self.console_letter_spacing {
            target.console_letter_spacing = *v;
        }
        if let Some(v) = &self.max_log_lines {
            target.max_log_lines = *v;
        }
        if let Some(v) = &self.background_image {
            target.background_image = v.clone();
        }
        if let Some(v) = &self.background_opacity {
            target.background_opacity = *v;
        }
        if let Some(v) = &self.background_blur {
            target.background_blur = *v;
        }
        if let Some(v) = &self.background_brightness {
            target.background_brightness = *v;
        }
        if let Some(v) = &self.background_size {
            target.background_size = v.clone();
        }
        if let Some(v) = &self.acrylic_enabled {
            target.acrylic_enabled = *v;
        }
        if let Some(v) = &self.theme {
            target.theme = v.clone();
        }
        if let Some(v) = &self.color {
            target.color = v.clone();
        }
        if let Some(v) = &self.font_size {
            target.font_size = *v;
        }
        if let Some(v) = &self.font_family {
            target.font_family = v.clone();
        }
        if let Some(v) = &self.minimal_mode {
            target.minimal_mode = *v;
        }
        if let Some(v) = &self.window_width {
            target.window_width = Some(*v);
        }
        if let Some(v) = &self.window_height {
            target.window_height = Some(*v);
        }
        if let Some(v) = &self.window_x {
            target.window_x = Some(*v);
        }
        if let Some(v) = &self.window_y {
            target.window_y = Some(*v);
        }
        if let Some(v) = &self.window_maximized {
            target.window_maximized = Some(*v);
        }
        if let Some(v) = &self.language {
            target.language = v.clone();
        }
        if let Some(v) = &self.locales_base_url {
            target.locales_base_url = Some(v.clone());
        }
        if let Some(v) = &self.developer_mode {
            target.developer_mode = *v;
        }
        if let Some(v) = &self.last_run_path {
            target.last_run_path = v.clone();
        }
        if let Some(v) = &self.agreed_to_terms {
            target.agreed_to_terms = *v;
        }
        if let Some(v) = &self.plugin_allowed_commands {
            target.plugin_allowed_commands = v.clone();
        }
        if let Some(v) = &self.plugin_blocked_commands {
            target.plugin_blocked_commands = v.clone();
        }
    }
}

// ---------------------------------------------------------------------------
// 更新结果
// ---------------------------------------------------------------------------

/// 设置更新的结果，包含更新后的设置和变更分组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    pub settings: AppSettings,
    pub changed_groups: Vec<SettingsGroup>,
}

// ---------------------------------------------------------------------------
// 辅助类型
// ---------------------------------------------------------------------------

/// Java 环境信息（用于缓存 Java 检测结果）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JavaInfo {
    pub path: String,
    pub version: String,
    pub vendor: String,
    pub is_64bit: bool,
    pub major_version: u32,
    /// Java 安装信息的规则置信度，范围为 0 到 100。
    #[serde(default)]
    pub confidence: u8,
}

#[cfg(test)]
mod tests {
    use super::JavaInfo;

    #[test]
    fn legacy_java_cache_defaults_confidence() {
        let info: JavaInfo = serde_json::from_str(
            r#"{
                "path": "/opt/jdk/bin/java",
                "version": "21.0.1",
                "vendor": "OpenJDK",
                "is_64bit": true,
                "major_version": 21
            }"#,
        )
        .expect("legacy Java info should remain readable");

        assert_eq!(info.confidence, 0);
    }
}

// ===========================================================================
// 服务器实例类型
// ===========================================================================

/// 服务器运行时状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

impl ServerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Error => "error",
        }
    }
}

/// 启动模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StartupMode {
    Jar,
    Bat,
    Sh,
    Ps1,
    Starter,
    Custom,
}

impl StartupMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Jar => "jar",
            Self::Bat => "bat",
            Self::Sh => "sh",
            Self::Ps1 => "ps1",
            Self::Starter => "starter",
            Self::Custom => "custom",
        }
    }
}

/// 实例列表的包装类型
///
/// 持久化 `core` 的领域模型 [`Instance`]，`version` 字段保留用于未来结构迁移。
///
/// [`Instance`]: sealantern_core::instance::Instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InstanceList {
    pub version: u32,
    pub instances: Vec<sealantern_core::instance::Instance>,
}

impl Default for InstanceList {
    fn default() -> Self {
        Self { version: 1, instances: Vec::new() }
    }
}

/// 运行状态快照（不持久化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusInfo {
    pub id: String,
    pub status: ServerStatus,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
}

/// 导入已有服务器的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRequest {
    pub name: String,
    pub jar_path: String,
    pub java_path: String,
    pub startup_mode: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub port: u16,
}

/// 扫描启动候选文件的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupScanResult {
    pub parsed_core: ParsedCoreInfo,
    pub candidates: Vec<StartupCandidate>,
    pub detected_core_type_key: Option<String>,
    pub core_type_options: Vec<String>,
    pub mc_version_options: Vec<String>,
    pub detected_mc_version: Option<String>,
    pub mc_version_detection_failed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCoreInfo {
    pub core_type: String,
    pub main_class: Option<String>,
    pub jar_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupCandidate {
    pub id: String,
    pub mode: String,
    pub label: String,
    pub detail: String,
    pub path: String,
    pub recommended: u8,
}

/// 服务器路径验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatePathResult {
    pub valid: bool,
    pub message: String,
    pub jar_path: Option<String>,
    pub startup_mode: Option<String>,
}

/// 配置文件发现结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredConfig {
    pub relative_path: String,
    pub kind: String,
    pub known_role: Option<String>,
}
