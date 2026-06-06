use std::collections::HashMap;
use std::fs;

use crate::types::{ConfigEntry, ServerProperties};

pub fn parse_properties_content(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value, _separator)) = split_property_line(trimmed) {
            map.insert(key, value);
        }
    }

    map
}

pub fn render_properties_content(original: &str, values: &HashMap<String, String>) -> String {
    let mut output = String::new();
    let mut written_keys: Vec<String> = Vec::new();

    for line in original.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            output.push_str(line);
            output.push('\n');
            continue;
        }

        if let Some((key, _value, separator)) = split_property_line(trimmed) {
            if let Some(new_value) = values.get(&key) {
                output.push_str(&format!("{}{}{}\n", key, separator, new_value));
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

    let mut missing_entries = values
        .iter()
        .filter(|(key, _)| !written_keys.contains(key))
        .collect::<Vec<_>>();
    missing_entries.sort_by_key(|(key, _)| *key);

    for (key, value) in missing_entries {
        output.push_str(&format!("{}={}\n", key, value));
    }

    output
}

fn split_property_line(line: &str) -> Option<(String, String, char)> {
    let separator_index = line.find(['=', ':'])?;
    let separator = line[separator_index..].chars().next()?;
    let key = line[..separator_index].trim().to_string();
    let value = line[separator_index + separator.len_utf8()..].trim().to_string();
    Some((key, value, separator))
}

pub fn read_raw_text(file_path: &str) -> Result<String, String> {
    fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))
}

pub fn write_raw_text(file_path: &str, content: &str) -> Result<(), String> {
    fs::write(file_path, content).map_err(|e| format!("Failed to write file: {}", e))
}

pub fn preview_properties_write(
    file_path: &str,
    values: &HashMap<String, String>,
) -> Result<String, String> {
    let original = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("Failed to read file: {}", error)),
    };
    Ok(render_properties_content(&original, values))
}

pub fn preview_properties_write_from_source(
    source: &str,
    values: &HashMap<String, String>,
) -> Result<String, String> {
    Ok(render_properties_content(source, values))
}

pub fn read_properties(file_path: &str) -> Result<HashMap<String, String>, String> {
    let content = read_raw_text(file_path)?;
    Ok(parse_properties_content(&content))
}

pub fn write_properties(file_path: &str, values: &HashMap<String, String>) -> Result<(), String> {
    let output = preview_properties_write(file_path, values)?;
    write_raw_text(file_path, &output)
}

pub fn parse_server_properties(file_path: &str) -> Result<ServerProperties, String> {
    let content = read_raw_text(file_path)?;
    parse_server_properties_from_source(&content)
}

pub fn parse_server_properties_from_source(source: &str) -> Result<ServerProperties, String> {
    let raw = parse_properties_content(source);
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

    entries.sort_by(|a, b| a.category.cmp(&b.category).then(a.key.cmp(&b.key)));

    Ok(ServerProperties { entries, raw })
}

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

#[cfg(test)]
mod tests {
    use super::{
        parse_properties_content, parse_server_properties_from_source, preview_properties_write,
        render_properties_content, write_properties,
    };
    use std::collections::HashMap;

    #[test]
    fn render_properties_content_updates_motd_as_plain_text() {
        let original = "motd=Old Server\nmax-players=20\n";
        let mut values = HashMap::new();
        values.insert("motd".to_string(), "Hello Sea Lantern".to_string());
        values.insert("max-players".to_string(), "20".to_string());

        let rendered = render_properties_content(original, &values);

        assert!(rendered.contains("motd=Hello Sea Lantern\n"));
    }

    #[test]
    fn parse_properties_content_keeps_value_text_after_first_equals() {
        let parsed = parse_properties_content("motd=hello=world\n");

        assert_eq!(parsed.get("motd"), Some(&"hello=world".to_string()));
    }

    #[test]
    fn parse_properties_content_supports_colon_separator() {
        let parsed = parse_properties_content("server-port:25565\nmotd: Hello Sea Lantern\n");

        assert_eq!(parsed.get("server-port"), Some(&"25565".to_string()));
        assert_eq!(parsed.get("motd"), Some(&"Hello Sea Lantern".to_string()));
    }

    #[test]
    fn parse_server_properties_from_source_sorts_entries_by_category_then_key() {
        let parsed = parse_server_properties_from_source(
            "view-distance=10\nmotd=Hello\nmax-players=20\n",
        )
        .unwrap();

        let keys = parsed.entries.iter().map(|entry| entry.key.as_str()).collect::<Vec<_>>();
        assert_eq!(keys, vec!["motd", "view-distance", "max-players"]);
    }

    #[test]
    fn write_properties_preserves_existing_comments() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.properties");
        std::fs::write(&path, "# header\nmotd=Old\n").unwrap();

        let mut values = HashMap::new();
        values.insert("motd".to_string(), "New".to_string());

        write_properties(&path.to_string_lossy(), &values).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();

        assert!(content.starts_with("# header\n"));
        assert!(content.contains("motd=New\n"));
    }

    #[test]
    fn render_properties_content_appends_missing_keys_in_sorted_order() {
        let mut values = HashMap::new();
        values.insert("white-list".to_string(), "true".to_string());
        values.insert("allow-flight".to_string(), "false".to_string());
        values.insert("motd".to_string(), "Hello".to_string());

        let rendered = render_properties_content("motd=Old\n", &values);

        assert_eq!(
            rendered,
            "motd=Hello\nallow-flight=false\nwhite-list=true\n"
        );
    }

    #[test]
    fn render_properties_content_preserves_colon_separator_for_existing_keys() {
        let mut values = HashMap::new();
        values.insert("server-port".to_string(), "25570".to_string());

        let rendered = render_properties_content("server-port:25565\n", &values);

        assert_eq!(rendered, "server-port:25570\n");
    }

    #[test]
    fn preview_properties_write_treats_missing_file_as_empty_source() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.properties");
        let mut values = HashMap::new();
        values.insert("motd".to_string(), "Hello".to_string());

        let rendered = preview_properties_write(&path.to_string_lossy(), &values).unwrap();

        assert_eq!(rendered, "motd=Hello\n");
    }

    #[test]
    fn preview_properties_write_propagates_non_missing_read_errors() {
        let dir = tempfile::tempdir().unwrap();
        let mut values = HashMap::new();
        values.insert("motd".to_string(), "Hello".to_string());

        let error = preview_properties_write(&dir.path().to_string_lossy(), &values)
            .expect_err("directory path should not be treated as empty file");

        assert!(error.contains("Failed to read file:"));
    }
}
