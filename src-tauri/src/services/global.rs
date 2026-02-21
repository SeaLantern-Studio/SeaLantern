use super::i18n::I18nService;
use super::join_manager::JoinManager;
use super::mod_manager::ModManager;
use super::server_id_manager::ServerIdManager;
use super::server_manager::ServerManager;
use super::settings_manager::SettingsManager;
use std::sync::OnceLock;

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

#[allow(dead_code)]
pub fn mod_manager() -> &'static ModManager {
    static INSTANCE: OnceLock<ModManager> = OnceLock::new();
    INSTANCE.get_or_init(|| ModManager::new().expect("Failed to initialize ModManager"))
}

#[allow(dead_code)]
pub fn join_manager() -> &'static JoinManager {
    static INSTANCE: OnceLock<JoinManager> = OnceLock::new();
    INSTANCE.get_or_init(JoinManager::new)
}

#[allow(dead_code)]
pub fn server_id_manager() -> &'static ServerIdManager {
    static INSTANCE: OnceLock<ServerIdManager> = OnceLock::new();
    INSTANCE.get_or_init(ServerIdManager::new)
}
