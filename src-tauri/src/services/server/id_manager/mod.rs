//! 服务器短 ID 管理

mod models;
mod ops;
mod shared;

pub use models::{CreateServerIdRequest, ServerIdEntry, ServerIdResponse};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use ops::{
    create_id, deactivate_id, delete_id, get_id, list_ids, resolve_id, search_ids, update_id,
};

pub(crate) type SharedServerIdEntries = Arc<RwLock<HashMap<String, ServerIdEntry>>>;

/// 服务器短 ID 管理器
pub struct ServerIdManager {
    entries: SharedServerIdEntries,
}

impl ServerIdManager {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建一个新的服务器短 ID
    pub async fn create_id(&self, req: CreateServerIdRequest) -> Result<ServerIdEntry, String> {
        create_id(&self.entries, req).await
    }

    /// 解析服务器短 ID
    pub async fn resolve_id(&self, id: &str) -> Result<(String, u16), String> {
        resolve_id(&self.entries, id).await
    }

    /// 读取短 ID 详情
    pub async fn get_id(&self, id: &str) -> Result<ServerIdEntry, String> {
        get_id(&self.entries, id).await
    }

    /// 列出全部启用中的短 ID
    pub async fn list_ids(&self) -> Vec<ServerIdEntry> {
        list_ids(&self.entries).await
    }

    /// 更新短 ID 信息
    pub async fn update_id(
        &self,
        id: &str,
        name: Option<String>,
        address: Option<String>,
        port: Option<u16>,
    ) -> Result<ServerIdEntry, String> {
        update_id(&self.entries, id, name, address, port).await
    }

    /// 停用短 ID
    pub async fn deactivate_id(&self, id: &str) -> Result<(), String> {
        deactivate_id(&self.entries, id).await
    }

    /// 删除短 ID
    pub async fn delete_id(&self, id: &str) -> Result<(), String> {
        delete_id(&self.entries, id).await
    }

    /// 搜索短 ID
    pub async fn search_ids(&self, query: &str) -> Vec<ServerIdEntry> {
        search_ids(&self.entries, query).await
    }
}

#[cfg(test)]
mod tests;
