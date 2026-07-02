use super::common::{server_t, server_t1};
use crate::services::global;
use crate::services::server::log_pipeline::map_domain_event;
use crate::services::server::manager::ServerManager;
use crate::services::server::player as player_manager;
use crate::services::server::player::{
    BanEntry, BannedIpEntry, OpEntry, PlayerEntry, ServerPlayerSummary,
};
use serde::Serialize;
use sl_server_info::log::{parse_log_line, LogLineInput, LogStream};
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ParsedPlayerLogEvent {
    pub event_kind: Option<String>,
    pub player: Option<String>,
}

fn validate_player_name(name: &str) -> Result<(), String> {
    if name.len() < 3 || name.len() > 16 {
        return Err(server_t("server.players.name_length_invalid"));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(server_t("server.players.name_chars_invalid"));
    }
    Ok(())
}

fn remove_player_from_whitelist_file(server_path: &Path, name: &str) -> Result<(), String> {
    let whitelist_path = server_path.join("whitelist.json");
    if !whitelist_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&whitelist_path)
        .map_err(|e| server_t1("server.players.whitelist_read_failed", e.to_string()))?;
    let trimmed = content.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Ok(());
    }

    let mut list = serde_json::from_str::<Vec<player_manager::PlayerEntry>>(trimmed)
        .map_err(|e| server_t1("server.players.whitelist_parse_failed", e.to_string()))?;
    list.retain(|player| !player.name.eq_ignore_ascii_case(name));

    let json = serde_json::to_string_pretty(&list)
        .map_err(|e| server_t1("server.players.whitelist_serialize_failed", e.to_string()))?;
    std::fs::write(&whitelist_path, json)
        .map_err(|e| server_t1("server.players.whitelist_write_failed", e.to_string()))
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
pub fn get_banned_ips(server_path: String) -> Result<Vec<BannedIpEntry>, String> {
    player_manager::read_banned_ips(&server_path)
}

#[tauri::command]
pub fn get_ops(server_path: String) -> Result<Vec<OpEntry>, String> {
    player_manager::read_ops(&server_path)
}

#[tauri::command]
pub fn get_server_player_summary(server_path: String) -> Result<ServerPlayerSummary, String> {
    player_manager::read_player_summary(&server_path)
}

#[tauri::command]
/// Parse historical player log lines into structured events.
///
/// Note:
/// - Historical logs are treated as having an `Unknown` `LogStream`.
/// - Live events from the log pipeline carry their actual stream (`Stdout` / `Stderr` / `Unknown`).
/// If `parse_log_line` introduces stream-sensitive behavior in the future,
/// callers must account for potential differences between historical and live parsing.
pub fn parse_player_log_events(lines: Vec<String>) -> Vec<ParsedPlayerLogEvent> {
    lines
        .into_iter()
        .map(|line| {
            // Historical log parsing does not know the original stream (stdout/stderr),
            // so we explicitly mark it as `Unknown` to distinguish it from live events,
            // which carry their actual stream information.
            let parsed =
                parse_log_line(None, LogLineInput { raw: line, stream: LogStream::Unknown });

            let mapped = map_domain_event(parsed.event);

            ParsedPlayerLogEvent {
                event_kind: mapped.event_kind,
                player: mapped.player,
            }
        })
        .collect()
}

// ---- Modify via server console commands ----

#[tauri::command]
pub fn add_to_whitelist(server_id: String, name: String) -> Result<String, String> {
    validate_player_name(&name)?;
    let cmd = format!("whitelist add {}", name);
    let manager = global::server_manager();
    manager.send_command(&server_id, &cmd)?;
    // Force save whitelist to file and reload
    let _ = manager.send_command(&server_id, "whitelist reload");
    Ok(format!("Sent: {}", cmd))
}

#[tauri::command]
pub fn remove_from_whitelist(server_id: String, name: String) -> Result<String, String> {
    remove_from_whitelist_in(global::server_manager(), &server_id, &name)
}

fn remove_from_whitelist_in(
    manager: &ServerManager,
    server_id: &str,
    name: &str,
) -> Result<String, String> {
    validate_player_name(name)?;
    let cmd = format!("whitelist remove {}", name);
    if manager.send_command(server_id, &cmd).is_ok() {
        let _ = manager.send_command(server_id, "whitelist reload");
        return Ok(format!("Removed: {}", name));
    }

    let server_path = manager.find_server_clone(server_id)?.path;

    remove_player_from_whitelist_file(Path::new(&server_path), name)?;

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
    global::server_manager().send_command(&server_id, &cmd)?;
    Ok(format!("Sent: {}", cmd))
}

#[tauri::command]
pub fn unban_player(server_id: String, name: String) -> Result<String, String> {
    validate_player_name(&name)?;
    let cmd = format!("pardon {}", name);
    global::server_manager().send_command(&server_id, &cmd)?;
    Ok(format!("Sent: {}", cmd))
}

#[tauri::command]
pub fn add_op(server_id: String, name: String) -> Result<String, String> {
    validate_player_name(&name)?;
    let cmd = format!("op {}", name);
    global::server_manager().send_command(&server_id, &cmd)?;
    Ok(format!("Sent: {}", cmd))
}

