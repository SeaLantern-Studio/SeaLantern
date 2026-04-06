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
