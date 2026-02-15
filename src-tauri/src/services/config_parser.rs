use std::collections::HashMap;
use std::fs;

use crate::models::config::*;

/// Read a .properties file into a HashMap
pub fn read_properties(file_path: &str) -> Result<HashMap<String, String>, String> {
    let content =
        fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let mut map = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_string();
            let value = trimmed[eq_pos + 1..].trim().to_string();
            map.insert(key, value);
        }
    }

    Ok(map)
}

/// Write a HashMap to a .properties file, preserving comments
pub fn write_properties(file_path: &str, values: &HashMap<String, String>) -> Result<(), String> {
    let original = fs::read_to_string(file_path).unwrap_or_default();

    let mut output = String::new();
    let mut written_keys: Vec<String> = Vec::new();

    for line in original.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            output.push_str(line);
            output.push('\n');
            continue;
        }
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();
            if let Some(new_value) = values.get(key) {
                output.push_str(&format!("{}={}\n", key, new_value));
                written_keys.push(key.to_string());
            } else {
                output.push_str(line);
                output.push('\n');
            }
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    // Add new keys that weren't in the original file
    for (key, value) in values {
        if !written_keys.contains(key) {
            output.push_str(&format!("{}={}\n", key, value));
        }
    }

    fs::write(file_path, output).map_err(|e| format!("Failed to write file: {}", e))
}

/// Parse server.properties with descriptions
pub fn parse_server_properties(file_path: &str) -> Result<ServerProperties, String> {
    let raw = read_properties(file_path)?;
    let mut entries = Vec::new();

    let descriptions = get_property_descriptions();

    for (key, value) in &raw {
        let desc_info = descriptions.get(key.as_str());
        entries.push(ConfigEntry {
            key: key.clone(),
            value: value.clone(),
            description: desc_info.map(|d| d.0.to_string()).unwrap_or_default(),
            value_type: desc_info
                .map(|d| d.1.to_string())
                .unwrap_or_else(|| "string".to_string()),
            default_value: desc_info.map(|d| d.2.to_string()).unwrap_or_default(),
            category: desc_info
                .map(|d| d.3.to_string())
                .unwrap_or_else(|| "other".to_string()),
        });
    }

    // Sort by category
    entries.sort_by(|a, b| a.category.cmp(&b.category).then(a.key.cmp(&b.key)));

    Ok(ServerProperties { entries, raw })
}

/// Property descriptions: (description, type, default, category)
fn get_property_descriptions(
) -> HashMap<&'static str, (&'static str, &'static str, &'static str, &'static str)> {
    let mut m = HashMap::new();
    m.insert("server-port", ("服务器端口", "number", "25565", "network"));
    m.insert("server-ip", ("服务器绑定IP，留空表示所有", "string", "", "network"));
    m.insert("max-players", ("最大玩家数", "number", "20", "player"));
    m.insert("online-mode", ("正版验证", "boolean", "true", "player"));
    m.insert("white-list", ("启用白名单", "boolean", "false", "player"));
    m.insert("enforce-whitelist", ("强制白名单", "boolean", "false", "player"));
    m.insert("gamemode", ("默认游戏模式", "select", "survival", "game"));
    m.insert("difficulty", ("游戏难度", "select", "easy", "game"));
    m.insert("hardcore", ("极限模式", "boolean", "false", "game"));
    m.insert("pvp", ("允许PVP", "boolean", "true", "game"));
    m.insert("allow-flight", ("允许飞行", "boolean", "false", "game"));
    m.insert("allow-nether", ("允许下界", "boolean", "true", "world"));
    m.insert("spawn-monsters", ("生成怪物", "boolean", "true", "world"));
    m.insert("spawn-animals", ("生成动物", "boolean", "true", "world"));
    m.insert("spawn-npcs", ("生成NPC", "boolean", "true", "world"));
    m.insert("generate-structures", ("生成结构", "boolean", "true", "world"));
    m.insert("level-name", ("世界名称", "string", "world", "world"));
    m.insert("level-seed", ("世界种子", "string", "", "world"));
    m.insert("level-type", ("世界类型", "string", "minecraft:normal", "world"));
    m.insert("view-distance", ("视距", "number", "10", "performance"));
    m.insert("simulation-distance", ("模拟距离", "number", "10", "performance"));
    m.insert(
        "max-tick-time",
        ("最大tick时间(ms)，-1为禁用", "number", "60000", "performance"),
    );
    m.insert(
        "network-compression-threshold",
        ("网络压缩阈值", "number", "256", "performance"),
    );
    m.insert("motd", ("服务器描述(MOTD)", "string", "A Minecraft Server", "display"));
    m.insert("enable-command-block", ("启用命令方块", "boolean", "false", "game"));
    m.insert("enable-query", ("启用Query协议", "boolean", "false", "network"));
    m.insert("enable-rcon", ("启用RCON远程控制", "boolean", "false", "network"));
    m.insert("enable-status", ("启用服务器列表状态", "boolean", "true", "network"));
    m.insert("force-gamemode", ("强制游戏模式", "boolean", "false", "game"));
    m.insert("spawn-protection", ("出生点保护半径", "number", "16", "world"));
    m.insert("sync-chunk-writes", ("同步区块写入", "boolean", "true", "performance"));
    m
}
