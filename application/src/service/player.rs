use async_trait::async_trait;
use sealantern_infra::net::{NetError, global_client};
use sealantern_interface::{PlayerLookupError, PlayerLookupService, PlayerProfile};
/// Mojang API 返回的 JSON 格式。
#[derive(serde::Deserialize)]
struct MojangProfileResponse {
    name: String,
    id: String,
}

pub struct CorePlayerService;

impl CorePlayerService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PlayerLookupService for CorePlayerService {
    async fn lookup(&self, username: String) -> Result<PlayerProfile, PlayerLookupError> {
        // 1. 校验 username（空或含非法字符 → InvalidInput）
        let username = username.trim();
        if username.is_empty() || !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(PlayerLookupError::InvalidInput);
        }
        // 2. 拿客户端：let client = global_client().map_err(|_| PlayerLookupError::ServiceUnavailable)?;
        let client = global_client().map_err(|_| PlayerLookupError::ServiceUnavailable)?;
        // 3. 发 GET 请求到 Mojang API
        let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", username);
        // 4. 根据状态码判断：
        let result = client
            .get(&url)
            .map_err(|_| PlayerLookupError::ServiceUnavailable)?
            .send()
            .await;
        match result {
            Ok(resp) => {
                // 200 和 204 都走这里，要区分
                if resp.status().as_u16() == 204 {
                    return Err(PlayerLookupError::NotFound);
                }
                // 200 -> 解析json
                let mojang: MojangProfileResponse = resp
                    .json()
                    .await
                    .map_err(|_| PlayerLookupError::ServiceUnavailable)?;
                Ok(PlayerProfile { name: mojang.name, uuid: mojang.id })
            }
            Err(NetError::Response(429, _)) => Err(PlayerLookupError::RateLimited),
            Err(_) => Err(PlayerLookupError::ServiceUnavailable),
        }
    }
}
