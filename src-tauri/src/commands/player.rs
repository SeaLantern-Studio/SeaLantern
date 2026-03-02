use crate::services::global;
use crate::services::player_manager;
use crate::services::player_manager::{BanEntry, OpEntry, PlayerEntry};

fn manager() -> &'static crate::services::server_manager::ServerManager {
    global::server_manager()
}

fn validate_player_name(name: &str) -> Result<(), String> {
    if name.len() < 3 || name.len() > 16 {
        return Err("Player name must be 3-16 characters".to_string());
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Player name can only contain letters, numbers and underscores".to_string());
    }
    Ok(())
}

// ---- Read lists from files ----

#[tauri::command]
pub fn get_whitelist(server_path: String) -> Result<Vec<PlayerEntry>, String> {
    player_manager::read_whitelist(&server_path)
}

#[tauri::command]
pub fn get_banned_players(server_path: String) -> Result<Vec<BanEntry>, String> {
    player_manager::read_banned_players(&server_path)
}

#[tauri::command]
pub fn get_ops(server_path: String) -> Result<Vec<OpEntry>, String> {
    player_manager::read_ops(&server_path)
}

// ---- Modify via server console commands ----

#[tauri::command]
pub fn add_to_whitelist(server_id: String, name: String) -> Result<String, String> {
    validate_player_name(&name)?;
    let cmd = format!("whitelist add {}", name);
    manager().send_command(&server_id, &cmd)?;
    // Force save whitelist to file and reload
    let _ = manager().send_command(&server_id, "whitelist reload");
    Ok(format!("Sent: {}", cmd))
}

#[tauri::command]
pub fn remove_from_whitelist(server_id: String, name: String) -> Result<String, String> {
    validate_player_name(&name)?;
    // First, try to remove via command
    let cmd = format!("whitelist remove {}", name);
    let _ = manager().send_command(&server_id, &cmd);

    // Also manually remove from file to ensure it's gone
    let servers = manager().get_server_list();
    if let Some(server) = servers.iter().find(|s| s.id == server_id) {
        let whitelist_path = std::path::Path::new(&server.path).join("whitelist.json");
        if whitelist_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&whitelist_path) {
                if let Ok(mut list) =
                    serde_json::from_str::<Vec<player_manager::PlayerEntry>>(&content)
                {
                    // Remove player from list (case-insensitive comparison)
                    list.retain(|p| !p.name.eq_ignore_ascii_case(&name));
                    // Write back to file
                    if let Ok(json) = serde_json::to_string_pretty(&list) {
                        let _ = std::fs::write(&whitelist_path, json);
                    }
                }
            }
        }
    }

    // Reload whitelist
    let _ = manager().send_command(&server_id, "whitelist reload");

    Ok(format!("Removed: {}", name))
}

#[tauri::command]
pub fn ban_player(server_id: String, name: String, reason: String) -> Result<String, String> {
    validate_player_name(&name)?;
    let cmd = if reason.is_empty() {
        format!("ban {}", name)
    } else {
        format!("ban {} {}", name, reason)
    };
    manager().send_command(&server_id, &cmd)?;
    Ok(format!("Sent: {}", cmd))
}

#[tauri::command]
pub fn unban_player(server_id: String, name: String) -> Result<String, String> {
    validate_player_name(&name)?;
    let cmd = format!("pardon {}", name);
    manager().send_command(&server_id, &cmd)?;
    Ok(format!("Sent: {}", cmd))
}

#[tauri::command]
pub fn add_op(server_id: String, name: String) -> Result<String, String> {
    validate_player_name(&name)?;
    let cmd = format!("op {}", name);
    manager().send_command(&server_id, &cmd)?;
    Ok(format!("Sent: {}", cmd))
}

#[tauri::command]
pub fn remove_op(server_id: String, name: String) -> Result<String, String> {
    validate_player_name(&name)?;
    let cmd = format!("deop {}", name);
    manager().send_command(&server_id, &cmd)?;
    Ok(format!("Sent: {}", cmd))
}

#[tauri::command]
pub fn kick_player(server_id: String, name: String, reason: String) -> Result<String, String> {
    validate_player_name(&name)?;
    let cmd = if reason.is_empty() {
        format!("kick {}", name)
    } else {
        format!("kick {} {}", name, reason)
    };
    manager().send_command(&server_id, &cmd)?;
    Ok(format!("Sent: {}", cmd))
}

#[tauri::command]
pub fn export_logs(logs: Vec<String>, save_path: String) -> Result<(), String> {
    let save = std::path::Path::new(&save_path);

    let allowed_root = dirs_next::home_dir().ok_or_else(|| "无法获取用户目录".to_string())?;

    let parent = save.parent().ok_or_else(|| "无效的保存路径".to_string())?;
    let canonical_parent =
        std::fs::canonicalize(parent).map_err(|e| format!("无效的保存路径: {}", e))?;
    let canonical_root =
        std::fs::canonicalize(&allowed_root).map_err(|e| format!("无法规范化用户目录: {}", e))?;

    if !canonical_parent.starts_with(&canonical_root) {
        return Err("保存路径必须在用户目录内".to_string());
    }

    let content = logs.join("\n");
    std::fs::write(&save_path, content).map_err(|e| format!("保存失败: {}", e))
}
