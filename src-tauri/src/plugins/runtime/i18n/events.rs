use super::common::{callbacks_table, remove_locale_callback_token, I18nContext};
use crate::services::global::i18n_service;
use mlua::{Function, Table, Value};

const NEXT_CALLBACK_ID_FIELD: &str = "__next_callback_id";

pub(super) fn on_locale_change(ctx: &I18nContext, callback: Function) -> mlua::Result<i64> {
    let registry_key = ctx.callbacks_registry_key();
    let token_key = ctx.token_registry_key();
    let callbacks_table = callbacks_table(&ctx.lua, &registry_key)?;

    let next_id = callbacks_table
        .get::<i64>(NEXT_CALLBACK_ID_FIELD)
        .unwrap_or(1)
        .max(1);
    callbacks_table.set(next_id, callback)?;
    callbacks_table.set(NEXT_CALLBACK_ID_FIELD, next_id + 1)?;

    ctx.lua
        .set_named_registry_value(&registry_key, callbacks_table)
        .map_err(|e| mlua::Error::runtime(format!("Failed to store callback function: {}", e)))?;

    ensure_locale_listener_registered(ctx, &token_key)?;

    Ok(next_id)
}

pub(super) fn off_locale_change(ctx: &I18nContext, callback_id: usize) -> mlua::Result<bool> {
    let registry_key = ctx.callbacks_registry_key();
    let token_key = ctx.token_registry_key();

    if callback_id == 0 {
        return Ok(false);
    }

    let Ok(callbacks_table) = ctx.lua.named_registry_value::<Table>(&registry_key) else {
        return Ok(false);
    };

    if matches!(callbacks_table.raw_get::<Value>(callback_id)?, Value::Nil) {
        return Ok(false);
    }

    callbacks_table.raw_set(callback_id, Value::Nil)?;

    if has_registered_callbacks(&callbacks_table)? {
        ctx.lua
            .set_named_registry_value(&registry_key, callbacks_table)
            .map_err(|e| mlua::Error::runtime(format!("Failed to update callbacks: {}", e)))?;
    } else {
        let _ = ctx.lua.set_named_registry_value(&registry_key, Value::Nil);
        if let Ok(token_id) = ctx.lua.named_registry_value::<usize>(&token_key) {
            let _ = ctx.lua.set_named_registry_value(&token_key, Value::Nil);
            remove_locale_callback_token(token_id);
        }
    }

    Ok(true)
}

fn ensure_locale_listener_registered(ctx: &I18nContext, token_key: &str) -> mlua::Result<()> {
    if ctx.lua.named_registry_value::<usize>(token_key).is_ok() {
        return Ok(());
    }

    let callback_plugin_id = ctx.plugin_id.clone();
    let lua_ref = ctx.lua.clone();
    let token = i18n_service().on_locale_change(move |_old_locale, new_locale| {
        let Ok(callbacks) = lua_ref.named_registry_value::<Table>(&format!(
            "_locale_change_callbacks_{}",
            callback_plugin_id
        )) else {
            return;
        };

        let Ok(pairs) = callbacks
            .pairs::<Value, Function>()
            .collect::<mlua::Result<Vec<_>>>()
        else {
            return;
        };

        for (key, callback) in pairs {
            if matches!(key, Value::String(_)) {
                continue;
            }

            if let Err(e) = callback.call::<()>(new_locale) {
                eprintln!("i18n callback error: {}", e);
            }
        }
    });

    ctx.lua
        .set_named_registry_value(token_key, token.0)
        .map_err(|e| mlua::Error::runtime(format!("Failed to store token: {}", e)))
}

fn has_registered_callbacks(callbacks_table: &Table) -> mlua::Result<bool> {
    for pair in callbacks_table.pairs::<Value, Value>() {
        let (key, value) = pair?;
        if matches!(key, Value::String(_)) {
            continue;
        }

        if !matches!(value, Value::Nil) {
            return Ok(true);
        }
    }

    Ok(false)
}
