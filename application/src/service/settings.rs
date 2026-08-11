//! 设置信息服务实现。
//!
//! 实现 [`sealantern_interface::SettingsService`] 能力端口，统一提供设置概览、
//! 读取、更新、重置与导入导出能力。
//!
//! 错误分层：内部以应用层主错误 [`SettingsError`] 为源头，暴露
//! [`SettingsService`] 时统一转为接口契约错误 [`SettingsServiceError`]。

use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use sealantern_extra::config::SettingsManager;
use sealantern_extra::models::{
    AppSettings, PartialAppSettings, UpdateResult, DEFAULT_ACRYLIC_BLUR_LEVEL,
};
use sealantern_interface::settings::{
    SettingsEntry, SettingsEntryType, SettingsGroupInfo, SettingsOption, SettingsOverview,
};
use sealantern_interface::{SettingsService, SettingsServiceError};

use crate::error::SettingsError;
use crate::service::network_settings::{
    commit_persisted_proxy, GlobalNetworkSettingsRuntime, NetworkSettingsRuntime,
    PreparedProxyUpdate,
};

/// 基于 `extra` 配置管理的设置服务实现。
pub struct CoreSettingsService {
    /// 惰性加载的唯一设置管理器；首次真实配置操作时初始化。
    manager: tokio::sync::OnceCell<tokio::sync::Mutex<SettingsManager>>,
    /// 持久化代理设置只向网络运行时初始化一次；失败后允许重试。
    network_initialized: tokio::sync::OnceCell<()>,
    /// 持久化成功但运行时提交失败时，下一次操作先修复同步。
    network_desynchronized: AtomicBool,
    network_runtime: Arc<dyn NetworkSettingsRuntime>,
}

impl CoreSettingsService {
    /// 创建使用默认配置位置的惰性设置服务。
    pub fn new() -> Self {
        Self {
            manager: tokio::sync::OnceCell::new(),
            network_initialized: tokio::sync::OnceCell::new(),
            network_desynchronized: AtomicBool::new(false),
            network_runtime: Arc::new(GlobalNetworkSettingsRuntime),
        }
    }

    /// 使用既有设置管理器构造服务，供测试和受控注入使用。
    pub fn with_manager(manager: SettingsManager) -> Self {
        Self {
            manager: tokio::sync::OnceCell::new_with(Some(tokio::sync::Mutex::new(manager))),
            network_initialized: tokio::sync::OnceCell::new(),
            network_desynchronized: AtomicBool::new(false),
            network_runtime: Arc::new(GlobalNetworkSettingsRuntime),
        }
    }

    #[cfg(test)]
    fn with_manager_and_runtime(
        manager: SettingsManager,
        network_runtime: Arc<dyn NetworkSettingsRuntime>,
    ) -> Self {
        Self {
            manager: tokio::sync::OnceCell::new_with(Some(tokio::sync::Mutex::new(manager))),
            network_initialized: tokio::sync::OnceCell::new(),
            network_desynchronized: AtomicBool::new(false),
            network_runtime,
        }
    }

    /// 获取设置管理器；并发首次调用只执行一次初始化，失败后允许重试。
    async fn manager(&self) -> Result<&tokio::sync::Mutex<SettingsManager>, SettingsError> {
        let manager = self
            .manager
            .get_or_try_init(|| async {
                SettingsManager::load_default()
                    .await
                    .map(tokio::sync::Mutex::new)
                    .map_err(SettingsError::from)
            })
            .await?;
        self.network_initialized
            .get_or_try_init(|| async {
                let proxy = manager.lock().await.get().proxy.clone();
                let prepared = self.network_runtime.prepare(proxy.clone())?;
                commit_persisted_proxy(self.network_runtime.as_ref(), prepared, proxy)?;
                Ok::<(), SettingsError>(())
            })
            .await?;
        Ok(manager)
    }

    /// 加载持久化设置并同步代理运行时，供网络消费者建立启动屏障。
    pub async fn initialize(&self) -> Result<(), SettingsServiceError> {
        self.lock_synchronized_manager()
            .await
            .map(|_| ())
            .map_err(Self::contract_error)
    }

