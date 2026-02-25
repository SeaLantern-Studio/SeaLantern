use serde::{Deserialize, Serialize};

const MSL_API_BASE: &str = "https://api.mslmc.cn/v3";
const USER_AGENT: &str = "SeaLantern/0.6.5";

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    code: i32,
    message: String,
    data: T,
}

#[derive(Debug, Deserialize)]
struct ServerTypesData {
    types: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ServerDescriptionData {
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerClassify {
    #[serde(rename = "pluginsCore")]
    pub plugins_core: Vec<String>,
    #[serde(rename = "pluginsAndModsCore")]
    pub plugins_and_mods_core: Vec<String>,
    #[serde(rename = "modsCore_Forge")]
    pub mods_core_forge: Vec<String>,
    #[serde(rename = "modsCore_Fabric")]
    pub mods_core_fabric: Vec<String>,
    #[serde(rename = "vanillaCore")]
    pub vanilla_core: Vec<String>,
    #[serde(rename = "bedrockCore")]
    pub bedrock_core: Vec<String>,
    #[serde(rename = "proxyCore")]
    pub proxy_core: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ServerVersionsData {
    #[serde(rename = "versionList")]
    version_list: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ServerBuildsData {
    builds: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub url: String,
    pub sha256: Option<String>,
}

/// 获取所有支持的服务端类型
#[tauri::command]
pub async fn get_msl_server_types() -> Result<Vec<String>, String> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let url = format!("{}/query/available_server_types", MSL_API_BASE);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API 请求失败: {}", response.status()));
    }

    let result: ApiResponse<ServerTypesData> = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if result.code != 200 {
        return Err(format!("API 返回错误: {}", result.message));
    }

    Ok(result.data.types)
}

/// 获取特定服务端的简介
#[tauri::command]
pub async fn get_msl_server_description(server: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let url = format!("{}/query/servers_description/{}", MSL_API_BASE, server);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API 请求失败: {}", response.status()));
    }

    let result: ApiResponse<ServerDescriptionData> = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if result.code != 200 {
        return Err(format!("API 返回错误: {}", result.message));
    }

    Ok(result.data.description)
}

/// 获取服务端分类
#[tauri::command]
pub async fn get_msl_server_classify() -> Result<ServerClassify, String> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let url = format!("{}/query/server_classify", MSL_API_BASE);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API 请求失败: {}", response.status()));
    }

    let result: ApiResponse<ServerClassify> = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if result.code != 200 {
        return Err(format!("API 返回错误: {}", result.message));
    }

    Ok(result.data)
}

/// 获取特定服务端支持的 MC 版本列表
#[tauri::command]
pub async fn get_msl_server_versions(server: String) -> Result<Vec<String>, String> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let url = format!("{}/query/available_versions/{}", MSL_API_BASE, server);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API 请求失败: {}", response.status()));
    }

    let result: ApiResponse<ServerVersionsData> = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if result.code != 200 {
        return Err(format!("API 返回错误: {}", result.message));
    }

    Ok(result.data.version_list)
}

/// 获取服务端的所有构建信息
#[tauri::command]
pub async fn get_msl_server_builds(server: String, version: String) -> Result<Vec<String>, String> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let url = format!("{}/query/server/{}/{}", MSL_API_BASE, server, version);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API 请求失败: {}", response.status()));
    }

    let result: ApiResponse<ServerBuildsData> = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if result.code != 200 {
        return Err(format!("API 返回错误: {}", result.message));
    }

    Ok(result.data.builds)
}

/// 获取服务端下载地址
#[tauri::command]
pub async fn get_msl_server_download_url(
    server: String,
    version: String,
    build: String,
) -> Result<DownloadInfo, String> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let url = format!("{}/download/server/{}/{}?build={}", MSL_API_BASE, server, version, build);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API 请求失败: {}", response.status()));
    }

    let result: ApiResponse<DownloadInfo> = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if result.code != 200 {
        return Err(format!("API 返回错误: {}", result.message));
    }

    Ok(result.data)
}
