use serde::{Deserialize, Serialize};

/// 服务器短 ID 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerIdEntry {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub description: Option<String>,
    pub created_at: u64,
    pub last_accessed_at: Option<u64>,
    pub is_active: bool,
    pub tags: Vec<String>,
}

/// 创建短 ID 的请求
#[derive(Debug, Deserialize)]
pub struct CreateServerIdRequest {
    pub id: Option<String>,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// 短 ID 操作响应
#[derive(Debug, Serialize)]
pub struct ServerIdResponse {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub created_at: u64,
    pub is_active: bool,
}
