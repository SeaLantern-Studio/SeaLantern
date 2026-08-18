//! Server.properties 配置文件管理
//!
//! 提供读取、写入、解析 server.properties 文件的能力

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

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
        let raw = parse_properties(&content)?;

        // 将原始配置转换为条目列表
        let entries = raw
            .iter()
            .map(|(key, value)| ConfigEntry {
                key: key.clone(),
                value: value.clone(),
                description: String::new(),
                value_type: "string".to_string(),
                default_value: String::new(),
                category: "general".to_string(),
            })
            .collect();

        debug!("读取 server.properties 成功，共 {} 项", raw.len());
        Ok(ServerProperties { entries, raw })
    }

    /// 写入服务器配置文件
    pub fn write(&self, values: &BTreeMap<String, String>) -> Result<(), ServerPropertiesError> {
        let file_path = self.properties_file();

        // 如果文件存在，先读取现有内容以保留注释和顺序
        let mut lines = if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            content.lines().map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            vec!["#Minecraft server properties".to_string()]
        };

        // 更新已有的键值对
        for (key, value) in values {
            let found = lines.iter_mut().find(|line| {
                let line = line.trim();
                !line.starts_with('#') && line.contains('=') && line.starts_with(key)
            });

            if let Some(line) = found {
                *line = format!("{}={}", key, value);
            } else {
                lines.push(format!("{}={}", key, value));
            }
        }

        // 写回文件
        let content = lines.join("\n");
        fs::write(&file_path, content)?;

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
        fs::write(&file_path, source)?;
        debug!("写入 server.properties 原始文本成功");
        Ok(())
    }

    /// 解析原始文本
    pub fn parse_source(source: &str) -> Result<ServerProperties, ServerPropertiesError> {
        let raw = parse_properties(source)?;
        let entries = raw
            .iter()
            .map(|(key, value)| ConfigEntry {
                key: key.clone(),
                value: value.clone(),
                description: String::new(),
                value_type: "string".to_string(),
                default_value: String::new(),
                category: "general".to_string(),
            })
            .collect();

        Ok(ServerProperties { entries, raw })
    }

    /// 预览写入后的文本
    pub fn preview_write(
        &self,
        values: &BTreeMap<String, String>,
    ) -> Result<String, ServerPropertiesError> {
        let file_path = self.properties_file();

        let mut lines = if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            content.lines().map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            vec!["#Minecraft server properties".to_string()]
        };

        for (key, value) in values {
            let found = lines.iter_mut().find(|line| {
                let line = line.trim();
                !line.starts_with('#') && line.contains('=') && line.starts_with(key)
            });

            if let Some(line) = found {
                *line = format!("{}={}", key, value);
            } else {
                lines.push(format!("{}={}", key, value));
            }
        }

        Ok(lines.join("\n"))
    }

    /// 从源码预览写入
    pub fn preview_write_from_source(
        source: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<String, ServerPropertiesError> {
        let mut lines = source.lines().map(|s| s.to_string()).collect::<Vec<_>>();

        for (key, value) in values {
            let found = lines.iter_mut().find(|line| {
                let line = line.trim();
                !line.starts_with('#') && line.contains('=') && line.starts_with(key)
            });

            if let Some(line) = found {
                *line = format!("{}={}", key, value);
            } else {
                lines.push(format!("{}={}", key, value));
            }
        }

        Ok(lines.join("\n"))
    }
}

/// 解析 Java Properties 格式
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
}
