use crate::hardcode_data::app_files::PLUGIN_MARKET_TEMP_DIR_NAME;
use crate::hardcode_data::plugin_market::GITHUB_PROFILE_BASE_URL;

pub(super) fn build_market_async_client(user_agent: &str) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(user_agent)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

pub(super) async fn fetch_market_index_async(
    client: &reqwest::Client,
    base_url: &str,
) -> Result<serde_json::Value, String> {
    let index_url = format!("{}/api/plugins.json", base_url);
    let response = client
        .get(&index_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch plugins index: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Plugins index returned status: {}", response.status()));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse plugins index: {}", e))
}

pub(super) fn fill_market_plugin_defaults(
    plugin: &mut crate::models::plugin::MarketPluginInfo,
    plugin_path: &str,
) {
    let parts: Vec<&str> = plugin_path.split('/').collect();
    if parts.len() < 3 {
        plugin._path = Some(plugin_path.to_string());
        return;
    }

    let username = parts[1];
    let plugin_folder = parts[2];

    if plugin.repo.is_empty() {
        plugin.repo = format!("{}/{}", username, plugin_folder);
    }

    if plugin.author.name.is_empty() {
        plugin.author.name = username.to_string();
        if plugin.author.url.is_none() {
            plugin.author.url = Some(format!("{}/{}", GITHUB_PROFILE_BASE_URL, username));
        }
    }

    plugin._path = Some(plugin_path.to_string());
}

pub(super) fn remove_download_temp_files(zip_path: &std::path::Path) {
    if let Err(e) = std::fs::remove_file(zip_path) {
        eprintln!("[WARN] Failed to remove temporary zip file: {}", e);
    }

    let temp_dir = std::env::temp_dir().join(PLUGIN_MARKET_TEMP_DIR_NAME);
    if let Err(e) = std::fs::remove_dir(&temp_dir) {
        if !e.to_string().contains("directory not empty") {
            eprintln!("[WARN] Failed to remove temporary directory: {}", e);
        }
    }
}
