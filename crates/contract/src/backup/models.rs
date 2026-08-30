//! 备份管理契约模型。
//!
//! 定义宿主消费的备份项、创建请求与备份设置等模型，全部可序列化，
//! 供跨传输面传递。

use serde::{Deserialize, Serialize};

/// 备份格式
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackupFormat {
    #[default]
    Zip,
    #[serde(rename = "tar.gz")]
    TarGz,
}

impl BackupFormat {
    /// 备份文件的扩展名（含点号前缀的格式名）。
    pub fn extension(&self) -> &'static str {
        match self {
            BackupFormat::Zip => "zip",
            BackupFormat::TarGz => "tar.gz",
        }
    }
}

impl std::fmt::Display for BackupFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupFormat::Zip => write!(f, "zip"),
            BackupFormat::TarGz => write!(f, "tar.gz"),
        }
    }
}

/// 压缩级别
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionLevel {
    Low,
    #[default]
    Medium,
    High,
}

/// 备份内容类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackupContentType {
    Core,
    Config,
    Plugins,
    World,
    Logs,
}

impl BackupContentType {
    /// 对应服务器目录下的子目录名（`Core` 为根目录）。
    pub fn directory_name(&self) -> &'static str {
        match self {
            BackupContentType::Core => ".",
            BackupContentType::Config => "config",
            BackupContentType::Plugins => "plugins",
            BackupContentType::World => "world",
            BackupContentType::Logs => "logs",
        }
    }
}

/// 备份项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupItem {
    /// 备份ID
    pub id: String,
    /// 服务器ID
    pub server_id: String,
    /// 备份文件名
    pub name: String,
    /// 压缩格式
    pub format: BackupFormat,
    /// 文件大小（字节）
    pub size: u64,
    /// 创建时间（UTC，ISO 8601格式）
    pub created_at: String,
    /// 备份内容类型列表
    pub contents: Vec<BackupContentType>,
}

/// 创建备份请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBackupRequest {
    /// 服务器ID
    pub server_id: String,
    /// 备份内容类型列表
    pub contents: Vec<BackupContentType>,
    /// 压缩格式
    pub format: BackupFormat,
    /// 压缩级别
    pub compression_level: CompressionLevel,
    /// 可选的备份文件名（不传则自动生成）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// 备份设置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSettings {
    /// 最大备份数量（范围1-50）
    #[serde(default = "default_max_backups")]
    pub max_backups: u32,
    /// 自动备份开关
    #[serde(default)]
    pub auto_backup_enabled: bool,
    /// 自动备份间隔（小时，范围1-720）
    #[serde(default = "default_auto_backup_interval")]
    pub auto_backup_interval: u32,
    /// 自动备份内容
    #[serde(default = "default_auto_backup_contents")]
    pub auto_backup_contents: Vec<BackupContentType>,
    /// 默认压缩格式
    #[serde(default)]
    pub default_format: BackupFormat,
    /// 压缩级别
    #[serde(default)]
    pub compression_level: CompressionLevel,
}

const DEFAULT_MAX_BACKUPS: u32 = 10;
const DEFAULT_AUTO_BACKUP_INTERVAL: u32 = 24;
const DEFAULT_AUTO_BACKUP_CONTENTS: &[BackupContentType] =
    &[BackupContentType::Core, BackupContentType::Config, BackupContentType::World];

fn default_max_backups() -> u32 {
    DEFAULT_MAX_BACKUPS
}

fn default_auto_backup_interval() -> u32 {
    DEFAULT_AUTO_BACKUP_INTERVAL
}

fn default_auto_backup_contents() -> Vec<BackupContentType> {
    DEFAULT_AUTO_BACKUP_CONTENTS.to_vec()
}

impl Default for BackupSettings {
    fn default() -> Self {
        Self {
            max_backups: DEFAULT_MAX_BACKUPS,
            auto_backup_enabled: false,
            auto_backup_interval: DEFAULT_AUTO_BACKUP_INTERVAL,
            auto_backup_contents: DEFAULT_AUTO_BACKUP_CONTENTS.to_vec(),
            default_format: BackupFormat::Zip,
            compression_level: CompressionLevel::Medium,
        }
    }
}
