//! Server.properties 配置文件管理
//!
//! 提供读取、写入、解析 server.properties 文件的能力

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use sealantern_infra::fs::{FileLock, FsError, write_atomic_blocking};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// 配置条目信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub description: String,
    pub value_type: String,
    pub default_value: String,
    pub category: String,
}

/// 服务器配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProperties {
    pub entries: Vec<ConfigEntry>,
    pub raw: BTreeMap<String, String>,
}

/// server.properties 文件管理器
pub struct ServerPropertiesManager {
    server_path: std::path::PathBuf,
}

impl ServerPropertiesManager {
    pub fn new(server_path: impl AsRef<Path>) -> Self {
        Self {
            server_path: server_path.as_ref().to_path_buf(),
        }
    }

    /// 获取 server.properties 文件路径
    fn properties_file(&self) -> std::path::PathBuf {
        self.server_path.join("server.properties")
    }

    /// 读取服务器配置文件
    pub fn read(&self) -> Result<ServerProperties, ServerPropertiesError> {
        let file_path = self.properties_file();

        if !file_path.exists() {
            debug!("server.properties 文件不存在: {:?}", file_path);
            return Ok(ServerProperties { entries: vec![], raw: BTreeMap::new() });
        }

        let content = fs::read_to_string(&file_path)?;
        let properties = properties_from_source(&content)?;

        debug!("读取 server.properties 成功，共 {} 项", properties.raw.len());
        Ok(properties)
    }

    /// 写入服务器配置文件
    pub fn write(&self, values: &BTreeMap<String, String>) -> Result<(), ServerPropertiesError> {
        let file_path = self.properties_file();
        let _lock = lock_properties_file(&file_path)?;

        // 如果文件存在，先读取现有内容以保留注释和顺序
        let source = read_source_or_default(&file_path)?;
        let content = apply_values_to_source(&source, values);
        write_properties_file(&file_path, &content)?;

        debug!("写入 server.properties 成功");
        Ok(())
    }

    /// 读取原始文本
    pub fn read_source(&self) -> Result<String, ServerPropertiesError> {
        let file_path = self.properties_file();
        let content = fs::read_to_string(&file_path)?;
        Ok(content)
    }

    /// 写入原始文本
    pub fn write_source(&self, source: &str) -> Result<(), ServerPropertiesError> {
        let file_path = self.properties_file();
        let _lock = lock_properties_file(&file_path)?;
        write_properties_file(&file_path, source)?;
        debug!("写入 server.properties 原始文本成功");
        Ok(())
    }

    /// 解析原始文本
    pub fn parse_source(source: &str) -> Result<ServerProperties, ServerPropertiesError> {
        properties_from_source(source)
    }

    /// 预览写入后的文本
    pub fn preview_write(
        &self,
        values: &BTreeMap<String, String>,
    ) -> Result<String, ServerPropertiesError> {
        let file_path = self.properties_file();
        let source = read_source_or_default(&file_path)?;
        Ok(apply_values_to_source(&source, values))
    }

