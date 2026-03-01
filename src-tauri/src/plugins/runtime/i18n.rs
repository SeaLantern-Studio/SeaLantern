use super::PluginRuntime;
use mlua::Table;

impl PluginRuntime {
    pub(super) fn setup_i18n_namespace(&self, sl: &Table) -> Result<(), String> {
        use crate::services::global::i18n_service;

        let i18n_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create i18n table: {}", e))?;

        let plugin_id = self.plugin_id.clone();

        let get_locale_fn = self
            .lua
            .create_function(move |_lua, ()| {
                let i18n = i18n_service();
                Ok(i18n.get_locale())
            })
            .map_err(|e| format!("Failed to create i18n.getLocale: {}", e))?;
        i18n_table
            .set("getLocale", get_locale_fn)
            .map_err(|e| format!("Failed to set i18n.getLocale: {}", e))?;

        let t_fn = self
            .lua
            .create_function(move |_, args: mlua::Variadic<mlua::Value>| {
                let i18n = i18n_service();

                let key = match args.first() {
                    Some(mlua::Value::String(s)) => s
                        .to_str()
                        .map(|s| s.to_string())
                        .map_err(|_| mlua::Error::runtime("Failed to convert string to UTF-8"))?,
                    _ => {
                        return Err(mlua::Error::runtime(
                            "i18n.t requires a string key as first argument",
                        ))
                    }
                };

                if let Some(mlua::Value::Table(options)) = args.get(1) {
                    let mut opts = std::collections::HashMap::new();
                    for (k, v) in options.pairs::<String, String>().flatten() {
                        opts.insert(k, v);
                    }
                    Ok(i18n.t_with_options(&key, &opts))
                } else {
                    Ok(i18n.t(&key))
                }
            })
            .map_err(|e| format!("Failed to create i18n.t: {}", e))?;
        i18n_table
            .set("t", t_fn)
            .map_err(|e| format!("Failed to set i18n.t: {}", e))?;

        let lua_weak = self.lua.clone();
        let pid = plugin_id.clone();
        let on_locale_change_fn = self
            .lua
            .create_function(move |_, callback: mlua::Function| {
                let registry_key = format!("_locale_change_callbacks_{}", pid);
                let callbacks_table: mlua::Table = lua_weak
                    .named_registry_value(&registry_key)
                    .unwrap_or_else(|_| {
                        lua_weak
                            .create_table()
                            .expect("Failed to create callbacks table")
                    });

                let index = callbacks_table.len()? + 1;
                callbacks_table.set(index, callback)?;

                lua_weak
                    .set_named_registry_value(&registry_key, callbacks_table)
                    .map_err(|e| {
                        mlua::Error::runtime(format!("Failed to store callback function: {}", e))
                    })?;

                let cb_pid = pid.clone();
                let lua_ref = lua_weak.clone();
                let token_key_for_storage = format!("_locale_callback_token_{}", cb_pid);
                let token = i18n_service().on_locale_change(move |_old_locale, new_locale| {
                    let Ok(callbacks) = lua_ref.named_registry_value::<mlua::Table>(&format!(
                        "_locale_change_callbacks_{}",
                        cb_pid
                    )) else {
                        return;
                    };
                    let Ok(len) = callbacks.len() else {
                        return;
                    };
                    for i in 1..=len {
                        if let Ok(callback) = callbacks.get::<mlua::Function>(i) {
                            if let Err(e) = callback.call::<()>(new_locale) {
                                eprintln!("i18n callback error: {}", e);
                            }
                        }
                    }
                });

                lua_weak
                    .set_named_registry_value(&token_key_for_storage, token.0)
                    .map_err(|e| mlua::Error::runtime(format!("Failed to store token: {}", e)))?;

                Ok(index)
            })
            .map_err(|e| format!("Failed to create i18n.onLocaleChange: {}", e))?;
        i18n_table
            .set("onLocaleChange", on_locale_change_fn)
            .map_err(|e| format!("Failed to set i18n.onLocaleChange: {}", e))?;

        let lua_weak_off = self.lua.clone();
        let pid_off = plugin_id.clone();
        let off_locale_change_fn = self
            .lua
            .create_function(move |_, callback_id: usize| {
                let registry_key = format!("_locale_change_callbacks_{}", pid_off);
                let token_key = format!("_locale_callback_token_{}", pid_off);

                if let Ok(callbacks_table) =
                    lua_weak_off.named_registry_value::<mlua::Table>(&registry_key)
                {
                    let len = callbacks_table.len()?;
                    if callback_id > 0 && callback_id <= len as usize {
                        callbacks_table.set(callback_id, mlua::Value::Nil)?;

                        if callbacks_table.len()? == 0 {
                            let _ = lua_weak_off
                                .set_named_registry_value(&registry_key, mlua::Value::Nil);
                            if let Ok(_token) =
                                lua_weak_off.named_registry_value::<usize>(&token_key)
                            {
                                let _ = lua_weak_off
                                    .set_named_registry_value(&token_key, mlua::Value::Nil);
                                let i18n = i18n_service();
                                i18n.remove_locale_callback(
                                    &crate::services::i18n::LocaleCallbackToken(_token),
                                );
                            }
                        } else {
                            let _ = lua_weak_off
                                .set_named_registry_value(&registry_key, callbacks_table);
                        }

                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            })
            .map_err(|e| format!("Failed to create i18n.offLocaleChange: {}", e))?;
        i18n_table
            .set("offLocaleChange", off_locale_change_fn)
            .map_err(|e| format!("Failed to set i18n.offLocaleChange: {}", e))?;

        let get_all_translations_fn = self
            .lua
            .create_function(move |lua, ()| {
                let i18n = i18n_service();
                let translations = i18n.get_all_translations();
                let table = lua.create_table()?;
                for (k, v) in translations {
                    table.set(k, v)?;
                }
                Ok(table)
            })
            .map_err(|e| format!("Failed to create i18n.getAllTranslations: {}", e))?;
        i18n_table
            .set("getAllTranslations", get_all_translations_fn)
            .map_err(|e| format!("Failed to set i18n.getAllTranslations: {}", e))?;

        let get_available_locales_fn = self
            .lua
            .create_function(move |lua, ()| {
                let i18n = i18n_service();
                let locales = i18n.get_available_locales();
                let table = lua.create_table()?;
                for (i, locale) in locales.iter().enumerate() {
                    table.set(i + 1, locale.clone())?;
                }
                Ok(table)
            })
            .map_err(|e| format!("Failed to create i18n.getAvailableLocales: {}", e))?;
        i18n_table
            .set("getAvailableLocales", get_available_locales_fn)
            .map_err(|e| format!("Failed to set i18n.getAvailableLocales: {}", e))?;

        let pid_reg = plugin_id.clone();
        let register_locale_fn = self
            .lua
            .create_function(move |_, (locale, display_name): (String, String)| {
                use crate::plugins::api::emit_i18n_event;
                let i18n = i18n_service();
                i18n.register_locale(&pid_reg, &locale, &display_name);
                let payload = serde_json::json!({ "displayName": display_name }).to_string();
                if let Err(e) = emit_i18n_event(&pid_reg, "register_locale", &locale, &payload) {
                    eprintln!("Failed to emit i18n event: {}", e);
                }
                Ok(())
            })
            .map_err(|e| format!("Failed to create i18n.registerLocale: {}", e))?;
        i18n_table
            .set("registerLocale", register_locale_fn)
            .map_err(|e| format!("Failed to set i18n.registerLocale: {}", e))?;

        let pid_add = plugin_id.clone();
        let add_translations_fn = self
            .lua
            .create_function(move |_, (locale, entries): (String, mlua::Table)| {
                use crate::plugins::api::emit_i18n_event;
                let i18n = i18n_service();
                let mut map = std::collections::HashMap::new();
                for (k, v) in entries.pairs::<String, String>().flatten() {
                    map.insert(k, v);
                }
                let payload = serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string());
                i18n.add_plugin_translations(&pid_add, &locale, map);
                if let Err(e) = emit_i18n_event(&pid_add, "add_translations", &locale, &payload) {
                    eprintln!("Failed to emit i18n event: {}", e);
                }
                Ok(())
            })
            .map_err(|e| format!("Failed to create i18n.addTranslations: {}", e))?;
        i18n_table
            .set("addTranslations", add_translations_fn)
            .map_err(|e| format!("Failed to set i18n.addTranslations: {}", e))?;

        let pid_rm = plugin_id.clone();
        let remove_translations_fn = self
            .lua
            .create_function(move |_, ()| {
                use crate::plugins::api::emit_i18n_event;
                let i18n = i18n_service();
                i18n.remove_plugin_translations(&pid_rm);
                if let Err(e) = emit_i18n_event(&pid_rm, "remove_translations", "", "") {
                    eprintln!("Failed to emit i18n event: {}", e);
                }
                Ok(())
            })
            .map_err(|e| format!("Failed to create i18n.removeTranslations: {}", e))?;
        i18n_table
            .set("removeTranslations", remove_translations_fn)
            .map_err(|e| format!("Failed to set i18n.removeTranslations: {}", e))?;

        sl.set("i18n", i18n_table)
            .map_err(|e| format!("Failed to set sl.i18n: {}", e))?;

        Ok(())
    }
}
