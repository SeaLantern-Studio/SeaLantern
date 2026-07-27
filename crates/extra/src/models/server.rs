//! 服务器实例模型

use serde::{Deserialize, Serialize};

/// 服务器实例信息
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