//! 持久化基础设施。
//!
//! 五个子模块各管一件事：
//!
//! - [`config`]：按扩展名选择 JSON/TOML/YAML，提供加载、原子替换、备份，
//!   以及"锁内加载-修改-写回"原语；
//! - `coordination`：进程内按资源路径的异步读写协调（**不**跨进程）；
//! - `error`：统一错误类型；
//! - `instance_registry`：实例注册表的 SQLite 适配层；
//! - `sqlite`：通用 SQLite 访问、参数绑定、事务与版本化迁移。
//!
//! 本层不定义业务表，也不依赖 feature/core：SQL、表结构与迁移内容都由调用方
//! 提供。跨进程互斥不在这里实现——配置文件靠 `FileLock`，SQLite 靠自身的
//! 文件锁与 `busy_timeout`。

pub mod config;
mod coordination;
mod error;
mod instance_registry;
mod sqlite;

pub use config::{ConfigError, ConfigFile, ConfigFormat};
pub use coordination::{ProcessLockRegistry, ProcessResourceLock, process_lock_registry};
pub use error::PersistenceError;
pub use instance_registry::{InstanceRegistry, InstanceRegistryError, InstanceRegistryRecord};
pub use sqlite::{
    Migration, SqlValue, SqliteDatabase, SqliteLockingMode, SqliteOptions, SqliteSynchronousMode,
};
