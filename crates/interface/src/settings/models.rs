//! 设置信息契约模型。
//!
//! 定义宿主消费的设置相关模型，全部可序列化，供跨传输面传递。

use serde::{Deserialize, Serialize};

/// 设置分组信息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsGroupInfo {
    /// 分组标识。
    pub id: String,
    /// 分组显示名称（i18n key）。
    pub display_name: String,
    /// 分组描述（i18n key）。
    pub description: String,
    /// 分组内的设置项列表。
    pub entries: Vec<SettingsEntry>,
}

/// 设置项信息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsEntry {
    /// 设置项标识。
    pub id: String,
    /// 设置项显示名称（i18n key）。
    pub display_name: String,
    /// 设置项描述（i18n key）。
    pub description: String,
    /// 设置项类型。
    pub entry_type: SettingsEntryType,
    /// 是否必填。
    pub required: bool,
    /// 是否已设置。
    pub has_value: bool,
    /// 默认值（JSON 字符串）。
    pub default_value: Option<String>,
    /// 可选值列表（用于枚举类型）。
    pub options: Vec<SettingsOption>,
}

/// 设置项类型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingsEntryType {
    /// 字符串。
    String,
    /// 整数。
    Integer,
    /// 浮点数。
    Float,
    /// 布尔值。
    Boolean,
    /// 枚举值。
    Enum,
    /// 路径。
    Path,
    /// 多行文本。
    Text,
}

/// 设置选项（用于枚举类型）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsOption {
    /// 选项值。
    pub value: String,
    /// 选项显示名称（i18n key）。
    pub display_name: String,
}

/// 设置概览。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsOverview {
    /// 所有设置分组列表。
    pub groups: Vec<SettingsGroupInfo>,
    /// 总设置项数量。
    pub total_entries: usize,
    /// 已配置项数量。
    pub configured_entries: usize,
}