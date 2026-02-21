use crate::models::plugin::{
    BatchInstallError, BatchInstallResult, PluginInfo, PluginInstallResult, PluginUpdateInfo,
};
use crate::plugins::api::{
    BufferedComponentEvent, BufferedContextMenuEvent, BufferedSidebarEvent, BufferedUiEvent,
};
use crate::plugins::manager::PluginManager;
use std::sync::{Arc, Mutex};
use url::Url;

const MARKET_BASE_URL: &str = "https://sealantern-studio.needhelp.icu/";

const ALLOWED_DOWNLOAD_DOMAINS: &[&str] = &[
    "localhost",
    "sealanternpluginmarket.little100.top",
    "github.com",
    "raw.githubusercontent.com",
    "codeload.github.com",
    "api.github.com",
];

fn validate_plugin_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("Plugin ID cannot be empty".to_string());
    }
    if id.contains("..") {
        return Err(format!("Plugin ID '{}' contains invalid characters", id));
    }
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(format!("Plugin ID '{}' contains invalid characters", id));
    }
    Ok(())
}

#[tauri::command]
pub fn list_plugins(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<Vec<PluginInfo>, String> {
    let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    Ok(manager.get_plugin_list())
}

#[tauri::command]
pub fn scan_plugins(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<Vec<PluginInfo>, String> {
    let mut manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    manager.scan_plugins()
}

#[tauri::command]
pub fn enable_plugin(
    plugin_id: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    validate_plugin_id(&plugin_id)?;
    let mut manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    manager.enable_plugin(&plugin_id)
}

#[tauri::command]
pub fn disable_plugin(
    plugin_id: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<Vec<String>, String> {
    validate_plugin_id(&plugin_id)?;
    let mut manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    manager.disable_plugin(&plugin_id)
}

#[tauri::command]
pub fn get_plugin_nav_items(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<Vec<serde_json::Value>, String> {
    let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    Ok(manager.get_nav_items())
}

#[tauri::command]
pub fn install_plugin(
    path: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<PluginInstallResult, String> {
    let file_path = std::path::PathBuf::from(path);
    let is_zip = file_path.extension().and_then(|e| e.to_str()) == Some("zip");
    let is_manifest = file_path
        .file_name()
        .is_some_and(|name| name == "manifest.json");
    let is_dir = file_path.is_dir();

    if !is_zip && !is_manifest && !is_dir {
        return Err("不支持的文件格式，请提供 .zip 文件、manifest.json 或插件目录".to_string());
    }
    if (is_zip || is_manifest) && !file_path.is_file() {
        return Err("插件路径不存在或不是文件".to_string());
    }
    let mut mgr = manager.lock().unwrap_or_else(|e| e.into_inner());
    mgr.install_plugin(&file_path)
}

#[tauri::command]
pub fn get_plugin_icon(
    plugin_id: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<String, String> {
    validate_plugin_id(&plugin_id)?;
    let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    manager.get_plugin_icon(&plugin_id)
}

#[tauri::command]
pub fn get_plugin_settings(
    plugin_id: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<serde_json::Value, String> {
    validate_plugin_id(&plugin_id)?;
    let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    manager.get_plugin_settings(&plugin_id)
}

#[tauri::command]
pub fn set_plugin_settings(
    plugin_id: String,
    settings: serde_json::Value,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    validate_plugin_id(&plugin_id)?;
    let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    manager.set_plugin_settings(&plugin_id, settings)
}

#[tauri::command]
pub fn get_plugin_css(
    plugin_id: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<String, String> {
    validate_plugin_id(&plugin_id)?;
    let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    manager.get_plugin_css(&plugin_id)
}

#[tauri::command]
pub fn get_all_plugin_css(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<Vec<(String, String)>, String> {
    let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    manager.get_all_plugin_css()
}

#[tauri::command]
pub async fn delete_plugin(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
    plugin_id: String,
    delete_data: Option<bool>,
) -> Result<(), String> {
    validate_plugin_id(&plugin_id)?;
    let mut mgr = manager.lock().unwrap_or_else(|e| e.into_inner());
    mgr.delete_plugin(&plugin_id, delete_data.unwrap_or(false))
}

#[tauri::command]
pub async fn delete_plugins(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
    plugin_ids: Vec<String>,
    delete_data: Option<bool>,
) -> Result<(), String> {
    let delete_data = delete_data.unwrap_or(false);
    let mut mgr = manager.lock().unwrap_or_else(|e| e.into_inner());

    for plugin_id in &plugin_ids {
        validate_plugin_id(plugin_id)?;
    }

    for plugin_id in plugin_ids {
        mgr.delete_plugin(&plugin_id, delete_data)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn check_plugin_update(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
    plugin_id: String,
) -> Result<Option<PluginUpdateInfo>, String> {
    validate_plugin_id(&plugin_id)?;

    let current_version = {
        let mgr = manager.lock().unwrap_or_else(|e| e.into_inner());
        let plugin_info = mgr
            .plugins()
            .get(&plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;
        plugin_info.manifest.version.clone()
    };

    let pid = plugin_id.clone();

    tokio::task::spawn_blocking(move || {
        let client = reqwest::blocking::Client::builder()
            .user_agent("SeaLantern")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let index_url = format!("{}/api/plugins.json", MARKET_BASE_URL);
        let index: serde_json::Value = client
            .get(&index_url)
            .send()
            .map_err(|e| format!("Failed to fetch plugins index: {}", e))?
            .json()
            .map_err(|e| format!("Failed to parse plugins index: {}", e))?;

        let plugin_path = index
            .get("paths")
            .and_then(|v| v.as_array())
            .and_then(|arr| {
                arr.iter().find(|v| {
                    v.as_str()
                        .map(|s| {
                            let parts: Vec<&str> = s.split('/').collect();
                            parts.get(2).map(|p| *p == pid.as_str()).unwrap_or(false)
                        })
                        .unwrap_or(false)
                })
            })
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let plugin_path = match plugin_path {
            Some(p) => p,
            None => return Ok(None),
        };

        let plugin_url = format!("{}/{}", MARKET_BASE_URL, plugin_path);
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

        let latest = match market_info.version {
            Some(ref v) if !v.is_empty() => v.clone(),
            _ => return Ok(None),
        };
        if PluginManager::is_newer_version(&latest, &current_version) {
            Ok(Some(PluginUpdateInfo {
                plugin_id: pid,
                current_version,
                latest_version: latest,
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

#[tauri::command]
pub async fn check_all_plugin_updates(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<Vec<PluginUpdateInfo>, String> {
    let plugin_versions: Vec<(String, String)> = {
        let mgr = manager.lock().unwrap_or_else(|e| e.into_inner());
        mgr.plugins()
            .iter()
            .map(|(id, info)| (id.clone(), info.manifest.version.clone()))
            .collect()
    };

    tokio::task::spawn_blocking(move || {
        let client = reqwest::blocking::Client::builder()
            .user_agent("SeaLantern")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let index_url = format!("{}/api/plugins.json", MARKET_BASE_URL);
        let index: serde_json::Value = client
            .get(&index_url)
            .send()
            .map_err(|e| format!("Failed to fetch plugins index: {}", e))?
            .json()
            .map_err(|e| format!("Failed to parse plugins index: {}", e))?;

        let paths: Vec<String> = index
            .get("paths")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let path_map: std::collections::HashMap<String, String> = paths
            .iter()
            .filter_map(|p| {
                let parts: Vec<&str> = p.split('/').collect();
                parts.get(2).map(|id| (id.to_string(), p.clone()))
            })
            .collect();

        let mut updates = Vec::new();
        for (plugin_id, current_version) in plugin_versions {
            let plugin_path = match path_map.get(&plugin_id) {
                Some(p) => p.clone(),
                None => continue,
            };
            let url = format!("{}/{}", MARKET_BASE_URL, plugin_path);
            if let Ok(response) = client.get(&url).send() {
                if response.status().is_success() {
                    if let Ok(market_info) =
                        response.json::<crate::models::plugin::MarketPluginInfo>()
                    {
                        if let Some(ref latest) = market_info.version {
                            if PluginManager::is_newer_version(latest, &current_version) {
                                updates.push(PluginUpdateInfo {
                                    plugin_id,
                                    current_version,
                                    latest_version: latest.clone(),
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

#[tauri::command]
pub async fn fetch_market_plugins(
    _manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
    market_url: Option<String>,
) -> Result<Vec<crate::models::plugin::MarketPluginInfo>, String> {
    let base_url = market_url.unwrap_or_else(|| MARKET_BASE_URL.to_string());
    let base_url = base_url.trim_end_matches('/').to_string();
    tokio::task::spawn_blocking(move || {
        let client = reqwest::blocking::Client::builder()
            .user_agent("SeaLantern")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let index_url = format!("{}/api/plugins.json", base_url);
        let index_resp = client
            .get(&index_url)
            .send()
            .map_err(|e| format!("Failed to fetch plugins index: {}", e))?;
        if !index_resp.status().is_success() {
            return Err(format!("Plugins index returned status: {}", index_resp.status()));
        }
        let index_json: serde_json::Value = index_resp
            .json()
            .map_err(|e| format!("Failed to parse plugins index: {}", e))?;

        let paths: Vec<String> = index_json
            .get("paths")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mut all_plugins: Vec<crate::models::plugin::MarketPluginInfo> = Vec::new();

        for plugin_path in &paths {
            let plugin_url = format!("{}/{}", base_url, plugin_path);
            let plugin_resp = match client.get(&plugin_url).send() {
                Ok(r) if r.status().is_success() => r,
                _ => continue,
            };
            let plugin_json: serde_json::Value = match plugin_resp.json() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let Ok(mut plugin) =
                serde_json::from_value::<crate::models::plugin::MarketPluginInfo>(plugin_json)
            {
                let parts: Vec<&str> = plugin_path.split('/').collect();
                if parts.len() >= 3 {
                    let username = parts[1];
                    let plugin_folder = parts[2];
                    if plugin.repo.is_empty() {
                        plugin.repo = format!("{}/{}", username, plugin_folder);
                    }
                    if plugin.author.name.is_empty() {
                        plugin.author.name = username.to_string();
                        if plugin.author.url.is_none() {
                            plugin.author.url = Some(format!("https://github.com/{}", username));
                        }
                    }
                }
                plugin._path = Some(plugin_path.clone());
                all_plugins.push(plugin);
            }
        }

        Ok(all_plugins)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn fetch_market_categories(
    market_url: Option<String>,
) -> Result<serde_json::Value, String> {
    let base_url = market_url.unwrap_or_else(|| MARKET_BASE_URL.to_string());
    let base_url = base_url.trim_end_matches('/').to_string();
    tokio::task::spawn_blocking(move || {
        let url = format!("{}/api/categories.json", base_url);
        let response = reqwest::blocking::Client::builder()
            .user_agent("SeaLantern")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?
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

#[tauri::command]
pub async fn fetch_market_plugin_detail(
    _manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
    plugin_path: String,
) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let url = format!("{}/{}", MARKET_BASE_URL, plugin_path);
        let response = reqwest::blocking::get(&url)
            .map_err(|e| format!("Failed to fetch plugin detail: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Market API returned status: {}", response.status()));
        }

        let json: serde_json::Value = response
            .json()
            .map_err(|e| format!("Failed to parse plugin detail: {}", e))?;

        Ok(json)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[derive(Debug, serde::Deserialize)]
struct GitHubReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubReleaseAsset>,
    zipball_url: String,
}

#[allow(clippy::needless_option_as_deref)]
fn resolve_github_download_url(
    github: &str,
    download_type: Option<&str>,
    release_asset: Option<&str>,
    branch: Option<&str>,
    version: Option<&str>,
) -> Result<(String, String), String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("SeaLantern")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let download_type = download_type.unwrap_or("release");

    if download_type == "release" {
        let api_url = match version.as_deref() {
            None | Some("latest") => {
                format!("https://api.github.com/repos/{}/releases/latest", github)
            }
            Some(tag) => format!("https://api.github.com/repos/{}/releases/tags/{}", github, tag),
        };

        let response = client
            .get(&api_url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .map_err(|e| format!("Failed to fetch GitHub release: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API returned status: {}", response.status()));
        }

        let release: GitHubRelease = response
            .json()
            .map_err(|e| format!("Failed to parse GitHub release: {}", e))?;

        let detected_version = release.tag_name.clone();

        let download_url = if let Some(asset_name) = release_asset {
            let url = release
                .assets
                .iter()
                .find(|a| a.name == asset_name || a.name.contains(asset_name))
                .map(|a| a.browser_download_url.clone())
                .ok_or_else(|| format!("Asset '{}' not found in release", asset_name))?;

            let parsed =
                Url::parse(&url).map_err(|e| format!("Invalid browser_download_url: {}", e))?;
            let host = parsed.host_str().unwrap_or("");
            if !ALLOWED_DOWNLOAD_DOMAINS.contains(&host) {
                return Err(format!(
                    "browser_download_url domain '{}' is not in the allowed list",
                    host
                ));
            }
            url
        } else {
            release.zipball_url
        };

        Ok((download_url, detected_version))
    } else {
        let branch = branch.unwrap_or("main");
        let download_url =
            format!("https://codeload.github.com/{}/zip/refs/heads/{}", github, branch);

        Ok((download_url, "source".to_string()))
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn install_from_market(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
    plugin_id: String,
    download_url: Option<String>,
    repo: Option<String>,
    download_type: Option<String>,
    release_asset: Option<String>,
    branch: Option<String>,
    version: Option<String>,
) -> Result<PluginInstallResult, String> {
    validate_plugin_id(&plugin_id)?;

    {
        let mgr = manager.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = mgr.plugins().get(&plugin_id) {
            if matches!(existing.state, crate::models::plugin::PluginState::Enabled) {
                return Err(format!(
                    "插件 '{}' 正在运行中，请先禁用后再进行更新",
                    existing.manifest.name
                ));
            }
        }
    }

    let untrusted_url = if let Some(ref url) = download_url {
        match Url::parse(url) {
            Ok(parsed_url) => {
                let hostname = parsed_url.host_str().unwrap_or("");
                let trusted = ALLOWED_DOWNLOAD_DOMAINS.iter().any(|domain| {
                    hostname == *domain || hostname.ends_with(&format!(".{}", domain))
                });
                !trusted
            }
            Err(_) => true,
        }
    } else {
        false
    };

    let pid = plugin_id.clone();

    let zip_path = tokio::task::spawn_blocking(move || {
        let temp_dir = std::env::temp_dir().join("sealantern_market_downloads");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        let zip_path = temp_dir.join(format!("{}.zip", pid));

        let final_download_url = if let Some(url) = download_url {
            url
        } else if let Some(ref r) = repo {
            if r.starts_with("http://") || r.starts_with("https://") {
                r.clone()
            } else {
                let (url, _version) = resolve_github_download_url(
                    r,
                    download_type.as_deref(),
                    release_asset.as_deref(),
                    branch.as_deref(),
                    version.as_deref(),
                )?;
                url
            }
        } else {
            return Err("No download source specified".to_string());
        };

        const MAX_DOWNLOAD_SIZE: u64 = 50 * 1024 * 1024;

        let client = reqwest::blocking::Client::builder()
            .user_agent("SeaLantern")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client
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
        Ok(p) => p,
        Err(e) => {
            let temp_dir = std::env::temp_dir().join("sealantern_market_downloads");
            let _ = std::fs::remove_file(temp_dir.join(format!("{}.zip", plugin_id)));
            return Err(e);
        }
    };

    let result = {
        let mut mgr = manager.lock().unwrap_or_else(|e| e.into_inner());
        mgr.install_plugin(&zip_path)
    };

    if let Err(e) = std::fs::remove_file(&zip_path) {
        eprintln!("[WARN] Failed to remove temporary zip file: {}", e);
    }
    let temp_dir = std::env::temp_dir().join("sealantern_market_downloads");
    if let Err(e) = std::fs::remove_dir(&temp_dir) {
        if !e.to_string().contains("directory not empty") {
            eprintln!("[WARN] Failed to remove temporary directory: {}", e);
        }
    }

    result.map(|mut r| {
        r.untrusted_url = untrusted_url;
        r
    })
}

#[tauri::command]
pub fn install_plugins_batch(
    paths: Vec<String>,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<BatchInstallResult, String> {
    let mut success = Vec::new();
    let mut failed = Vec::new();

    let mut mgr = manager.lock().unwrap_or_else(|e| e.into_inner());

    for path_str in paths {
        let path = std::path::PathBuf::from(&path_str);

        let result = if path.is_file() {
            if path_str.ends_with(".zip") {
                mgr.install_plugin(&path)
            } else if path_str.ends_with("manifest.json") {
                if let Some(parent) = path.parent() {
                    mgr.install_plugin(parent)
                } else {
                    Err("Invalid manifest.json path".to_string())
                }
            } else {
                Err("Unsupported file type. Only .zip and manifest.json are supported.".to_string())
            }
        } else if path.is_dir() {
            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                mgr.install_plugin(&path)
            } else {
                Err("Folder does not contain manifest.json".to_string())
            }
        } else {
            Err(format!("Path does not exist: {}", path_str))
        };

        match result {
            Ok(install_result) => {
                success.push(install_result);
            }
            Err(e) => {
                failed.push(BatchInstallError { path: path_str, error: e });
            }
        }
    }

    Ok(BatchInstallResult { success, failed })
}

#[tauri::command]
pub fn context_menu_hide_notify(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    let runtimes = manager.get_shared_runtimes();
    let runtimes_guard = runtimes.read().unwrap_or_else(|e| e.into_inner());
    for runtime in runtimes_guard.values() {
        let _ = runtime.call_context_menu_hide_callback();
    }
    Ok(())
}

#[tauri::command]
pub fn context_menu_show_notify(
    context: String,
    target_data: serde_json::Value,
    x: f64,
    y: f64,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
    let runtimes = manager.get_shared_runtimes();
    let runtimes_guard = runtimes.read().unwrap_or_else(|e| e.into_inner());

    for runtime in runtimes_guard.values() {
        let _ = runtime.call_context_menu_show_callback(&context, target_data.clone(), x, y);
    }

    Ok(())
}

#[tauri::command]
pub fn context_menu_callback(
    plugin_id: String,
    context: String,
    item_id: String,
    target_data: serde_json::Value,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    validate_plugin_id(&plugin_id)?;

    let manager = manager.lock().unwrap_or_else(|e| e.into_inner());

    let runtimes = manager.get_shared_runtimes();
    let runtimes_guard = runtimes.read().unwrap_or_else(|e| e.into_inner());

    let runtime = runtimes_guard
        .get(&plugin_id)
        .ok_or_else(|| format!("插件 '{}' 的运行时不存在", plugin_id))?;

    runtime.call_context_menu_callback(&context, &item_id, target_data)
}

#[tauri::command]
pub fn on_locale_changed(
    locale: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    use crate::services::global::i18n_service;

    let i18n = i18n_service();
    i18n.set_locale(&locale);

    let mgr = manager.lock().unwrap_or_else(|e| e.into_inner());
    mgr.notify_locale_changed(&locale);

    Ok(())
}

#[tauri::command]
pub fn component_mirror_register(id: String, component_type: String) {
    crate::plugins::api::component_mirror_register(&id, &component_type);
}

#[tauri::command]
pub fn component_mirror_unregister(id: String) {
    crate::plugins::api::component_mirror_unregister(&id);
}

#[tauri::command]
pub fn component_mirror_clear() {
    crate::plugins::api::component_mirror_clear();
}

#[tauri::command]
pub fn on_page_changed(
    path: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    let mgr = manager.lock().unwrap_or_else(|e| e.into_inner());
    mgr.notify_page_changed(&path);
    Ok(())
}

#[tauri::command]
pub fn get_plugin_component_snapshot() -> Vec<BufferedComponentEvent> {
    crate::plugins::api::take_component_event_snapshot()
}

#[tauri::command]
pub fn get_plugin_ui_snapshot() -> Vec<BufferedUiEvent> {
    crate::plugins::api::take_ui_event_snapshot()
}

#[tauri::command]
pub fn get_plugin_sidebar_snapshot() -> Vec<BufferedSidebarEvent> {
    crate::plugins::api::take_sidebar_event_snapshot()
}

#[tauri::command]
pub fn get_plugin_context_menu_snapshot() -> Vec<BufferedContextMenuEvent> {
    crate::plugins::api::take_context_menu_snapshot()
}
