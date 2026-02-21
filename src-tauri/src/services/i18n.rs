use std::collections::HashMap;
use std::sync::RwLock;

pub const SUPPORTED_LOCALES: &[&str] = &["zh-CN", "en-US", "zh-TW"];

// 类型别名，简化复杂类型定义
type LocaleCallback = Box<dyn Fn(&str, &str) + Send + Sync>;
type TranslationsMap = HashMap<String, HashMap<String, String>>;
type PluginTranslationsMap = HashMap<String, TranslationsMap>;

pub struct I18nService {
    translations: RwLock<TranslationsMap>,
    locale: RwLock<String>,
    change_callbacks: RwLock<HashMap<usize, LocaleCallback>>,
    next_callback_id: RwLock<usize>,
    plugin_locale_owners: RwLock<HashMap<String, String>>,
    plugin_locale_names: RwLock<HashMap<String, String>>,
    plugin_translations: RwLock<PluginTranslationsMap>,
}

#[derive(Clone, Debug)]
pub struct LocaleCallbackToken(pub usize);

impl I18nService {
    pub fn new() -> Self {
        let mut translations = HashMap::new();

        translations.insert("zh-CN".to_string(), Self::load_zh_cn());
        translations.insert("en-US".to_string(), Self::load_en_us());
        translations.insert("zh-TW".to_string(), Self::load_zh_tw());

        Self {
            translations: RwLock::new(translations),
            locale: RwLock::new("zh-CN".to_string()),
            change_callbacks: RwLock::new(HashMap::new()),
            next_callback_id: RwLock::new(1),
            plugin_locale_owners: RwLock::new(HashMap::new()),
            plugin_locale_names: RwLock::new(HashMap::new()),
            plugin_translations: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_locale(&self) -> String {
        self.locale.read().unwrap().clone()
    }

    pub fn set_locale(&self, locale: &str) {
        let old_locale = self.locale.read().unwrap().clone();
        *self.locale.write().unwrap() = locale.to_string();

        let callbacks = self.change_callbacks.read().unwrap();
        for callback in callbacks.values() {
            callback(&old_locale, locale);
        }
    }

    pub fn on_locale_change<F>(&self, callback: F) -> LocaleCallbackToken
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        let id = {
            let mut next_id = self.next_callback_id.write().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        self.change_callbacks
            .write()
            .unwrap()
            .insert(id, Box::new(callback));

        LocaleCallbackToken(id)
    }

    pub fn remove_locale_callback(&self, token: &LocaleCallbackToken) {
        self.change_callbacks.write().unwrap().remove(&token.0);
    }

    pub fn t(&self, key: &str) -> String {
        let locale = self.get_locale();

        if let Some(locale_translations) = self.translations.read().unwrap().get(&locale) {
            if let Some(value) = locale_translations.get(key) {
                return value.clone();
            }
        }

        {
            let plugin_trans = self.plugin_translations.read().unwrap();
            for plugin_map in plugin_trans.values() {
                if let Some(locale_map) = plugin_map.get(&locale) {
                    if let Some(value) = locale_map.get(key) {
                        return value.clone();
                    }
                }
            }
        }

        if locale != "zh-CN" {
            if let Some(zh_cn) = self.translations.read().unwrap().get("zh-CN") {
                if let Some(value) = zh_cn.get(key) {
                    return value.clone();
                }
            }

            let plugin_trans = self.plugin_translations.read().unwrap();
            for plugin_map in plugin_trans.values() {
                if let Some(locale_map) = plugin_map.get("zh-CN") {
                    if let Some(value) = locale_map.get(key) {
                        return value.clone();
                    }
                }
            }
        }

        key.to_string()
    }

    pub fn t_with_options(&self, key: &str, options: &HashMap<String, String>) -> String {
        let mut result = self.t(key);
        for (k, v) in options {
            result = result.replace(&format!("{{{}}}", k), v);
        }
        result
    }

    pub fn get_all_translations(&self) -> HashMap<String, String> {
        let locale = self.get_locale();
        let mut merged = self
            .translations
            .read()
            .unwrap()
            .get(&locale)
            .cloned()
            .unwrap_or_default();

        let plugin_trans = self.plugin_translations.read().unwrap();
        for plugin_map in plugin_trans.values() {
            if let Some(locale_map) = plugin_map.get(&locale) {
                for (k, v) in locale_map {
                    merged.entry(k.clone()).or_insert_with(|| v.clone());
                }
            }
        }

        merged
    }

    pub fn get_available_locales(&self) -> Vec<String> {
        let mut locales: Vec<String> = SUPPORTED_LOCALES.iter().map(|s| s.to_string()).collect();
        let owners = self.plugin_locale_owners.read().unwrap();
        for locale in owners.keys() {
            if !locales.contains(locale) {
                locales.push(locale.clone());
            }
        }
        locales
    }

    #[allow(dead_code)]
    pub fn get_locale_display_name(&self, locale: &str) -> Option<String> {
        self.plugin_locale_names
            .read()
            .unwrap()
            .get(locale)
            .cloned()
    }

    pub fn register_locale(&self, plugin_id: &str, locale: &str, display_name: &str) {
        self.plugin_locale_owners
            .write()
            .unwrap()
            .insert(locale.to_string(), plugin_id.to_string());
        self.plugin_locale_names
            .write()
            .unwrap()
            .insert(locale.to_string(), display_name.to_string());
    }

    pub fn add_plugin_translations(
        &self,
        plugin_id: &str,
        locale: &str,
        entries: HashMap<String, String>,
    ) {
        let mut plugin_trans = self.plugin_translations.write().unwrap();
        let plugin_map = plugin_trans.entry(plugin_id.to_string()).or_default();
        let locale_map = plugin_map.entry(locale.to_string()).or_default();
        locale_map.extend(entries);
    }

    pub fn remove_plugin_translations(&self, plugin_id: &str) {
        self.plugin_translations.write().unwrap().remove(plugin_id);

        let locales_to_remove: Vec<String> = self
            .plugin_locale_owners
            .read()
            .unwrap()
            .iter()
            .filter(|(_, owner)| owner.as_str() == plugin_id)
            .map(|(locale, _)| locale.clone())
            .collect();

        {
            let mut owners = self.plugin_locale_owners.write().unwrap();
            let mut names = self.plugin_locale_names.write().unwrap();
            for locale in &locales_to_remove {
                owners.remove(locale);
                names.remove(locale);
            }
        }
    }

    fn load_zh_cn() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("app.title".to_string(), "SeaLantern".to_string());
        map.insert("app.subtitle".to_string(), "Minecraft 服务器管理器".to_string());
        map
    }

    fn load_en_us() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("app.title".to_string(), "SeaLantern".to_string());
        map.insert("app.subtitle".to_string(), "Minecraft Server Manager".to_string());
        map
    }

    fn load_zh_tw() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("app.title".to_string(), "SeaLantern".to_string());
        map.insert("app.subtitle".to_string(), "Minecraft 伺服器管理器".to_string());
        map
    }
}

impl Default for I18nService {
    fn default() -> Self {
        Self::new()
    }
}
