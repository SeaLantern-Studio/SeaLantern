use crate::services::global::i18n_service;
use mlua::{Lua, Table, Value, Variadic};
use std::collections::HashMap;

pub(super) fn get_locale(_: &Lua, (): ()) -> mlua::Result<String> {
    Ok(i18n_service().get_locale())
}

pub(super) fn translate(_: &Lua, args: Variadic<Value>) -> mlua::Result<String> {
    let i18n = i18n_service();

    let key = match args.first() {
        Some(Value::String(s)) => s
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| mlua::Error::runtime("Failed to convert string to UTF-8"))?,
        _ => return Err(mlua::Error::runtime("i18n.t requires a string key as first argument")),
    };

    if let Some(Value::Table(options)) = args.get(1) {
        Ok(i18n.t_with_options(&key, &table_to_options(options)?))
    } else {
        Ok(i18n.t(&key))
    }
}

pub(super) fn get_all_translations(lua: &Lua, (): ()) -> mlua::Result<Table> {
    let translations = i18n_service().get_all_translations();
    let table = lua.create_table()?;
    for (k, v) in translations {
        table.set(k, v)?;
    }
    Ok(table)
}

pub(super) fn get_available_locales(lua: &Lua, (): ()) -> mlua::Result<Table> {
    let locales = i18n_service().get_available_locales();
    let table = lua.create_table()?;
    for (i, locale) in locales.iter().enumerate() {
        table.set(i + 1, locale.clone())?;
    }
    Ok(table)
}

fn table_to_options(options: &Table) -> mlua::Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for (k, v) in options.pairs::<String, String>().flatten() {
        map.insert(k, v);
    }
    Ok(map)
}
