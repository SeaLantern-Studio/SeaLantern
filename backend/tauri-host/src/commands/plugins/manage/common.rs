use crate::plugins::manager::i18n::{plugin_t, plugin_t1};
use crate::plugins::manager::PluginManager;
use std::sync::{Arc, Mutex, MutexGuard};
use url::Url;

pub(super) fn validate_plugin_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err(plugin_t("plugin.common.id_empty"));
    }
    if id.contains("..") {
        return Err(plugin_t1("plugin.common.id_invalid_chars", id));
    }
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(plugin_t1("plugin.common.id_invalid_chars", id));
    }
    Ok(())
}

pub(super) fn lock_manager<'a>(
    manager: &'a tauri::State<'a, Arc<Mutex<PluginManager>>>,
) -> MutexGuard<'a, PluginManager> {
    manager.lock().unwrap_or_else(|e| e.into_inner())
}

pub(super) fn trim_market_base_url(market_url: Option<String>, default_base_url: &str) -> String {
    market_url
        .unwrap_or_else(|| default_base_url.to_string())
        .trim_end_matches('/')
        .to_string()
}

pub(super) fn is_trusted_download_url(url: &Url, allowed_domains: &[&str]) -> bool {
    let hostname = url.host_str().unwrap_or("");
    allowed_domains
        .iter()
        .any(|domain| hostname == *domain || hostname.ends_with(&format!(".{}", domain)))
}
