use std::collections::HashMap;
use std::sync::RwLock;

pub const SUPPORTED_LOCALES: &[&str] = &["zh-CN", "en-US"];

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

        map.insert("console.server_not_found".to_string(), "服务器不存在: {0}".to_string());
        map.insert("console.server_not_running".to_string(), "服务器未运行".to_string());
        map.insert("console.command_forbidden".to_string(), "命令 '{0}' 被禁止执行".to_string());
        map.insert(
            "console.command_not_allowed".to_string(),
            "命令 '{0}' 不在允许列表中。允许的命令: {1}".to_string(),
        );
        map.insert(
            "console.command_has_forbidden_chars".to_string(),
            "命令包含非法字符".to_string(),
        );
        map.insert("console.empty_command".to_string(), "空命令".to_string());
        map.insert("console.send_command_failed".to_string(), "发送命令失败: {0}".to_string());
        map.insert("console.create_table_failed".to_string(), "创建控制台表失败: {0}".to_string());
        map.insert(
            "console.create_send_failed".to_string(),
            "创建 console.send 失败: {0}".to_string(),
        );
        map.insert(
            "console.set_send_failed".to_string(),
            "设置 console.send 失败: {0}".to_string(),
        );
        map.insert(
            "console.create_get_logs_failed".to_string(),
            "创建 console.get_logs 失败: {0}".to_string(),
        );
        map.insert(
            "console.set_get_logs_failed".to_string(),
            "设置 console.get_logs 失败: {0}".to_string(),
        );
        map.insert(
            "console.create_get_status_failed".to_string(),
            "创建 console.get_status 失败: {0}".to_string(),
        );
        map.insert(
            "console.set_get_status_failed".to_string(),
            "设置 console.get_status 失败: {0}".to_string(),
        );
        map.insert(
            "console.set_console_failed".to_string(),
            "设置 sl.console 失败: {0}".to_string(),
        );

        map.insert("element.create_table_failed".to_string(), "创建元素表失败: {0}".to_string());
        map.insert(
            "element.create_get_text_failed".to_string(),
            "创建 element.get_text 失败: {0}".to_string(),
        );
        map.insert(
            "element.set_get_text_failed".to_string(),
            "设置 element.get_text 失败: {0}".to_string(),
        );
        map.insert(
            "element.create_get_value_failed".to_string(),
            "创建 element.get_value 失败: {0}".to_string(),
        );
        map.insert(
            "element.set_get_value_failed".to_string(),
            "设置 element.get_value 失败: {0}".to_string(),
        );
        map.insert(
            "element.create_get_attribute_failed".to_string(),
            "创建 element.get_attribute 失败: {0}".to_string(),
        );
        map.insert(
            "element.set_get_attribute_failed".to_string(),
            "设置 element.get_attribute 失败: {0}".to_string(),
        );
        map.insert(
            "element.create_get_attributes_failed".to_string(),
            "创建 element.get_attributes 失败: {0}".to_string(),
        );
        map.insert(
            "element.set_get_attributes_failed".to_string(),
            "设置 element.get_attributes 失败: {0}".to_string(),
        );
        map.insert(
            "element.create_click_failed".to_string(),
            "创建 element.click 失败: {0}".to_string(),
        );
        map.insert(
            "element.set_click_failed".to_string(),
            "设置 element.click 失败: {0}".to_string(),
        );
        map.insert("element.click_error".to_string(), "点击元素失败: {0}".to_string());
        map.insert(
            "element.create_set_value_failed".to_string(),
            "创建 element.set_value 失败: {0}".to_string(),
        );
        map.insert(
            "element.set_set_value_failed".to_string(),
            "设置 element.set_value 失败: {0}".to_string(),
        );
        map.insert("element.set_value_error".to_string(), "设置元素值失败: {0}".to_string());
        map.insert(
            "element.create_check_failed".to_string(),
            "创建 element.check 失败: {0}".to_string(),
        );
        map.insert(
            "element.set_check_failed".to_string(),
            "设置 element.check 失败: {0}".to_string(),
        );
        map.insert("element.check_error".to_string(), "勾选元素失败: {0}".to_string());
        map.insert(
            "element.create_select_failed".to_string(),
            "创建 element.select 失败: {0}".to_string(),
        );
        map.insert(
            "element.set_select_failed".to_string(),
            "设置 element.select 失败: {0}".to_string(),
        );
        map.insert("element.select_error".to_string(), "选择元素失败: {0}".to_string());
        map.insert(
            "element.create_focus_failed".to_string(),
            "创建 element.focus 失败: {0}".to_string(),
        );
        map.insert(
            "element.set_focus_failed".to_string(),
            "设置 element.focus 失败: {0}".to_string(),
        );
        map.insert("element.focus_error".to_string(), "聚焦元素失败: {0}".to_string());
        map.insert(
            "element.create_blur_failed".to_string(),
            "创建 element.blur 失败: {0}".to_string(),
        );
        map.insert(
            "element.set_blur_failed".to_string(),
            "设置 element.blur 失败: {0}".to_string(),
        );
        map.insert("element.blur_error".to_string(), "失焦元素失败: {0}".to_string());
        map.insert(
            "element.create_on_change_failed".to_string(),
            "创建 element.on_change 失败: {0}".to_string(),
        );
        map.insert(
            "element.set_on_change_failed".to_string(),
            "设置 element.on_change 失败: {0}".to_string(),
        );
        map.insert("element.on_change_error".to_string(), "设置元素变更监听失败: {0}".to_string());
        map.insert("element.store_callback_failed".to_string(), "存储回调失败: {0}".to_string());
        map.insert("element.cleanup_callback_failed".to_string(), "清理回调失败: {0}".to_string());
        map.insert(
            "element.set_element_failed".to_string(),
            "设置 sl.element 失败: {0}".to_string(),
        );

        map.insert(
            "server.permission_denied".to_string(),
            "权限被拒绝: 需要 'server' 权限".to_string(),
        );
        map.insert("server.server_not_found".to_string(), "服务器不存在: {0}".to_string());
        map.insert(
            "server.failed_to_get_metadata".to_string(),
            "无法获取文件信息: {0}".to_string(),
        );
        map.insert("server.file_too_large".to_string(), "文件过大 (最大 10MB)".to_string());
        map.insert("server.failed_to_read_file".to_string(), "读取文件失败: {0}".to_string());
        map.insert("server.failed_to_create_dir".to_string(), "创建目录失败: {0}".to_string());
        map.insert("server.failed_to_write_file".to_string(), "写入文件失败: {0}".to_string());
        map.insert("server.path_not_directory".to_string(), "路径不是目录".to_string());
        map.insert("server.failed_to_read_dir".to_string(), "读取目录失败: {0}".to_string());
        map.insert("server.failed_to_read_entry".to_string(), "读取目录项失败: {0}".to_string());
        map.insert("server.create_table_failed".to_string(), "创建服务器表失败: {0}".to_string());
        map.insert(
            "server.create_list_failed".to_string(),
            "创建 server.list 失败: {0}".to_string(),
        );
        map.insert("server.set_list_failed".to_string(), "设置 server.list 失败: {0}".to_string());
        map.insert(
            "server.create_get_path_failed".to_string(),
            "创建 server.get_path 失败: {0}".to_string(),
        );
        map.insert(
            "server.set_get_path_failed".to_string(),
            "设置 server.get_path 失败: {0}".to_string(),
        );
        map.insert(
            "server.create_read_file_failed".to_string(),
            "创建 server.read_file 失败: {0}".to_string(),
        );
        map.insert(
            "server.set_read_file_failed".to_string(),
            "设置 server.read_file 失败: {0}".to_string(),
        );
        map.insert(
            "server.create_write_file_failed".to_string(),
            "创建 server.write_file 失败: {0}".to_string(),
        );
        map.insert(
            "server.set_write_file_failed".to_string(),
            "设置 server.write_file 失败: {0}".to_string(),
        );
        map.insert(
            "server.create_list_dir_failed".to_string(),
            "创建 server.list_dir 失败: {0}".to_string(),
        );
        map.insert(
            "server.set_list_dir_failed".to_string(),
            "设置 server.list_dir 失败: {0}".to_string(),
        );
        map.insert(
            "server.create_exists_failed".to_string(),
            "创建 server.exists 失败: {0}".to_string(),
        );
        map.insert(
            "server.set_exists_failed".to_string(),
            "设置 server.exists 失败: {0}".to_string(),
        );
        map.insert(
            "server.create_logs_table_failed".to_string(),
            "创建 server.logs 表失败: {0}".to_string(),
        );
        map.insert(
            "server.create_logs_get_failed".to_string(),
            "创建 server.logs.get 失败: {0}".to_string(),
        );
        map.insert(
            "server.set_logs_get_failed".to_string(),
            "设置 server.logs.get 失败: {0}".to_string(),
        );
        map.insert(
            "server.create_logs_getall_failed".to_string(),
            "创建 server.logs.getAll 失败: {0}".to_string(),
        );
        map.insert(
            "server.set_logs_getall_failed".to_string(),
            "设置 server.logs.getAll 失败: {0}".to_string(),
        );
        map.insert("server.set_logs_failed".to_string(), "设置 server.logs 失败: {0}".to_string());
        map.insert("server.set_server_failed".to_string(), "设置 sl.server 失败: {0}".to_string());

        map.insert("log.create_table_failed".to_string(), "创建日志表失败: {0}".to_string());
        map.insert("log.create_debug_failed".to_string(), "创建 log.debug 失败: {0}".to_string());
        map.insert("log.set_debug_failed".to_string(), "设置 log.debug 失败: {0}".to_string());
        map.insert(
            "log.create_debug_noop_failed".to_string(),
            "创建 log.debug (noop) 失败: {0}".to_string(),
        );
        map.insert(
            "log.set_debug_noop_failed".to_string(),
            "设置 log.debug (noop) 失败: {0}".to_string(),
        );
        map.insert("log.create_info_failed".to_string(), "创建 log.info 失败: {0}".to_string());
        map.insert("log.set_info_failed".to_string(), "设置 log.info 失败: {0}".to_string());
        map.insert("log.create_warn_failed".to_string(), "创建 log.warn 失败: {0}".to_string());
        map.insert("log.set_warn_failed".to_string(), "设置 log.warn 失败: {0}".to_string());
        map.insert("log.create_error_failed".to_string(), "创建 log.error 失败: {0}".to_string());
        map.insert("log.set_error_failed".to_string(), "设置 log.error 失败: {0}".to_string());
        map.insert("log.set_log_failed".to_string(), "设置 sl.log 失败: {0}".to_string());

        map.insert("storage.create_table_failed".to_string(), "创建存储表失败: {0}".to_string());
        map.insert(
            "storage.create_get_failed".to_string(),
            "创建 storage.get 失败: {0}".to_string(),
        );
        map.insert("storage.set_get_failed".to_string(), "设置 storage.get 失败: {0}".to_string());
        map.insert(
            "storage.create_set_failed".to_string(),
            "创建 storage.set 失败: {0}".to_string(),
        );
        map.insert("storage.set_set_failed".to_string(), "设置 storage.set 失败: {0}".to_string());
        map.insert("storage.key_too_long".to_string(), "存储键超过256字节限制".to_string());
        map.insert("storage.value_too_large".to_string(), "存储值超过1MB限制".to_string());
        map.insert("storage.total_too_large".to_string(), "存储总大小超过10MB限制".to_string());
        map.insert(
            "storage.create_remove_failed".to_string(),
            "创建 storage.remove 失败: {0}".to_string(),
        );
        map.insert(
            "storage.set_remove_failed".to_string(),
            "设置 storage.remove 失败: {0}".to_string(),
        );
        map.insert(
            "storage.create_keys_failed".to_string(),
            "创建 storage.keys 失败: {0}".to_string(),
        );
        map.insert(
            "storage.set_keys_failed".to_string(),
            "设置 storage.keys 失败: {0}".to_string(),
        );
        map.insert(
            "storage.set_storage_failed".to_string(),
            "设置 sl.storage 失败: {0}".to_string(),
        );

        map.insert("system.create_table_failed".to_string(), "创建系统表失败: {0}".to_string());
        map.insert(
            "system.create_get_os_failed".to_string(),
            "创建 system.get_os 失败: {0}".to_string(),
        );
        map.insert(
            "system.set_get_os_failed".to_string(),
            "设置 system.get_os 失败: {0}".to_string(),
        );
        map.insert(
            "system.create_get_arch_failed".to_string(),
            "创建 system.get_arch 失败: {0}".to_string(),
        );
        map.insert(
            "system.set_get_arch_failed".to_string(),
            "设置 system.get_arch 失败: {0}".to_string(),
        );
        map.insert(
            "system.create_get_app_version_failed".to_string(),
            "创建 system.get_app_version 失败: {0}".to_string(),
        );
        map.insert(
            "system.set_get_app_version_failed".to_string(),
            "设置 system.get_app_version 失败: {0}".to_string(),
        );
        map.insert(
            "system.create_get_memory_failed".to_string(),
            "创建 system.get_memory 失败: {0}".to_string(),
        );
        map.insert(
            "system.set_get_memory_failed".to_string(),
            "设置 system.get_memory 失败: {0}".to_string(),
        );
        map.insert(
            "system.create_get_cpu_failed".to_string(),
            "创建 system.get_cpu 失败: {0}".to_string(),
        );
        map.insert(
            "system.set_get_cpu_failed".to_string(),
            "设置 system.get_cpu 失败: {0}".to_string(),
        );
        map.insert("system.set_system_failed".to_string(), "设置 sl.system 失败: {0}".to_string());

        map
    }

    fn load_en_us() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("app.title".to_string(), "SeaLantern".to_string());
        map.insert("app.subtitle".to_string(), "Minecraft Server Manager".to_string());

        map.insert("console.server_not_found".to_string(), "Server not found: {0}".to_string());
        map.insert("console.server_not_running".to_string(), "Server is not running".to_string());
        map.insert(
            "console.command_forbidden".to_string(),
            "Command '{0}' is forbidden".to_string(),
        );
        map.insert(
            "console.command_not_allowed".to_string(),
            "Command '{0}' is not in the allowed list. Allowed commands: {1}".to_string(),
        );
        map.insert(
            "console.command_has_forbidden_chars".to_string(),
            "Command contains forbidden characters".to_string(),
        );
        map.insert("console.empty_command".to_string(), "Empty command".to_string());
        map.insert(
            "console.send_command_failed".to_string(),
            "Failed to send command: {0}".to_string(),
        );
        map.insert(
            "console.create_table_failed".to_string(),
            "Failed to create console table: {0}".to_string(),
        );
        map.insert(
            "console.create_send_failed".to_string(),
            "Failed to create console.send: {0}".to_string(),
        );
        map.insert(
            "console.set_send_failed".to_string(),
            "Failed to set console.send: {0}".to_string(),
        );
        map.insert(
            "console.create_get_logs_failed".to_string(),
            "Failed to create console.get_logs: {0}".to_string(),
        );
        map.insert(
            "console.set_get_logs_failed".to_string(),
            "Failed to set console.get_logs: {0}".to_string(),
        );
        map.insert(
            "console.create_get_status_failed".to_string(),
            "Failed to create console.get_status: {0}".to_string(),
        );
        map.insert(
            "console.set_get_status_failed".to_string(),
            "Failed to set console.get_status: {0}".to_string(),
        );
        map.insert(
            "console.set_console_failed".to_string(),
            "Failed to set sl.console: {0}".to_string(),
        );

        map.insert(
            "element.create_table_failed".to_string(),
            "Failed to create element table: {0}".to_string(),
        );
        map.insert(
            "element.create_get_text_failed".to_string(),
            "Failed to create element.get_text: {0}".to_string(),
        );
        map.insert(
            "element.set_get_text_failed".to_string(),
            "Failed to set element.get_text: {0}".to_string(),
        );
        map.insert(
            "element.create_get_value_failed".to_string(),
            "Failed to create element.get_value: {0}".to_string(),
        );
        map.insert(
            "element.set_get_value_failed".to_string(),
            "Failed to set element.get_value: {0}".to_string(),
        );
        map.insert(
            "element.create_get_attribute_failed".to_string(),
            "Failed to create element.get_attribute: {0}".to_string(),
        );
        map.insert(
            "element.set_get_attribute_failed".to_string(),
            "Failed to set element.get_attribute: {0}".to_string(),
        );
        map.insert(
            "element.create_get_attributes_failed".to_string(),
            "Failed to create element.get_attributes: {0}".to_string(),
        );
        map.insert(
            "element.set_get_attributes_failed".to_string(),
            "Failed to set element.get_attributes: {0}".to_string(),
        );
        map.insert(
            "element.create_click_failed".to_string(),
            "Failed to create element.click: {0}".to_string(),
        );
        map.insert(
            "element.set_click_failed".to_string(),
            "Failed to set element.click: {0}".to_string(),
        );
        map.insert("element.click_error".to_string(), "Failed to click element: {0}".to_string());
        map.insert(
            "element.create_set_value_failed".to_string(),
            "Failed to create element.set_value: {0}".to_string(),
        );
        map.insert(
            "element.set_set_value_failed".to_string(),
            "Failed to set element.set_value: {0}".to_string(),
        );
        map.insert(
            "element.set_value_error".to_string(),
            "Failed to set element value: {0}".to_string(),
        );
        map.insert(
            "element.create_check_failed".to_string(),
            "Failed to create element.check: {0}".to_string(),
        );
        map.insert(
            "element.set_check_failed".to_string(),
            "Failed to set element.check: {0}".to_string(),
        );
        map.insert("element.check_error".to_string(), "Failed to check element: {0}".to_string());
        map.insert(
            "element.create_select_failed".to_string(),
            "Failed to create element.select: {0}".to_string(),
        );
        map.insert(
            "element.set_select_failed".to_string(),
            "Failed to set element.select: {0}".to_string(),
        );
        map.insert("element.select_error".to_string(), "Failed to select element: {0}".to_string());
        map.insert(
            "element.create_focus_failed".to_string(),
            "Failed to create element.focus: {0}".to_string(),
        );
        map.insert(
            "element.set_focus_failed".to_string(),
            "Failed to set element.focus: {0}".to_string(),
        );
        map.insert("element.focus_error".to_string(), "Failed to focus element: {0}".to_string());
        map.insert(
            "element.create_blur_failed".to_string(),
            "Failed to create element.blur: {0}".to_string(),
        );
        map.insert(
            "element.set_blur_failed".to_string(),
            "Failed to set element.blur: {0}".to_string(),
        );
        map.insert("element.blur_error".to_string(), "Failed to blur element: {0}".to_string());
        map.insert(
            "element.create_on_change_failed".to_string(),
            "Failed to create element.on_change: {0}".to_string(),
        );
        map.insert(
            "element.set_on_change_failed".to_string(),
            "Failed to set element.on_change: {0}".to_string(),
        );
        map.insert(
            "element.on_change_error".to_string(),
            "Failed to set element change listener: {0}".to_string(),
        );
        map.insert(
            "element.store_callback_failed".to_string(),
            "Failed to store callback: {0}".to_string(),
        );
        map.insert(
            "element.cleanup_callback_failed".to_string(),
            "Failed to cleanup callback: {0}".to_string(),
        );
        map.insert(
            "element.set_element_failed".to_string(),
            "Failed to set sl.element: {0}".to_string(),
        );

        map.insert(
            "server.permission_denied".to_string(),
            "Permission denied: 'server' permission required".to_string(),
        );
        map.insert("server.server_not_found".to_string(), "Server not found: {0}".to_string());
        map.insert(
            "server.failed_to_get_metadata".to_string(),
            "Failed to get file metadata: {0}".to_string(),
        );
        map.insert("server.file_too_large".to_string(), "File too large (max 10MB)".to_string());
        map.insert(
            "server.failed_to_read_file".to_string(),
            "Failed to read file: {0}".to_string(),
        );
        map.insert(
            "server.failed_to_create_dir".to_string(),
            "Failed to create directory: {0}".to_string(),
        );
        map.insert(
            "server.failed_to_write_file".to_string(),
            "Failed to write file: {0}".to_string(),
        );
        map.insert("server.path_not_directory".to_string(), "Path is not a directory".to_string());
        map.insert(
            "server.failed_to_read_dir".to_string(),
            "Failed to read directory: {0}".to_string(),
        );
        map.insert(
            "server.failed_to_read_entry".to_string(),
            "Failed to read directory entry: {0}".to_string(),
        );
        map.insert(
            "server.create_table_failed".to_string(),
            "Failed to create server table: {0}".to_string(),
        );
        map.insert(
            "server.create_list_failed".to_string(),
            "Failed to create server.list: {0}".to_string(),
        );
        map.insert(
            "server.set_list_failed".to_string(),
            "Failed to set server.list: {0}".to_string(),
        );
        map.insert(
            "server.create_get_path_failed".to_string(),
            "Failed to create server.get_path: {0}".to_string(),
        );
        map.insert(
            "server.set_get_path_failed".to_string(),
            "Failed to set server.get_path: {0}".to_string(),
        );
        map.insert(
            "server.create_read_file_failed".to_string(),
            "Failed to create server.read_file: {0}".to_string(),
        );
        map.insert(
            "server.set_read_file_failed".to_string(),
            "Failed to set server.read_file: {0}".to_string(),
        );
        map.insert(
            "server.create_write_file_failed".to_string(),
            "Failed to create server.write_file: {0}".to_string(),
        );
        map.insert(
            "server.set_write_file_failed".to_string(),
            "Failed to set server.write_file: {0}".to_string(),
        );
        map.insert(
            "server.create_list_dir_failed".to_string(),
            "Failed to create server.list_dir: {0}".to_string(),
        );
        map.insert(
            "server.set_list_dir_failed".to_string(),
            "Failed to set server.list_dir: {0}".to_string(),
        );
        map.insert(
            "server.create_exists_failed".to_string(),
            "Failed to create server.exists: {0}".to_string(),
        );
        map.insert(
            "server.set_exists_failed".to_string(),
            "Failed to set server.exists: {0}".to_string(),
        );
        map.insert(
            "server.create_logs_table_failed".to_string(),
            "Failed to create server.logs table: {0}".to_string(),
        );
        map.insert(
            "server.create_logs_get_failed".to_string(),
            "Failed to create server.logs.get: {0}".to_string(),
        );
        map.insert(
            "server.set_logs_get_failed".to_string(),
            "Failed to set server.logs.get: {0}".to_string(),
        );
        map.insert(
            "server.create_logs_getall_failed".to_string(),
            "Failed to create server.logs.getAll: {0}".to_string(),
        );
        map.insert(
            "server.set_logs_getall_failed".to_string(),
            "Failed to set server.logs.getAll: {0}".to_string(),
        );
        map.insert(
            "server.set_logs_failed".to_string(),
            "Failed to set server.logs: {0}".to_string(),
        );
        map.insert(
            "server.set_server_failed".to_string(),
            "Failed to set sl.server: {0}".to_string(),
        );

        map.insert(
            "log.create_table_failed".to_string(),
            "Failed to create log table: {0}".to_string(),
        );
        map.insert(
            "log.create_debug_failed".to_string(),
            "Failed to create log.debug: {0}".to_string(),
        );
        map.insert("log.set_debug_failed".to_string(), "Failed to set log.debug: {0}".to_string());
        map.insert(
            "log.create_debug_noop_failed".to_string(),
            "Failed to create log.debug (noop): {0}".to_string(),
        );
        map.insert(
            "log.set_debug_noop_failed".to_string(),
            "Failed to set log.debug (noop): {0}".to_string(),
        );
        map.insert(
            "log.create_info_failed".to_string(),
            "Failed to create log.info: {0}".to_string(),
        );
        map.insert("log.set_info_failed".to_string(), "Failed to set log.info: {0}".to_string());
        map.insert(
            "log.create_warn_failed".to_string(),
            "Failed to create log.warn: {0}".to_string(),
        );
        map.insert("log.set_warn_failed".to_string(), "Failed to set log.warn: {0}".to_string());
        map.insert(
            "log.create_error_failed".to_string(),
            "Failed to create log.error: {0}".to_string(),
        );
        map.insert("log.set_error_failed".to_string(), "Failed to set log.error: {0}".to_string());
        map.insert("log.set_log_failed".to_string(), "Failed to set sl.log: {0}".to_string());

        map.insert(
            "storage.create_table_failed".to_string(),
            "Failed to create storage table: {0}".to_string(),
        );
        map.insert(
            "storage.create_get_failed".to_string(),
            "Failed to create storage.get: {0}".to_string(),
        );
        map.insert(
            "storage.set_get_failed".to_string(),
            "Failed to set storage.get: {0}".to_string(),
        );
        map.insert(
            "storage.create_set_failed".to_string(),
            "Failed to create storage.set: {0}".to_string(),
        );
        map.insert(
            "storage.set_set_failed".to_string(),
            "Failed to set storage.set: {0}".to_string(),
        );
        map.insert(
            "storage.key_too_long".to_string(),
            "Storage key exceeds 256 bytes limit".to_string(),
        );
        map.insert(
            "storage.value_too_large".to_string(),
            "Storage value exceeds 1MB limit".to_string(),
        );
        map.insert(
            "storage.total_too_large".to_string(),
            "Storage total size exceeds 10MB limit".to_string(),
        );
        map.insert(
            "storage.create_remove_failed".to_string(),
            "Failed to create storage.remove: {0}".to_string(),
        );
        map.insert(
            "storage.set_remove_failed".to_string(),
            "Failed to set storage.remove: {0}".to_string(),
        );
        map.insert(
            "storage.create_keys_failed".to_string(),
            "Failed to create storage.keys: {0}".to_string(),
        );
        map.insert(
            "storage.set_keys_failed".to_string(),
            "Failed to set storage.keys: {0}".to_string(),
        );
        map.insert(
            "storage.set_storage_failed".to_string(),
            "Failed to set sl.storage: {0}".to_string(),
        );

        map.insert(
            "system.create_table_failed".to_string(),
            "Failed to create system table: {0}".to_string(),
        );
        map.insert(
            "system.create_get_os_failed".to_string(),
            "Failed to create system.get_os: {0}".to_string(),
        );
        map.insert(
            "system.set_get_os_failed".to_string(),
            "Failed to set system.get_os: {0}".to_string(),
        );
        map.insert(
            "system.create_get_arch_failed".to_string(),
            "Failed to create system.get_arch: {0}".to_string(),
        );
        map.insert(
            "system.set_get_arch_failed".to_string(),
            "Failed to set system.get_arch: {0}".to_string(),
        );
        map.insert(
            "system.create_get_app_version_failed".to_string(),
            "Failed to create system.get_app_version: {0}".to_string(),
        );
        map.insert(
            "system.set_get_app_version_failed".to_string(),
            "Failed to set system.get_app_version: {0}".to_string(),
        );
        map.insert(
            "system.create_get_memory_failed".to_string(),
            "Failed to create system.get_memory: {0}".to_string(),
        );
        map.insert(
            "system.set_get_memory_failed".to_string(),
            "Failed to set system.get_memory: {0}".to_string(),
        );
        map.insert(
            "system.create_get_cpu_failed".to_string(),
            "Failed to create system.get_cpu: {0}".to_string(),
        );
        map.insert(
            "system.set_get_cpu_failed".to_string(),
            "Failed to set system.get_cpu: {0}".to_string(),
        );
        map.insert(
            "system.set_system_failed".to_string(),
            "Failed to set sl.system: {0}".to_string(),
        );

        map
    }
}

impl Default for I18nService {
    fn default() -> Self {
        Self::new()
    }
}
