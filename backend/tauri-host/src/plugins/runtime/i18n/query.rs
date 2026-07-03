use super::common::{i18n_err, i18n_err1, plugin_i18n_namespace};
use crate::plugins::runtime::host_api::{
    host_get_all_translations, host_get_available_locales, host_get_locale,
    host_get_translations_for_locale, host_has_translation, host_has_translation_for_locale,
    host_t, host_t_with_options,
};
use mlua::{Lua, Table, Value, Variadic};
use std::collections::HashMap;

pub(super) fn get_locale(_: &Lua, (): ()) -> mlua::Result<String> {
    Ok(host_get_locale())
}

pub(super) fn translate(_: &Lua, args: Variadic<Value>) -> mlua::Result<String> {
    let key = required_string_arg(&args, 0, "plugins.runtime.i18n.translate_key_string_required")?;

    match args.get(1) {
        Some(Value::Table(options)) => Ok(host_t_with_options(&key, &table_to_options(options)?)),
        Some(Value::Nil) | None => Ok(host_t(&key)),
        Some(_) => Err(i18n_err("plugins.runtime.i18n.translate_options_table_required")),
    }
}

pub(super) fn has_translation(_: &Lua, args: Variadic<Value>) -> mlua::Result<bool> {
    let key =
        required_string_arg(&args, 0, "plugins.runtime.i18n.has_translation_key_string_required")?;

    match optional_string_arg(&args, 1)? {
        Some(locale) => Ok(host_has_translation_for_locale(&locale, &key)),
        None => Ok(host_has_translation(&key)),
    }
}

pub(super) fn t_or_default(_: &Lua, args: Variadic<Value>) -> mlua::Result<String> {
    let key = required_string_arg(
        &args,
        0,
        "plugins.runtime.i18n.translate_or_default_key_string_required",
    )?;
    let default_value = required_string_arg(
        &args,
        1,
        "plugins.runtime.i18n.translate_or_default_value_string_required",
    )?;

    let options = match args.get(2) {
        Some(Value::Table(options)) => Some(table_to_options(options)?),
        Some(Value::Nil) | None => None,
        Some(_) => {
            return Err(i18n_err(
                "plugins.runtime.i18n.translate_or_default_options_table_required",
            ));
        }
    };

    if host_has_translation(&key) {
        Ok(match options {
            Some(options) => host_t_with_options(&key, &options),
            None => host_t(&key),
        })
    } else {
        Ok(default_value)
    }
}

pub(super) fn get_all_translations(lua: &Lua, (): ()) -> mlua::Result<Table> {
    map_to_lua_table(lua, host_get_all_translations())
}

pub(super) fn get_translations(lua: &Lua, locale: String) -> mlua::Result<Table> {
    map_to_lua_table(lua, host_get_translations_for_locale(&locale))
}

pub(super) fn get_available_locales(lua: &Lua, (): ()) -> mlua::Result<Table> {
    let locales = host_get_available_locales();
    let table = lua.create_table()?;
    for (i, locale) in locales.iter().enumerate() {
        table.set(i + 1, locale.clone())?;
    }
    Ok(table)
}

pub(super) fn translate_plugin(
    _: &Lua,
    (plugin_id, key): (String, String),
) -> mlua::Result<String> {
    Ok(host_t(&plugin_i18n_namespace(&plugin_id, &key)))
}

fn required_string_arg(
    args: &Variadic<Value>,
    index: usize,
    err_key: &str,
) -> mlua::Result<String> {
    match args.get(index) {
        Some(Value::String(s)) => s
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| i18n_err("plugins.runtime.i18n.invalid_utf8_string")),
        _ => Err(i18n_err(err_key)),
    }
}

fn optional_string_arg(args: &Variadic<Value>, index: usize) -> mlua::Result<Option<String>> {
    match args.get(index) {
        Some(Value::String(s)) => s
            .to_str()
            .map(|s| Some(s.to_string()))
            .map_err(|_| i18n_err("plugins.runtime.i18n.invalid_utf8_string")),
        Some(Value::Nil) | None => Ok(None),
        Some(_) => Err(i18n_err1(
            "plugins.runtime.i18n.string_or_nil_expected",
            (index + 1).to_string(),
        )),
    }
}

fn map_to_lua_table(lua: &Lua, translations: HashMap<String, String>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (k, v) in translations {
        table.set(k, v)?;
    }
    Ok(table)
}

fn table_to_options(options: &Table) -> mlua::Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for pair in options.pairs::<String, String>() {
        let (k, v) = pair?;
        map.insert(k, v);
    }
    Ok(map)
}
