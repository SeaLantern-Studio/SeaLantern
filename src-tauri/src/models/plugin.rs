use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Default)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
}

impl SemVer {
    pub fn parse(version: &str) -> Option<Self> {
        let clean = version.trim().trim_start_matches('v');
        if clean.is_empty() {
            return None;
        }
        let parts: Vec<&str> = clean.split('.').collect();

        let major = parts.first().and_then(|s| s.parse().ok())?;
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

        let (patch, prerelease) = if let Some(patch_str) = parts.get(2) {
            if let Some(idx) = patch_str.find('-') {
                let (p, pre) = patch_str.split_at(idx);
                (p.parse().unwrap_or(0), Some(pre[1..].to_string()))
            } else {
                (patch_str.parse().unwrap_or(0), None)
            }
        } else {
            (0, None)
        };
        Some(Self { major, minor, patch, prerelease })
    }

    pub fn satisfies(&self, requirement: &str) -> bool {
        let req = requirement.trim();

        if let Some(stripped) = req.strip_prefix(">=") {
            if let Some(target) = SemVer::parse(stripped) {
                return self.compare(&target) != Ordering::Less;
            }
        } else if let Some(stripped) = req.strip_prefix('>') {
            if let Some(target) = SemVer::parse(stripped) {
                return self.compare(&target) == Ordering::Greater;
            }
        } else if let Some(stripped) = req.strip_prefix("<=") {
            if let Some(target) = SemVer::parse(stripped) {
                return self.compare(&target) != Ordering::Greater;
            }
        } else if let Some(stripped) = req.strip_prefix('<') {
            if let Some(target) = SemVer::parse(stripped) {
                return self.compare(&target) == Ordering::Less;
            }
        } else if let Some(stripped) = req.strip_prefix('=') {
            if let Some(target) = SemVer::parse(stripped) {
                return self.compare(&target) == Ordering::Equal;
            }
        } else if let Some(stripped) = req.strip_prefix('^') {
            if let Some(target) = SemVer::parse(stripped) {
                if self.compare(&target) == Ordering::Less {
                    return false;
                }
                if target.major == 0 {
                    if target.minor == 0 {
                        return self.major == 0 && self.minor == 0 && self.patch == target.patch;
                    }
                    return self.major == 0 && self.minor == target.minor;
                }
                return self.major == target.major;
            }
        } else if let Some(stripped) = req.strip_prefix('~') {
            if let Some(target) = SemVer::parse(stripped) {
                if self.compare(&target) == Ordering::Less {
                    return false;
                }
                return self.major == target.major && self.minor == target.minor;
            }
        } else if let Some(target) = SemVer::parse(req) {
            return self.compare(&target) == Ordering::Equal;
        }

        false
    }

    fn compare(&self, other: &SemVer) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }

        match (&self.prerelease, &other.prerelease) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginDependency {
    Simple(String),

    WithVersion {
        id: String,
        #[serde(default)]
        version: Option<String>,
    },
}

impl PluginDependency {
    pub fn id(&self) -> &str {
        match self {
            PluginDependency::Simple(id) => id,
            PluginDependency::WithVersion { id, .. } => id,
        }
    }

    pub fn version_requirement(&self) -> Option<&str> {
        match self {
            PluginDependency::Simple(_) => None,
            PluginDependency::WithVersion { version, .. } => version.as_deref(),
        }
    }

