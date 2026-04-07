use super::PluginRuntime;
use mlua::Table;

mod common;
mod events;
mod query;
mod write;

use common::{create_i18n_table, set_i18n_function, set_i18n_table, I18nContext};

impl PluginRuntime {
    pub(super) fn setup_i18n_namespace(&self, sl: &Table) -> Result<(), String> {
        let i18n_table = create_i18n_table(&self.lua)?;
        let ctx = I18nContext::new(self.plugin_id.clone(), self.lua.clone());

        set_i18n_function(
            &i18n_table,
            "getLocale",
            self.lua
                .create_function(query::get_locale)
                .map_err(|e| format!("Failed to create i18n.getLocale: {}", e))?,
            "getLocale",
        )?;

        set_i18n_function(
            &i18n_table,
            "t",
            self.lua
                .create_function(query::translate)
                .map_err(|e| format!("Failed to create i18n.t: {}", e))?,
            "t",
        )?;

        set_i18n_function(
            &i18n_table,
            "hasTranslation",
            self.lua
                .create_function(query::has_translation)
                .map_err(|e| format!("Failed to create i18n.hasTranslation: {}", e))?,
            "hasTranslation",
        )?;

        set_i18n_function(
            &i18n_table,
            "tOrDefault",
            self.lua
                .create_function(query::t_or_default)
                .map_err(|e| format!("Failed to create i18n.tOrDefault: {}", e))?,
            "tOrDefault",
        )?;

        set_i18n_function(
            &i18n_table,
            "onLocaleChange",
            self.lua
                .create_function({
                    let ctx = ctx.clone();
                    move |_, callback| events::on_locale_change(&ctx, callback)
                })
                .map_err(|e| format!("Failed to create i18n.onLocaleChange: {}", e))?,
            "onLocaleChange",
        )?;

        set_i18n_function(
            &i18n_table,
            "offLocaleChange",
            self.lua
                .create_function({
                    let ctx = ctx.clone();
                    move |_, callback_id| events::off_locale_change(&ctx, callback_id)
                })
                .map_err(|e| format!("Failed to create i18n.offLocaleChange: {}", e))?,
            "offLocaleChange",
        )?;

        set_i18n_function(
            &i18n_table,
            "getAllTranslations",
            self.lua
                .create_function(query::get_all_translations)
                .map_err(|e| format!("Failed to create i18n.getAllTranslations: {}", e))?,
            "getAllTranslations",
        )?;

        set_i18n_function(
            &i18n_table,
            "getTranslations",
            self.lua
                .create_function(query::get_translations)
                .map_err(|e| format!("Failed to create i18n.getTranslations: {}", e))?,
            "getTranslations",
        )?;

        set_i18n_function(
            &i18n_table,
            "getAvailableLocales",
            self.lua
                .create_function(query::get_available_locales)
                .map_err(|e| format!("Failed to create i18n.getAvailableLocales: {}", e))?,
            "getAvailableLocales",
        )?;

        set_i18n_function(
            &i18n_table,
            "tp",
            self.lua
                .create_function(query::translate_plugin)
                .map_err(|e| format!("Failed to create i18n.tp: {}", e))?,
            "tp",
        )?;

        set_i18n_function(
            &i18n_table,
            "registerLocale",
            self.lua
                .create_function({
                    let ctx = ctx.clone();
                    move |_, args| write::register_locale(&ctx, args)
                })
                .map_err(|e| format!("Failed to create i18n.registerLocale: {}", e))?,
            "registerLocale",
        )?;

        set_i18n_function(
            &i18n_table,
            "addTranslations",
            self.lua
                .create_function({
                    let ctx = ctx.clone();
                    move |_, args| write::add_translations(&ctx, args)
                })
                .map_err(|e| format!("Failed to create i18n.addTranslations: {}", e))?,
            "addTranslations",
        )?;

        set_i18n_function(
            &i18n_table,
            "removeTranslations",
            self.lua
                .create_function({
                    let ctx = ctx.clone();
                    move |_, ()| write::remove_translations(&ctx)
                })
                .map_err(|e| format!("Failed to create i18n.removeTranslations: {}", e))?,
            "removeTranslations",
        )?;

        set_i18n_table(sl, i18n_table)
    }
}
