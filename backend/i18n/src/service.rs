use crate::{embedded_table, SUPPORTED_LOCALES};
use std::collections::HashMap;
use std::sync::RwLock;

type LocaleChangeCallback = dyn Fn(&str, &str) + Send + Sync;
type LocaleChangeCallbackMap = HashMap<usize, Box<LocaleChangeCallback>>;
type LocaleTranslations = HashMap<String, String>;
type PluginLocaleTranslations = HashMap<String, LocaleTranslations>;
type PluginTranslationStore = HashMap<String, PluginLocaleTranslations>;

pub struct I18nService {
    translations: RwLock<HashMap<String, HashMap<String, String>>>,
    locale: RwLock<String>,
    change_callbacks: RwLock<LocaleChangeCallbackMap>,
    next_callback_id: RwLock<usize>,
    plugin_locale_owners: RwLock<HashMap<String, String>>,
    plugin_locale_names: RwLock<HashMap<String, String>>,
    plugin_translations: RwLock<PluginTranslationStore>,
    fallback_locale: &'static str,
}

#[derive(Clone, Debug)]
pub struct LocaleCallbackToken(pub usize);

impl I18nService {
    pub fn new() -> Self {
        Self::with_embedded_locales(SUPPORTED_LOCALES, "zh-CN")
    }

    pub fn with_embedded_locales(locales: &[&str], fallback_locale: &'static str) -> Self {
        let mut translations = HashMap::new();

        for &locale in locales {
            translations.insert(locale.to_string(), embedded_table(locale));
        }

        Self::with_translations(translations, fallback_locale)
    }

    pub fn with_translations(
        translations: HashMap<String, HashMap<String, String>>,
        fallback_locale: &'static str,
    ) -> Self {
        let initial_locale = if translations.contains_key(fallback_locale) {
            fallback_locale.to_string()
        } else {
            translations
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| fallback_locale.to_string())
        };

