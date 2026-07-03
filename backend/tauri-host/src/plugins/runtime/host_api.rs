use sea_lantern_lua_runtime_core::host::runtime_host_api;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

type LocaleChangeCallback = std::sync::Arc<dyn Fn(&str, &str) + Send + Sync>;

pub(crate) fn host_t(key: &str) -> String {
    runtime_host_api().i18n().t(key)
}

pub(crate) fn host_t_with_options(key: &str, options: &HashMap<String, String>) -> String {
    runtime_host_api().i18n().t_with_options(key, options)
}

pub(crate) fn host_get_locale() -> String {
    runtime_host_api().i18n().get_locale()
}

pub(crate) fn host_has_translation(key: &str) -> bool {
    runtime_host_api().i18n().has_translation(key)
}

pub(crate) fn host_has_translation_for_locale(locale: &str, key: &str) -> bool {
    runtime_host_api()
        .i18n()
        .has_translation_for_locale(locale, key)
}

pub(crate) fn host_get_all_translations() -> HashMap<String, String> {
    runtime_host_api().i18n().get_all_translations()
}

pub(crate) fn host_get_translations_for_locale(locale: &str) -> HashMap<String, String> {
    runtime_host_api()
        .i18n()
        .get_translations_for_locale(locale)
}

pub(crate) fn host_get_available_locales() -> Vec<String> {
    runtime_host_api().i18n().get_available_locales()
}

pub(crate) fn host_register_locale(plugin_id: &str, locale: &str, display_name: &str) {
    runtime_host_api()
        .i18n()
        .register_locale(plugin_id, locale, display_name);
}

pub(crate) fn host_add_plugin_translations(
    plugin_id: &str,
    locale: &str,
    entries: HashMap<String, String>,
) {
    runtime_host_api()
        .i18n()
        .add_plugin_translations(plugin_id, locale, entries);
}

pub(crate) fn host_plugin_translation_entry_count(plugin_id: &str) -> usize {
    runtime_host_api()
        .i18n()
        .plugin_translation_entry_count(plugin_id)
}

pub(crate) fn host_remove_plugin_translations(plugin_id: &str) {
    runtime_host_api()
        .i18n()
        .remove_plugin_translations(plugin_id);
}

pub(crate) fn host_on_locale_change(callback: LocaleChangeCallback) -> usize {
    runtime_host_api().i18n().on_locale_change(callback)
}

pub(crate) fn host_remove_locale_callback(token_id: usize) {
    runtime_host_api().i18n().remove_locale_callback(token_id);
}

pub(crate) fn host_call_api(
    source_plugin_id: &str,
    target_plugin_id: &str,
    api_name: &str,
    args: Vec<JsonValue>,
) -> Result<JsonValue, String> {
    runtime_host_api()
        .plugin()
        .call_api(source_plugin_id, target_plugin_id, api_name, args)
}

pub(crate) fn host_emit_ui_event(
    plugin_id: &str,
    action: &str,
    target: &str,
    payload: &str,
) -> Result<(), String> {
    runtime_host_api()
        .plugin()
        .emit_ui_event(plugin_id, action, target, payload)
}

pub(crate) fn host_emit_component_event(plugin_id: &str, payload: &str) -> Result<(), String> {
    runtime_host_api()
        .plugin()
        .emit_component_event(plugin_id, payload)
}

pub(crate) fn host_emit_context_menu_event(
    plugin_id: &str,
    action: &str,
    context: &str,
    payload: &str,
) -> Result<(), String> {
    runtime_host_api()
        .plugin()
        .emit_context_menu_event(plugin_id, action, context, payload)
}

#[allow(dead_code)]
pub(crate) fn host_emit_sidebar_event(
    plugin_id: &str,
    action: &str,
    label: &str,
    icon: &str,
) -> Result<(), String> {
    runtime_host_api()
        .plugin()
        .emit_sidebar_event(plugin_id, action, label, icon)
}

pub(crate) fn host_emit_i18n_event(
    plugin_id: &str,
    action: &str,
    locale: &str,
    payload: &str,
) -> Result<(), String> {
    runtime_host_api()
        .plugin()
        .emit_i18n_event(plugin_id, action, locale, payload)
}

#[allow(dead_code)]
pub(crate) fn host_emit_log_event(
    plugin_id: &str,
    level: &str,
    message: &str,
) -> Result<(), String> {
    runtime_host_api()
        .plugin()
        .emit_log_event(plugin_id, level, message)
}

pub(crate) fn host_emit_permission_log(
    plugin_id: &str,
    category: &str,
    api_name: &str,
    resource: &str,
) -> Result<(), String> {
    runtime_host_api()
        .plugin()
        .emit_permission_log(plugin_id, category, api_name, resource)
}

pub(crate) fn host_component_mirror_list(
    page_filter: Option<&str>,
) -> Vec<sea_lantern_lua_runtime_core::host::RuntimeComponentEntry> {
    runtime_host_api()
        .plugin()
        .component_mirror_list(page_filter)
}

#[allow(dead_code)]
pub(crate) fn host_element_response_create() -> (u64, std::sync::mpsc::Receiver<String>) {
    runtime_host_api().plugin().element_response_create()
}
