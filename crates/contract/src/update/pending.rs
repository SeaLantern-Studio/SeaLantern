//! 已下载、待安装的更新模型。

use serde::{Deserialize, Serialize};

/// 已下载但尚未安装的更新。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingUpdate {
    pub file_path: String,
    pub version: String,
}