    async fn lock_synchronized_manager(
        &self,
    ) -> Result<tokio::sync::MutexGuard<'_, SettingsManager>, SettingsError> {
        let manager = self.manager().await?;
        let manager = manager.lock().await;
        self.repair_network_if_needed(manager.get())?;
        Ok(manager)
    }

    fn repair_network_if_needed(&self, settings: &AppSettings) -> Result<(), SettingsError> {
        if !self.network_desynchronized.load(Ordering::Acquire) {
            return Ok(());
        }
        let proxy = settings.proxy.clone();
        let prepared = self.network_runtime.prepare(proxy.clone())?;
        commit_persisted_proxy(self.network_runtime.as_ref(), prepared, proxy)?;
        self.network_desynchronized.store(false, Ordering::Release);
        Ok(())
    }

    fn prepare_proxy_change(
        &self,
        current: &AppSettings,
        candidate: &AppSettings,
    ) -> Result<Option<Box<dyn PreparedProxyUpdate>>, SettingsError> {
        if current.proxy == candidate.proxy {
            return Ok(None);
        }
        self.network_runtime
            .prepare(candidate.proxy.clone())
            .map(Some)
    }

    fn commit_proxy_change(
        &self,
        prepared: Option<Box<dyn PreparedProxyUpdate>>,
        previous: &sealantern_infra::net::proxy::ProxySettings,
        persisted: &sealantern_infra::net::proxy::ProxySettings,
    ) -> Result<(), SettingsError> {
        if previous == persisted {
            return Ok(());
        }
        // 部分更新会在配置锁内重读磁盘。若其他进程刚修改了代理，本次调用即使
        // 没携带 proxy，也可能观察到真实的 Network 变更；此时应按最终持久化值
        // 重新准备，而不是把正常的跨进程合并误报为内部不变量错误。
        let prepared = match prepared {
            Some(prepared) => prepared,
            None => self.network_runtime.prepare(persisted.clone())?,
        };
        self.commit_prepared_persisted_proxy(prepared, persisted.clone())
    }

    fn commit_prepared_persisted_proxy(
        &self,
        prepared: Box<dyn PreparedProxyUpdate>,
        persisted: sealantern_infra::net::proxy::ProxySettings,
    ) -> Result<(), SettingsError> {
        self.network_desynchronized.store(true, Ordering::Release);
        let result = commit_persisted_proxy(self.network_runtime.as_ref(), prepared, persisted);
        if result.is_ok() {
            self.network_desynchronized.store(false, Ordering::Release);
        }
        result
    }

    /// 将应用层设置错误记录并收敛为宿主契约错误。
    fn contract_error(error: SettingsError) -> SettingsServiceError {
        tracing::error!(
            target: "sealantern.application.settings",
            error = %error,
            "settings operation failed"
        );
        SettingsServiceError::from(error)
    }

    /// 构造设置概览。
    fn build_overview_inner() -> SettingsOverview {
        // 构建所有设置分组及其设置项
        let groups = vec![
            build_general_group(),
            build_server_defaults_group(),
            build_console_group(),
            build_appearance_group(),
            build_window_group(),
            build_developer_group(),
        ];

        // 统计总项数和已配置项数
        let total_entries: usize = groups.iter().map(|g| g.entries.len()).sum();
        let configured_entries: usize = groups
            .iter()
            .flat_map(|g| g.entries.iter())
            .filter(|e| e.has_value)
            .count();

        SettingsOverview {
            groups,
            total_entries,
            configured_entries,
        }
    }
}

impl Default for CoreSettingsService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SettingsService for CoreSettingsService {
    async fn settings_overview(&self) -> Result<SettingsOverview, SettingsServiceError> {
        Ok(Self::build_overview_inner())
    }

    async fn get(&self) -> Result<AppSettings, SettingsServiceError> {
        let manager = self
            .lock_synchronized_manager()
            .await
            .map_err(Self::contract_error)?;
        Ok(manager.get().clone())
    }

    async fn update(&self, settings: AppSettings) -> Result<UpdateResult, SettingsServiceError> {
        settings
            .validate()
            .map_err(sealantern_extra::config::SettingsError::from)
            .map_err(SettingsError::from)
            .map_err(Self::contract_error)?;
        let mut manager = self
            .lock_synchronized_manager()
            .await
            .map_err(Self::contract_error)?;
        let previous_proxy = manager.get().proxy.clone();
        let prepared = self
            .prepare_proxy_change(manager.get(), &settings)
            .map_err(Self::contract_error)?;
        let result = manager
            .update(settings)
            .await
            .map_err(SettingsError::from)
            .map_err(Self::contract_error)?;
        self.commit_proxy_change(prepared, &previous_proxy, &result.settings.proxy)
            .map_err(Self::contract_error)?;
        Ok(result)
    }

