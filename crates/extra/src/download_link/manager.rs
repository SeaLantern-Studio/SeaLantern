//! 下载链接管理器实现

use super::{BaseDownloadLinks, DownloadLink, TypeDownloadLinks};
use sealantern_infra::download::fetch_to_bytes;
use sealantern_infra::net::client::{ClientConfig, NetClient};
use serde_json::Value;
use tokio::sync::{Mutex, OnceCell};

/// 下载链接配置 URL
const DOWNLOAD_LINK_LIST_URL: &str =
    "https://raw.githubusercontent.com/SeaLantern-Studio/SeaLanternData/main/server_download.json";

static DOWNLOAD_LINKS: OnceCell<BaseDownloadLinks> = OnceCell::const_new();
static INIT_LOCK: Mutex<()> = Mutex::const_new(());

/// 下载链接管理器
pub struct LinkManager;

impl LinkManager {
    /// 获取下载链接数据（懒加载）
    pub async fn get() -> Result<&'static BaseDownloadLinks, String> {
        if DOWNLOAD_LINKS.get().is_none() {
            let _guard = INIT_LOCK.lock().await;
            // 双重检查：在锁内再次确认
            if DOWNLOAD_LINKS.get().is_none() {
                let links = Self::init().await?;
                DOWNLOAD_LINKS.set(links).ok();
            }
        }

        DOWNLOAD_LINKS
            .get()
            .ok_or_else(|| "download links not initialized".to_string())
    }

    /// 从远程加载下载链接配置
    async fn init() -> Result<BaseDownloadLinks, String> {
        let config = ClientConfig {
            user_agent: crate::update::UPDATE_HTTP_USER_AGENT.to_string(),
            ..Default::default()
        };

        let client = NetClient::from_config(&config)
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response_body = fetch_to_bytes(&client, DOWNLOAD_LINK_LIST_URL)
            .await
            .map_err(|e| format!("Failed to download config: {}", e))?;

        let body_str = String::from_utf8_lossy(&response_body);
        let root_json: Value = serde_json::from_str(&body_str)
            .map_err(|e| format!("Failed to parse config: {}", e))?;

        let mut all_server_types = Vec::new();
        let mut type_download_groups = Vec::new();

        if let Some(type_name_list) = root_json.get("types").and_then(|t| t.as_array()) {
            for type_node in type_name_list {
                let server_type_name = type_node.as_str().unwrap_or_default().to_string();
                all_server_types.push(server_type_name.clone());

                if let Some(type_detail_data) = root_json.get(&server_type_name) {
                    let mut mc_versions_under_type = Vec::new();
                    let mut download_links_under_type = Vec::new();

                    if let Some(version_list_node) =
                        type_detail_data.get("versions").and_then(|v| v.as_array())
                    {
                        for version_node in version_list_node {
                            let mc_version_str =
                                version_node.as_str().unwrap_or_default().to_string();
                            mc_versions_under_type.push(mc_version_str.clone());

                            if let Some(file_mapping) = type_detail_data
                                .get(&mc_version_str)
                                .and_then(|f| f.as_object())
                            {
                                for (file_key_name, file_url_value) in file_mapping {
                                    let download_entry = DownloadLink::new(
                                        mc_version_str.clone(),
                                        file_key_name.clone(),
                                        file_url_value.as_str().unwrap_or_default().to_string(),
                                    );
                                    download_links_under_type.push(download_entry);
                                }
                            }
                        }
                    }

                    let type_group = TypeDownloadLinks::new(
                        server_type_name,
                        mc_versions_under_type,
                        download_links_under_type,
                    );
                    type_download_groups.push(type_group);
                }
            }
        }

        Ok(BaseDownloadLinks::new(all_server_types, type_download_groups))
    }

    /// 获取所有服务器类型
    pub async fn get_server_types() -> Result<Vec<String>, String> {
        Ok(Self::get().await?.get_types())
    }

    /// 根据名称获取类型详情
    pub async fn get_type_by_name(name: &str) -> Result<TypeDownloadLinks, String> {
        let links = Self::get().await?;
        links
            .get_type_by_name(name)
            .cloned()
            .ok_or_else(|| format!("Type {} not found", name))
    }

    /// 获取指定类型的版本列表
    pub async fn get_versions_by_type(type_name: &str) -> Result<Vec<String>, String> {
        Ok(Self::get_type_by_name(type_name).await?.get_versions())
    }
}
