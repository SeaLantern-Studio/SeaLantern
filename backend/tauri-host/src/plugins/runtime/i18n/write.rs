use super::common::{
    i18n_err, i18n_err1, i18n_t2, plugin_i18n_namespace, validate_locale, validate_translation_key,
    I18nContext,
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
            return Err(i18n_err1(
                "plugins.runtime.i18n.max_entries_per_call_exceeded",
                MAX_TRANSLATION_ENTRIES_PER_CALL.to_string(),
            ));
        }
    }

    Ok(map)
}

fn validate_locale_input(locale: &str) -> mlua::Result<()> {
    if locale.is_empty() || locale.len() > MAX_LOCALE_LEN {
        return Err(i18n_err1(
            "plugins.runtime.i18n.locale_length_invalid",
            MAX_LOCALE_LEN.to_string(),
        ));
    }

    if !validate_locale(locale) {
        return Err(i18n_err("plugins.runtime.i18n.locale_format_invalid"));
    }

    Ok(())
}

fn validate_display_name(display_name: &str) -> mlua::Result<()> {
    let trimmed = display_name.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_DISPLAY_NAME_LEN {
        return Err(i18n_err1(
            "plugins.runtime.i18n.display_name_length_invalid",
            MAX_DISPLAY_NAME_LEN.to_string(),
        ));
    }

    if trimmed.chars().any(|ch| ch.is_control()) {
        return Err(i18n_err("plugins.runtime.i18n.display_name_control_chars_forbidden"));
    }

    Ok(())
}

fn validate_translation_key_input(key: &str) -> mlua::Result<()> {
    if key.is_empty() || key.len() > MAX_TRANSLATION_KEY_LEN {
        return Err(i18n_err1(
            "plugins.runtime.i18n.translation_key_length_invalid",
            MAX_TRANSLATION_KEY_LEN.to_string(),
        ));
    }

    if !validate_translation_key(key) {
        return Err(i18n_err("plugins.runtime.i18n.translation_key_format_invalid"));
    }

    Ok(())
}

fn validate_translation_value(value: &str) -> mlua::Result<()> {
    if value.len() > MAX_TRANSLATION_VALUE_LEN {
        return Err(i18n_err1(
            "plugins.runtime.i18n.translation_value_too_long",
            MAX_TRANSLATION_VALUE_LEN.to_string(),
        ));
    }

    if value.chars().any(|ch| ch == '\u{0000}') {
        return Err(i18n_err("plugins.runtime.i18n.translation_value_null_forbidden"));
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
        return Err(mlua::Error::runtime(i18n_t2(
            "plugins.runtime.i18n.plugin_quota_exceeded",
            locale,
            MAX_TRANSLATION_ENTRIES_PER_PLUGIN.to_string(),
        )));
    }

    Ok(())
}
