use crate::services::global::i18n_service;
use mlua::{Function, Lua, Table};

#[derive(Clone)]
pub(super) struct I18nContext {
    pub(super) plugin_id: String,
    pub(super) lua: Lua,
}

impl I18nContext {
    pub(super) fn new(plugin_id: String, lua: Lua) -> Self {
        Self { plugin_id, lua }
    }

    pub(super) fn callbacks_registry_key(&self) -> String {
        format!("_locale_change_callbacks_{}", self.plugin_id)
    }

    pub(super) fn token_registry_key(&self) -> String {
        format!("_locale_callback_token_{}", self.plugin_id)
    }
}

pub(super) fn validate_locale(locale: &str) -> bool {
    let parts: Vec<&str> = locale.split('-').collect();
    if parts.is_empty() || parts.len() > 3 {
        return false;
    }

    let is_alpha_segment = |segment: &str, min: usize, max: usize| {
        !segment.is_empty()
            && segment.len() >= min
            && segment.len() <= max
            && segment.chars().all(|ch| ch.is_ascii_alphabetic())
    };

    if !is_alpha_segment(parts[0], 2, 3) {
        return false;
    }

    if let Some(region_or_script) = parts.get(1) {
        let valid = (region_or_script.len() == 2
            && region_or_script.chars().all(|ch| ch.is_ascii_alphabetic()))
            || (region_or_script.len() == 4
                && region_or_script.chars().all(|ch| ch.is_ascii_alphabetic()));
        if !valid {
            return false;
        }
    }

    if let Some(region) = parts.get(2) {
        let valid = (region.len() == 2 && region.chars().all(|ch| ch.is_ascii_alphabetic()))
            || (region.len() == 3 && region.chars().all(|ch| ch.is_ascii_digit()));
        if !valid {
            return false;
        }
    }

    true
}

pub(super) fn validate_translation_key(key: &str) -> bool {
    key.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | ':'))
}

pub(super) fn plugin_i18n_namespace(plugin_id: &str, key: &str) -> String {
    format!("plugins.{}.{}", plugin_id, key)
}

pub(super) fn create_i18n_table(lua: &Lua) -> Result<Table, String> {
    lua.create_table()
        .map_err(|e| format!("Failed to create i18n table: {}", e))
}

pub(super) fn set_i18n_function(
    table: &Table,
    name: &str,
    function: Function,
    error_suffix: &str,
) -> Result<(), String> {
    table
        .set(name, function)
        .map_err(|e| format!("Failed to set i18n.{}: {}", error_suffix, e))
}

pub(super) fn set_i18n_table(sl: &Table, table: Table) -> Result<(), String> {
    sl.set("i18n", table)
        .map_err(|e| format!("Failed to set sl.i18n: {}", e))
}

pub(super) fn callbacks_table(lua: &Lua, registry_key: &str) -> mlua::Result<Table> {
    lua.named_registry_value(registry_key)
        .or_else(|_| lua.create_table())
}

pub(super) fn remove_locale_callback_token(token_id: usize) {
    i18n_service().remove_locale_callback(&crate::services::i18n::LocaleCallbackToken(token_id));
}