    async fn update_partial(
        &self,
        partial: PartialAppSettings,
    ) -> Result<UpdateResult, SettingsServiceError> {
        let mut manager = self
            .lock_synchronized_manager()
            .await
            .map_err(Self::contract_error)?;
        let previous_proxy = manager.get().proxy.clone();
        let prepared = match partial.proxy.as_ref() {
            Some(proxy) => Some(
                self.network_runtime
                    .prepare(proxy.clone())
                    .map_err(Self::contract_error)?,
            ),
            _ => None,
        };
        let result = manager
            .update_partial(partial)
            .await
            .map_err(SettingsError::from)
            .map_err(Self::contract_error)?;
        self.commit_proxy_change(prepared, &previous_proxy, &result.settings.proxy)
            .map_err(Self::contract_error)?;
        Ok(result)
    }

    async fn reset(&self) -> Result<AppSettings, SettingsServiceError> {
        let mut manager = self
            .lock_synchronized_manager()
            .await
            .map_err(Self::contract_error)?;
        let default = AppSettings::default();
        let prepared = self
            .prepare_proxy_change(manager.get(), &default)
            .map_err(Self::contract_error)?;
        let previous_proxy = manager.get().proxy.clone();
        let settings = manager
            .reset()
            .await
            .map_err(SettingsError::from)
            .map_err(Self::contract_error)?;
        if settings.proxy != previous_proxy {
            let prepared = match prepared {
                Some(prepared) => prepared,
                None => self
                    .network_runtime
                    .prepare(settings.proxy.clone())
                    .map_err(Self::contract_error)?,
            };
            self.commit_prepared_persisted_proxy(prepared, settings.proxy.clone())
                .map_err(Self::contract_error)?;
        }
        Ok(settings)
    }

    async fn export_json(&self) -> Result<String, SettingsServiceError> {
        let manager = self
            .lock_synchronized_manager()
            .await
            .map_err(Self::contract_error)?;
        manager
            .export_json()
            .map_err(SettingsError::from)
            .map_err(Self::contract_error)
    }

    async fn import_json(&self, json: &str) -> Result<UpdateResult, SettingsServiceError> {
        let candidate: AppSettings = serde_json::from_str(json).map_err(|error| {
            Self::contract_error(SettingsError::InvalidInput {
                source: sealantern_extra::config::SettingsError::invalid_input(
                    "json",
                    error.to_string(),
                ),
            })
        })?;
        candidate
            .validate()
            .map_err(sealantern_extra::config::SettingsError::from)
            .map_err(SettingsError::from)
            .map_err(Self::contract_error)?;
        let mut manager = self
            .lock_synchronized_manager()
            .await
            .map_err(Self::contract_error)?;
        let previous_proxy = manager.get().proxy.clone();
        let prepared = self
            .prepare_proxy_change(manager.get(), &candidate)
            .map_err(Self::contract_error)?;
        let result = manager
            .import_json(json)
            .await
            .map_err(SettingsError::from)
            .map_err(Self::contract_error)?;
        self.commit_proxy_change(prepared, &previous_proxy, &result.settings.proxy)
            .map_err(Self::contract_error)?;
        Ok(result)
    }
}

