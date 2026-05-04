use serde::{Deserialize, Serialize};

use super::dependencies::PluginDependency;

/// 插件权限名
pub type PluginPermission = String;

/// 插件清单
///
/// 对应插件目录里的 `manifest.json`
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

/// 插件语言条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginLocaleEntry {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// 插件设置字段
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

/// 插件设置选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettingOption {
    pub value: String,
    pub label: String,
}

/// 插件作者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAuthor {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

/// 插件运行环境要求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEngines {
    #[serde(default)]
    pub sealantern: Option<String>,
}

/// 插件 UI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUiConfig {
    #[serde(default)]
    pub pages: Vec<PluginPage>,
    #[serde(default)]
    pub sidebar: Option<PluginSidebarConfig>,
    #[serde(default, rename = "contextMenus")]
    pub context_menus: Vec<PluginContextMenu>,
}

/// 插件页面配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPage {
    pub id: String,
    pub title: String,
    pub path: String,
    #[serde(default)]
    pub icon: Option<String>,
}

/// 插件侧边栏入口配置
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

/// 侧边栏模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SidebarMode {
    #[default]
    None,
    #[serde(rename = "self")]
    SelfPage,
    Category,
}

/// 侧边栏分类配置
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

/// 插件右键菜单配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContextMenu {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub contexts: Vec<String>,
}

/// 插件命令配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub shortcut: Option<String>,
}

fn default_show_dependents() -> bool {
    true
}
