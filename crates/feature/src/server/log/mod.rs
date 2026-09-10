//! 服务器日志数据库的存储、读取与批量写入。
//!
//! 按服务器目录持久化控制台日志：`store` 提供日志库初始化与增量读取，
//! `writer` 提供高频输出场景下的批量写入。数据访问复用 `infra` 的
//! [`SqliteDatabase`]，本模块不绑定任何宿主。

mod store;
mod writer;

pub use store::{LOG_DATABASE_FILE, LogLine, open_log_database, read_logs};
pub use writer::LogWriter;

/// 日志来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSource {
    /// Sea Lantern 自身写入的说明性日志。
    SeaLantern,
    /// 服务器进程输出（stdout / stderr）。
    Server,
}

impl LogSource {
    /// 数据库中的来源标识。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SeaLantern => "sealantern",
            Self::Server => "server",
        }
    }
}
