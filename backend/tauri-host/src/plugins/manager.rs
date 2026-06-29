mod assets;
mod dependency_state;
mod driver;
mod driver_builtin;
mod driver_local;
pub(crate) mod i18n;
mod install;
mod lifecycle;
mod notify;
mod resource_copy;
mod source;
mod source_builtin;
mod source_local;
mod versioning;

pub(crate) use crate::models::plugin::PluginState;
use crate::models::plugin::{
    PluginActions, PluginDistributionClass, PluginInfo, PluginInstallResult, PluginManifest,
    PluginSource,
};
use crate::plugins::api::new_api_registry;
use crate::plugins::runtime::PluginRuntime;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

/// 插件管理器
///
/// 负责插件扫描、启停、安装、资源读取和通知分发
pub struct PluginManager {
    plugins: HashMap<String, PluginInfo>,
    runtimes: Arc<RwLock<HashMap<String, PluginRuntime>>>,
    plugins_dir: PathBuf,
    data_dir: PathBuf,
    api_registry: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
    server_event_subscription_id: Option<u64>,
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
    #[allow(dead_code)]
    pub fn new(plugins_dir: PathBuf, data_dir: PathBuf) -> Self {
        Self::new_checked(plugins_dir, data_dir)
            .unwrap_or_else(|error| panic!("Failed to initialize PluginManager: {}", error))
    }

    pub fn new_checked(plugins_dir: PathBuf, data_dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&plugins_dir).map_err(|e| {
            format!("Failed to create plugins directory '{}': {}", plugins_dir.display(), e)
        })?;
        fs::create_dir_all(&data_dir).map_err(|e| {
            format!("Failed to create data directory '{}': {}", data_dir.display(), e)
        })?;

