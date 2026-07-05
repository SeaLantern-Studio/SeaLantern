use crate::plugins::api::{
    call_api, component_mirror_list, element_response_create, emit_component_event,
    emit_context_menu_event, emit_i18n_event, emit_log_event, emit_permission_log,
    emit_sidebar_event, emit_ui_event,
};
use crate::services::global::i18n_service;
use sea_lantern_i18n_core::LocaleCallbackToken;
use sea_lantern_lua_runtime_core::host::{
    install_runtime_host_api, LocaleChangeCallback, RuntimeComponentEntry, RuntimeHostApi,
    RuntimeI18nApi, RuntimePluginApi,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

struct TauriRuntimeI18nApi;

impl RuntimeI18nApi for TauriRuntimeI18nApi {
    fn t(&self, key: &str) -> String {
        i18n_service().t(key)
    }

    fn t_with_options(&self, key: &str, options: &HashMap<String, String>) -> String {
        i18n_service().t_with_options(key, options)
    }

    fn get_locale(&self) -> String {
        i18n_service().get_locale()
    }

    fn has_translation(&self, key: &str) -> bool {
        i18n_service().has_translation(key)
    }

    fn has_translation_for_locale(&self, locale: &str, key: &str) -> bool {
        i18n_service().has_translation_for_locale(locale, key)
    }

    fn get_all_translations(&self) -> HashMap<String, String> {
        i18n_service().get_all_translations()
    }

    fn get_translations_for_locale(&self, locale: &str) -> HashMap<String, String> {
        i18n_service().get_translations_for_locale(locale)
    }

    fn get_available_locales(&self) -> Vec<String> {
        i18n_service().get_available_locales()
    }

    fn register_locale(&self, plugin_id: &str, locale: &str, display_name: &str) {
        i18n_service().register_locale(plugin_id, locale, display_name);
    }

    fn add_plugin_translations(
        &self,
        plugin_id: &str,
        locale: &str,
        entries: HashMap<String, String>,
    ) {
        i18n_service().add_plugin_translations(plugin_id, locale, entries);
    }

    fn plugin_translation_entry_count(&self, plugin_id: &str) -> usize {
        i18n_service().plugin_translation_entry_count(plugin_id)
    }

    fn remove_plugin_translations(&self, plugin_id: &str) {
        i18n_service().remove_plugin_translations(plugin_id);
    }

    fn on_locale_change(&self, callback: LocaleChangeCallback) -> usize {
        i18n_service()
            .on_locale_change(move |old_locale, new_locale| callback(old_locale, new_locale))
            .0
    }

    fn remove_locale_callback(&self, token_id: usize) {
        i18n_service().remove_locale_callback(&LocaleCallbackToken(token_id));
    }
}

struct TauriRuntimePluginApi;

impl RuntimePluginApi for TauriRuntimePluginApi {
    fn call_api(
        &self,
        source_plugin_id: &str,
        target_plugin_id: &str,
        api_name: &str,
        args: Vec<JsonValue>,
    ) -> Result<JsonValue, String> {
        call_api(source_plugin_id, target_plugin_id, api_name, args)
    }

    fn emit_ui_event(
        &self,
        plugin_id: &str,
        action: &str,
        target: &str,
        payload: &str,
    ) -> Result<(), String> {
        emit_ui_event(plugin_id, action, target, payload)
    }

    fn emit_component_event(&self, plugin_id: &str, payload: &str) -> Result<(), String> {
        emit_component_event(plugin_id, payload)
    }

    fn emit_context_menu_event(
        &self,
        plugin_id: &str,
        action: &str,
        context: &str,
        payload: &str,
    ) -> Result<(), String> {
        emit_context_menu_event(plugin_id, action, context, payload)
    }

    fn emit_sidebar_event(
        &self,
        plugin_id: &str,
        action: &str,
        label: &str,
        icon: &str,
    ) -> Result<(), String> {
        emit_sidebar_event(plugin_id, action, label, icon)
    }

    fn emit_i18n_event(
        &self,
        plugin_id: &str,
        action: &str,
        locale: &str,
        payload: &str,
    ) -> Result<(), String> {
        emit_i18n_event(plugin_id, action, locale, payload)
    }

    fn emit_log_event(&self, plugin_id: &str, level: &str, message: &str) -> Result<(), String> {
        emit_log_event(plugin_id, level, message)
    }

    fn emit_permission_log(
        &self,
        plugin_id: &str,
        category: &str,
        api_name: &str,
        resource: &str,
    ) -> Result<(), String> {
        emit_permission_log(plugin_id, category, api_name, resource)
    }

    fn component_mirror_list(&self, page_filter: Option<&str>) -> Vec<RuntimeComponentEntry> {
        component_mirror_list(page_filter)
            .into_iter()
            .map(|entry| RuntimeComponentEntry {
                id: entry.id,
                component_type: entry.component_type,
            })
            .collect()
    }

    fn element_response_create(&self) -> (u64, std::sync::mpsc::Receiver<String>) {
        element_response_create()
    }
}

struct TauriRuntimeHostApi {
    i18n: TauriRuntimeI18nApi,
    plugin: TauriRuntimePluginApi,
}

impl RuntimeHostApi for TauriRuntimeHostApi {
    fn i18n(&self) -> &dyn RuntimeI18nApi {
        &self.i18n
    }

    fn plugin(&self) -> &dyn RuntimePluginApi {
        &self.plugin
    }
}

pub(crate) fn ensure_runtime_host_api_installed() {
    let _ = install_runtime_host_api(Arc::new(TauriRuntimeHostApi {
        i18n: TauriRuntimeI18nApi,
        plugin: TauriRuntimePluginApi,
    }));
}