    pub fn is_satisfied_by(&self, version: &str) -> bool {
        match self.version_requirement() {
            Some(req) => {
                if let Some(ver) = SemVer::parse(version) {
                    ver.satisfies(req)
                } else {
                    false
                }
            }
            None => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdateInfo {
    pub plugin_id: String,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: Option<String>,
    pub changelog: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPluginInfo {
    pub id: String,
    pub name: serde_json::Value,
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default = "default_empty_value")]
    pub description: serde_json::Value,
    #[serde(default)]
    pub author: MarketAuthorInfo,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub download_url: Option<String>,
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub download_type: Option<String>,
    #[serde(default)]
    pub release_asset: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,

    #[serde(default, skip_deserializing)]
    pub _path: Option<String>,
}

fn default_empty_value() -> serde_json::Value {
    serde_json::Value::String(String::new())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketAuthorInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PluginState {
    Loaded,
    Enabled,
    Disabled,
    Error(String),
}

pub type PluginPermission = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum PermissionDangerLevel {
    Normal,

    Dangerous,

    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PermissionMeta {
    pub id: &'static str,

    pub name: &'static str,

    pub description: &'static str,

    pub danger_level: PermissionDangerLevel,

    pub icon: &'static str,
}

#[allow(dead_code)]
pub fn get_all_permission_meta() -> Vec<PermissionMeta> {
    vec![
        PermissionMeta {
            id: "log",
            name: "日志",
            description: "允许使用日志 API 输出调试信息",
            danger_level: PermissionDangerLevel::Normal,
            icon: "scroll-text",
        },
        PermissionMeta {
            id: "storage",
            name: "存储",
            description: "允许使用持久化存储 API",
            danger_level: PermissionDangerLevel::Normal,
            icon: "database",
        },
        PermissionMeta {
            id: "ui",
            name: "界面控制",
            description: "允许操作应用界面元素",
            danger_level: PermissionDangerLevel::Normal,
            icon: "layout",
        },
        PermissionMeta {
            id: "element",
            name: "元素交互",
            description: "允许读取和操作页面元素",
            danger_level: PermissionDangerLevel::Normal,
            icon: "mouse-pointer",
        },
        PermissionMeta {
            id: "fs",
            name: "文件系统",
            description: "允许在插件自身目录内读写文件",
            danger_level: PermissionDangerLevel::Dangerous,
            icon: "folder",
        },
        PermissionMeta {
            id: "api",
            name: "插件间通信",
            description: "允许调用其他插件注册的 API",
            danger_level: PermissionDangerLevel::Normal,
            icon: "plug",
        },
        PermissionMeta {
            id: "network",
            name: "网络访问",
            description: "允许发送 HTTP 请求",
            danger_level: PermissionDangerLevel::Dangerous,
            icon: "globe",
        },
        PermissionMeta {
            id: "system",
            name: "系统信息",
            description: "允许获取操作系统和硬件信息",
            danger_level: PermissionDangerLevel::Normal,
            icon: "cpu",
        },
        PermissionMeta {
            id: "server",
            name: "服务器管理",
            description: "允许管理 Minecraft 服务器",
            danger_level: PermissionDangerLevel::Dangerous,
            icon: "server",
        },
        PermissionMeta {
            id: "console",
            name: "控制台",
            description: "允许向服务器控制台发送命令",
            danger_level: PermissionDangerLevel::Dangerous,
            icon: "terminal",
        },
        PermissionMeta {
            id: "execute_program",
            name: "程序执行",
            description: "允许在插件目录内执行外部程序，可能带来严重安全风险",
            danger_level: PermissionDangerLevel::Critical,
            icon: "shield-alert",
        },
        PermissionMeta {
            id: "plugin_folder_access",
            name: "插件文件夹访问",
            description: "允许读写其他插件的文件和数据，可能影响其他插件的正常运行",
            danger_level: PermissionDangerLevel::Critical,
            icon: "folder-key",
        },
        PermissionMeta {
            id: "ui.component.read",
            name: "UI 组件读取",
            description: "允许读取页面内已注册的 UI 组件属性、列表及监听事件",
            danger_level: PermissionDangerLevel::Normal,
            icon: "eye",
        },
        PermissionMeta {
            id: "ui.component.write",
            name: "UI 组件写入",
            description: "允许修改页面内已注册的 UI 组件属性并调用其方法",
            danger_level: PermissionDangerLevel::Dangerous,
            icon: "pencil",
        },
        PermissionMeta {
            id: "ui.component.proxy",
            name: "UI 组件代理",
            description: "允许拦截并修改 UI 组件的交互事件，可阻断原始行为，需用户确认",
            danger_level: PermissionDangerLevel::Critical,
            icon: "shield",
        },
        PermissionMeta {
            id: "ui.component.create",
            name: "UI 组件创建",
            description: "允许插件动态创建新的 UI 组件并渲染到应用中",
            danger_level: PermissionDangerLevel::Dangerous,
            icon: "layout-grid",
        },
    ]
}

#[allow(dead_code)]
pub fn get_permission_danger_level(permission_id: &str) -> PermissionDangerLevel {
    match permission_id {
        "execute_program" | "plugin_folder_access" | "ui.component.proxy" => {
            PermissionDangerLevel::Critical
        }
        "fs" | "network" | "server" | "console" | "ui.component.write" | "ui.component.create" => {
            PermissionDangerLevel::Dangerous
        }
        _ => PermissionDangerLevel::Normal,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: PluginAuthor,
    pub main: String,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub engines: Option<PluginEngines>,
    #[serde(default)]
    pub permissions: Vec<PluginPermission>,
    #[serde(default)]
    pub ui: Option<PluginUiConfig>,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub commands: Vec<PluginCommand>,

    #[serde(default)]
    pub dependencies: Vec<PluginDependency>,

    #[serde(default)]
    pub optional_dependencies: Vec<PluginDependency>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub settings: Option<Vec<PluginSettingField>>,

    #[serde(default)]
    pub sidebar: Option<SidebarCategoryConfig>,

    #[serde(default)]
    pub locales: Option<std::collections::HashMap<String, PluginLocaleEntry>>,

    #[serde(default)]
    pub include: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginLocaleEntry {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettingField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub display: Option<String>,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub options: Option<Vec<PluginSettingOption>>,

    #[serde(default)]
    pub rows: Option<u32>,

    #[serde(default)]
    pub maxlength: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettingOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAuthor {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEngines {
    #[serde(default)]
    pub sealantern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUiConfig {
    #[serde(default)]
    pub pages: Vec<PluginPage>,
    #[serde(default)]
    pub sidebar: Option<PluginSidebarConfig>,
    #[serde(default, rename = "contextMenus")]
    pub context_menus: Vec<PluginContextMenu>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPage {
    pub id: String,
    pub title: String,
    pub path: String,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSidebarConfig {
    #[serde(default)]
    pub group: Option<String>,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SidebarMode {
    #[default]
    None,

    #[serde(rename = "self")]
    SelfPage,

    Category,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarCategoryConfig {
    #[serde(default)]
    pub mode: SidebarMode,

    pub label: String,

    #[serde(default)]
    pub icon: Option<String>,

    #[serde(default = "default_show_dependents")]
    pub show_dependents: bool,

    #[serde(default)]
    pub priority: Option<i32>,

    #[serde(default)]
    pub after: Option<String>,
}

fn default_show_dependents() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContextMenu {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub contexts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub shortcut: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub path: String,

    #[serde(default)]
    pub missing_dependencies: Vec<MissingDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingDependency {
    pub id: String,

    pub version_requirement: Option<String>,

    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallResult {
    pub plugin: PluginInfo,

    pub missing_dependencies: Vec<MissingDependency>,

    #[serde(default)]
    pub untrusted_url: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInstallError {
    pub path: String,

    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInstallResult {
    pub success: Vec<PluginInstallResult>,

    pub failed: Vec<BatchInstallError>,
}
