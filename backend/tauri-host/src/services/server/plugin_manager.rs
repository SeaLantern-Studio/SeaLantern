//! 服务器插件文件管理

mod common;
mod config_files;
mod listing;
mod ops;
mod parser;

use crate::models::mcs_plugin::{m_PluginConfigFile, m_PluginInfo};

/// 服务器插件管理器
pub struct ServerPluginManager;

impl ServerPluginManager {
    /// 创建插件管理器
    pub fn new() -> Self {
        Self
    }

    /// 读取服务器里的插件列表
    pub async fn get_plugins(&self, server_path: &str) -> Result<Vec<m_PluginInfo>, String> {
        listing::get_plugins(server_path).await
    }

    /// 读取插件配置文件列表
    pub fn get_plugin_config_files(
        &self,
        server_path: &str,
        plugin_name: &str,
    ) -> Result<Vec<m_PluginConfigFile>, String> {
        config_files::get_plugin_config_files(server_path, plugin_name)
    }

    /// 启用或禁用插件文件
    pub fn toggle_plugin(
        &self,
        server_path: &str,
        file_name: &str,
        enabled: bool,
    ) -> Result<(), String> {
        ops::toggle_plugin(server_path, file_name, enabled)
    }

    /// 删除插件文件
    pub fn delete_plugin(&self, server_path: &str, file_name: &str) -> Result<(), String> {
        ops::delete_plugin(server_path, file_name)
    }

    /// 安装插件文件
    pub async fn install_plugin(
        &self,
        server_path: &str,
        file_data: Vec<u8>,
        file_name: &str,
    ) -> Result<(), String> {
        ops::install_plugin(server_path, file_data, file_name).await
    }
}
