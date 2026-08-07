//! 设置信息服务实现。
//!
//! 实现 [`sealantern_interface::SettingsService`] 能力端口，提供设置分组、
//! 设置项列表等查询能力。
//!
//! 错误分层：内部以应用层主错误 [`SettingsError`] 为源头，暴露
//! [`SettingsService`] 时统一转为接口契约错误 [`SettingsServiceError`]。

use async_trait::async_trait;
use sealantern_interface::settings::{
    SettingsEntry, SettingsEntryType, SettingsGroupInfo, SettingsOption, SettingsOverview,
};
use sealantern_interface::{SettingsService, SettingsServiceError};

use crate::error::SettingsError;

/// 基于 `extra` 配置管理的设置信息服务实现。
#[derive(Debug, Default)]
pub struct CoreSettingsService;

impl CoreSettingsService {
    /// 构造设置概览，返回应用层主错误。
    fn build_overview_inner() -> Result<SettingsOverview, SettingsError> {
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

        Ok(SettingsOverview {
            groups,
            total_entries,
            configured_entries,
        })
    }
}

#[async_trait]
impl SettingsService for CoreSettingsService {
    async fn settings_overview(&self) -> Result<SettingsOverview, SettingsServiceError> {
        // 设置概览构造为纯计算，经 spawn_blocking 调度到阻塞线程池，
        // 避免阻塞运行时的核心线程。
        let overview = tokio::task::spawn_blocking(Self::build_overview_inner)
            .await
            .map_err(SettingsError::from)?
            .map_err(|e| SettingsServiceError::from(e))?;

        Ok(overview)
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
                options: vec![
                    SettingsOption {
                        value: "default".to_string(),
                        display_name: "settings.color_default".to_string(),
                    },
                ],
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
                display_name: "settings.acrylic_enabled".to_string(),
                description: "settings.acrylic_enabled_desc".to_string(),
                entry_type: SettingsEntryType::Boolean,
                required: false,
                has_value: true,
                default_value: Some("false".to_string()),
                options: vec![],
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