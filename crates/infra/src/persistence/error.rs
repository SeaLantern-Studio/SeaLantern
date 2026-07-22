use std::fmt;
use std::path::PathBuf;

/// 持久化基础设施操作返回的错误。
#[derive(Debug)]
pub enum PersistenceError {
    /// 数据库文件的父目录无法创建。
    CreateParent {
        path: PathBuf,
        source: std::io::Error,
    },
    /// SQLite 操作失败。
    Sqlite {
        operation: &'static str,
        path: PathBuf,
        message: String,
    },
    /// 阻塞数据库任务未能完成。
    Task {
        operation: &'static str,
        message: String,
    },
    /// 进程内协调器状态异常。
    Coordination { resource: PathBuf, message: String },
    /// 迁移清单不满足版本顺序约束。
    InvalidMigration { version: i64, reason: &'static str },
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateParent { path, source } => {
                write!(
                    formatter,
                    "failed to create database parent for '{}': {source}",
                    path.display()
                )
            }
            Self::Sqlite { operation, path, message } => {
                write!(
                    formatter,
                    "SQLite operation '{operation}' failed for '{}': {message}",
                    path.display()
                )
            }
            Self::Task { operation, message } => {
                write!(formatter, "database task '{operation}' failed: {message}")
            }
            Self::Coordination { resource, message } => {
                write!(
                    formatter,
                    "failed to coordinate access to '{}': {message}",
                    resource.display()
                )
            }
            Self::InvalidMigration { version, reason } => {
                write!(formatter, "migration {version} is invalid: {reason}")
            }
        }
    }
}

impl std::error::Error for PersistenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CreateParent { source, .. } => Some(source),
            _ => None,
        }
    }
}
