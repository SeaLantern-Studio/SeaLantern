use crate::services::global;
use crate::services::mod_manager::SearchModsResult;
use std::path::PathBuf;
use tauri::command;

#[command]
pub async fn search_mods(
    query: String,
    game_version: String,
    loader: String,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<SearchModsResult, String> {
    let mod_manager = global::mod_manager();
    mod_manager
        .search_modrinth(
            &query,
            &game_version,
            &loader,
            page.unwrap_or(1),
            page_size.unwrap_or(10),
        )
        .await
}

#[command]
pub async fn install_mod(
    server_id: String,
    download_url: String,
    file_name: String,
) -> Result<(), String> {
    let server_path = {
        let server_manager = global::server_manager();
        let servers = server_manager.servers.lock().unwrap();
        let server = servers
            .iter()
            .find(|s| s.id == server_id)
            .ok_or("Server not found")?;
        server.path.clone()
    }; // MutexGuard 在这里被释放

    let mods_dir = PathBuf::from(&server_path).join("mods");
    let target_path = mods_dir.join(file_name);

    let mod_manager = global::mod_manager();
    mod_manager.download_mod(&download_url, &target_path).await
}
