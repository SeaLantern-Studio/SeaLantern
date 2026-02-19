use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub download_url: String,
    pub file_name: String,
    pub source: String, // "modrinth" or "curseforge"
    pub icon_url: Option<String>,
    pub downloads: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchModsResult {
    pub items: Vec<ModInfo>,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
}

pub struct ModManager {
    client: Client,
}

impl ModManager {
    pub fn new() -> Self {
        ModManager {
            client: Client::builder()
                .user_agent("SeaLantern/0.5.0 (contact@manus.im)")
                .build()
                .unwrap(),
        }
    }

    pub async fn search_modrinth(
        &self,
        query: &str,
        game_version: &str,
        loader: &str,
        project_type: &str,
        page: u32,
        page_size: u32,
    ) -> Result<SearchModsResult, String> {
        let limit = page_size.clamp(1, 50);
        let offset = page.saturating_sub(1).saturating_mul(limit);
        let url = format!(
            "https://api.modrinth.com/v2/search?query={}&limit={}&offset={}&facets=[[\"versions:{}\"],[\"categories:{}\"],[\"project_type:{}\"]]",
            query, limit, offset, game_version, loader.to_lowercase(), project_type
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let data: ModrinthSearchResponse = resp.json().await.map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for hit in data.hits {
            if let Ok(version) = self
                .get_latest_modrinth_version(&hit.project_id, game_version, loader)
                .await
            {
                results.push(ModInfo {
                    id: hit.project_id,
                    name: hit.title,
                    summary: hit.description,
                    download_url: version.file.url,
                    file_name: version.file.filename,
                    source: "modrinth".to_string(),
                    icon_url: hit.icon_url,
                    downloads: hit.downloads,
                });
            }
        }
        Ok(SearchModsResult {
            items: results,
            total: data.total_hits,
            offset,
            limit,
        })
    }

    async fn get_latest_modrinth_version(
        &self,
        project_id: &str,
        game_version: &str,
        loader: &str,
    ) -> Result<ModrinthResolvedVersion, String> {
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
                return Ok(ModrinthResolvedVersion {
                    file: ModrinthVersionFile {
                        url: file.url.clone(),
                        filename: file.filename.clone(),
                    },
                    dependencies: version.dependencies.clone(),
                });
            }
        }
        Err("No compatible version found".to_string())
    }

    pub async fn install_mod_with_dependencies(
        &self,
        root_project_id: &str,
        game_version: &str,
        loader: &str,
        mods_dir: &Path,
    ) -> Result<(), String> {
        fs::create_dir_all(mods_dir).map_err(|e| e.to_string())?;

        let mut queue = VecDeque::from([root_project_id.to_string()]);
        let mut visited = HashSet::new();

        while let Some(project_id) = queue.pop_front() {
            if !visited.insert(project_id.clone()) {
                continue;
            }

            let resolved = self
                .get_latest_modrinth_version(&project_id, game_version, loader)
                .await?;

            let target_path = mods_dir.join(&resolved.file.filename);
            if !target_path.exists() {
                self.download_mod(&resolved.file.url, &target_path).await?;
            }

            for dependency in resolved.dependencies {
                if dependency.dependency_type != "required" {
                    continue;
                }

                if let Some(dep_project_id) = dependency.project_id {
                    if !visited.contains(&dep_project_id) {
                        queue.push_back(dep_project_id);
                    }
                }
            }
        }

        Ok(())
    }

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
    total_hits: u64,
}

#[derive(Deserialize)]
struct ModrinthHit {
    project_id: String,
    title: String,
    description: String,
    icon_url: Option<String>,
    downloads: u64,
}

#[derive(Deserialize)]
struct ModrinthVersion {
    files: Vec<ModrinthFile>,
    #[serde(default)]
    dependencies: Vec<ModrinthDependency>,
}

#[derive(Deserialize)]
struct ModrinthFile {
    url: String,
    filename: String,
    primary: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct ModrinthDependency {
    #[serde(default)]
    project_id: Option<String>,
    dependency_type: String,
}

struct ModrinthVersionFile {
    url: String,
    filename: String,
}

struct ModrinthResolvedVersion {
    file: ModrinthVersionFile,
    dependencies: Vec<ModrinthDependency>,
}
