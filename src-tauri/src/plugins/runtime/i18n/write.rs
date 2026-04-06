use super::common::I18nContext;
use crate::plugins::api::emit_i18n_event;
use crate::services::global::i18n_service;
use mlua::Table;
use std::collections::HashMap;

pub(super) fn register_locale(
    ctx: &I18nContext,
    (locale, display_name): (String, String),
) -> mlua::Result<()> {
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
    let i18n = i18n_service();
    let map = table_to_map(&entries);
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

fn table_to_map(entries: &Table) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (k, v) in entries.pairs::<String, String>().flatten() {
        map.insert(k, v);
    }
    map
}
