pub(crate) use crate::models::download::{
    BaseDownloadLinks, DownloadLink, LinkManager, TypeDownloadLinks,
};
use crate::utils::constants::USER_AGENT_EXAMPLE;
use crate::utils::downloader::SingleThreadDownloader;
use serde_json::Value;
use tokio::sync::OnceCell;

///此处常量见 utils/constants.rs
use crate::utils::constants::DOWNLOAD_LINK_LIST_URL;

static DOWNLOAD_LINKS: OnceCell<BaseDownloadLinks> = OnceCell::const_new();

impl LinkManager {
    pub async fn get() -> Result<&'static BaseDownloadLinks, String> {
        if DOWNLOAD_LINKS.get().is_none() {
            let links = Self::init().await?;
            DOWNLOAD_LINKS
                .set(links)
                .map_err(|_| "download links already initialized".to_string())?;
        }

        DOWNLOAD_LINKS
            .get()
            .ok_or_else(|| "download links not initialized".to_string())
    }

    pub async fn init() -> Result<BaseDownloadLinks, String> {
        let downloader = SingleThreadDownloader::new(USER_AGENT_EXAMPLE);
        let response_body = downloader.read_to_string(DOWNLOAD_LINK_LIST_URL).await?;

        let root_json: Value =
            serde_json::from_str(&response_body).map_err(|e| format!("解析下载配置失败: {}", e))?;

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

    pub async fn get_server_types() -> Result<Vec<String>, String> {
        Ok(Self::get().await?.get_types())
    }

    pub async fn get_type_by_name(name: &str) -> Result<TypeDownloadLinks, String> {
        let links = Self::get().await?;
        links
            .get_type_by_name(name)
            .cloned()
            .ok_or_else(|| format!("Type {} not found", name))
    }

    pub async fn get_versions_by_type(type_name: &str) -> Result<Vec<String>, String> {
        Ok(Self::get_type_by_name(type_name).await?.get_versions())
    }
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

impl DownloadLink {
    pub fn new(version: String, file_name: String, url: String) -> Self {
        Self { version, file_name, url }
    }
}

#[tokio::test]
pub async fn test() -> Result<(), String> {
    let data = LinkManager::get().await?;
    for i in &data.links {
        println!("==================");
        println!("type={}", i.server_type);
        println!("versions= | {}", i.versions.join(" | "));
        for j in &i.links {
            println!("------------------");
            println!("version= {}, file={}", j.version, j.file_name);
            println!("link= {}", j.url);
        }
    }
    Ok(())
}
