//! 全局单例访问入口：提供 server_manager / settings_manager / i18n_service 等静态句柄。
//!
//! 所有函数都基于 OnceLock 懒初始化，在进程生命周期内保持 `&'static` 引用
use super::i18n::I18nService;
use super::mod_manager::ModManager;
use super::server::id_manager::ServerIdManager;
use super::server::join::JoinManager;
use super::server::manager::ServerManager;
use super::server::plugin_manager::ServerPluginManager;
use super::settings_manager::SettingsManager;
use crate::plugins::manager::PluginManager;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn server_manager() -> &'static ServerManager {
    static INSTANCE: OnceLock<ServerManager> = OnceLock::new();
    INSTANCE.get_or_init(ServerManager::new)
}

pub fn settings_manager() -> &'static SettingsManager {
    static INSTANCE: OnceLock<SettingsManager> = OnceLock::new();
    INSTANCE.get_or_init(SettingsManager::new)
}

pub fn i18n_service() -> &'static I18nService {
    static INSTANCE: OnceLock<I18nService> = OnceLock::new();
    INSTANCE.get_or_init(I18nService::new)
}

pub fn mod_manager() -> &'static ModManager {
    static INSTANCE: OnceLock<ModManager> = OnceLock::new();
    INSTANCE.get_or_init(|| match ModManager::new() {
        Ok(manager) => manager,
        Err(error) => {
            eprintln!("Failed to initialize ModManager: {}", error);
            ModManager::fallback()
        }
    })
}

pub fn join_manager() -> &'static JoinManager {
    static INSTANCE: OnceLock<JoinManager> = OnceLock::new();
    INSTANCE.get_or_init(JoinManager::new)
}

pub fn server_id_manager() -> &'static ServerIdManager {
    static INSTANCE: OnceLock<ServerIdManager> = OnceLock::new();
    INSTANCE.get_or_init(ServerIdManager::new)
}

pub fn server_plugin_manager() -> &'static ServerPluginManager {
    static INSTANCE: OnceLock<ServerPluginManager> = OnceLock::new();
    INSTANCE.get_or_init(ServerPluginManager::new)
}

pub fn plugin_manager() -> &'static Arc<Mutex<PluginManager>> {
    static INSTANCE: OnceLock<Arc<Mutex<PluginManager>>> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let app_data_dir = crate::utils::path::get_app_data_dir();
        let plugins_dir = app_data_dir.join("plugins");
        let data_dir = app_data_dir.join("plugin_data");

        let mut plugin_manager = PluginManager::new(plugins_dir, data_dir);
        if let Err(error) = plugin_manager.scan_plugins() {
            eprintln!("Failed to scan plugins: {}", error);
        }

        Arc::new(Mutex::new(plugin_manager))
    })
}

static FRONTEND_LAST_HEARTBEAT: OnceLock<AtomicU64> = OnceLock::new();

fn heartbeat_storage() -> &'static AtomicU64 {
    FRONTEND_LAST_HEARTBEAT.get_or_init(|| AtomicU64::new(0))
}

/// 更新前端心跳时间为当前 Unix 秒时间戳。
pub fn update_frontend_heartbeat() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    heartbeat_storage().store(now, Ordering::Relaxed);
}

/// 获取最近一次前端心跳的 Unix 秒时间戳；0 表示尚未收到心跳。
pub fn last_frontend_heartbeat() -> u64 {
    heartbeat_storage().load(Ordering::Relaxed)
}
