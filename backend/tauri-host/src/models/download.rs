use crate::hardcode_data::external_services::COMMON_HTTP_BROWSER_USER_AGENT;
use crate::utils::constants::DOWNLOAD_LINK_LIST_URL;
use crate::utils::downloader::SingleThreadDownloader;
pub(crate) use sea_lantern_server_download_links_core::{
    BaseDownloadLinks, DownloadLink, TypeDownloadLinks,
};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use uuid::Uuid;

static DOWNLOAD_LINKS: OnceCell<BaseDownloadLinks> = OnceCell::const_new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Downloading,
    Completed,
    Error(String),
}

/// 用于 API 返回的简易快照
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgressResponse {
    pub id: Uuid,
    pub total_size: u64,
    pub downloaded: u64,
    pub progress: f64,
    pub status: TaskStatus,
    pub is_finished: bool,
}

pub struct LinkManager;

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
        let downloader = SingleThreadDownloader::new(COMMON_HTTP_BROWSER_USER_AGENT);
        let response_body = downloader.read_to_string(DOWNLOAD_LINK_LIST_URL).await?;

        sea_lantern_server_download_links_core::parse_base_download_links(&response_body)
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

#[cfg(test)]
mod tests {
    use super::{TaskProgressResponse, TaskStatus, TypeDownloadLinks};

    #[test]
    fn task_progress_response_serializes_in_camel_case() {
        let value = serde_json::to_value(TaskProgressResponse {
            id: uuid::Uuid::nil(),
            total_size: 1024,
            downloaded: 512,
            progress: 50.0,
            status: TaskStatus::Downloading,
            is_finished: false,
        })
        .expect("task progress response should serialize");

        assert_eq!(value["totalSize"], 1024);
        assert_eq!(value["downloaded"], 512);
        assert_eq!(value["progress"], 50.0);
        assert_eq!(value["status"], "Downloading");
        assert_eq!(value["isFinished"], false);
    }

    #[test]
    fn type_download_links_get_link_by_version_returns_matching_entry() {
        let links = TypeDownloadLinks {
            server_type: "paper".to_string(),
            versions: vec!["1.20.1".to_string(), "1.21.1".to_string()],
            links: vec![
                super::DownloadLink {
                    version: "1.20.1".to_string(),
                    file_name: "paper-1.20.1.jar".to_string(),
                    url: "https://example.com/paper-1.20.1.jar".to_string(),
                },
                super::DownloadLink {
                    version: "1.21.1".to_string(),
                    file_name: "paper-1.21.1.jar".to_string(),
                    url: "https://example.com/paper-1.21.1.jar".to_string(),
                },
            ],
        };

        let link = links
            .get_link_by_version("1.21.1")
            .expect("matching version should be found");

        assert_eq!(link.file_name, "paper-1.21.1.jar");
    }
}
