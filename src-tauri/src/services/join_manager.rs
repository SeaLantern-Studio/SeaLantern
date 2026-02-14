use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use crate::services::global;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerAddress {
    pub host: String,
    pub port: u16,
}

pub struct JoinManager {
    // 模拟一个远程 ID 映射，实际应用中可以对接一个简单的 KV 存储或 API
    // 这里先实现本地映射逻辑
    id_map: Mutex<HashMap<String, ServerAddress>>,
}

impl JoinManager {
    pub fn new() -> Self {
        JoinManager {
            id_map: Mutex::new(HashMap::new()),
        }
    }

    /// 通过 ID 获取服务器地址
    pub async fn resolve_id(&self, id: &str) -> Result<ServerAddress, String> {
        // 1. 检查本地已有的服务器 ID
        let server_manager = global::server_manager();
        let servers = server_manager.servers.lock().unwrap();
        if let Some(server) = servers.iter().find(|s| s.id == id || s.name == id) {
            return Ok(ServerAddress {
                host: "127.0.0.1".to_string(),
                port: server.port,
            });
        }

        // 2. 检查手动添加的映射
        let map = self.id_map.lock().unwrap();
        if let Some(addr) = map.get(id) {
            return Ok(addr.clone());
        }

        // 3. 模拟远程解析 (未来可以扩展为访问中心化 API)
        // 示例：如果 ID 是 "manus-test"，返回一个测试地址
        if id == "manus-test" {
            return Ok(ServerAddress {
                host: "play.manus.im".to_string(),
                port: 25565,
            });
        }

        Err(format!("无法解析服务器 ID: {}", id))
    }

    /// 注册一个新的 ID 映射（本地）
    pub fn register_id(&self, id: String, host: String, port: u16) {
        let mut map = self.id_map.lock().unwrap();
        map.insert(id, ServerAddress { host, port });
    }
}
