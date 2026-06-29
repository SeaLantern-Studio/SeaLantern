use serde::{Deserialize, Serialize};

use super::manifest::PluginManifest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginSource {
    Local,
    Builtin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginRuntimeKind {
    Lua,
    Rust,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginActions {
    #[serde(default = "default_true")]
    pub can_toggle: bool,
    #[serde(default = "default_true")]
    pub can_delete: bool,
    #[serde(default = "default_true")]
    pub can_check_update: bool,
}

/// 插件当前状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PluginState {
    Loaded,
    Enabled,
    Disabled,
    Error(String),
}

/// 已载入插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub path: String,
    #[serde(default = "default_plugin_source")]
    pub source: PluginSource,
    #[serde(default = "default_plugin_runtime_kind")]
    pub runtime: PluginRuntimeKind,
    #[serde(default = "default_plugin_actions")]
    pub actions: PluginActions,
    #[serde(default)]
    pub missing_dependencies: Vec<MissingDependency>,
}

/// 缺失依赖信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingDependency {
    pub id: String,
    pub version_requirement: Option<String>,
    pub required: bool,
}

/// 单个插件安装结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallResult {
    pub plugin: PluginInfo,
    pub missing_dependencies: Vec<MissingDependency>,
    #[serde(default)]
    pub untrusted_url: bool,
}

/// 批量安装里的单项失败信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInstallError {
    pub path: String,
    pub error: String,
}

/// 批量插件安装结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInstallResult {
    pub success: Vec<PluginInstallResult>,
    pub failed: Vec<BatchInstallError>,
}

fn default_true() -> bool {
    true
}

fn default_plugin_source() -> PluginSource {
    PluginSource::Local
}

fn default_plugin_runtime_kind() -> PluginRuntimeKind {
    PluginRuntimeKind::Lua
}

fn default_plugin_actions() -> PluginActions {
    PluginActions {
        can_toggle: true,
        can_delete: true,
        can_check_update: true,
    }
}