        Self {
            translations: RwLock::new(translations),
            locale: RwLock::new(initial_locale),
            change_callbacks: RwLock::new(HashMap::new()),
            next_callback_id: RwLock::new(1),
            plugin_locale_owners: RwLock::new(HashMap::new()),
            plugin_locale_names: RwLock::new(HashMap::new()),
            plugin_translations: RwLock::new(HashMap::new()),
            fallback_locale,
        }
    }

    pub fn get_locale(&self) -> String {
        self.locale
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn set_locale(&self, locale: &str) {
        let old_locale = self
            .locale
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        *self.locale.write().unwrap_or_else(|e| e.into_inner()) = locale.to_string();

        let callbacks = self
            .change_callbacks
            .read()
            .unwrap_or_else(|e| e.into_inner());
        for callback in callbacks.values() {
            callback(&old_locale, locale);
        }
    }

    pub fn on_locale_change<F>(&self, callback: F) -> LocaleCallbackToken
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        let id = {
            let mut next_id = self
                .next_callback_id
                .write()
                .unwrap_or_else(|e| e.into_inner());
            let id = *next_id;
            *next_id += 1;
            id
        };

        self.change_callbacks
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, Box::new(callback));

        LocaleCallbackToken(id)
    }

    pub fn remove_locale_callback(&self, token: &LocaleCallbackToken) {
        self.change_callbacks
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&token.0);
    }

    pub fn t(&self, key: &str) -> String {
        self.translate_for_locale(&self.get_locale(), key)
            .unwrap_or_else(|| key.to_string())
    }

    pub fn t_for_locale(&self, locale: &str, key: &str) -> String {
        self.translate_for_locale(locale, key)
            .unwrap_or_else(|| key.to_string())
    }

    pub fn t_with_options(&self, key: &str, options: &HashMap<String, String>) -> String {
        let mut result = self.t(key);
        for (k, v) in options {
            result = result.replace(&format!("{{{}}}", k), v);
        }
        result
    }

    pub fn t_with_options_for_locale(
        &self,
        locale: &str,
        key: &str,
        options: &HashMap<String, String>,
    ) -> String {
        let mut result = self.t_for_locale(locale, key);
        for (k, v) in options {
            result = result.replace(&format!("{{{}}}", k), v);
        }
        result
    }

    pub fn has_translation(&self, key: &str) -> bool {
        self.has_translation_for_locale(&self.get_locale(), key)
    }

    pub fn has_translation_for_locale(&self, locale: &str, key: &str) -> bool {
        self.translate_for_locale(locale, key).is_some()
    }

    pub fn get_all_translations(&self) -> HashMap<String, String> {
        self.get_translations_for_locale(&self.get_locale())
    }

    pub fn get_translations_for_locale(&self, locale: &str) -> HashMap<String, String> {
        self.merge_translations_for_locale(locale)
    }

    pub fn get_available_locales(&self) -> Vec<String> {
        let translations = self.translations.read().unwrap_or_else(|e| e.into_inner());
        let mut locales: Vec<String> = SUPPORTED_LOCALES
            .iter()
            .filter(|locale| translations.contains_key(**locale))
            .map(|locale| (*locale).to_string())
            .collect();

        for locale in translations.keys() {
            if !locales.contains(locale) {
                locales.push(locale.clone());
            }
        }

        let owners = self
            .plugin_locale_owners
            .read()
            .unwrap_or_else(|e| e.into_inner());
        for locale in owners.keys() {
            if !locales.contains(locale) {
                locales.push(locale.clone());
            }
        }
        locales
    }

    pub fn get_locale_display_name(&self, locale: &str) -> Option<String> {
        self.plugin_locale_names
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(locale)
            .cloned()
    }

    pub fn register_locale(&self, plugin_id: &str, locale: &str, display_name: &str) {
        self.plugin_locale_owners
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(locale.to_string(), plugin_id.to_string());
        self.plugin_locale_names
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(locale.to_string(), display_name.to_string());
    }

    pub fn add_plugin_translations(
        &self,
        plugin_id: &str,
        locale: &str,
        entries: HashMap<String, String>,
    ) {
        let mut plugin_trans = self
            .plugin_translations
            .write()
            .unwrap_or_else(|e| e.into_inner());
        let plugin_map = plugin_trans.entry(plugin_id.to_string()).or_default();
        let locale_map = plugin_map.entry(locale.to_string()).or_default();
        locale_map.extend(entries);
    }

    pub fn plugin_translation_entry_count(&self, plugin_id: &str) -> usize {
        self.plugin_translations
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(plugin_id)
            .map(|locale_map| locale_map.values().map(HashMap::len).sum())
            .unwrap_or(0)
    }

    pub fn remove_plugin_translations(&self, plugin_id: &str) {
        self.plugin_translations
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .remove(plugin_id);

        let locales_to_remove: Vec<String> = self
            .plugin_locale_owners
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .filter(|(_, owner)| owner.as_str() == plugin_id)
            .map(|(locale, _)| locale.clone())
            .collect();

        {
            let mut owners = self
                .plugin_locale_owners
                .write()
                .unwrap_or_else(|e| e.into_inner());
            let mut names = self
                .plugin_locale_names
                .write()
                .unwrap_or_else(|e| e.into_inner());
            for locale in &locales_to_remove {
                owners.remove(locale);
                names.remove(locale);
            }
        }
    }

    fn translate_for_locale(&self, locale: &str, key: &str) -> Option<String> {
        self.resolve_translation_for_locale(locale, key)
            .or_else(|| {
                if locale != self.fallback_locale {
                    self.resolve_translation_for_locale(self.fallback_locale, key)
                } else {
                    None
                }
            })
    }

    fn resolve_translation_for_locale(&self, locale: &str, key: &str) -> Option<String> {
        if let Some(locale_translations) = self
            .translations
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(locale)
        {
            if let Some(value) = locale_translations.get(key) {
                return Some(value.clone());
            }
        }

        let plugin_trans = self
            .plugin_translations
            .read()
            .unwrap_or_else(|e| e.into_inner());
        for plugin_map in plugin_trans.values() {
            if let Some(locale_map) = plugin_map.get(locale) {
                if let Some(value) = locale_map.get(key) {
                    return Some(value.clone());
                }
            }
        }

        None
    }

    fn merge_translations_for_locale(&self, locale: &str) -> HashMap<String, String> {
        let mut merged = self
            .translations
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(locale)
            .cloned()
            .unwrap_or_default();

        let plugin_trans = self
            .plugin_translations
            .read()
            .unwrap_or_else(|e| e.into_inner());
        for plugin_map in plugin_trans.values() {
            if let Some(locale_map) = plugin_map.get(locale) {
                for (k, v) in locale_map {
                    merged.entry(k.clone()).or_insert_with(|| v.clone());
                }
            }
        }

        merged
    }
}

impl Default for I18nService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::I18nService;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[test]
    fn falls_back_to_default_locale_for_missing_key() {
        let mut translations = HashMap::new();
        translations.insert(
            "zh-CN".to_string(),
            HashMap::from([(String::from("demo.key"), String::from("中文默认值"))]),
        );
        translations.insert("en-US".to_string(), HashMap::new());

        let service = I18nService::with_translations(translations, "zh-CN");
        service.set_locale("en-US");

        assert_eq!(service.t("demo.key"), "中文默认值");
    }

    #[test]
    fn plugin_translations_are_merged_without_overriding_embedded_keys() {
        let service = I18nService::new();
        let mut entries = HashMap::new();
        entries.insert("plugins.demo.custom".to_string(), "Plugin custom text".to_string());
        entries.insert(
            "console.empty_command".to_string(),
            "Plugin override should not win".to_string(),
        );

        service.add_plugin_translations("demo", "en-US", entries);

        let merged = service.get_translations_for_locale("en-US");
        assert_eq!(merged.get("plugins.demo.custom"), Some(&"Plugin custom text".to_string()));
        assert_ne!(
            merged.get("console.empty_command"),
            Some(&"Plugin override should not win".to_string())
        );
    }

    #[test]
    fn locale_change_callbacks_can_be_removed() {
        let service = I18nService::new();
        let seen = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
        let seen_ref = Arc::clone(&seen);

        let token = service.on_locale_change(move |old_locale, new_locale| {
            seen_ref
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push((old_locale.to_string(), new_locale.to_string()));
        });

        service.set_locale("en-US");
        service.remove_locale_callback(&token);
        service.set_locale("ja-JP");

        let events = seen.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], ("zh-CN".to_string(), "en-US".to_string()));
    }
}
