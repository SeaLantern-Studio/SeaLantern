use super::shared::{build_market_client, fetch_market_index, fill_market_plugin_defaults};
use crate::hardcode_data::plugin_market::{PLUGIN_MARKET_BASE_URL, PLUGIN_MARKET_HTTP_USER_AGENT};
use crate::models::plugin::PluginUpdateInfo;
use crate::plugins::manager::PluginManager;

pub(super) async fn check_plugin_update(
    current_version: String,
    plugin_id: String,
) -> Result<Option<PluginUpdateInfo>, String> {
    let plugin_id_for_task = plugin_id.clone();

    tokio::task::spawn_blocking(move || {
        let client = build_market_client(PLUGIN_MARKET_HTTP_USER_AGENT)?;
        let index = fetch_market_index(&client, PLUGIN_MARKET_BASE_URL)?;

        let plugin_path = index
            .get("paths")
            .and_then(|value| value.as_array())
            .and_then(|items| {
                items.iter().find(|value| {
                    value
                        .as_str()
                        .map(|path| {
                            let parts: Vec<&str> = path.split('/').collect();
                            parts
                                .get(2)
                                .map(|part| *part == plugin_id_for_task.as_str())
                                .unwrap_or(false)
                        })
                        .unwrap_or(false)
                })
            })
            .and_then(|value| value.as_str())
            .map(|path| path.to_string());

        let plugin_path = match plugin_path {
            Some(path) => path,
            None => return Ok(None),
        };

        let plugin_url = format!("{}/{}", PLUGIN_MARKET_BASE_URL, plugin_path);
        let response = client
            .get(&plugin_url)
            .send()
            .map_err(|e| format!("Failed to fetch plugin info: {}", e))?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Ok(None);
            }
            return Err(format!("Market API returned error: {}", response.status()));
        }

        let market_info: crate::models::plugin::MarketPluginInfo = response
            .json()
            .map_err(|e| format!("Failed to parse market response: {}", e))?;

        let latest_version = match market_info.version {
            Some(ref version) if !version.is_empty() => version.clone(),
            _ => return Ok(None),
        };

        if PluginManager::is_newer_version(&latest_version, &current_version) {
            Ok(Some(PluginUpdateInfo {
                plugin_id: plugin_id_for_task,
                current_version,
                latest_version,
                download_url: market_info.download_url,
                changelog: market_info.changelog,
            }))
        } else {
            Ok(None)
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

pub(super) async fn check_all_plugin_updates(
    plugin_versions: Vec<(String, String)>,
) -> Result<Vec<PluginUpdateInfo>, String> {
    tokio::task::spawn_blocking(move || {
        let client = build_market_client(PLUGIN_MARKET_HTTP_USER_AGENT)?;
        let index = fetch_market_index(&client, PLUGIN_MARKET_BASE_URL)?;

        let paths: Vec<String> = index
            .get("paths")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|value| value.as_str().map(|path| path.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let path_map: std::collections::HashMap<String, String> = paths
            .iter()
            .filter_map(|path| {
                let parts: Vec<&str> = path.split('/').collect();
                parts.get(2).map(|id| (id.to_string(), path.clone()))
            })
            .collect();

        let mut updates = Vec::new();
        for (plugin_id, current_version) in plugin_versions {
            let plugin_path = match path_map.get(&plugin_id) {
                Some(path) => path.clone(),
                None => continue,
            };

            let url = format!("{}/{}", PLUGIN_MARKET_BASE_URL, plugin_path);
            if let Ok(response) = client.get(&url).send() {
                if response.status().is_success() {
                    if let Ok(market_info) =
                        response.json::<crate::models::plugin::MarketPluginInfo>()
                    {
                        if let Some(ref latest_version) = market_info.version {
                            if PluginManager::is_newer_version(latest_version, &current_version) {
                                updates.push(PluginUpdateInfo {
                                    plugin_id,
                                    current_version,
                                    latest_version: latest_version.clone(),
                                    download_url: market_info.download_url,
                                    changelog: market_info.changelog,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(updates)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

pub(super) async fn fetch_market_plugins(
    base_url: String,
) -> Result<Vec<crate::models::plugin::MarketPluginInfo>, String> {
    tokio::task::spawn_blocking(move || {
        let client = build_market_client(PLUGIN_MARKET_HTTP_USER_AGENT)?;
        let index_json = fetch_market_index(&client, &base_url)?;

        let paths: Vec<String> = index_json
            .get("paths")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|value| value.as_str().map(|path| path.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mut all_plugins = Vec::new();

        for plugin_path in &paths {
            let plugin_url = format!("{}/{}", base_url, plugin_path);
            let plugin_response = match client.get(&plugin_url).send() {
                Ok(response) if response.status().is_success() => response,
                _ => continue,
            };

            let plugin_json: serde_json::Value = match plugin_response.json() {
                Ok(value) => value,
                Err(_) => continue,
            };

            if let Ok(mut plugin) =
                serde_json::from_value::<crate::models::plugin::MarketPluginInfo>(plugin_json)
            {
                fill_market_plugin_defaults(&mut plugin, plugin_path);
                all_plugins.push(plugin);
            }
        }

        Ok(all_plugins)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

pub(super) async fn fetch_market_categories(base_url: String) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let url = format!("{}/api/categories.json", base_url);
        let response = build_market_client(PLUGIN_MARKET_HTTP_USER_AGENT)?
            .get(&url)
            .send()
            .map_err(|e| format!("Failed to fetch categories: {}", e))?;

        if !response.status().is_success() {
            return Ok(serde_json::Value::Object(Default::default()));
        }

        response
            .json()
            .map_err(|e| format!("Failed to parse categories: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

pub(super) async fn fetch_market_plugin_detail(
    base_url: String,
    plugin_path: String,
) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let url = format!("{}/{}", base_url, plugin_path);
        let response = reqwest::blocking::get(&url)
            .map_err(|e| format!("Failed to fetch plugin detail: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Market API returned status: {}", response.status()));
        }

        response
            .json()
            .map_err(|e| format!("Failed to parse plugin detail: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