    /// 从源码预览写入
    pub fn preview_write_from_source(
        source: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<String, ServerPropertiesError> {
        Ok(apply_values_to_source(source, values))
    }
}

const DEFAULT_SOURCE: &str = "#Minecraft server properties";

#[derive(Clone, Copy)]
struct PropertyMetadata {
    value_type: &'static str,
    default_value: &'static str,
    category: &'static str,
}

/// 仅为编辑器已经支持的常见字段提供小型 schema；未知字段仍保持可编辑字符串。
fn property_metadata(key: &str, value: &str) -> PropertyMetadata {
    let value_type = match key {
        "server-port"
        | "max-players"
        | "view-distance"
        | "simulation-distance"
        | "max-tick-time"
        | "network-compression-threshold"
        | "spawn-protection"
        | "max-world-size"
        | "op-permission-level"
        | "player-idle-timeout"
        | "rate-limit"
        | "management-server-port"
        | "function-permission-level"
        | "entity-broadcast-range-percentage" => "number",
        "online-mode"
        | "white-list"
        | "enforce-whitelist"
        | "hardcore"
        | "pvp"
        | "allow-flight"
        | "allow-nether"
        | "spawn-monsters"
        | "spawn-animals"
        | "spawn-npcs"
        | "generate-structures"
        | "enable-command-block"
        | "enable-query"
        | "enable-rcon"
        | "enable-status"
        | "force-gamemode"
        | "sync-chunk-writes"
        | "accepts-transfers"
        | "management-server-enabled"
        | "management-server-tls-enabled"
        | "use-native-transport"
        | "log-ips"
        | "prevent-proxy-connections"
        | "require-resource-pack"
        | "hide-online-players" => "boolean",
        _ if matches!(value, "true" | "false") => "boolean",
        _ => "string",
    };

    let category = match key {
        "server-port"
        | "server-ip"
        | "online-mode"
        | "enable-query"
        | "enable-rcon"
        | "management-server-enabled"
        | "management-server-host"
        | "management-server-port"
        | "management-server-secret"
        | "management-server-tls-enabled"
        | "management-server-tls-keystore"
        | "management-server-tls-keystore-password" => "network",
        "max-players"
        | "white-list"
        | "enforce-whitelist"
        | "op-permission-level"
        | "player-idle-timeout"
        | "hide-online-players"
        | "function-permission-level" => "player",
        "gamemode"
        | "difficulty"
        | "hardcore"
        | "pvp"
        | "allow-flight"
        | "allow-nether"
        | "spawn-monsters"
        | "spawn-animals"
        | "spawn-npcs"
        | "generate-structures"
        | "enable-command-block"
        | "enable-status"
        | "force-gamemode"
        | "accepts-transfers"
        | "require-resource-pack" => "game",
        "level-name"
        | "level-seed"
        | "level-type"
        | "spawn-protection"
        | "max-world-size"
        | "resource-pack"
        | "resource-pack-id"
        | "resource-pack-prompt"
        | "resource-pack-sha1" => "world",
        "view-distance"
        | "simulation-distance"
        | "max-tick-time"
        | "network-compression-threshold"
        | "sync-chunk-writes"
        | "use-native-transport"
        | "max-chained-neighbor-updates"
        | "entity-broadcast-range-percentage" => "performance",
        "motd" => "display",
        _ => "other",
    };

    let default_value = match key {
        "server-port" => "25565",
        "max-players" => "20",
        "gamemode" => "survival",
        "difficulty" => "easy",
        _ => "",
    };

    PropertyMetadata { value_type, default_value, category }
}

fn properties_from_source(source: &str) -> Result<ServerProperties, ServerPropertiesError> {
    let raw = parse_properties(source)?;
    let entries = raw
        .iter()
        .map(|(key, value)| {
            let metadata = property_metadata(key, value);
            ConfigEntry {
                key: key.clone(),
                value: value.clone(),
                description: String::new(),
                value_type: metadata.value_type.to_string(),
                default_value: metadata.default_value.to_string(),
                category: metadata.category.to_string(),
            }
        })
        .collect();

    Ok(ServerProperties { entries, raw })
}

fn read_source_or_default(path: &Path) -> Result<String, ServerPropertiesError> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(DEFAULT_SOURCE.to_owned()),
        Err(error) => Err(error.into()),
    }
}

fn apply_values_to_source(source: &str, values: &BTreeMap<String, String>) -> String {
    let mut lines = source.lines().map(str::to_owned).collect::<Vec<_>>();

    for (key, value) in values {
        // 解析规则是重复键后者生效，因此写回也更新最后一个匹配项。
        if let Some(index) = lines.iter().rposition(|line| line_matches_key(line, key)) {
            lines[index] = format!("{key}={value}");
        } else {
            lines.push(format!("{key}={value}"));
        }
    }

    lines.join("\n")
}

fn lock_properties_file(path: &Path) -> Result<FileLock, ServerPropertiesError> {
    FileLock::try_acquire(path).map_err(storage_error)
}

fn write_properties_file(path: &Path, content: &str) -> Result<(), ServerPropertiesError> {
    write_atomic_blocking(path, content.as_bytes()).map_err(storage_error)
}

fn storage_error(error: FsError) -> ServerPropertiesError {
    std::io::Error::other(error.to_string()).into()
}

