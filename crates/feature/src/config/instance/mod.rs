//! 实例级配置文档（`sl.json`）。
//!
//! `sl.json` 是实例的权威配置，跟随服务器目录移动；本模块提供文档模型、
//! schema 版本门禁与基于 `ConfigFile` 的锁内读写。

mod document;
mod error;

pub use document::{
    CURRENT_SCHEMA_VERSION, CoreSpec, DOCUMENT_FILE_NAME, DocumentStore, InstanceCronEntry,
    InstanceDocument, MemorySpec, StartupSpec,
};
pub use error::DocumentError;
