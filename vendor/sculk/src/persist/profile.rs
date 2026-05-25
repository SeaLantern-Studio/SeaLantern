//! 用户偏好 Profile，以 TOML 格式持久化到 `{data_dir}/sculk/profile.toml`。

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::data_dir;
use crate::Result;
use crate::error::PersistError;

const PROFILE_FILE: &str = "profile.toml";

/// 用户偏好配置根结构，序列化为 `profile.toml`。
///
/// 各字段均实现 [`Default`]，未出现在文件中的键自动取默认值，
/// 因此增删字段不会导致旧版配置文件解析失败。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub host: HostProfile,
    #[serde(default)]
    pub join: JoinProfile,
    #[serde(default)]
    pub relay: RelayProfile,
}

/// host 端偏好配置，对应 `[host]` TOML 节。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostProfile {
    /// 本地 Minecraft 服务端监听端口，默认 [`DEFAULT_MC_PORT`](crate::DEFAULT_MC_PORT)。
    #[serde(default = "default_mc_port")]
    pub port: u16,
}

/// join 端偏好配置，对应 `[join]` TOML 节。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinProfile {
    /// 本地入站监听端口，默认 [`DEFAULT_INLET_PORT`](crate::DEFAULT_INLET_PORT)。
    #[serde(default = "default_inlet_port")]
    pub port: u16,
    /// 上次成功加入的票据，序列化时若为 `None` 则省略。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_ticket: Option<String>,
}

/// relay 偏好配置，对应 `[relay]` TOML 节。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelayProfile {
    /// `true` 启用自建中继，`false` 使用 iroh 内置 n0 中继服务器组。
    #[serde(default)]
    pub custom: bool,
    /// 自建中继地址，仅 `custom = true` 时生效。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl Default for HostProfile {
    fn default() -> Self {
        Self {
            port: default_mc_port(),
        }
    }
}

impl Default for JoinProfile {
    fn default() -> Self {
        Self {
            port: default_inlet_port(),
            last_ticket: None,
        }
    }
}

fn default_mc_port() -> u16 {
    crate::DEFAULT_MC_PORT
}

fn default_inlet_port() -> u16 {
    crate::DEFAULT_INLET_PORT
}

impl Profile {
    /// 配置文件路径。
    pub fn path() -> Result<std::path::PathBuf> {
        Ok(data_dir()?.join(PROFILE_FILE))
    }

    /// 加载配置。文件不存在时创建默认配置并写入磁盘。
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        Self::load_from(&path)
    }

    /// 从指定路径加载配置。文件不存在时写入默认值。
    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            let profile = Self::default();
            profile.save_to(path)?;
            return Ok(profile);
        }
        let content = std::fs::read_to_string(path).map_err(|e| PersistError::PathIo {
            op: "read profile",
            path: path.to_path_buf(),
            source: e,
        })?;
        let profile: Self = toml::from_str(&content).map_err(|e| PersistError::ProfileParse {
            path: path.to_path_buf(),
            source: e,
        })?;
        Ok(profile)
    }

    /// 保存配置到默认路径。
    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        self.save_to(&path)
    }

    /// 保存配置到指定路径。
    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| PersistError::PathIo {
                op: "create config dir",
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        let content = toml::to_string_pretty(self).map_err(PersistError::ProfileSerialize)?;
        std::fs::write(path, content).map_err(|e| PersistError::PathIo {
            op: "write profile",
            path: path.to_path_buf(),
            source: e,
        })?;
        Ok(())
    }

    /// 解析最终使用的 relay URL，优先级从高到低：
    /// 1. 参数 `custom` 中显式传入的 URL；
    /// 2. `self.relay.custom == true` 时读取 `self.relay.url`；
    /// 3. `None`，使用 iroh 内置 n0 中继服务器组。
    pub fn resolve_relay_url(
        &self,
        custom: Option<&str>,
    ) -> Result<Option<crate::types::RelayUrl>> {
        let url_str = custom.or(if self.relay.custom {
            self.relay.url.as_deref()
        } else {
            None
        });
        match url_str {
            Some(s) => {
                let url: crate::types::RelayUrl = s
                    .parse::<crate::types::RelayUrl>()
                    .map_err(|e| PersistError::RelayUrlParse(e.to_string()))?;
                Ok(Some(url))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_values() {
        let p = Profile::default();
        assert_eq!(p.host.port, crate::DEFAULT_MC_PORT);
        assert_eq!(p.join.port, crate::DEFAULT_INLET_PORT);
        assert!(p.join.last_ticket.is_none());
        assert!(!p.relay.custom);
        assert!(p.relay.url.is_none());
    }

    #[test]
    fn roundtrip_toml() {
        let mut p = Profile::default();
        p.host.port = 12345;
        p.join.last_ticket = Some("sculk://test".to_string());
        p.relay.custom = true;
        p.relay.url = Some("https://relay.example.com".to_string());

        let s_res = toml::to_string_pretty(&p);
        assert!(s_res.is_ok(), "serialize profile failed");
        let s = if let Ok(v) = s_res { v } else { return };
        let p2_res: std::result::Result<Profile, toml::de::Error> = toml::from_str(&s);
        assert!(p2_res.is_ok(), "deserialize profile failed");
        let p2 = if let Ok(v) = p2_res { v } else { return };

        assert_eq!(p2.host.port, 12345);
        assert_eq!(p2.join.last_ticket.as_deref(), Some("sculk://test"));
        assert!(p2.relay.custom);
        assert_eq!(p2.relay.url.as_deref(), Some("https://relay.example.com"));
    }

    #[test]
    fn partial_toml_uses_defaults() {
        let s = "[host]\nport = 9999\n";
        let p_res: std::result::Result<Profile, toml::de::Error> = toml::from_str(s);
        assert!(p_res.is_ok(), "deserialize partial profile failed");
        let p = if let Ok(v) = p_res { v } else { return };
        assert_eq!(p.host.port, 9999);
        assert_eq!(p.join.port, crate::DEFAULT_INLET_PORT);
        assert!(p.relay.url.is_none());
    }

    #[test]
    fn save_and_load_file() {
        let dir = std::env::temp_dir().join("sculk_test_profile");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("profile.toml");

        let mut p = Profile::default();
        p.host.port = 11111;
        let save_res = p.save_to(&path);
        assert!(save_res.is_ok(), "save profile failed");

        let loaded_res = Profile::load_from(&path);
        assert!(loaded_res.is_ok(), "load profile failed");
        let loaded = if let Ok(v) = loaded_res { v } else { return };
        assert_eq!(loaded.host.port, 11111);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_missing_file_creates_default() {
        let dir = std::env::temp_dir().join("sculk_test_load_missing");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("profile.toml");

        let p_res = Profile::load_from(&path);
        assert!(p_res.is_ok(), "load missing profile failed");
        let p = if let Ok(v) = p_res { v } else { return };
        assert_eq!(p.host.port, crate::DEFAULT_MC_PORT);
        // 文件应该已被创建
        assert!(path.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
