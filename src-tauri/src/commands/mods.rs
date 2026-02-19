use crate::services::global;
use crate::services::mod_manager::SearchModsResult;
use std::path::PathBuf;
use tauri::command;

#[command]
pub async fn search_mods(
    query: String,
    game_version: String,
    loader: String,
    project_type: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<SearchModsResult, String> {
    let mod_manager = global::mod_manager();
    mod_manager
        .search_modrinth(
            &query,
            &game_version,
            &loader,
            project_type.as_deref().unwrap_or("mod"),
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
    project_id: Option<String>,
    game_version: Option<String>,
    loader: Option<String>,
) -> Result<(), String> {
    let server_path = {
        let server_manager = global::server_manager();
        let servers = server_manager.servers.lock().unwrap();
        let server = servers
            .iter()
            .find(|s| s.id == server_id)
            .ok_or("Server not found")?;
        server.path.clone()
    };

    let mods_dir = PathBuf::from(&server_path).join("mods");
    let mod_manager = global::mod_manager();

    if let (Some(project_id), Some(game_version), Some(loader)) = (project_id, game_version, loader)
    {
        return mod_manager
            .install_mod_with_dependencies(&project_id, &game_version, &loader, &mods_dir)
            .await;
    }

    let target_path = mods_dir.join(file_name);
    mod_manager.download_mod(&download_url, &target_path).await
}
