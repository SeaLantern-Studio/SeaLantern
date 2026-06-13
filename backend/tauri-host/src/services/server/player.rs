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
pub struct OpEntry {
    pub uuid: String,
    pub name: String,
    #[serde(default)]
    pub level: u32,
    #[serde(default, alias = "bypassesPlayerLimit", alias = "bypass_player_limit")]
    pub bypasses_player_limit: bool,
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

fn map_whitelist_entries(entries: Vec<sl_libscv::state::WhitelistEntry>) -> Vec<PlayerEntry> {
    entries
        .into_iter()
        .map(|entry| PlayerEntry {
            uuid: entry.uuid,
            name: entry.name,
        })
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
        sl_libscv::StateFileError::Io(message) | sl_libscv::StateFileError::ParseFailed(message) => {
            message
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{read_ops, read_whitelist};

    #[test]
    fn whitelist_reader_uses_sl_libscv_conventions() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        std::fs::write(
            temp_dir.path().join("whitelist.json"),
            r#"[{"uuid":"1","name":"Alex"}]"#,
        )
        .expect("whitelist should write");

        let entries = read_whitelist(&temp_dir.path().to_string_lossy()).expect("whitelist should parse");

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
}