#[tauri::command]
pub fn remove_op(server_id: String, name: String) -> Result<String, String> {
    validate_player_name(&name)?;
    let cmd = format!("deop {}", name);
    global::server_manager().send_command(&server_id, &cmd)?;
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
    global::server_manager().send_command(&server_id, &cmd)?;
    Ok(format!("Sent: {}", cmd))
}

#[tauri::command]
pub fn export_logs(logs: Vec<String>, save_path: String) -> Result<(), String> {
    let save = std::path::Path::new(&save_path);

    let allowed_root =
        dirs_next::home_dir().ok_or_else(|| server_t("server.players.user_home_unavailable"))?;

    let parent = save
        .parent()
        .ok_or_else(|| server_t("server.players.invalid_save_path"))?;
    let canonical_parent = std::fs::canonicalize(parent)
        .map_err(|e| server_t1("server.players.invalid_save_path_with_detail", e.to_string()))?;
    let canonical_root = std::fs::canonicalize(&allowed_root)
        .map_err(|e| server_t1("server.players.user_home_canonicalize_failed", e.to_string()))?;

    if !canonical_parent.starts_with(&canonical_root) {
        return Err(server_t("server.players.save_path_must_be_within_home"));
    }

    let content = logs.join("\n");
    std::fs::write(&save_path, content)
        .map_err(|e| server_t1("server.players.save_failed", e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        parse_player_log_events, remove_from_whitelist_in, remove_player_from_whitelist_file,
    };
    use crate::models::server::{LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig};
    use crate::services::server::manager::ServerManager;
    use std::sync::Arc;

    fn test_server(path: String) -> ServerInstance {
        ServerInstance {
            id: "players-remove-whitelist".to_string(),
            name: "Players Remove Whitelist".to_string(),
            aliases: Vec::new(),
            core_type: "fabric".to_string(),
            core_version: "fabric".to_string(),
            mc_version: "1.20.1".to_string(),
            path,
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "java".to_string(),
                jvm_args: Vec::new(),
                cpu_policy: crate::models::server::CpuPolicyConfig::default(),
                jvm_preset: crate::models::server::JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn remove_player_from_whitelist_file_removes_name_case_insensitively() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let whitelist_path = temp_dir.path().join("whitelist.json");
        std::fs::write(
            &whitelist_path,
            r#"[
  {"uuid":"1","name":"Alice"},
  {"uuid":"2","name":"Bob"}
]"#,
        )
        .expect("whitelist should write");

        remove_player_from_whitelist_file(temp_dir.path(), "alice")
            .expect("whitelist update should succeed");

        let updated = std::fs::read_to_string(&whitelist_path).expect("whitelist should read");
        let entries: Vec<crate::services::server::player::PlayerEntry> =
            serde_json::from_str(&updated).expect("whitelist json should parse");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Bob");
    }

    #[test]
    fn remove_player_from_whitelist_file_surfaces_parse_failures() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let whitelist_path = temp_dir.path().join("whitelist.json");
        std::fs::write(&whitelist_path, "{not json}").expect("invalid whitelist should write");

        let error = remove_player_from_whitelist_file(temp_dir.path(), "alice")
            .expect_err("invalid whitelist json should not be silently downgraded");

        assert!(error.contains("解析白名单文件失败"), "unexpected error: {}", error);
    }

    #[test]
    fn remove_from_whitelist_surfaces_server_list_lock_failures_instead_of_fake_not_found() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let manager = Arc::new(ServerManager::new_checked().expect("manager should initialize"));
        manager
            .lock_servers()
            .expect("servers lock should succeed")
            .push(test_server(temp_dir.path().to_string_lossy().to_string()));

        let poison_manager = Arc::clone(&manager);
        let poison_thread = std::thread::spawn(move || {
            let _guard = poison_manager
                .servers
                .lock()
                .expect("servers lock should be acquired");
            panic!("poison server list lock");
        });
        assert!(poison_thread.join().is_err(), "poison thread should panic");

        let error = remove_from_whitelist_in(&manager, "players-remove-whitelist", "Alice")
            .expect_err("lock failure should not be flattened into a fake server-not-found error");

        assert_eq!(error, "servers lock poisoned");
    }

    #[test]
    fn parse_player_log_events_uses_shared_log_semantics() {
        let events = parse_player_log_events(vec![
            "[12:00:00] [Server thread/INFO]: Starting minecraft server".to_string(),
            "[12:00:05] [Server thread/INFO]: Done (5.123s)! For help, type \"help\"".to_string(),
            "[12:00:06] [Server thread/INFO]: Alex joined the game".to_string(),
            "[12:00:07] [Server thread/INFO]: Alex left the game".to_string(),
        ]);

        assert_eq!(events[0].event_kind, None);
        assert_eq!(events[1].event_kind.as_deref(), Some("server_ready"));
        assert_eq!(events[2].event_kind.as_deref(), Some("player_join"));
        assert_eq!(events[2].player.as_deref(), Some("Alex"));
        assert_eq!(events[3].event_kind.as_deref(), Some("player_leave"));
        assert_eq!(events[3].player.as_deref(), Some("Alex"));
    }
}
