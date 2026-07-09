use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerEntry {
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanEntry {
    pub uuid: String,
    pub name: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub expires: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannedIpEntry {
    pub ip: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub expires: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpEntry {
    pub uuid: String,
    pub name: String,
    #[serde(default)]
    pub level: u32,
    #[serde(default, alias = "bypassesPlayerLimit", alias = "bypass_player_limit")]
    pub bypasses_player_limit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPlayerSummary {
    pub whitelist: Vec<PlayerEntry>,
    pub banned_players: Vec<BanEntry>,
    pub banned_ips: Vec<BannedIpEntry>,
    pub ops: Vec<OpEntry>,
}

pub fn read_whitelist(server_path: &str) -> Result<Vec<PlayerEntry>, String> {
    let files = sl_libscv::state::discover_state_files(server_path);
    sl_libscv::state::read_whitelist(&files.whitelist_path)
        .map(map_whitelist_entries)
        .map_err(render_state_error)
}

pub fn read_banned_players(server_path: &str) -> Result<Vec<BanEntry>, String> {
    let files = sl_libscv::state::discover_state_files(server_path);
    sl_libscv::state::read_banned_players(&files.banned_players_path)
        .map(map_ban_entries)
        .map_err(render_state_error)
}

pub fn read_ops(server_path: &str) -> Result<Vec<OpEntry>, String> {
    let files = sl_libscv::state::discover_state_files(server_path);
    sl_libscv::state::read_ops(&files.ops_path)
        .map(map_op_entries)
        .map_err(render_state_error)
}

pub fn read_banned_ips(server_path: &str) -> Result<Vec<BannedIpEntry>, String> {
    let banned_ips_path = Path::new(server_path).join("banned-ips.json");
    if !banned_ips_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&banned_ips_path)
        .map_err(|error| format!("Failed to read banned-ips.json: {error}"))?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str::<Vec<BannedIpEntry>>(trimmed)
        .map_err(|error| format!("Failed to parse banned-ips.json: {error}"))
}

pub fn read_player_summary(server_path: &str) -> Result<ServerPlayerSummary, String> {
    Ok(ServerPlayerSummary {
        whitelist: read_whitelist(server_path)?,
        banned_players: read_banned_players(server_path)?,
        banned_ips: read_banned_ips(server_path)?,
        ops: read_ops(server_path)?,
    })
}

fn map_whitelist_entries(entries: Vec<sl_libscv::state::WhitelistEntry>) -> Vec<PlayerEntry> {
    entries
        .into_iter()
        .map(|entry| PlayerEntry { uuid: entry.uuid, name: entry.name })
        .collect()
}

fn map_ban_entries(entries: Vec<sl_libscv::state::BannedPlayerEntry>) -> Vec<BanEntry> {
    entries
        .into_iter()
        .map(|entry| BanEntry {
            uuid: entry.uuid,
            name: entry.name,
            reason: entry.reason,
            source: entry.source,
            created: entry.created,
            expires: entry.expires,
        })
        .collect()
}

fn map_op_entries(entries: Vec<sl_libscv::state::OpEntry>) -> Vec<OpEntry> {
    entries
        .into_iter()
        .map(|entry| OpEntry {
            uuid: entry.uuid,
            name: entry.name,
            level: entry.level,
            bypasses_player_limit: entry.bypasses_player_limit,
        })
        .collect()
}

fn render_state_error(error: sl_libscv::StateFileError) -> String {
    match error {
        sl_libscv::StateFileError::Io(message)
        | sl_libscv::StateFileError::ParseFailed(message) => message,
    }
}

#[cfg(test)]
mod tests {
    use super::{read_banned_ips, read_ops, read_player_summary, read_whitelist};

    #[test]
    fn whitelist_reader_uses_sl_libscv_conventions() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        std::fs::write(temp_dir.path().join("whitelist.json"), r#"[{"uuid":"1","name":"Alex"}]"#)
            .expect("whitelist should write");

        let entries =
            read_whitelist(&temp_dir.path().to_string_lossy()).expect("whitelist should parse");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Alex");
    }

    #[test]
    fn ops_reader_keeps_bypass_alias_compatibility() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        std::fs::write(
            temp_dir.path().join("ops.json"),
            r#"[{"uuid":"1","name":"Alex","level":4,"bypassesPlayerLimit":true}]"#,
        )
        .expect("ops should write");

        let entries = read_ops(&temp_dir.path().to_string_lossy()).expect("ops should parse");

        assert_eq!(entries.len(), 1);
        assert!(entries[0].bypasses_player_limit);
    }

    #[test]
    fn banned_ips_reader_uses_standard_banned_ips_file() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        std::fs::write(
            temp_dir.path().join("banned-ips.json"),
            r#"[{"ip":"203.0.113.10","reason":"proxy","source":"Console","created":"now","expires":"forever"}]"#,
        )
        .expect("banned ips should write");

        let entries =
            read_banned_ips(&temp_dir.path().to_string_lossy()).expect("banned ips should parse");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].ip, "203.0.113.10");
        assert_eq!(entries[0].reason, "proxy");
    }

    #[test]
    fn player_summary_aggregates_file_backed_lists() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        std::fs::write(temp_dir.path().join("whitelist.json"), r#"[{"uuid":"1","name":"Alex"}]"#)
            .expect("whitelist should write");
        std::fs::write(
            temp_dir.path().join("banned-players.json"),
            r#"[{"uuid":"2","name":"Steve","reason":"grief","source":"Console","created":"now","expires":"forever"}]"#,
        )
        .expect("banned players should write");
        std::fs::write(
            temp_dir.path().join("banned-ips.json"),
            r#"[{"ip":"203.0.113.10","reason":"proxy","source":"Console","created":"now","expires":"forever"}]"#,
        )
        .expect("banned ips should write");
        std::fs::write(
            temp_dir.path().join("ops.json"),
            r#"[{"uuid":"3","name":"Admin","level":4,"bypassesPlayerLimit":true}]"#,
        )
        .expect("ops should write");

        let summary =
            read_player_summary(&temp_dir.path().to_string_lossy()).expect("summary should parse");

        assert_eq!(summary.whitelist.len(), 1);
        assert_eq!(summary.banned_players.len(), 1);
        assert_eq!(summary.banned_ips.len(), 1);
        assert_eq!(summary.ops.len(), 1);
    }
}
