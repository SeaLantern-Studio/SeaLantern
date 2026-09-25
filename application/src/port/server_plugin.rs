//! 服务器插件管理能力端口。
//!
//! 所有方法都以实例标识定位：服务器目录由应用层从实例注册表解析，
//! 因此调用方（无论 Tauri 还是 HTTP）都无法指定任意路径。

use async_trait::async_trait;

use sealantern_contract::ServerPluginServiceError;
use sealantern_contract::server_plugin::{PluginConfigFile, PluginSummary};
use sealantern_core::instance::InstanceId;

/// 服务器插件（`plugins` 目录）管理能力。
#[async_trait]
pub trait ServerPluginService: Send + Sync {
    /// 列出实例的全部服务器插件，包含已禁用（`.jar.disabled`）的文件。
    async fn list(&self, id: &InstanceId) -> Result<Vec<PluginSummary>, ServerPluginServiceError>;

    /// 读取某个插件配置目录下的文本文件。
    ///
    /// `plugin_name` 是插件声明的名称（配置目录名），由调用方提供。
    async fn read_config_files(
        &self,
        id: &InstanceId,
        file_name: &str,
        plugin_name: &str,
    ) -> Result<Vec<PluginConfigFile>, ServerPluginServiceError>;

    /// 启用或禁用一个插件（重命名 `.jar` ↔ `.jar.disabled`）。
    async fn set_enabled(
        &self,
        id: &InstanceId,
        file_name: &str,
        enabled: bool,
    ) -> Result<(), ServerPluginServiceError>;

    /// 永久删除一个插件。
    async fn delete(
        &self,
        id: &InstanceId,
        file_name: &str,
    ) -> Result<(), ServerPluginServiceError>;

    /// 安装一个插件：把字节内容写入 `plugins/<file_name>`。
    async fn install(
        &self,
        id: &InstanceId,
        file_name: &str,
        data: &[u8],
    ) -> Result<(), ServerPluginServiceError>;
}
