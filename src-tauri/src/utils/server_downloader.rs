use crate::models::download::{BaseDownloadLinks, DownloadLink, TypeDownloadLinks};
use serde_json::Value;
use tokio::sync::OnceCell;
use crate::utils::downloader::{SingleThreadDownloader, USER_AGENT_EXAMPLE};

const DOWNLOAD_LINK_LIST_URL: &str = crate::services::starter_installer_links::STARTER_INSTALLER_LINKS_URL;
static DOWNLOAD_LINKS: OnceCell<BaseDownloadLinks> = OnceCell::const_new();
pub async fn get_download_links() -> &'static BaseDownloadLinks {
    DOWNLOAD_LINKS.get_or_init(|| async {
        BaseDownloadLinks::new().await.expect("初始化失败")
    }).await
}

impl BaseDownloadLinks {
    pub async fn new() -> Result<Self, String> {
        let downloader = SingleThreadDownloader::new(USER_AGENT_EXAMPLE);
        let json = downloader.read_to_string(DOWNLOAD_LINK_LIST_URL).await?;

        let v: Value = serde_json::from_str(json.as_str())
            .map_err(|e| format!("JSON 解析失败: {}", e))?;

        let mut server_types = Vec::new();
        let mut links = Vec::new();

        if let Some(types_array) = v.get("types").and_then(|t| t.as_array()) {
            for t in types_array {
                let type_name = t.as_str().unwrap_or_default().to_string();
                server_types.push(type_name.clone());

                if let Some(type_data) = v.get(&type_name) {
                    let mut version_strings = Vec::new(); // 存放 Vec<String>
                    let mut download_entries = Vec::new(); // 存放 Vec<DownloadLink>

                    if let Some(versions_list) = type_data.get("versions").and_then(|v| v.as_array()) {
                        for ver_val in versions_list {
                            let mc_ver = ver_val.as_str().unwrap_or_default().to_string();

                           version_strings.push(mc_ver.clone());

                            if let Some(files_obj) = type_data.get(&mc_ver).and_then(|f| f.as_object()) {
                                for (fname, url_val) in files_obj {
                                    download_entries.push(DownloadLink {
                                        version: mc_ver.clone(),
                                        file_name: fname.clone(),
                                        url: url_val.as_str().unwrap_or_default().to_string(),
                                    });
                                }
                            }
                        }
                    }

                    links.push(TypeDownloadLinks {
                        server_type: type_name,
                        versions: version_strings,
                        links: download_entries,
                    });
                }
            }
        }
        Ok(Self { server_types, links })
    }
}

#[tokio::test]
pub async fn test() -> Result<(), String> {
    get_download_links().await;
    for i in DOWNLOAD_LINKS.get().unwrap().links.iter() {
        println!("==================");
        println!("type={}", i.server_type);
        println!("versions= | {}", i.versions.join(" | "));
        for j in i.links.iter() {
            println!("------------------");
            println!("version= {}", j.version);
            println!("link= {}", j.url);
        }
    }
    Ok(())
}