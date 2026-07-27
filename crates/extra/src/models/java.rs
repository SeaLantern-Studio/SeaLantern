//! Java 信息模型

use serde::{Deserialize, Serialize};

/// Java 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaInfo {
    pub version: String,
    pub path: String,
    #[serde(default)]
    pub major_version: Option<u32>,
}