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
    read_json_list(server_path, "whitelist.json")
}

pub fn read_banned_players(server_path: &str) -> Result<Vec<BanEntry>, String> {
    read_json_list(server_path, "banned-players.json")
}

pub fn read_ops(server_path: &str) -> Result<Vec<OpEntry>, String> {
    read_json_list(server_path, "ops.json")
}

fn read_json_list<T: serde::de::DeserializeOwned>(
    server_path: &str,
    filename: &str,
) -> Result<Vec<T>, String> {
    let path = std::path::Path::new(server_path).join(filename);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("读取{}失败: {}", filename, e))?;
    let trimmed = content.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Ok(Vec::new());
    }
    serde_json::from_str(trimmed).map_err(|e| format!("解析{}失败: {}", filename, e))
}