        Ok(Self {
            plugins: HashMap::new(),
            runtimes: Arc::new(RwLock::new(HashMap::new())),
            plugins_dir,
            data_dir,
            api_registry: new_api_registry(),
            server_event_subscription_id: None,
        })
    }

    pub fn reload_roots(&mut self, plugins_dir: PathBuf, data_dir: PathBuf) -> Result<(), String> {
        fs::create_dir_all(&plugins_dir).map_err(|e| {
            format!("Failed to create plugins directory '{}': {}", plugins_dir.display(), e)
        })?;
        fs::create_dir_all(&data_dir).map_err(|e| {
            format!("Failed to create data directory '{}': {}", data_dir.display(), e)
        })?;

        self.disable_all_plugins_for_shutdown();
        self.plugins.clear();
        {
            let mut runtimes = self.runtimes.write().unwrap_or_else(|e| e.into_inner());
            runtimes.clear();
        }
        self.plugins_dir = plugins_dir;
        self.data_dir = data_dir;
        Ok(())
    }

    pub(crate) fn get_shared_runtimes(&self) -> Arc<RwLock<HashMap<String, PluginRuntime>>> {
        Arc::clone(&self.runtimes)
    }

    /// 读取 API 注册表
    ///
    /// # Returns
    ///
    /// 返回当前插件系统共用的 API 注册表
    pub fn get_api_registry(&self) -> Arc<Mutex<HashMap<String, HashMap<String, String>>>> {
        Arc::clone(&self.api_registry)
    }

    /// 扫描插件目录并刷新插件列表
    ///
    /// # Returns
    ///
    /// 返回刷新后的插件列表
    pub fn scan_plugins(&mut self) -> Result<Vec<PluginInfo>, String> {
        lifecycle::scan::scan_plugins(self)
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
    pub fn enable_plugin(
        &mut self,
        plugin_id: &str,
        confirmation: Option<crate::models::plugin::PluginEnableConfirmation>,
    ) -> Result<crate::models::plugin::PluginEnableResult, String> {
        let plugin = self
            .plugins
            .get(plugin_id)
            .cloned()
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;
        self.driver_for(&plugin).enable(self, plugin_id, confirmation)
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
        let plugin = self
            .plugins
            .get(plugin_id)
            .cloned()
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;
        self.driver_for(&plugin).disable(self, plugin_id)
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
    #[allow(dead_code)]
    pub fn auto_enable_plugins(&mut self) {
        lifecycle::persistence::auto_enable_plugins(self);
    }

    pub fn auto_enable_plugins_checked(&mut self) -> Result<(), String> {
        lifecycle::persistence::auto_enable_plugins_checked(self)
    }

    /// 应用退出前停用全部插件
    pub fn disable_all_plugins_for_shutdown(&mut self) {
        lifecycle::persistence::disable_all_plugins_for_shutdown(self);
    }

    /// 读取当前插件列表
    ///
    /// # Returns
    ///
    /// 返回当前管理器中的插件信息列表
    pub fn get_plugin_list(&self) -> Vec<PluginInfo> {
        lifecycle::dependencies::get_plugin_list(self)
    }

    pub(crate) fn normalize_plugin_info(&self, plugin: PluginInfo) -> PluginInfo {
        let plugin = crate::plugins::manager::source::apply_source_capabilities(
            plugin.clone(),
            self.source_driver_for_source(plugin.source.clone())
                .capabilities(),
        );
        self.apply_runtime_capabilities(plugin)
    }

    pub(crate) fn make_local_plugin_info(
        &self,
        manifest: PluginManifest,
        state: PluginState,
        path: String,
        distribution_class: PluginDistributionClass,
        archive_sha256: Option<&str>,
        missing_dependencies: Vec<crate::models::plugin::MissingDependency>,
    ) -> PluginInfo {
        let trust_assessment =
            crate::services::plugin_trusted_catalog::assess_plugin(
                &manifest,
                distribution_class.clone(),
                archive_sha256,
            );

        self.normalize_plugin_info(PluginInfo {
            manifest,
            state,
            path,
            source: PluginSource::Local,
            runtime: crate::models::plugin::PluginRuntimeKind::Lua,
            actions: PluginActions {
                can_toggle: true,
                can_delete: true,
                can_check_update: true,
            },
            missing_dependencies,
            trust_level_display: trust_assessment.trust_level_display,
            execution_class: trust_assessment.execution_class,
            review_status: trust_assessment.review_status,
            integrity_status: trust_assessment.integrity_status,
            trusted_policy_source: trust_assessment.trusted_policy_source,
            permission_profile: trust_assessment.permission_profile,
            publisher_id: trust_assessment.publisher_id,
            distribution_class,
            trusted_catalog_matched: trust_assessment.trusted_catalog_matched,
            hash_matched: trust_assessment.hash_matched,
            verified_hash: trust_assessment.verified_hash,
            verified_signature: trust_assessment.verified_signature,
            reviewed_at: trust_assessment.reviewed_at,
            revoked: trust_assessment.revoked,
            exceeds_standard_sandbox: trust_assessment.exceeds_standard_sandbox,
            requires_explicit_consent: trust_assessment.requires_explicit_consent,
        })
    }

    pub(crate) fn apply_runtime_capabilities(&self, mut plugin: PluginInfo) -> PluginInfo {
        let capabilities = self.driver_for(&plugin).capabilities();
        plugin.actions.can_toggle = capabilities.can_toggle;
        plugin
    }

    /// 读取插件侧边栏导航项
    ///
    /// # Returns
    ///
    /// 返回前端可直接使用的导航项 JSON 列表
    pub fn get_nav_items(&self) -> Vec<serde_json::Value> {
        lifecycle::catalog::get_nav_items(self)
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
        self.install_plugin_with_metadata(
            path,
            crate::services::plugin_trusted_catalog::PluginInstallMetadata::default(),
        )
    }

    pub fn install_plugin_with_metadata(
        &mut self,
        path: &Path,
        metadata: crate::services::plugin_trusted_catalog::PluginInstallMetadata,
    ) -> Result<PluginInstallResult, String> {
        self.source_driver_for_install_path(path)?.install(self, path, &metadata)
    }

    fn get_missing_dependencies(
        &self,
        manifest: &crate::models::plugin::PluginManifest,
    ) -> Vec<crate::models::plugin::MissingDependency> {
        dependency_state::get_missing_dependencies(self, manifest)
    }

    /// 读取插件设置
    pub fn get_plugin_settings(&self, plugin_id: &str) -> Result<serde_json::Value, String> {
        let plugin = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;
        if !self.driver_for(plugin).capabilities().has_settings {
            return Ok(serde_json::json!({}));
        }
        self.driver_for(plugin).get_settings(self, plugin)
    }

    /// 写入插件设置
    pub fn set_plugin_settings(
        &self,
        plugin_id: &str,
        settings: serde_json::Value,
    ) -> Result<(), String> {
        let plugin = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;
        if !self.driver_for(plugin).capabilities().has_settings {
            return Err(format!("Plugin '{}' does not support settings", plugin_id));
        }
        self.driver_for(plugin).set_settings(self, plugin, settings)
    }

    /// 读取插件图标
    pub fn get_plugin_icon(&self, plugin_id: &str) -> Result<String, String> {
        let plugin = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;
        if !self.driver_for(plugin).capabilities().has_icon {
            return Ok(String::new());
        }
        self.driver_for(plugin).get_icon(self, plugin)
    }

    /// 读取插件样式
    pub fn get_plugin_css(&self, plugin_id: &str) -> Result<String, String> {
        let plugin = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;
        if !self.driver_for(plugin).capabilities().has_css {
            return Ok(String::new());
        }
        self.driver_for(plugin).get_css(self, plugin)
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
        let plugin = self
            .plugins
            .get(plugin_id)
            .cloned()
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;
        self.driver_for(&plugin)
            .delete(self, plugin_id, delete_data)
    }

    pub fn check_plugin_update(&self, plugin_id: &str) -> Result<Option<(String, String)>, String> {
        let plugin = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;
        Ok(self.driver_for(plugin).collect_update_version(plugin))
    }

    pub fn collect_update_versions(&self) -> Vec<(String, String)> {
        self.plugins
            .values()
            .filter_map(|plugin| self.driver_for(plugin).collect_update_version(plugin))
            .collect()
    }

    pub(crate) fn driver_for(
        &self,
        plugin: &PluginInfo,
    ) -> &'static dyn crate::plugins::manager::driver::PluginDriver {
        match crate::plugins::manager::driver::driver_kind_for(plugin) {
            crate::plugins::manager::driver::PluginDriverKind::LuaLocal => {
                &crate::plugins::manager::driver_local::LuaLocalPluginDriver
            }
            crate::plugins::manager::driver::PluginDriverKind::BuiltinRust => {
                &crate::plugins::manager::driver_builtin::BuiltinRustPluginDriver
            }
        }
    }

    pub(crate) fn source_driver_for_source(
        &self,
        source: PluginSource,
    ) -> &'static dyn crate::plugins::manager::source::PluginSourceDriver {
        match source {
            PluginSource::Local => {
                &crate::plugins::manager::source_local::LocalFilesystemPluginSourceDriver
            }
            PluginSource::Builtin => {
                &crate::plugins::manager::source_builtin::BuiltinPluginSourceDriver
            }
        }
    }

    pub(crate) fn source_driver_for_install_path(
        &self,
        path: &Path,
    ) -> Result<&'static dyn crate::plugins::manager::source::PluginSourceDriver, String> {
        let source = crate::plugins::manager::source::source_kind_for_install_path(path)
            .ok_or_else(crate::hardcode_data::plugin_manifest::unsupported_plugin_source_message)?;
        Ok(self.source_driver_for_source(source))
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

    pub fn notify_server_event(&self, event: &crate::services::events::ServerEventEnvelope) {
        notify::notify_server_event(self, event);
    }

    pub fn notify_context_menu_show(
        &self,
        context: &str,
        target_data: &serde_json::Value,
        x: f64,
        y: f64,
    ) {
        notify::notify_context_menu_show(self, context, target_data, x, y);
    }

    pub fn notify_context_menu_hide(&self) {
        notify::notify_context_menu_hide(self);
    }

    pub fn dispatch_context_menu_callback(
        &self,
        plugin_id: &str,
        context: &str,
        item_id: &str,
        target_data: serde_json::Value,
    ) -> Result<(), String> {
        let plugin = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;
        if !self.driver_for(plugin).capabilities().supports_context_menu {
            return Err(format!("Plugin '{}' does not support context menu callbacks", plugin_id));
        }
        self.driver_for(plugin).dispatch_context_menu_callback(
            self,
            plugin_id,
            context,
            item_id,
            target_data,
        )
    }

    pub fn set_server_event_subscription_id(&mut self, subscriber_id: u64) {
        self.server_event_subscription_id = Some(subscriber_id);
    }
}

