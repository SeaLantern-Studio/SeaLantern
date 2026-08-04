//! 旧公开模型路径的兼容定义。

#![allow(deprecated)]

use serde::{Deserialize, Serialize};

/// 旧服务器实例信息。
#[deprecated(note = "请使用 sealantern_core::instance::Instance")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInstance {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub core_type: Option<String>,
    #[serde(default)]
    pub mc_version: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
}

/// 旧服务器运行时状态。
#[deprecated(note = "请在实际运行时 API 边界定义状态模型")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

#[allow(deprecated)]
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

/// 旧启动模式。
#[deprecated(note = "请使用 sealantern_core::instance::StartupMode")]
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

#[allow(deprecated)]
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

/// 旧运行状态快照。
#[deprecated(note = "请在实际运行时 API 边界定义状态响应")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusInfo {
    pub id: String,
    pub status: ServerStatus,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
}

/// 旧服务器导入请求。
#[deprecated(note = "请使用 sealantern_core::instance::InstanceImportRequest")]
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

/// 旧启动候选扫描结果。
#[deprecated(note = "请在实际导入 API 边界定义扫描响应")]
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

/// 旧核心解析结果。
#[deprecated(note = "请在实际导入 API 边界定义解析响应")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCoreInfo {
    pub core_type: String,
    pub main_class: Option<String>,
    pub jar_path: Option<String>,
}

/// 旧启动候选项。
#[deprecated(note = "请在实际导入 API 边界定义候选项")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupCandidate {
    pub id: String,
    pub mode: String,
    pub label: String,
    pub detail: String,
    pub path: String,
    pub recommended: u8,
}

/// 旧服务器路径验证结果。
#[deprecated(note = "请在实际导入 API 边界定义验证响应")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatePathResult {
    pub valid: bool,
    pub message: String,
    pub jar_path: Option<String>,
    pub startup_mode: Option<String>,
}

/// 旧配置文件发现结果。
#[deprecated(note = "请在实际配置发现 API 边界定义响应")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredConfig {
    pub relative_path: String,
    pub kind: String,
    pub known_role: Option<String>,
}
