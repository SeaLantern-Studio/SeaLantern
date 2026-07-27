//! 下载链接数据模型

use serde::{Deserialize, Serialize};

/// 单个下载链接
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 类型下载链接集合
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
        self.links.iter().find(|i| i.version == version)
    }
}

/// 基础下载链接数据
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
        self.links.iter().find(|i| i.server_type == type_name)
    }
}
