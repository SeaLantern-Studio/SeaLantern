//! 全局单例访问入口：提供 server_manager / settings_manager / i18n_service 等静态句柄。
//!
//! 所有函数都基于 OnceLock 懒初始化，在进程生命周期内保持 `&'static` 引用
use super::event_consumer_registry::EventConsumerRegistryService;
use super::i18n::I18nService;
use super::mod_manager::ModManager;
use super::events::{EventConsumer, EventConsumerKind, EventConsumerMetadata, EventManager};
use super::server::id_manager::ServerIdManager;
use super::server::join::JoinManager;
use super::server::manager::ServerManager;
use super::settings_manager::SettingsManager;
use crate::plugins::manager::PluginManager;
use crate::utils::logger::{log_error_ctx, log_fatal_ctx, log_warn_ctx};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

fn exit_with_global_init_error(component: &str, error: impl AsRef<str>) -> ! {
    log_fatal_ctx(
        "services.global",
        "exit_with_global_init_error",
        &format!("component={} error={}", component, error.as_ref()),
    );
    std::process::exit(1);
}

fn init_settings_manager_or_exit() -> SettingsManager {
    SettingsManager::new_checked()
        .unwrap_or_else(|error| exit_with_global_init_error("SettingsManager", error))
}

fn init_server_manager_or_exit() -> ServerManager {
    ServerManager::new_checked()
        .unwrap_or_else(|error| exit_with_global_init_error("ServerManager", error))
}

fn init_plugin_manager_or_exit() -> Arc<Mutex<PluginManager>> {
    let app_data_dir = crate::utils::path::get_or_create_app_data_dir_checked()
        .map(PathBuf::from)
        .unwrap_or_else(|error| exit_with_global_init_error("plugin app data directory", error));
    let plugins_dir = app_data_dir.join("plugins");
    let data_dir = app_data_dir.join("plugin_data");

    let mut plugin_manager = PluginManager::new_checked(plugins_dir, data_dir)
        .unwrap_or_else(|error| exit_with_global_init_error("PluginManager", error));
    if let Err(error) = plugin_manager.scan_plugins() {
        log_warn_ctx(
            "services.global",
            "init_plugin_manager_or_exit",
            &format!("plugin scan failed during bootstrap: {}", error),
        );
    }

    let manager = Arc::new(Mutex::new(plugin_manager));

    let manager_for_events = Arc::clone(&manager);
    let plugin_registration = event_manager().register_named_consumer_with_metadata(
        "plugin_manager.server_events",
        EventConsumer::server(Arc::new(move |event| {
            let guard = manager_for_events.lock().unwrap_or_else(|e| e.into_inner());
            guard.notify_server_event(event);
            Ok(())
        })),
        EventConsumerMetadata::new(
            EventConsumerKind::PluginRuntime,
            "plugin_manager",
            "Dispatch backend server events to enabled plugins.",
        ),
    );

    let _ = event_manager().register_named_consumer_with_metadata(
        "online.onebot.server_events",
        EventConsumer::server(Arc::new(move |event| {
            crate::services::online::onebot::handle_server_event(event);
            Ok(())
        })),
        EventConsumerMetadata::new(
            EventConsumerKind::ProtocolAdapter,
            "online.onebot",
            "Forward selected server events to the thin OneBot HTTP adapter.",
        ),
    );

    if let Ok(mut guard) = manager.lock() {
        if let Some(subscriber_id) = plugin_registration.server_subscription_id {
            guard.set_server_event_subscription_id(subscriber_id);
        }
    }

    manager
}

pub fn server_manager() -> &'static ServerManager {
    static INSTANCE: OnceLock<ServerManager> = OnceLock::new();
    INSTANCE.get_or_init(init_server_manager_or_exit)
}

pub fn settings_manager() -> &'static SettingsManager {
    static INSTANCE: OnceLock<SettingsManager> = OnceLock::new();
    INSTANCE.get_or_init(init_settings_manager_or_exit)
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
            log_error_ctx(
                "services.global",
                "mod_manager",
                &format!("failed to initialize ModManager: {}", error),
            );
            ModManager::fallback()
        }
    })
}

pub fn event_manager() -> &'static EventManager {
    static INSTANCE: OnceLock<EventManager> = OnceLock::new();
    INSTANCE.get_or_init(EventManager::new)
}

#[allow(dead_code)]
pub fn event_consumer_registry_service() -> &'static EventConsumerRegistryService {
    static INSTANCE: OnceLock<EventConsumerRegistryService> = OnceLock::new();
    INSTANCE.get_or_init(EventConsumerRegistryService::new)
}

pub fn join_manager() -> &'static JoinManager {
    static INSTANCE: OnceLock<JoinManager> = OnceLock::new();
    INSTANCE.get_or_init(JoinManager::new)
}

pub fn server_id_manager() -> &'static ServerIdManager {
    static INSTANCE: OnceLock<ServerIdManager> = OnceLock::new();
    INSTANCE.get_or_init(ServerIdManager::new)
}

pub fn plugin_manager() -> &'static Arc<Mutex<PluginManager>> {
    static INSTANCE: OnceLock<Arc<Mutex<PluginManager>>> = OnceLock::new();
    INSTANCE.get_or_init(init_plugin_manager_or_exit)
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