/// 构建常规设置分组。
fn build_general_group() -> SettingsGroupInfo {
    SettingsGroupInfo {
        id: "General".to_string(),
        display_name: "settings.group_general".to_string(),
        description: "settings.group_general_desc".to_string(),
        entries: vec![
            SettingsEntry {
                id: "close_servers_on_exit".to_string(),
                display_name: "settings.close_servers_on_exit".to_string(),
                description: "settings.close_servers_on_exit_desc".to_string(),
                entry_type: SettingsEntryType::Boolean,
                required: false,
                has_value: true,
                default_value: Some("true".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "close_servers_on_update".to_string(),
                display_name: "settings.close_servers_on_update".to_string(),
                description: "settings.close_servers_on_update_desc".to_string(),
                entry_type: SettingsEntryType::Boolean,
                required: false,
                has_value: true,
                default_value: Some("true".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "auto_accept_eula".to_string(),
                display_name: "settings.auto_accept_eula".to_string(),
                description: "settings.auto_accept_eula_desc".to_string(),
                entry_type: SettingsEntryType::Boolean,
                required: false,
                has_value: true,
                default_value: Some("true".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "close_action".to_string(),
                display_name: "settings.close_action".to_string(),
                description: "settings.close_action_desc".to_string(),
                entry_type: SettingsEntryType::Enum,
                required: false,
                has_value: true,
                default_value: Some("\"ask\"".to_string()),
                options: vec![
                    SettingsOption {
                        value: "ask".to_string(),
                        display_name: "settings.close_action_ask".to_string(),
                    },
                    SettingsOption {
                        value: "exit".to_string(),
                        display_name: "settings.close_action_exit".to_string(),
                    },
                    SettingsOption {
                        value: "tray".to_string(),
                        display_name: "settings.close_action_tray".to_string(),
                    },
                ],
            },
        ],
    }
}

/// 构建服务器默认设置分组。
fn build_server_defaults_group() -> SettingsGroupInfo {
    SettingsGroupInfo {
        id: "ServerDefaults".to_string(),
        display_name: "settings.group_server_defaults".to_string(),
        description: "settings.group_server_defaults_desc".to_string(),
        entries: vec![
            SettingsEntry {
                id: "default_max_memory".to_string(),
                display_name: "settings.default_max_memory".to_string(),
                description: "settings.default_max_memory_desc".to_string(),
                entry_type: SettingsEntryType::Integer,
                required: false,
                has_value: true,
                default_value: Some("2048".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "default_min_memory".to_string(),
                display_name: "settings.default_min_memory".to_string(),
                description: "settings.default_min_memory_desc".to_string(),
                entry_type: SettingsEntryType::Integer,
                required: false,
                has_value: true,
                default_value: Some("512".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "default_port".to_string(),
                display_name: "settings.default_port".to_string(),
                description: "settings.default_port_desc".to_string(),
                entry_type: SettingsEntryType::Integer,
                required: false,
                has_value: true,
                default_value: Some("25565".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "default_java_path".to_string(),
                display_name: "settings.default_java_path".to_string(),
                description: "settings.default_java_path_desc".to_string(),
                entry_type: SettingsEntryType::Path,
                required: false,
                has_value: false,
                default_value: None,
                options: vec![],
            },
            SettingsEntry {
                id: "default_jvm_args".to_string(),
                display_name: "settings.default_jvm_args".to_string(),
                description: "settings.default_jvm_args_desc".to_string(),
                entry_type: SettingsEntryType::Text,
                required: false,
                has_value: false,
                default_value: None,
                options: vec![],
            },
        ],
    }
}

/// 构建控制台设置分组。
fn build_console_group() -> SettingsGroupInfo {
    SettingsGroupInfo {
        id: "Console".to_string(),
        display_name: "settings.group_console".to_string(),
        description: "settings.group_console_desc".to_string(),
        entries: vec![
            SettingsEntry {
                id: "console_font_size".to_string(),
                display_name: "settings.console_font_size".to_string(),
                description: "settings.console_font_size_desc".to_string(),
                entry_type: SettingsEntryType::Integer,
                required: false,
                has_value: true,
                default_value: Some("13".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "console_font_family".to_string(),
                display_name: "settings.console_font_family".to_string(),
                description: "settings.console_font_family_desc".to_string(),
                entry_type: SettingsEntryType::String,
                required: false,
                has_value: false,
                default_value: None,
                options: vec![],
            },
            SettingsEntry {
                id: "console_letter_spacing".to_string(),
                display_name: "settings.console_letter_spacing".to_string(),
                description: "settings.console_letter_spacing_desc".to_string(),
                entry_type: SettingsEntryType::Integer,
                required: false,
                has_value: true,
                default_value: Some("0".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "max_log_lines".to_string(),
                display_name: "settings.max_log_lines".to_string(),
                description: "settings.max_log_lines_desc".to_string(),
                entry_type: SettingsEntryType::Integer,
                required: false,
                has_value: true,
                default_value: Some("5000".to_string()),
                options: vec![],
            },
        ],
    }
}

/// 构建外观设置分组。
fn build_appearance_group() -> SettingsGroupInfo {
    SettingsGroupInfo {
        id: "Appearance".to_string(),
        display_name: "settings.group_appearance".to_string(),
        description: "settings.group_appearance_desc".to_string(),
        entries: vec![
            SettingsEntry {
                id: "theme".to_string(),
                display_name: "settings.theme".to_string(),
                description: "settings.theme_desc".to_string(),
                entry_type: SettingsEntryType::Enum,
                required: false,
                has_value: true,
                default_value: Some("\"auto\"".to_string()),
                options: vec![
                    SettingsOption {
                        value: "auto".to_string(),
                        display_name: "settings.theme_auto".to_string(),
                    },
                    SettingsOption {
                        value: "light".to_string(),
                        display_name: "settings.theme_light".to_string(),
                    },
                    SettingsOption {
                        value: "dark".to_string(),
                        display_name: "settings.theme_dark".to_string(),
                    },
                ],
            },
            SettingsEntry {
                id: "color".to_string(),
                display_name: "settings.color".to_string(),
                description: "settings.color_desc".to_string(),
                entry_type: SettingsEntryType::Enum,
                required: false,
                has_value: true,
                default_value: Some("\"default\"".to_string()),
                options: vec![SettingsOption {
                    value: "default".to_string(),
                    display_name: "settings.color_default".to_string(),
                }],
            },
            SettingsEntry {
                id: "font_size".to_string(),
                display_name: "settings.font_size".to_string(),
                description: "settings.font_size_desc".to_string(),
                entry_type: SettingsEntryType::Integer,
                required: false,
                has_value: true,
                default_value: Some("14".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "font_family".to_string(),
                display_name: "settings.font_family".to_string(),
                description: "settings.font_family_desc".to_string(),
                entry_type: SettingsEntryType::String,
                required: false,
                has_value: false,
                default_value: None,
                options: vec![],
            },
            SettingsEntry {
                id: "background_image".to_string(),
                display_name: "settings.background_image".to_string(),
                description: "settings.background_image_desc".to_string(),
                entry_type: SettingsEntryType::Path,
                required: false,
                has_value: false,
                default_value: None,
                options: vec![],
            },
            SettingsEntry {
                id: "background_opacity".to_string(),
                display_name: "settings.background_opacity".to_string(),
                description: "settings.background_opacity_desc".to_string(),
                entry_type: SettingsEntryType::Float,
                required: false,
                has_value: true,
                default_value: Some("0.3".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "background_blur".to_string(),
                display_name: "settings.background_blur".to_string(),
                description: "settings.background_blur_desc".to_string(),
                entry_type: SettingsEntryType::Integer,
                required: false,
                has_value: true,
                default_value: Some("0".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "acrylic_enabled".to_string(),
                display_name: "settings.advanced_material".to_string(),
                description: "settings.advanced_material_desc".to_string(),
                entry_type: SettingsEntryType::Boolean,
                required: false,
                has_value: true,
                default_value: Some("false".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "acrylic_blur_level".to_string(),
                display_name: "settings.acrylic_blur".to_string(),
                description: "settings.acrylic_blur_desc".to_string(),
                entry_type: SettingsEntryType::Enum,
                required: false,
                has_value: true,
                default_value: Some(format!("\"{DEFAULT_ACRYLIC_BLUR_LEVEL}\"")),
                options: vec![
                    SettingsOption {
                        value: "off".to_string(),
                        display_name: "settings.acrylic_blur_options.off".to_string(),
                    },
                    SettingsOption {
                        value: "low".to_string(),
                        display_name: "settings.acrylic_blur_options.low".to_string(),
                    },
                    SettingsOption {
                        value: "medium".to_string(),
                        display_name: "settings.acrylic_blur_options.medium".to_string(),
                    },
                    SettingsOption {
                        value: "high".to_string(),
                        display_name: "settings.acrylic_blur_options.high".to_string(),
                    },
                ],
            },
            SettingsEntry {
                id: "minimal_mode".to_string(),
                display_name: "settings.minimal_mode".to_string(),
                description: "settings.minimal_mode_desc".to_string(),
                entry_type: SettingsEntryType::Boolean,
                required: false,
                has_value: true,
                default_value: Some("false".to_string()),
                options: vec![],
            },
        ],
    }
}

/// 构建窗口设置分组。
fn build_window_group() -> SettingsGroupInfo {
    SettingsGroupInfo {
        id: "Window".to_string(),
        display_name: "settings.group_window".to_string(),
        description: "settings.group_window_desc".to_string(),
        entries: vec![
            SettingsEntry {
                id: "window_width".to_string(),
                display_name: "settings.window_width".to_string(),
                description: "settings.window_width_desc".to_string(),
                entry_type: SettingsEntryType::Integer,
                required: false,
                has_value: true,
                default_value: Some("1200".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "window_height".to_string(),
                display_name: "settings.window_height".to_string(),
                description: "settings.window_height_desc".to_string(),
                entry_type: SettingsEntryType::Integer,
                required: false,
                has_value: true,
                default_value: Some("720".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "window_maximized".to_string(),
                display_name: "settings.window_maximized".to_string(),
                description: "settings.window_maximized_desc".to_string(),
                entry_type: SettingsEntryType::Boolean,
                required: false,
                has_value: true,
                default_value: Some("false".to_string()),
                options: vec![],
            },
        ],
    }
}

/// 构建开发者设置分组。
fn build_developer_group() -> SettingsGroupInfo {
    SettingsGroupInfo {
        id: "Developer".to_string(),
        display_name: "settings.group_developer".to_string(),
        description: "settings.group_developer_desc".to_string(),
        entries: vec![
            SettingsEntry {
                id: "language".to_string(),
                display_name: "settings.language".to_string(),
                description: "settings.language_desc".to_string(),
                entry_type: SettingsEntryType::Enum,
                required: false,
                has_value: true,
                default_value: Some("\"zh-CN\"".to_string()),
                options: vec![
                    SettingsOption {
                        value: "zh-CN".to_string(),
                        display_name: "settings.language_zh_cn".to_string(),
                    },
                    SettingsOption {
                        value: "en-US".to_string(),
                        display_name: "settings.language_en_us".to_string(),
                    },
                ],
            },
            SettingsEntry {
                id: "developer_mode".to_string(),
                display_name: "settings.developer_mode".to_string(),
                description: "settings.developer_mode_desc".to_string(),
                entry_type: SettingsEntryType::Boolean,
                required: false,
                has_value: true,
                default_value: Some("false".to_string()),
                options: vec![],
            },
            SettingsEntry {
                id: "last_run_path".to_string(),
                display_name: "settings.last_run_path".to_string(),
                description: "settings.last_run_path_desc".to_string(),
                entry_type: SettingsEntryType::Path,
                required: false,
                has_value: false,
                default_value: None,
                options: vec![],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex as StdMutex;

    use super::*;
    use sealantern_extra::models::SettingsGroup;
    use sealantern_infra::net::proxy::{ProxyMode, ProxySettings};
    use sealantern_infra::net::NetworkCommitError;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum NetworkEvent {
        Prepare(ProxySettings),
        Commit,
    }

    struct RecordingPrepared {
        events: Arc<StdMutex<Vec<NetworkEvent>>>,
        outcome: Result<(), NetworkCommitError>,
    }

    impl PreparedProxyUpdate for RecordingPrepared {
        fn commit(self: Box<Self>) -> Result<(), NetworkCommitError> {
            self.events
                .lock()
                .expect("测试事件锁不应污染")
                .push(NetworkEvent::Commit);
            self.outcome
        }
    }

    struct RecordingRuntime {
        events: Arc<StdMutex<Vec<NetworkEvent>>>,
        prepare_results: StdMutex<VecDeque<Result<(), &'static str>>>,
        commit_results: StdMutex<VecDeque<Result<(), NetworkCommitError>>>,
    }

    impl RecordingRuntime {
        fn successful() -> (Arc<Self>, Arc<StdMutex<Vec<NetworkEvent>>>) {
            let events = Arc::new(StdMutex::new(Vec::new()));
            (
                Arc::new(Self {
                    events: events.clone(),
                    prepare_results: StdMutex::new(VecDeque::new()),
                    commit_results: StdMutex::new(VecDeque::new()),
                }),
                events,
            )
        }

        fn fail_next_prepare(&self) {
            self.prepare_results
                .lock()
                .expect("测试准备结果锁不应污染")
                .push_back(Err("synthetic prepare failure"));
        }

        fn push_commit_result(&self, result: Result<(), NetworkCommitError>) {
            self.commit_results
                .lock()
                .expect("测试提交结果锁不应污染")
                .push_back(result);
        }
    }

    impl NetworkSettingsRuntime for RecordingRuntime {
        fn prepare(
            &self,
            settings: ProxySettings,
        ) -> Result<Box<dyn PreparedProxyUpdate>, SettingsError> {
            self.events
                .lock()
                .expect("测试事件锁不应污染")
                .push(NetworkEvent::Prepare(settings));
            if let Some(Err(message)) = self
                .prepare_results
                .lock()
                .expect("测试准备结果锁不应污染")
                .pop_front()
            {
                return Err(SettingsError::OperationFailed {
                    source: Box::new(std::io::Error::other(message)),
                });
            }
            Ok(Box::new(RecordingPrepared {
                events: self.events.clone(),
                outcome: self
                    .commit_results
                    .lock()
                    .expect("测试提交结果锁不应污染")
                    .pop_front()
                    .unwrap_or(Ok(())),
            }))
        }
    }

    async fn recording_service(
        path: &std::path::Path,
    ) -> (CoreSettingsService, Arc<RecordingRuntime>, Arc<StdMutex<Vec<NetworkEvent>>>) {
        let manager = SettingsManager::load(path).await.expect("设置管理器应加载");
        let (runtime, events) = RecordingRuntime::successful();
        let service = CoreSettingsService::with_manager_and_runtime(manager, runtime.clone());
        service.get().await.expect("首次代理同步应成功");
        events.lock().expect("测试事件锁不应污染").clear();
        (service, runtime, events)
    }

    #[tokio::test]
    async fn overview_does_not_initialize_settings_manager() {
        let service = CoreSettingsService::new();
        assert!(service.manager.get().is_none());

        let overview = service
            .settings_overview()
            .await
            .expect("settings overview should be available without storage");

        assert!(!overview.groups.is_empty());
        assert!(service.manager.get().is_none());
    }

    #[tokio::test]
    async fn manages_settings_through_the_service_contract() {
        let root = tempfile::tempdir().expect("temporary settings directory should be created");
        let path = root.path().join("settings.json");
        let manager = SettingsManager::load(&path)
            .await
            .expect("settings manager should load");
        let service = CoreSettingsService::with_manager_and_runtime(
            manager,
            Arc::new(crate::service::network_settings::NoopNetworkSettingsRuntime),
        );

        let initial = service
            .get()
            .await
            .expect("default settings should be returned");
        assert_eq!(initial.theme, "auto");
        assert_eq!(initial.language, "zh-CN");

        let mut replacement = initial;
        replacement.theme = "dark".to_string();
        replacement.language = "en-US".to_string();
        let updated = service
            .update(replacement)
            .await
            .expect("full settings update should succeed");
        assert!(updated.changed_groups.contains(&SettingsGroup::Appearance));
        assert!(updated.changed_groups.contains(&SettingsGroup::Developer));

        let partial = PartialAppSettings {
            default_port: Some(25566),
            ..PartialAppSettings::default()
        };
        let partially_updated = service
            .update_partial(partial)
            .await
            .expect("partial settings update should succeed");
        assert_eq!(partially_updated.settings.default_port, 25566);
        assert_eq!(partially_updated.settings.theme, "dark");
        assert!(partially_updated
            .changed_groups
            .contains(&SettingsGroup::ServerDefaults));

        let exported = service
            .export_json()
            .await
            .expect("settings should export as JSON");
        let reset = service
            .reset()
            .await
            .expect("settings reset should succeed");
        assert_eq!(reset.theme, AppSettings::default().theme);
        assert_eq!(reset.default_port, AppSettings::default().default_port);

        let imported = service
            .import_json(&exported)
            .await
            .expect("exported settings should import");
        assert_eq!(imported.settings.theme, "dark");
        assert_eq!(imported.settings.language, "en-US");
        assert_eq!(imported.settings.default_port, 25566);

        let reloaded = SettingsManager::load(&path)
            .await
            .expect("persisted settings should reload");
        assert_eq!(reloaded.get().theme, "dark");
        assert_eq!(reloaded.get().default_port, 25566);
    }

    #[tokio::test]
    async fn proxy_update_prepares_before_persisting_and_commits_after_success() {
        let root = tempfile::tempdir().expect("临时目录应创建");
        let path = root.path().join("settings.json");
        let (service, _runtime, events) = recording_service(&path).await;
        let mut settings = service.get().await.expect("设置应可读取");
        settings.proxy = ProxySettings { mode: ProxyMode::Disabled };

        let result = service.update(settings).await.expect("代理设置更新应成功");

        assert!(result.changed_groups.contains(&SettingsGroup::Network));
        assert_eq!(
            *events.lock().expect("测试事件锁不应污染"),
            vec![
                NetworkEvent::Prepare(ProxySettings { mode: ProxyMode::Disabled }),
                NetworkEvent::Commit,
            ]
        );
        let persisted = SettingsManager::load(&path)
            .await
            .expect("持久化设置应可重载");
        assert_eq!(persisted.get().proxy.mode, ProxyMode::Disabled);
    }

    #[tokio::test]
    async fn proxy_prepare_failure_does_not_persist_settings() {
        let root = tempfile::tempdir().expect("临时目录应创建");
        let path = root.path().join("settings.json");
        let (service, runtime, events) = recording_service(&path).await;
        runtime.fail_next_prepare();
        let mut settings = service.get().await.expect("设置应可读取");
        settings.proxy = ProxySettings { mode: ProxyMode::Disabled };

        assert!(matches!(
            service.update(settings).await,
            Err(SettingsServiceError::OperationFailed)
        ));

        assert_eq!(events.lock().expect("测试事件锁不应污染").len(), 1);
        let persisted = SettingsManager::load(&path).await.expect("原设置应可重载");
        assert_eq!(persisted.get().proxy, ProxySettings::default());
    }

    #[tokio::test]
    async fn persistence_failure_does_not_commit_prepared_proxy() {
        let root = tempfile::tempdir().expect("临时目录应创建");
        let path = root.path().join("settings.json");
        let (service, _runtime, events) = recording_service(&path).await;
        std::fs::remove_file(&path).expect("测试设置文件应可删除");
        std::fs::create_dir(&path).expect("测试设置路径应可替换为目录");
        let mut settings = service.get().await.expect("内存设置应可读取");
        settings.proxy = ProxySettings { mode: ProxyMode::Disabled };

        assert!(matches!(
            service.update(settings).await,
            Err(SettingsServiceError::StorageFailed)
        ));

        assert_eq!(
            *events.lock().expect("测试事件锁不应污染"),
            vec![NetworkEvent::Prepare(ProxySettings { mode: ProxyMode::Disabled })]
        );
    }

    #[tokio::test]
    async fn partial_reset_and_import_synchronize_proxy_changes() {
        let root = tempfile::tempdir().expect("临时目录应创建");
        let path = root.path().join("settings.json");
        let (service, _runtime, events) = recording_service(&path).await;

        service
            .update_partial(PartialAppSettings {
                proxy: Some(ProxySettings { mode: ProxyMode::Disabled }),
                ..PartialAppSettings::default()
            })
            .await
            .expect("部分代理更新应成功");
        assert_eq!(events.lock().expect("测试事件锁不应污染").len(), 2);

        events.lock().expect("测试事件锁不应污染").clear();
        service.reset().await.expect("重置应成功");
        assert_eq!(events.lock().expect("测试事件锁不应污染").len(), 2);

        events.lock().expect("测试事件锁不应污染").clear();
        let mut imported = service.get().await.expect("设置应可读取");
        imported.proxy = ProxySettings { mode: ProxyMode::Disabled };
        let json = serde_json::to_string(&imported).expect("测试设置应可序列化");
        service.import_json(&json).await.expect("导入应成功");
        assert_eq!(events.lock().expect("测试事件锁不应污染").len(), 2);
    }

    #[tokio::test]
    async fn network_settings_initialize_only_once_for_all_settings_operations() {
        let root = tempfile::tempdir().expect("临时目录应创建");
        let path = root.path().join("settings.json");
        let manager = SettingsManager::load(&path)
            .await
            .expect("设置管理器应加载");
        let (runtime, events) = RecordingRuntime::successful();
        let service = CoreSettingsService::with_manager_and_runtime(manager, runtime);

        service.initialize().await.expect("显式初始化应成功");
        service.get().await.expect("后续读取应成功");
        service.initialize().await.expect("重复初始化应成功");

        assert_eq!(
            *events.lock().expect("测试事件锁不应污染"),
            vec![NetworkEvent::Prepare(ProxySettings::default()), NetworkEvent::Commit,]
        );
    }

    #[tokio::test]
    async fn later_operation_repairs_runtime_after_persisted_commit_conflicts() {
        let root = tempfile::tempdir().expect("临时目录应创建");
        let path = root.path().join("settings.json");
        let (service, runtime, events) = recording_service(&path).await;
        for revision in 1..=3 {
            runtime.push_commit_result(Err(NetworkCommitError::Conflict {
                expected_revision: revision,
                actual_revision: revision + 1,
            }));
        }
        let mut settings = service.get().await.expect("设置应可读取");
        settings.proxy = ProxySettings { mode: ProxyMode::Disabled };

        assert!(matches!(service.update(settings).await, Err(SettingsServiceError::Unavailable)));
        let persisted = SettingsManager::load(&path)
            .await
            .expect("已持久化设置应可重载");
        assert_eq!(persisted.get().proxy.mode, ProxyMode::Disabled);

        let repaired = service.get().await.expect("后续读取应先修复运行时同步");
        assert_eq!(repaired.proxy.mode, ProxyMode::Disabled);
        assert!(!service.network_desynchronized.load(Ordering::Acquire));
        assert_eq!(
            events
                .lock()
                .expect("测试事件锁不应污染")
                .iter()
                .filter(|event| matches!(event, NetworkEvent::Commit))
                .count(),
            4
        );
    }

    #[tokio::test]
    async fn non_network_partial_update_adopts_and_applies_external_proxy_change() {
        let root = tempfile::tempdir().expect("临时目录应创建");
        let path = root.path().join("settings.json");
        let (service, _runtime, events) = recording_service(&path).await;
        let mut external = SettingsManager::load(&path)
            .await
            .expect("外部设置管理器应加载");
        external
            .update_partial(PartialAppSettings {
                proxy: Some(ProxySettings { mode: ProxyMode::Disabled }),
                ..PartialAppSettings::default()
            })
            .await
            .expect("外部代理更新应持久化");

        let result = service
            .update_partial(PartialAppSettings {
                default_port: Some(25566),
                ..PartialAppSettings::default()
            })
            .await
            .expect("非网络部分更新应合并外部代理并同步运行时");

        assert_eq!(result.settings.proxy.mode, ProxyMode::Disabled);
        assert_eq!(result.settings.default_port, 25566);
        assert_eq!(
            *events.lock().expect("测试事件锁不应污染"),
            vec![
                NetworkEvent::Prepare(ProxySettings { mode: ProxyMode::Disabled }),
                NetworkEvent::Commit,
            ]
        );
    }
}
