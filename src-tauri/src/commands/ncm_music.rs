use ncmapi::*;
use crate::services::ncm_music::*;

#[tauri::command]
pub async fn search_ncm_music(keyword: String) -> Result<Vec<NCMMusic>, String> {
    ncm_main(keyword)
}

#[tauri::command]
pub async fn url_ncm_music(music_id: u64) -> Result<String, String>{
    let api = NcmApi::new(true, std::time::Duration::from_secs(60), std::time::Duration::from_secs(120), false, "./cookies.json", );
    let resp = api.song_url(&vec![music_id as usize]).await.map_err(|e| format!("{}", e))?;
    let url = serde_json::to_value(resp.deserialize_to_implict()).map_err(|e| format!("{}", e))?;

    if let Some(url) = url["data"][0]["url"].as_str() {
        Ok(url.to_string())
    } else {
        Err("该歌曲无法播放".to_string())
    }
}