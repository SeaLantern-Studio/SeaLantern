use super::github::resolve_plugin_download_url;
use super::shared::{build_market_client, remove_download_temp_files};
use super::super::common::{is_trusted_download_url, lock_manager, PluginManagerState};
use crate::hardcode_data::app_files::PLUGIN_MARKET_TEMP_DIR_NAME;
use crate::hardcode_data::plugin_market::{
    PLUGIN_MARKET_ALLOWED_DOWNLOAD_DOMAINS, PLUGIN_MARKET_HTTP_USER_AGENT,
};
use crate::models::plugin::{MarketPluginInfo, PluginInstallResult};
use url::Url;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct InstallFromMarketRequest {
    pub plugin_id: String,
    pub download_url: Option<String>,
    pub repo: Option<String>,
    pub download_type: Option<String>,
    pub release_asset: Option<String>,
    pub branch: Option<String>,
    pub version: Option<String>,
}

impl From<MarketPluginInfo> for InstallFromMarketRequest {
    fn from(value: MarketPluginInfo) -> Self {
        Self {
            plugin_id: value.id,
            download_url: value.download_url,
            repo: Some(value.repo),
            download_type: value.download_type,
            release_asset: value.release_asset,
            branch: value.branch,
            version: value.version,
        }
    }
}

pub(super) async fn install_from_market(
    manager: PluginManagerState<'_>,
    req: InstallFromMarketRequest,
) -> Result<PluginInstallResult, String> {
    let InstallFromMarketRequest {
        plugin_id,
        download_url,
        repo,
        download_type,
        release_asset,
        branch,
        version,
    } = req;

    {
        let manager = lock_manager(&manager);
        if let Some(existing) = manager.plugins().get(&plugin_id) {
            if matches!(existing.state, crate::models::plugin::PluginState::Enabled) {
                return Err(format!(
                "插件 '{}' 正在运行中，请先禁用后再进行更新",
                existing.manifest.name
            ));
        }
    }
    }

    let untrusted_url = download_url
        .as_ref()
        .map(|url| {
            Url::parse(url)
                .map(|parsed_url| {
                    !is_trusted_download_url(&parsed_url, PLUGIN_MARKET_ALLOWED_DOWNLOAD_DOMAINS)
                })
                .unwrap_or(true)
        })
        .unwrap_or(false);

    let plugin_id_for_task = plugin_id.clone();

    let zip_path = tokio::task::spawn_blocking(move || {
        let temp_dir = std::env::temp_dir().join(PLUGIN_MARKET_TEMP_DIR_NAME);
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        let zip_path = temp_dir.join(format!("{}.zip", plugin_id_for_task));
        let final_download_url = resolve_plugin_download_url(
            download_url,
            repo,
            download_type,
            release_asset,
            branch,
            version,
        )?;

        const MAX_DOWNLOAD_SIZE: u64 = 50 * 1024 * 1024;

        let response = build_market_client(PLUGIN_MARKET_HTTP_USER_AGENT)?
            .get(&final_download_url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .map_err(|e| format!("Failed to download plugin: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Download failed with status: {}", response.status()));
        }

        let bytes = response
            .bytes()
            .map_err(|e| format!("Failed to read download response: {}", e))?;

        if bytes.len() as u64 > MAX_DOWNLOAD_SIZE {
            return Err(format!(
                "Downloaded file exceeds max size ({}MB > 50MB)",
                bytes.len() / 1024 / 1024
            ));
        }

        std::fs::write(&zip_path, &bytes)
            .map_err(|e| format!("Failed to save downloaded file: {}", e))?;

        Ok::<std::path::PathBuf, String>(zip_path)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    let zip_path = match zip_path {
        Ok(path) => path,
        Err(error) => {
            let temp_dir = std::env::temp_dir().join(PLUGIN_MARKET_TEMP_DIR_NAME);
            let _ = std::fs::remove_file(temp_dir.join(format!("{}.zip", plugin_id)));
            return Err(error);
        }
    };

    let result = {
        let mut manager = lock_manager(&manager);
        manager.install_plugin(&zip_path)
    };

    remove_download_temp_files(&zip_path);

    result.map(|mut install_result| {
        install_result.untrusted_url = untrusted_url;
        install_result
    })
}
