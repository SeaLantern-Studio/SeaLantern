use async_trait::async_trait;
use sealantern_interface::{PlayerLookupError, PlayerLookupService, PlayerProfile};
use std::path::Path;

/// usercache.json 里每条记录的格式。
#[derive(serde::Deserialize)]
struct UserCacheEntry {
    name: String,
    uuid: String,
}

pub struct CorePlayerService;

impl CorePlayerService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CorePlayerService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlayerLookupService for CorePlayerService {
    async fn lookup(
        &self,
        server_path: String,
        username: String,
    ) -> Result<PlayerProfile, PlayerLookupError> {
        // 1. 校验用户名：不能空，只能字母数字下划线
        let username = username.trim();
        if username.is_empty() || !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(PlayerLookupError::InvalidInput);
        }

        // 2. 校验服务器路径
        if server_path.trim().is_empty() {
            return Err(PlayerLookupError::ServerNotSelected);
        }

        // 3. 读 usercache.json
        let cache_path = Path::new(&server_path).join("usercache.json");
        let content = tokio::fs::read_to_string(&cache_path)
            .await
            .map_err(|_| PlayerLookupError::ServiceUnavailable)?;

        // 4. 解析 JSON 数组，按用户名查找（不区分大小写）
        let entries: Vec<UserCacheEntry> =
            serde_json::from_str(&content).map_err(|_| PlayerLookupError::ServiceUnavailable)?;

        let found = entries
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(username));

        match found {
            Some(entry) => {
                // usercache.json 的 UUID 是 8-4-4-4-12 带连字符格式
                // 去掉连字符，保持无连字符形式
                let uuid = entry.uuid.replace('-', "");
                Ok(PlayerProfile { name: entry.name.clone(), uuid })
            }
            None => Err(PlayerLookupError::NotFound),
        }
    }
}
