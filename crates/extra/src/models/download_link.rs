//! 服务器核心下载链接模型。
//!
//! 数据按「服务器核心类型 → Minecraft 版本 → 下载链接」三级结构组织，
//! 字段统一采用 snake_case 命名，便于 JSON 序列化交换。

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

/// 同一服务器核心类型的版本和下载链接。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDownloadLinks {
    /// 服务器核心类型名称。
    pub server_type: String,
    /// 该类型支持的 Minecraft 版本列表。
    pub versions: Vec<String>,
    /// 各版本对应的下载链接。
    pub links: Vec<DownloadLink>,
}

impl TypeDownloadLinks {
    /// 构造指定类型的版本与下载链接数据。
    pub fn new(server_type: String, versions: Vec<String>, links: Vec<DownloadLink>) -> Self {
        Self { server_type, versions, links }
    }

    /// 返回该类型支持的版本列表。
    pub fn get_versions(&self) -> Vec<String> {
        self.versions.clone()
    }

    /// 按版本号查找对应的下载链接。
    pub fn get_link_by_version(&self, version: &str) -> Option<&DownloadLink> {
        self.links.iter().find(|link| link.version == version)
    }
}

/// 所有服务器核心类型的下载链接数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseDownloadLinks {
    /// 全部服务器核心类型名称。
    pub server_types: Vec<String>,
    /// 各类型的版本与下载链接数据。
    pub links: Vec<TypeDownloadLinks>,
}

impl BaseDownloadLinks {
    /// 构造完整的下载链接数据集合。
    pub fn new(server_types: Vec<String>, links: Vec<TypeDownloadLinks>) -> Self {
        Self { server_types, links }
    }

    /// 返回全部服务器核心类型名称。
    pub fn get_types(&self) -> Vec<String> {
        self.server_types.clone()
    }

    /// 按类型名称查找对应的版本与下载链接数据。
    pub fn get_type_by_name(&self, type_name: &str) -> Option<&TypeDownloadLinks> {
        self.links.iter().find(|link| link.server_type == type_name)
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
