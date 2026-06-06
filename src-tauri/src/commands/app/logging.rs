use crate::services::global;
use crate::utils::logger::{LogEntry, GLOBAL_LOG_COLLECTOR};
use tauri::command;

#[command]
pub fn get_logs(limit: Option<usize>) -> Vec<LogEntry> {
    GLOBAL_LOG_COLLECTOR.get_logs(limit)
}

#[command]
pub fn clear_logs() {
    GLOBAL_LOG_COLLECTOR.clear();
}

#[command]
pub fn check_developer_mode() -> bool {
    global::settings_manager().get().developer_mode
}
