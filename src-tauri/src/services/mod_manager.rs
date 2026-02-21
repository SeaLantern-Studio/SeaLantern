use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub download_url: String,
    pub file_name: String,
    pub source: String, // "modrinth" or "curseforge"
}

#[allow(dead_code)]
pub struct ModManager {
    client: Client,
}

impl ModManager {
    #[allow(dead_code)]
    pub fn new() -> Result<Self, String> {
        Ok(ModManager {
            client: Client::builder()
                .user_agent("SeaLantern/0.5.0 (contact@manus.im)")
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?,
        })
    }

    #[allow(dead_code)]
    pub async fn search_modrinth(
        &self,
        query: &str,
        game_version: &str,
        loader: &str,
    ) -> Result<Vec<ModInfo>, String> {
        let facets = format!(
            "[[\"versions:{}\"],[\"categories:{}\"],[\"project_type:mod\"]]",
            game_version,
            loader.to_lowercase()
        );
        let resp = self
            .client
            .get("https://api.modrinth.com/v2/search")
            .query(&[("query", query), ("facets", facets.as_str())])
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let data: ModrinthSearchResponse = resp.json().await.map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for hit in data.hits {
            // 获取最新版本以获取下载链接
            if let Ok(version) = self
                .get_latest_modrinth_version(&hit.project_id, game_version, loader)
                .await
            {
                results.push(ModInfo {
                    id: hit.project_id,
                    name: hit.title,
                    summary: hit.description,
                    download_url: version.url,
                    file_name: version.filename,
                    source: "modrinth".to_string(),
                });
            }
        }
        Ok(results)
    }

    #[allow(dead_code)]
    async fn get_latest_modrinth_version(
        &self,
        project_id: &str,
        game_version: &str,
        loader: &str,
    ) -> Result<ModrinthVersionFile, String> {
        let url = format!(
            "https://api.modrinth.com/v2/project/{}/version?loaders=[\"{}\"]&game_versions=[\"{}\"]",
            project_id, loader.to_lowercase(), game_version
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let versions: Vec<ModrinthVersion> = resp.json().await.map_err(|e| e.to_string())?;

        if let Some(version) = versions.first() {
            if let Some(file) = version
                .files
                .iter()
                .find(|f| f.primary)
                .or(version.files.first())
            {
                return Ok(ModrinthVersionFile {
                    url: file.url.clone(),
                    filename: file.filename.clone(),
                });
            }
        }
        Err("No compatible version found".to_string())
    }

    #[allow(dead_code)]
    pub async fn download_mod(&self, download_url: &str, target_path: &Path) -> Result<(), String> {
        let resp = self
            .client
            .get(download_url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let bytes = resp.bytes().await.map_err(|e| e.to_string())?;

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        fs::write(target_path, bytes).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthHit>,
}

#[derive(Deserialize)]
struct ModrinthHit {
    project_id: String,
    title: String,
    description: String,
}

#[derive(Deserialize)]
struct ModrinthVersion {
    files: Vec<ModrinthFile>,
}

#[derive(Deserialize)]
struct ModrinthFile {
    url: String,
    filename: String,
    primary: bool,
}

struct ModrinthVersionFile {
    url: String,
    filename: String,
}
