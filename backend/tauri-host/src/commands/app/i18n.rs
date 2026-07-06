use crate::services::global::i18n_service;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct LocaleBundleResponse {
    pub locale: String,
    pub entries: HashMap<String, String>,
    pub available_locales: Vec<String>,
}

#[tauri::command]
pub fn get_locale_bundle(locale: Option<String>) -> Result<LocaleBundleResponse, String> {
    let i18n = i18n_service();
    let resolved_locale = locale.unwrap_or_else(|| i18n.get_locale());
    Ok(LocaleBundleResponse {
        entries: i18n.get_translations_for_locale(&resolved_locale),
        available_locales: i18n.get_available_locales(),
        locale: resolved_locale,
    })
}