#[cfg(test)]
mod tests {
    use super::PluginManager;
    use std::sync::Arc;

    #[test]
    fn new_checked_rejects_file_backed_plugins_dir() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = temp_dir.path().join("plugins-file");
        let data_dir = temp_dir.path().join("plugin-data");
        std::fs::write(&plugins_dir, b"not a directory")
            .expect("file-backed plugins dir should exist");

        let error = PluginManager::new_checked(plugins_dir, data_dir)
            .err()
            .expect("file-backed plugins dir should not be silently downgraded");

        assert!(
            error.contains("Failed to create plugins directory"),
            "unexpected error: {}",
            error
        );
        assert!(error.contains("plugins-file"), "unexpected error: {}", error);
    }

    #[test]
    fn reload_roots_preserves_shared_runtime_and_api_registry_handles() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = temp_dir.path().join("plugins-a");
        let data_dir = temp_dir.path().join("plugin-data-a");
        let mut manager = PluginManager::new_checked(plugins_dir, data_dir)
            .expect("plugin manager should initialize");

        let shared_runtimes = manager.get_shared_runtimes();
        let api_registry = manager.get_api_registry();

        manager
            .reload_roots(temp_dir.path().join("plugins-b"), temp_dir.path().join("plugin-data-b"))
            .expect("reload_roots should succeed");

        assert!(Arc::ptr_eq(&shared_runtimes, &manager.get_shared_runtimes()));
        assert!(Arc::ptr_eq(&api_registry, &manager.get_api_registry()));
    }
}
