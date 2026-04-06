use super::common::{callbacks_table, remove_locale_callback_token, I18nContext};
use crate::services::global::i18n_service;
use mlua::{Function, Table, Value};

pub(super) fn on_locale_change(ctx: &I18nContext, callback: Function) -> mlua::Result<i64> {
    let registry_key = ctx.callbacks_registry_key();
    let callbacks_table = callbacks_table(&ctx.lua, &registry_key)?;

    let index = callbacks_table.len()? + 1;
    callbacks_table.set(index, callback)?;

    ctx.lua
        .set_named_registry_value(&registry_key, callbacks_table)
        .map_err(|e| mlua::Error::runtime(format!("Failed to store callback function: {}", e)))?;

    let callback_plugin_id = ctx.plugin_id.clone();
    let lua_ref = ctx.lua.clone();
    let token_key = ctx.token_registry_key();
    let token = i18n_service().on_locale_change(move |_old_locale, new_locale| {
        let Ok(callbacks) = lua_ref.named_registry_value::<Table>(&format!(
            "_locale_change_callbacks_{}",
            callback_plugin_id
        )) else {
            return;
        };

        let Ok(len) = callbacks.len() else {
            return;
        };

        for i in 1..=len {
            if let Ok(callback) = callbacks.get::<Function>(i) {
                if let Err(e) = callback.call::<()>(new_locale) {
                    eprintln!("i18n callback error: {}", e);
                }
            }
        }
    });

    ctx.lua
        .set_named_registry_value(&token_key, token.0)
        .map_err(|e| mlua::Error::runtime(format!("Failed to store token: {}", e)))?;

    Ok(index)
}

pub(super) fn off_locale_change(ctx: &I18nContext, callback_id: usize) -> mlua::Result<bool> {
    let registry_key = ctx.callbacks_registry_key();
    let token_key = ctx.token_registry_key();

    if let Ok(callbacks_table) = ctx.lua.named_registry_value::<Table>(&registry_key) {
        let len = callbacks_table.len()?;
        if callback_id > 0 && callback_id <= len as usize {
            callbacks_table.set(callback_id, Value::Nil)?;

            if callbacks_table.len()? == 0 {
                let _ = ctx.lua.set_named_registry_value(&registry_key, Value::Nil);
                if let Ok(token_id) = ctx.lua.named_registry_value::<usize>(&token_key) {
                    let _ = ctx.lua.set_named_registry_value(&token_key, Value::Nil);
                    remove_locale_callback_token(token_id);
                }
            } else {
                let _ = ctx
                    .lua
                    .set_named_registry_value(&registry_key, callbacks_table);
            }

            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}
