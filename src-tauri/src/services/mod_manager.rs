//! 模组搜索和下载服务

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::hardcode_data::external_services::{
    MODRINTH_PROJECT_VERSION_API_BASE, MODRINTH_SEARCH_API_URL,
};
use crate::hardcode_data::plugin_market::PLUGIN_MARKET_HTTP_USER_AGENT;

/// 模组来源
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModSource {
    /// 来自 Modrinth
    Modrinth,
    /// 来自 CurseForge
    Curseforge,
}

impl std::fmt::Display for ModSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Modrinth => "modrinth",
            Self::Curseforge => "curseforge",
        };
        f.write_str(value)
    }
}

/// 前端使用的模组信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModInfo {
    /// 模组 ID
    pub id: String,
    /// 模组名称
    pub name: String,
    /// 简介
    pub summary: String,
    /// 下载地址
    pub download_url: String,
    /// 建议保存文件名
    pub file_name: String,
    /// 来源平台
    pub source: ModSource,
}

/// 模组查询和下载服务
pub struct ModManager {
    client: Client,
}

impl ModManager {
    /// 创建模组服务
    pub fn new() -> Result<Self, String> {
        Ok(ModManager {
            client: Client::builder()
                .user_agent(PLUGIN_MARKET_HTTP_USER_AGENT)
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?,
        })
    }

    /// 按版本和加载器搜索 Modrinth 模组
    ///
    /// # Parameters
    ///
    /// - `query`: 搜索关键词
    /// - `game_version`: 目标 MC 版本
    /// - `loader`: 目标加载器
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
            .get(MODRINTH_SEARCH_API_URL)
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
                    source: ModSource::Modrinth,
                });
            }
        }
        Ok(results)
    }

    /// 读取指定项目的最新可用版本文件
    async fn get_latest_modrinth_version(
        &self,
        project_id: &str,
        game_version: &str,
        loader: &str,
    ) -> Result<ModrinthVersionFile, String> {
        let url = format!(
            "{}/{}/version?loaders=[\"{}\"]&game_versions=[\"{}\"]",
            MODRINTH_PROJECT_VERSION_API_BASE,
            project_id,
            loader.to_lowercase(),
            game_version
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
        Err("没有找到兼容版本".to_string())
    }

    /// 下载模组文件到目标路径
    ///
    /// # Parameters
    ///
    /// - `download_url`: 模组下载地址
    /// - `target_path`: 目标保存路径
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

/// Modrinth 搜索响应
#[derive(Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthHit>,
}

/// 搜索结果里的单个项目
#[derive(Deserialize)]
struct ModrinthHit {
    project_id: String,
    title: String,
    description: String,
}

/// Modrinth 版本信息
#[derive(Deserialize)]
struct ModrinthVersion {
    files: Vec<ModrinthFile>,
}

/// 版本中的可下载文件
#[derive(Deserialize)]
struct ModrinthFile {
    url: String,
    filename: String,
    primary: bool,
}

/// 最终选中的下载文件
struct ModrinthVersionFile {
    url: String,
    filename: String,
}
