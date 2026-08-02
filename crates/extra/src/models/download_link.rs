//! 服务器核心下载链接模型。

use serde::{Deserialize, Serialize};

/// 单个下载链接。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadLink {
    pub version: String,
    pub file_name: String,
    pub url: String,
}

impl DownloadLink {
    pub fn new(version: String, file_name: String, url: String) -> Self {
        Self { version, file_name, url }
    }
}

/// 同一服务器核心类型的版本和下载链接。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDownloadLinks {
    pub server_type: String,
    pub versions: Vec<String>,
    pub links: Vec<DownloadLink>,
}

impl TypeDownloadLinks {
    pub fn new(server_type: String, versions: Vec<String>, links: Vec<DownloadLink>) -> Self {
        Self { server_type, versions, links }
    }

    pub fn get_versions(&self) -> Vec<String> {
        self.versions.clone()
    }

    pub fn get_link_by_version(&self, version: &str) -> Option<&DownloadLink> {
        self.links.iter().find(|link| link.version == version)
    }
}

/// 所有服务器核心类型的下载链接数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseDownloadLinks {
    pub server_types: Vec<String>,
    pub links: Vec<TypeDownloadLinks>,
}

impl BaseDownloadLinks {
    pub fn new(server_types: Vec<String>, links: Vec<TypeDownloadLinks>) -> Self {
        Self { server_types, links }
    }

    pub fn get_types(&self) -> Vec<String> {
        self.server_types.clone()
    }

    pub fn get_type_by_name(&self, type_name: &str) -> Option<&TypeDownloadLinks> {
        self.links.iter().find(|link| link.server_type == type_name)
    }
}

#[cfg(test)]
mod tests {
    use super::DownloadLink;

    #[test]
    fn download_link_uses_frontend_field_names() {
        let value = serde_json::to_value(DownloadLink::new(
            "1.21.1".to_string(),
            "server.jar".to_string(),
            "https://example.invalid/server.jar".to_string(),
        ))
        .expect("download link should serialize");

        assert_eq!(value["fileName"], "server.jar");
        assert!(value.get("file_name").is_none());
    }
}
