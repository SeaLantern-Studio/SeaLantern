//! 服务器配置（server.properties）服务端口。

use std::collections::BTreeMap;

use async_trait::async_trait;
use sealantern_contract::ServerConfigServiceError;
use sealantern_contract::server_config::ServerProperties;

/// 服务器配置（server.properties）宿主能力端口。
///
/// 提供配置文件的可视化结构与原始文本的双向读写、解析与写入预览。
/// 方法均为异步：文件读写涉及阻塞 IO，由实现方调度到阻塞线程池，
/// 不依赖任何具体宿主。
#[async_trait]
pub trait ServerConfigService: Send + Sync {
    /// 读取服务器目录下的 `server.properties` 为可视化配置结构。
    ///
    /// 文件不存在时返回空配置（不报错），与原有行为保持一致。
    async fn read(&self, server_path: &str) -> Result<ServerProperties, ServerConfigServiceError>;

    /// 按键值对更新服务器目录下的 `server.properties`（保留注释与顺序）。
    async fn write(
        &self,
        server_path: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<(), ServerConfigServiceError>;

    /// 读取 `server.properties` 原始文本。
    async fn read_source(&self, server_path: &str) -> Result<String, ServerConfigServiceError>;

    /// 直接写入 `server.properties` 原始文本。
    async fn write_source(
        &self,
        server_path: &str,
        source: &str,
    ) -> Result<(), ServerConfigServiceError>;

    /// 将原始文本解析为可视化配置结构。
    async fn parse_source(
        &self,
        source: &str,
    ) -> Result<ServerProperties, ServerConfigServiceError>;

    /// 预览可视化配置写回后的最终文本（基于服务器目录下的现有内容）。
    async fn preview_write(
        &self,
        server_path: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<String, ServerConfigServiceError>;

    /// 基于给定源码预览可视化配置写回后的最终文本。
    async fn preview_write_from_source(
        &self,
        source: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<String, ServerConfigServiceError>;
}
