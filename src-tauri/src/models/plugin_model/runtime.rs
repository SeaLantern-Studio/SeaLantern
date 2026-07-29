use serde::{Deserialize, Serialize};

use super::manifest::PluginManifest;

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