/// 判断一行是否匹配指定的键（精确匹配，忽略前导空白和注释）
fn line_matches_key(line: &str, key: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.starts_with('#') || trimmed.starts_with('!') {
        return false;
    }
    match trimmed.find('=') {
        Some(pos) => trimmed[..pos].trim() == key,
        None => false,
    }
}

/// 解析 Minecraft `server.properties` 使用的 `key=value` 子集。
fn parse_properties(content: &str) -> Result<BTreeMap<String, String>, ServerPropertiesError> {
    let mut map = BTreeMap::new();

    for line in content.lines() {
        let line = line.trim();

        // 跳过空行和注释
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }

        // 分割键值对
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim();
            let value = line[pos + 1..].trim();

            if !key.is_empty() {
                map.insert(key.to_string(), value.to_string());
            }
        }
    }

    Ok(map)
}

/// server.properties 处理错误
#[derive(Debug, thiserror::Error)]
pub enum ServerPropertiesError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("解析错误: {0}")]
    Parse(String),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use proptest::prelude::*;

    use super::*;

    #[test]
    fn test_parse_properties() {
        let content = r#"
#Minecraft server properties
server-port=25565
max-players=20
"#;
        let result = parse_properties(content).unwrap();
        assert_eq!(result.get("server-port"), Some(&"25565".to_string()));
        assert_eq!(result.get("max-players"), Some(&"20".to_string()));
    }

    #[test]
    fn metadata_describes_known_editor_fields() {
        let properties = ServerPropertiesManager::parse_source(
            "server-port=25565\nonline-mode=true\ngamemode=survival\nunknown=value",
        )
        .unwrap();

        let by_key = properties
            .entries
            .into_iter()
            .map(|entry| (entry.key.clone(), entry))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(by_key["server-port"].value_type, "number");
        assert_eq!(by_key["server-port"].default_value, "25565");
        assert_eq!(by_key["server-port"].category, "network");
        assert_eq!(by_key["online-mode"].value_type, "boolean");
        assert_eq!(by_key["unknown"].value_type, "string");
        assert_eq!(by_key["unknown"].category, "other");
    }

    #[test]
    fn duplicate_keys_are_updated_using_the_same_last_value_parser_rule() {
        let mut values = BTreeMap::new();
        values.insert("motd".to_string(), "new".to_string());

        let source = "motd=old\n# keep\nmotd=latest";
        let preview = ServerPropertiesManager::preview_write_from_source(source, &values).unwrap();

        assert_eq!(preview, "motd=old\n# keep\nmotd=new");
        assert_eq!(parse_properties(&preview).unwrap()["motd"], "new");
    }

    #[test]
    fn file_writes_are_atomic_and_release_the_lock() {
        let root = tempfile::tempdir().expect("temporary server directory should be created");
        let manager = ServerPropertiesManager::new(root.path());

        manager
            .write_source("motd=Sea Lantern")
            .expect("server properties should be written");

        let path = root.path().join("server.properties");
        assert_eq!(
            std::fs::read_to_string(&path).expect("server properties should be readable"),
            "motd=Sea Lantern"
        );
        let lock = FileLock::try_acquire(&path).expect("write should release the file lock");
        drop(lock);
    }

    proptest! {
        #[test]
        fn applying_updates_matches_an_independent_map_model(
            initial in prop::collection::vec(("[a-z][a-z0-9-]{0,8}", "[a-zA-Z0-9._/=-]{0,12}"), 0..24),
            updates in prop::collection::vec(("[a-z][a-z0-9-]{0,8}", "[a-zA-Z0-9._/=-]{0,12}"), 0..24),
        ) {
            let initial = initial.into_iter().collect::<BTreeMap<_, _>>();
            let updates = updates.into_iter().collect::<BTreeMap<_, _>>();
            let source = initial
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join("\n");

            let mut expected = initial;
            expected.extend(updates.clone());

            let output = apply_values_to_source(&source, &updates);
            prop_assert_eq!(parse_properties(&output)?, expected);
            prop_assert_eq!(apply_values_to_source(&output, &updates), output);
        }
    }
}
