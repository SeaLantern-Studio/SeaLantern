use ncmapi::NcmApi;
use serde::{Serialize, Deserialize};
use tokio::task;
use futures::executor;
use tauri::AppHandle;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artist {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct Album {
    pub id: u64,
    pub name: String,
    pub picUrl: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NCMMusic {
    pub id: u64,
    pub name: String,
    #[serde(rename = "ar")]
    pub artists: Vec<Artist>,
    #[serde(rename = "al")]
    pub album: Album,
    #[serde(rename = "dt")]
    pub duration: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResult {
    pub songs: Vec<NCMMusic>,
    #[serde(rename = "songCount")]
    pub song_count: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse {
    pub result: SearchResult,
    pub code: i32,
}

pub async fn ncm_search(app: AppHandle, keyword: String) -> Result<Vec<NCMMusic>, String> {
    // 直接使用临时目录，不单独写函数
    let mut cookie_path = std::env::temp_dir();
    cookie_path.push("sea-lantern-cookies.json");
    let cookie_path_str = cookie_path.to_string_lossy().to_string();

    let result = task::spawn_blocking(move || -> Result<Vec<NCMMusic>, String> {
        let api = NcmApi::new(
            true,
            std::time::Duration::from_secs(60),
            std::time::Duration::from_secs(120),
            false,
            &cookie_path_str,
        );

        let rsp = futures::executor::block_on(api.search(keyword.as_str(), None))
            .map_err(|e| format!("搜索失败: {}", e))?;

        let raw_value = rsp.deserialize_to_implict();
        let value = serde_json::to_value(raw_value)
            .map_err(|e| format!("序列化失败: {}", e))?;

        let api_rsp: ApiResponse = serde_json::from_value(value)
            .map_err(|e| format!("反序列化失败: {}", e))?;

        Ok(api_rsp.result.songs)
    })
        .await
        .map_err(|e| format!("线程池错误: {}", e))??;
    Ok(result)
}

pub async fn ncm_geturl(app: AppHandle, music_id: u64) -> Result<String, String> {
    // 同样使用临时目录
    let mut cookie_path = std::env::temp_dir();
    cookie_path.push("sea-lantern-cookies.json");
    let cookie_path_str = cookie_path.to_string_lossy().to_string();

    let result = task::spawn_blocking(move || -> Result<String, String> {
        let api = NcmApi::new(
            true,
            std::time::Duration::from_secs(60),
            std::time::Duration::from_secs(120),
            false,
            &cookie_path_str,
        );

        let resp = executor::block_on(api.song_url(&vec![music_id as usize]))
            .map_err(|e| format!("获取 URL 失败: {}", e))?;

        let url_value = serde_json::to_value(resp.deserialize_to_implict())
            .map_err(|e| format!("序列化失败: {}", e))?;

        let url = url_value["data"][0]["url"]
            .as_str()
            .ok_or_else(|| "无法解析 URL".to_string())?
            .to_string();

        Ok(url)
    })
        .await
        .map_err(|e| format!("线程池错误: {}", e))??;
    Ok(result)
}