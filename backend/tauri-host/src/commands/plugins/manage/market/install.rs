use super::super::common::{is_trusted_download_url, lock_manager};
use super::github::resolve_plugin_download_url;
use super::shared::{build_market_async_client, remove_download_temp_files};
use crate::hardcode_data::app_files::PLUGIN_MARKET_TEMP_DIR_NAME;
use crate::hardcode_data::plugin_market::{
    PLUGIN_MARKET_ALLOWED_DOWNLOAD_DOMAINS, PLUGIN_MARKET_HTTP_USER_AGENT,
};
use crate::models::plugin::{MarketPluginInfo, PluginInstallResult};
use crate::plugins::manager::i18n::{plugin_t1, plugin_t2};
use crate::plugins::manager::PluginManager;
use crate::services::plugin_trusted_catalog::PluginInstallMetadata;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
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
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
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
        ensure_plugin_not_enabled(&manager, &plugin_id)?;
    }

    let (zip_path, untrusted_url, archive_bytes) = download_market_plugin(
        plugin_id.clone(),
        download_url,
        repo,
        download_type,
        release_asset,
        branch,
        version,
    )
    .await?;

    let result = {
        let mut manager = lock_manager(&manager);
        manager.install_plugin_with_metadata(
            &zip_path,
            PluginInstallMetadata {
                distribution_class: Some(crate::models::plugin::PluginDistributionClass::Market),
                archive_sha256: Some(sha256_hex(&archive_bytes)),
                installed_tree_sha256: None,
            },
        )
    };

    remove_download_temp_files(&zip_path);

    result.map(|mut install_result| {
        install_result.untrusted_url = untrusted_url;
        install_result.suggested_trust_level =
            Some(install_result.plugin.trust_level_display.clone());
        install_result.integrity_status = Some(install_result.plugin.integrity_status.clone());
        install_result.review_status = Some(install_result.plugin.review_status.clone());
        install_result.distribution_class = Some(install_result.plugin.distribution_class.clone());
        install_result.permission_profile = Some(install_result.plugin.permission_profile.clone());
        install_result.trusted_catalog_matched = install_result.plugin.trusted_catalog_matched;
        install_result.hash_matched = install_result.plugin.hash_matched;
        install_result.exceeds_standard_sandbox = install_result.plugin.exceeds_standard_sandbox;
        install_result
    })
}

#[cfg(feature = "docker")]
pub(crate) async fn install_from_market_for_http(
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
        let manager = crate::services::global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        ensure_plugin_not_enabled(&manager, &plugin_id)?;
    }

    let (zip_path, untrusted_url, archive_bytes) = download_market_plugin(
        plugin_id.clone(),
        download_url,
        repo,
        download_type,
        release_asset,
        branch,
        version,
    )
    .await?;

    let result = {
        let manager = crate::services::global::plugin_manager();
        let mut manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        manager.install_plugin_with_metadata(
            &zip_path,
            PluginInstallMetadata {
                distribution_class: Some(crate::models::plugin::PluginDistributionClass::Market),
                archive_sha256: Some(sha256_hex(&archive_bytes)),
                installed_tree_sha256: None,
            },
        )
    };

    remove_download_temp_files(&zip_path);

    result.map(|mut install_result| {
        install_result.untrusted_url = untrusted_url;
        install_result.suggested_trust_level =
            Some(install_result.plugin.trust_level_display.clone());
        install_result.integrity_status = Some(install_result.plugin.integrity_status.clone());
        install_result.review_status = Some(install_result.plugin.review_status.clone());
        install_result.distribution_class = Some(install_result.plugin.distribution_class.clone());
        install_result.permission_profile = Some(install_result.plugin.permission_profile.clone());
        install_result.trusted_catalog_matched = install_result.plugin.trusted_catalog_matched;
        install_result.hash_matched = install_result.plugin.hash_matched;
        install_result.exceeds_standard_sandbox = install_result.plugin.exceeds_standard_sandbox;
        install_result
    })
}

fn ensure_plugin_not_enabled(manager: &PluginManager, plugin_id: &str) -> Result<(), String> {
    if let Some(existing) = manager.plugins().get(plugin_id) {
        if matches!(existing.state, crate::models::plugin::PluginState::Enabled) {
            return Err(plugin_t1(
                "plugin.market.already_running_update",
                existing.manifest.name.clone(),
            ));
        }
    }

    Ok(())
}

async fn download_market_plugin(
    plugin_id: String,
    download_url: Option<String>,
    repo: Option<String>,
    download_type: Option<String>,
    release_asset: Option<String>,
    branch: Option<String>,
    version: Option<String>,
) -> Result<(std::path::PathBuf, bool, Vec<u8>), String> {
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

    let temp_dir = std::env::temp_dir().join(PLUGIN_MARKET_TEMP_DIR_NAME);

    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| plugin_t1("plugin.market.create_temp_dir_failed", e.to_string()))?;

    let zip_path = temp_dir.join(format!("{}.zip", plugin_id));
    let final_download_url = resolve_plugin_download_url(
        download_url,
        repo,
        download_type,
        release_asset,
        branch,
        version,
    )
    .await?;

    const MAX_DOWNLOAD_SIZE: u64 = 50 * 1024 * 1024;

    let client = build_market_async_client(PLUGIN_MARKET_HTTP_USER_AGENT)?;
    let response = client
        .get(&final_download_url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| plugin_t1("plugin.market.download_failed", e.to_string()))?;

    if !response.status().is_success() {
        return Err(plugin_t1(
            "plugin.market.download_status_failed",
            response.status().to_string(),
        ));
    }

    if let Some(content_length) = response.content_length() {
        if content_length > MAX_DOWNLOAD_SIZE {
            return Err(plugin_t2(
                "plugin.market.download_too_large",
                (content_length / 1024 / 1024).to_string(),
                (MAX_DOWNLOAD_SIZE / 1024 / 1024).to_string(),
            ));
        }
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| plugin_t1("plugin.market.read_download_response_failed", e.to_string()))?;

    if bytes.len() as u64 > MAX_DOWNLOAD_SIZE {
        return Err(plugin_t2(
            "plugin.market.download_too_large",
            (bytes.len() / 1024 / 1024).to_string(),
            (MAX_DOWNLOAD_SIZE / 1024 / 1024).to_string(),
        ));
    }

    if let Err(error) = tokio::fs::write(&zip_path, &bytes).await {
        let _ = tokio::fs::remove_file(&zip_path).await;
        return Err(plugin_t1("plugin.market.save_downloaded_file_failed", error.to_string()));
    }

    Ok((zip_path, untrusted_url, bytes.to_vec()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
