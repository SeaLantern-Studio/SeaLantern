use crate::plugins::runtime::host_api::{host_remove_locale_callback, host_t, host_t_with_options};
use mlua::{Function, Lua, Table};
use std::collections::HashMap;

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

pub(super) fn i18n_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    host_t_with_options(key, &m)
}

pub(super) fn i18n_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    host_t_with_options(key, &m)
}

pub(super) fn i18n_err(key: &str) -> mlua::Error {
    mlua::Error::runtime(host_t(key))
}

pub(super) fn i18n_err1(key: &str, a: impl Into<String>) -> mlua::Error {
    mlua::Error::runtime(i18n_t1(key, a))
}

pub(super) fn create_i18n_table(lua: &Lua) -> Result<Table, String> {
    lua.create_table()
        .map_err(|e| i18n_t1("plugins.runtime.i18n.create_table_failed", e.to_string()))
}

pub(super) fn set_i18n_function(
    table: &Table,
    name: &str,
    function: Function,
    error_suffix: &str,
) -> Result<(), String> {
    table
        .set(name, function)
        .map_err(|e| i18n_t2("plugins.runtime.i18n.set_api_failed", error_suffix, e.to_string()))
}

pub(super) fn set_i18n_table(sl: &Table, table: Table) -> Result<(), String> {
    sl.set("i18n", table)
        .map_err(|e| i18n_t1("plugins.runtime.i18n.set_namespace_failed", e.to_string()))
}

pub(super) fn callbacks_table(lua: &Lua, registry_key: &str) -> mlua::Result<Table> {
    lua.named_registry_value(registry_key)
        .or_else(|_| lua.create_table())
}

pub(super) fn remove_locale_callback_token(token_id: usize) {
    host_remove_locale_callback(token_id);
}
