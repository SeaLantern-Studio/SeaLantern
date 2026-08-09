//! 设置信息服务实现。
//!
//! 实现 [`sealantern_interface::SettingsService`] 能力端口，统一提供设置概览、
//! 读取、更新、重置与导入导出能力。
//!
//! 错误分层：内部以应用层主错误 [`SettingsError`] 为源头，暴露
//! [`SettingsService`] 时统一转为接口契约错误 [`SettingsServiceError`]。

use async_trait::async_trait;
use sealantern_extra::config::SettingsManager;
use sealantern_extra::models::{
    AppSettings, PartialAppSettings, UpdateResult, DEFAULT_ACRYLIC_BLUR_LEVEL,
};
use sealantern_interface::settings::{
    SettingsEntry, SettingsEntryType, SettingsGroupInfo, SettingsOption, SettingsOverview,
};
use sealantern_interface::{SettingsService, SettingsServiceError};

use crate::error::SettingsError;

/// 基于 `extra` 配置管理的设置服务实现。
pub struct CoreSettingsService {
    /// 惰性加载的唯一设置管理器；首次真实配置操作时初始化。
    manager: tokio::sync::OnceCell<tokio::sync::Mutex<SettingsManager>>,
}

impl CoreSettingsService {
    /// 创建使用默认配置位置的惰性设置服务。
    pub fn new() -> Self {
        Self { manager: tokio::sync::OnceCell::new() }
    }

    /// 使用既有设置管理器构造服务，供测试和受控注入使用。
    pub fn with_manager(manager: SettingsManager) -> Self {
        Self {
            manager: tokio::sync::OnceCell::new_with(Some(tokio::sync::Mutex::new(manager))),
        }
    }

    /// 获取设置管理器；并发首次调用只执行一次初始化，失败后允许重试。
    async fn manager(&self) -> Result<&tokio::sync::Mutex<SettingsManager>, SettingsError> {
        self.manager
            .get_or_try_init(|| async {
                SettingsManager::load_default()
                    .await
                    .map(tokio::sync::Mutex::new)
                    .map_err(SettingsError::from)
            })
            .await
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
        let manager = self.manager().await.map_err(Self::contract_error)?;
        let manager = manager.lock().await;
        Ok(manager.get().clone())
    }

    async fn update(&self, settings: AppSettings) -> Result<UpdateResult, SettingsServiceError> {
        let manager = self.manager().await.map_err(Self::contract_error)?;
        manager
            .lock()
            .await
            .update(settings)
            .await
            .map_err(SettingsError::from)
            .map_err(Self::contract_error)
    }

    async fn update_partial(
        &self,
        partial: PartialAppSettings,
    ) -> Result<UpdateResult, SettingsServiceError> {
        let manager = self.manager().await.map_err(Self::contract_error)?;
        manager
            .lock()
            .await
            .update_partial(partial)
            .await
            .map_err(SettingsError::from)
            .map_err(Self::contract_error)
    }

    async fn reset(&self) -> Result<AppSettings, SettingsServiceError> {
        let manager = self.manager().await.map_err(Self::contract_error)?;
        manager
            .lock()
            .await
            .reset()
            .await
            .map_err(SettingsError::from)
            .map_err(Self::contract_error)
    }

    async fn export_json(&self) -> Result<String, SettingsServiceError> {
        let manager = self.manager().await.map_err(Self::contract_error)?;
        manager
            .lock()
            .await
            .export_json()
            .map_err(SettingsError::from)
            .map_err(Self::contract_error)
    }

    async fn import_json(&self, json: &str) -> Result<UpdateResult, SettingsServiceError> {
        let manager = self.manager().await.map_err(Self::contract_error)?;
        manager
            .lock()
            .await
            .import_json(json)
            .await
            .map_err(SettingsError::from)
            .map_err(Self::contract_error)
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
    use super::*;
    use sealantern_extra::models::SettingsGroup;

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
        let service = CoreSettingsService::with_manager(manager);

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
}
