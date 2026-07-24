//! 应用配置管理模块
//!
//! 提供应用程序级别的配置管理，包括：
//! - 应用数据目录路径管理
//! - 应用配置的加载和保存
//! - 配置迁移和版本管理

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use sealantern_infra::fs::{get_app_data_dir, get_or_create_app_data_dir};

/// 应用程序主配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 配置版本，用于未来迁移
    pub version: u32,
    /// 用户偏好设置
    pub preferences: UserPreferences,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            preferences: UserPreferences::default(),
        }
    }
}

/// 用户偏好设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// 语言设置
    pub language: String,
    /// 主题设置
    pub theme: String,
    /// 是否启用开发者模式
    pub developer_mode: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(),
            theme: "default".to_string(),
            developer_mode: false,
        }
    }
}

impl AppConfig {
    /// 获取配置文件路径
    pub fn config_path() -> PathBuf {
        get_app_data_dir().join("config").join("app.json")
    }

    /// 加载或创建默认配置
    pub async fn load_or_default() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        
        if path.exists() {
            let content = tokio::fs::read_to_string(&path)
                .await
                .map_err(|e| ConfigError::Io(path.clone(), e.to_string()))?;
            let config: Self = serde_json::from_str(&content)
                .map_err(|e| ConfigError::Parse(e.to_string()))?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save().await?;
            Ok(config)
        }
    }

    /// 保存配置
    pub async fn save(&self) -> Result<(), ConfigError> {
        let path = Self::config_path();
        let parent = path.parent().unwrap();
        
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| ConfigError::Io(parent.to_path_buf(), e.to_string()))?;
        
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::Serialize(e.to_string()))?;
        
        tokio::fs::write(&path, content)
            .await
            .map_err(|e| ConfigError::Io(path.clone(), e.to_string()))?;
        
        Ok(())
    }
}

/// 配置错误类型
#[derive(Debug)]
pub enum ConfigError {
    /// IO 错误
    Io(PathBuf, String),
    /// 解析错误
    Parse(String),
    /// 序列化错误
    Serialize(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(path, msg) => write!(f, "IO error at {}: {}", path.display(), msg),
            ConfigError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::Serialize(msg) => write!(f, "Serialize error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}