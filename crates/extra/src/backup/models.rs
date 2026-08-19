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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSettings {
    /// 最大备份数量（范围1-50）
    #[serde(default = "default_max_backups")]
    pub max_backups: u32,
    /// 自动备份开关
    #[serde(default)]
    pub auto_backup_enabled: bool,
    /// 自动备份间隔（小时）
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

fn default_max_backups() -> u32 {
    10
}

fn default_auto_backup_interval() -> u32 {
    24
}

fn default_auto_backup_contents() -> Vec<BackupContentType> {
    vec![BackupContentType::Core, BackupContentType::Config, BackupContentType::World]
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
