//! 服务器实例持久化模型。

use sealantern_core::instance::Instance;
use serde::{Deserialize, Serialize};

/// 实例列表的持久化包装。
///
/// 实例本体由 `sealantern-core` 维护，`extra` 只拥有存储格式版本和集合边界。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InstanceList {
    pub version: u32,
    pub instances: Vec<Instance>,
}

impl Default for InstanceList {
    fn default() -> Self {
        Self { version: 1, instances: Vec::new() }
    }
}
