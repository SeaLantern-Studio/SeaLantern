pub const STARTER_DOWNLOAD_API_BASE_DEFAULT: &str = "https://api.mslmc.cn/v3/download/server";
pub const STARTER_DOWNLOAD_API_BASE_ENV: &str = "SEALANTERN_STARTER_DOWNLOAD_API_BASE";

use serde::Deserialize;
use std::time::Duration;

pub fn starter_download_api_base() -> String {
    std::env::var(STARTER_DOWNLOAD_API_BASE_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| STARTER_DOWNLOAD_API_BASE_DEFAULT.to_string())
}

#[derive(Debug, Deserialize)]
struct StarterDownloadApiResponse {
    data: Option<StarterDownloadApiData>,
}

#[derive(Debug, Deserialize)]
struct StarterDownloadApiData {
    url: Option<String>,
    sha256: Option<String>,
}

fn build_starter_download_url(
    core_type_key: &str,
    mc_version: &str,
) -> Result<reqwest::Url, String> {
    let core_type_trimmed = core_type_key.trim();
    if core_type_trimmed.is_empty() {
        return Err("核心类别不能为空".to_string());
    }
    let mc_version_trimmed = mc_version.trim();
    if mc_version_trimmed.is_empty() {
        return Err("游戏版本不能为空".to_string());
    }

    let mut url = reqwest::Url::parse(&starter_download_api_base())
        .map_err(|e| format!("构建 Starter 下载链接失败: {}", e))?;
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| "Starter 下载链接不支持路径段写入".to_string())?;
        segments.push(core_type_trimmed);
        segments.push(mc_version_trimmed);
    }
    Ok(url)
}

fn parse_starter_download_response(
    payload: StarterDownloadApiResponse,
) -> Result<(String, Option<String>), String> {
    let data = payload
        .data
        .ok_or_else(|| "Starter 下载接口缺少 data 字段".to_string())?;
    let installer_url = data
        .url
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Starter 下载接口未返回 data.url".to_string())?;
    let installer_sha256 = data
        .sha256
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    Ok((installer_url, installer_sha256))
}

pub async fn fetch_starter_download_info(
    core_type_key: &str,
    mc_version: &str,
) -> Result<(String, Option<String>), String> {
    let url = build_starter_download_url(core_type_key, mc_version)?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 Starter 请求客户端失败: {}", e))?;
    let response = client
        .get(url.clone())
        .send()
        .await
        .map_err(|e| format!("请求 Starter 下载信息失败: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("Starter 下载接口返回异常状态: {} ({})", status, url));
    }

    let payload: StarterDownloadApiResponse = response
        .json()
        .await
        .map_err(|e| format!("解析 Starter 下载信息失败: {}", e))?;

    parse_starter_download_response(payload)
}

pub fn fetch_starter_download_info_blocking(
    core_type_key: &str,
    mc_version: &str,
) -> Result<(String, Option<String>), String> {
    let url = build_starter_download_url(core_type_key, mc_version)?;

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 Starter 请求客户端失败: {}", e))?;
    let response = client
        .get(url.clone())
        .send()
        .map_err(|e| format!("请求 Starter 下载信息失败: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("Starter 下载接口返回异常状态: {} ({})", status, url));
    }

    let payload: StarterDownloadApiResponse = response
        .json()
        .map_err(|e| format!("解析 Starter 下载信息失败: {}", e))?;

    parse_starter_download_response(payload)
}
