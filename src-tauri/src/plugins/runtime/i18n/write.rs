use super::common::{
    plugin_i18n_namespace, validate_locale, validate_translation_key, I18nContext,
};
use crate::plugins::api::emit_i18n_event;
use crate::services::global::i18n_service;
use mlua::Table;
use std::collections::HashMap;

const MAX_LOCALE_LEN: usize = 32;
const MAX_DISPLAY_NAME_LEN: usize = 64;
const MAX_TRANSLATION_KEY_LEN: usize = 128;
const MAX_TRANSLATION_VALUE_LEN: usize = 4000;
const MAX_TRANSLATION_ENTRIES_PER_CALL: usize = 500;
const MAX_TRANSLATION_ENTRIES_PER_PLUGIN: usize = 5000;

pub(super) fn register_locale(
    ctx: &I18nContext,
    (locale, display_name): (String, String),
) -> mlua::Result<()> {
    validate_locale_input(&locale)?;
    validate_display_name(&display_name)?;

    let i18n = i18n_service();
    i18n.register_locale(&ctx.plugin_id, &locale, &display_name);

    let payload = serde_json::json!({ "displayName": display_name }).to_string();
    if let Err(e) = emit_i18n_event(&ctx.plugin_id, "register_locale", &locale, &payload) {
        eprintln!("Failed to emit i18n event: {}", e);
    }

    Ok(())
}

pub(super) fn add_translations(
    ctx: &I18nContext,
    (locale, entries): (String, Table),
) -> mlua::Result<()> {
    validate_locale_input(&locale)?;

    let i18n = i18n_service();
    let map = table_to_namespaced_map(ctx, &entries)?;
    enforce_plugin_translation_quota(&ctx.plugin_id, &locale, map.len())?;
    let payload = serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string());

    i18n.add_plugin_translations(&ctx.plugin_id, &locale, map);
    if let Err(e) = emit_i18n_event(&ctx.plugin_id, "add_translations", &locale, &payload) {
        eprintln!("Failed to emit i18n event: {}", e);
    }

    Ok(())
}

pub(super) fn remove_translations(ctx: &I18nContext) -> mlua::Result<()> {
    let i18n = i18n_service();
    i18n.remove_plugin_translations(&ctx.plugin_id);
    if let Err(e) = emit_i18n_event(&ctx.plugin_id, "remove_translations", "", "") {
        eprintln!("Failed to emit i18n event: {}", e);
    }

    Ok(())
}

fn table_to_namespaced_map(
    ctx: &I18nContext,
    entries: &Table,
) -> mlua::Result<HashMap<String, String>> {
    let mut map = HashMap::new();

    for pair in entries.pairs::<String, String>() {
        let (raw_key, value) = pair?;
        validate_translation_key_input(&raw_key)?;
        validate_translation_value(&value)?;

        let key = plugin_i18n_namespace(&ctx.plugin_id, &raw_key);
        map.insert(key, value);

        if map.len() > MAX_TRANSLATION_ENTRIES_PER_CALL {
            return Err(mlua::Error::runtime(format!(
                "i18n.addTranslations accepts at most {} entries per call",
                MAX_TRANSLATION_ENTRIES_PER_CALL
            )));
        }
    }

    Ok(map)
}

fn validate_locale_input(locale: &str) -> mlua::Result<()> {
    if locale.is_empty() || locale.len() > MAX_LOCALE_LEN {
        return Err(mlua::Error::runtime(format!(
            "Locale length must be between 1 and {} characters",
            MAX_LOCALE_LEN
        )));
    }

    if !validate_locale(locale) {
        return Err(mlua::Error::runtime(
            "Locale must match formats like en, en-US, zh-CN or sr-Latn".to_string(),
        ));
    }

    Ok(())
}

fn validate_display_name(display_name: &str) -> mlua::Result<()> {
    let trimmed = display_name.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_DISPLAY_NAME_LEN {
        return Err(mlua::Error::runtime(format!(
            "Display name length must be between 1 and {} characters",
            MAX_DISPLAY_NAME_LEN
        )));
    }

    if trimmed.chars().any(|ch| ch.is_control()) {
        return Err(mlua::Error::runtime(
            "Display name must not contain control characters".to_string(),
        ));
    }

    Ok(())
}

fn validate_translation_key_input(key: &str) -> mlua::Result<()> {
    if key.is_empty() || key.len() > MAX_TRANSLATION_KEY_LEN {
        return Err(mlua::Error::runtime(format!(
            "Translation key length must be between 1 and {} characters",
            MAX_TRANSLATION_KEY_LEN
        )));
    }

    if !validate_translation_key(key) {
        return Err(mlua::Error::runtime(
            "Translation key may only contain letters, digits, '.', '-', '_' or ':'".to_string(),
        ));
    }

    Ok(())
}

fn validate_translation_value(value: &str) -> mlua::Result<()> {
    if value.len() > MAX_TRANSLATION_VALUE_LEN {
        return Err(mlua::Error::runtime(format!(
            "Translation value length must not exceed {} characters",
            MAX_TRANSLATION_VALUE_LEN
        )));
    }

    if value.chars().any(|ch| ch == '\u{0000}') {
        return Err(mlua::Error::runtime(
            "Translation value must not contain null characters".to_string(),
        ));
    }

    Ok(())
}

fn enforce_plugin_translation_quota(
    plugin_id: &str,
    locale: &str,
    incoming_entries: usize,
) -> mlua::Result<()> {
    let i18n = i18n_service();
    let total_entries = i18n
        .plugin_translation_entry_count(plugin_id)
        .saturating_add(incoming_entries);

    if total_entries > MAX_TRANSLATION_ENTRIES_PER_PLUGIN {
        return Err(mlua::Error::runtime(format!(
            "Plugin translation quota exceeded for locale '{}': limit is {} entries per plugin",
            locale, MAX_TRANSLATION_ENTRIES_PER_PLUGIN
        )));
    }

    Ok(())
}
