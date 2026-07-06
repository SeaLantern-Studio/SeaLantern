use crate::services::global;
use crate::services::server::extensions::{
    read_server_extensions_summary, ServerExtensionsSummary,
};

#[tauri::command]
pub fn get_server_extensions_summary(
    server_path: String,
) -> Result<ServerExtensionsSummary, String> {
    let manager = global::server_manager();
    let server = manager
        .lock_servers()?
        .iter()
        .find(|item| item.path == server_path)
        .cloned();

    read_server_extensions_summary(&server_path, server.as_ref())
}
