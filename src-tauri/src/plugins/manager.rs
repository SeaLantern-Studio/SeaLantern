mod lifecycle;
mod install;
mod assets;
mod notify;
mod dependency_state;
mod resource_copy;
mod runtime_state;
mod versioning;

use crate::models::plugin::{PluginInfo, PluginInstallResult};
use crate::plugins::api::{new_api_registry, ApiRegistry};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
pub(crate) use runtime_state::SharedRuntimes;
use runtime_state::new_shared_runtimes;
pub(crate) use crate::models::plugin::PluginState;

/// 插件管理器
///
/// 负责插件扫描、启停、安装、资源读取和通知分发
pub struct PluginManager {
    plugins: HashMap<String, PluginInfo>,
    runtimes: SharedRuntimes,
    plugins_dir: PathBuf,
    data_dir: PathBuf,
    api_registry: ApiRegistry,
}

impl PluginManager {
    /// 创建插件管理器
    ///
    /// # Parameters
    ///
    /// - `plugins_dir`: 插件目录
    /// - `data_dir`: 插件数据目录
    ///
    /// # Returns
    ///
    /// 返回新的插件管理器实例
    pub fn new(plugins_dir: PathBuf, data_dir: PathBuf) -> Self {
        if let Err(e) = fs::create_dir_all(&plugins_dir) {
            eprintln!("[ERROR] Failed to create plugins directory: {}", e);
        }
        if let Err(e) = fs::create_dir_all(&data_dir) {
            eprintln!("[ERROR] Failed to create data directory: {}", e);
        }

        Self {
            plugins: HashMap::new(),
            runtimes: new_shared_runtimes(),
            plugins_dir,
            data_dir,
            api_registry: new_api_registry(),
        }
    }

    pub(crate) fn get_shared_runtimes(&self) -> SharedRuntimes {
        Arc::clone(&self.runtimes)
    }

    /// 读取 API 注册表
    ///
    /// # Returns
    ///
    /// 返回当前插件系统共用的 API 注册表
    pub fn get_api_registry(&self) -> ApiRegistry {
        Arc::clone(&self.api_registry)
    }

    /// 扫描插件目录并刷新插件列表
    ///
    /// # Returns
    ///
    /// 返回刷新后的插件列表
    pub fn scan_plugins(&mut self) -> Result<Vec<PluginInfo>, String> {
        lifecycle::scan_plugins(self)
    }

    /// 启用一个插件
    ///
    /// # Parameters
    ///
    /// - `plugin_id`: 插件 ID
    ///
    /// # Returns
    ///
    /// 启用成功时返回 `Ok(())`
    pub fn enable_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        lifecycle::enable_plugin(self, plugin_id)
    }

    /// 禁用一个插件
    ///
    /// # Parameters
    ///
    /// - `plugin_id`: 插件 ID
    ///
    /// # Returns
    ///
    /// 返回本次连带被禁用的插件 ID 列表
    pub fn disable_plugin(&mut self, plugin_id: &str) -> Result<Vec<String>, String> {
        lifecycle::disable_plugin(self, plugin_id)
    }

    fn copy_included_resources(
        plugin_dir: &Path,
        data_dir: &Path,
        includes: &[String],
    ) -> Result<(), String> {
        resource_copy::copy_included_resources(plugin_dir, data_dir, includes)
    }

    fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
        resource_copy::copy_dir_recursive(src, dest)
    }

    /// 按保存记录自动启用插件
    pub fn auto_enable_plugins(&mut self) {
        lifecycle::auto_enable_plugins(self);
    }

    /// 应用退出前停用全部插件
    pub fn disable_all_plugins_for_shutdown(&mut self) {
        lifecycle::disable_all_plugins_for_shutdown(self);
    }

    /// 读取当前插件列表
    ///
    /// # Returns
    ///
    /// 返回当前管理器中的插件信息列表
    pub fn get_plugin_list(&self) -> Vec<PluginInfo> {
        lifecycle::get_plugin_list(self)
    }

    /// 读取插件侧边栏导航项
    ///
    /// # Returns
    ///
    /// 返回前端可直接使用的导航项 JSON 列表
    pub fn get_nav_items(&self) -> Vec<serde_json::Value> {
        lifecycle::get_nav_items(self)
    }

    /// 从文件或压缩包安装插件
    ///
    /// # Parameters
    ///
    /// - `path`: 插件来源路径
    ///
    /// # Returns
    ///
    /// 返回安装结果和缺失依赖信息
    pub fn install_plugin(&mut self, path: &Path) -> Result<PluginInstallResult, String> {
        install::install_plugin(self, path)
    }

    fn get_missing_dependencies(
        &self,
        manifest: &crate::models::plugin::PluginManifest,
    ) -> Vec<crate::models::plugin::MissingDependency> {
        dependency_state::get_missing_dependencies(self, manifest)
    }

    /// 从目录安装插件
    ///
    /// # Parameters
    ///
    /// - `source_dir`: 插件目录
    ///
    /// # Returns
    ///
    /// 返回安装后的插件信息
    pub fn install_plugin_from_dir(&mut self, source_dir: &Path) -> Result<PluginInfo, String> {
        install::install_plugin_from_dir(self, source_dir)
    }

    /// 读取插件设置
    pub fn get_plugin_settings(&self, plugin_id: &str) -> Result<serde_json::Value, String> {
        assets::get_plugin_settings(self, plugin_id)
    }

    /// 写入插件设置
    pub fn set_plugin_settings(
        &self,
        plugin_id: &str,
        settings: serde_json::Value,
    ) -> Result<(), String> {
        assets::set_plugin_settings(self, plugin_id, settings)
    }

    /// 读取插件图标
    pub fn get_plugin_icon(&self, plugin_id: &str) -> Result<String, String> {
        assets::get_plugin_icon(self, plugin_id)
    }

    /// 读取插件样式
    pub fn get_plugin_css(&self, plugin_id: &str) -> Result<String, String> {
        assets::get_plugin_css(self, plugin_id)
    }

    /// 读取全部启用插件样式
    pub fn get_all_plugin_css(&self) -> Result<Vec<(String, String)>, String> {
        assets::get_all_plugin_css(self)
    }

    /// 删除插件
    ///
    /// # Parameters
    ///
    /// - `plugin_id`: 插件 ID
    /// - `delete_data`: 是否同时删除插件数据目录
    ///
    /// # Returns
    ///
    /// 删除成功时返回 `Ok(())`
    pub fn delete_plugin(&mut self, plugin_id: &str, delete_data: bool) -> Result<(), String> {
        install::delete_plugin(self, plugin_id, delete_data)
    }

    /// 判断远端版本是否比本地版本新
    ///
    /// # Parameters
    ///
    /// - `remote`: 远端版本
    /// - `local`: 本地版本
    ///
    /// # Returns
    ///
    /// 远端更新时返回 `true`
    pub fn is_newer_version(remote: &str, local: &str) -> bool {
        versioning::is_newer_version(remote, local)
    }

    /// 读取内部插件映射
    ///
    /// # Returns
    ///
    /// 返回插件 ID 到插件信息的只读映射
    pub fn plugins(&self) -> &HashMap<String, PluginInfo> {
        &self.plugins
    }

    /// 通知插件当前页面已切换
    pub fn notify_page_changed(&self, path: &str) {
        notify::notify_page_changed(self, path);
    }

    /// 通知插件当前语言已切换
    pub fn notify_locale_changed(&self, locale: &str) {
        notify::notify_locale_changed(self, locale);
    }
}
