use std::collections::HashMap;
use std::sync::RwLock;

pub const SUPPORTED_LOCALES: &[&str] = &["zh-CN", "en-US", "zh-TW"];

pub struct I18nService {
    translations: RwLock<HashMap<String, HashMap<String, String>>>,
    locale: RwLock<String>,
    change_callbacks: RwLock<HashMap<usize, Box<dyn Fn(&str, &str) + Send + Sync>>>,
    next_callback_id: RwLock<usize>,
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
        self.change_callbacks
            .write()
            .unwrap()
            .remove(&token.0);
    }

    pub fn t(&self, key: &str) -> String {
        let locale = self.get_locale();

        if let Some(locale_translations) = self.translations.read().unwrap().get(&locale) {
            if let Some(value) = locale_translations.get(key) {
                return value.clone();
            }
        }

        if locale != "zh-CN" {
            if let Some(zh_cn) = self.translations.read().unwrap().get("zh-CN") {
                if let Some(value) = zh_cn.get(key) {
                    return value.clone();
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
        self.translations
            .read()
            .unwrap()
            .get(&locale)
            .cloned()
            .unwrap_or_default()
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
