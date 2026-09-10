//! 服务器核心下载链接模型。
//!
//! 单个服务器核心下载链接模型，字段统一采用 snake_case 命名，便于 JSON 序列化交换。

use serde::{Deserialize, Serialize};

/// 单个下载链接，对应某 Minecraft 版本下的一个可下载文件。
///
/// 序列化时字段统一使用 snake_case 命名，如 `file_name`。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DownloadLink {
    /// Minecraft 版本号，如 `1.21.1`。
    pub version: String,
    /// 下载文件的名称，如 `server.jar`。
    pub file_name: String,
    /// 下载地址。
    pub url: String,
}

impl DownloadLink {
    /// 构造单个下载链接。
    pub fn new(version: String, file_name: String, url: String) -> Self {
        Self { version, file_name, url }
    }
}

#[cfg(test)]
mod tests {
    use super::DownloadLink;

    // 校验序列化输出统一使用 snake_case 字段名。
    #[test]
    fn download_link_uses_snake_case_field_names() {
        let value = serde_json::to_value(DownloadLink::new(
            "1.21.1".to_string(),
            "server.jar".to_string(),
            "https://example.invalid/server.jar".to_string(),
        ))
        .expect("download link should serialize");

        assert_eq!(value["file_name"], "server.jar");
        assert!(value.get("fileName").is_none());
    }
}
