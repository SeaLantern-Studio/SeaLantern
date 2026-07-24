use serde::{Deserialize, Serialize};

/// 备份格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackupFormat {
    Zip,
    #[serde(rename = "tar.gz")]
    TarGz,
}

impl BackupFormat {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionLevel {
    Low,
    Medium,
    High,
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::Medium
    }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSettings {
    /// 最大备份数量（范围1-50）
    pub max_backups: u32,
    /// 自动备份开关
    pub auto_backup_enabled: bool,
    /// 自动备份间隔（小时）
    pub auto_backup_interval: u32,
    /// 自动备份内容
    pub auto_backup_contents: Vec<BackupContentType>,
    /// 默认压缩格式
    pub default_format: BackupFormat,
    /// 压缩级别
    pub compression_level: CompressionLevel,
}

impl Default for BackupSettings {
    fn default() -> Self {
        Self {
            max_backups: 10,
            auto_backup_enabled: false,
            auto_backup_interval: 24,
            auto_backup_contents: vec![
                BackupContentType::Core,
                BackupContentType::Config,
                BackupContentType::World,
            ],
            default_format: BackupFormat::Zip,
            compression_level: CompressionLevel::Medium,
        }
    }
}