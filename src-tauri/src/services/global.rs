use super::server_manager::ServerManager;
use super::settings_manager::SettingsManager;
use super::mod_manager::ModManager;
use super::join_manager::JoinManager;
use super::server_id_manager::ServerIdManager;
use std::sync::OnceLock;

pub fn server_manager() -> &'static ServerManager {
    static INSTANCE: OnceLock<ServerManager> = OnceLock::new();
    INSTANCE.get_or_init(ServerManager::new)
}

pub fn settings_manager() -> &'static SettingsManager {
    static INSTANCE: OnceLock<SettingsManager> = OnceLock::new();
    INSTANCE.get_or_init(SettingsManager::new)
}

pub fn mod_manager() -> &'static ModManager {
    static INSTANCE: OnceLock<ModManager> = OnceLock::new();
    INSTANCE.get_or_init(ModManager::new)
}

pub fn join_manager() -> &'static JoinManager {
    static INSTANCE: OnceLock<JoinManager> = OnceLock::new();
    INSTANCE.get_or_init(JoinManager::new)
}

pub fn server_id_manager() -> &'static ServerIdManager {
    static INSTANCE: OnceLock<ServerIdManager> = OnceLock::new();
    INSTANCE.get_or_init(ServerIdManager::new)
}
