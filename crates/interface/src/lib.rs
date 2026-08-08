//! `sealantern-interface` 接口契约 crate。
//!
//! 定义宿主侧能力端口（trait）及其相关模型与契约错误，供 `server` 实现与上层复用，
//! 不依赖任何具体 RPC 传输。

#![forbid(unsafe_code)]

/// 下载任务管理相关模型与服务端口。
pub mod download;
/// 接口契约错误类型。
pub mod error;
/// 服务器实例记录相关模型与服务端口。
pub mod instance;
/// 服务器进程管理相关模型与服务端口。
pub mod server;
/// 系统资源信息相关模型与服务端口。
pub mod system;

/// 下载任务管理服务端口。
pub use download::DownloadService;
/// 下载任务管理错误枚举。
pub use error::DownloadServiceError;
/// 服务器实例管理错误枚举。
pub use error::InstanceServiceError;
/// 服务器进程管理错误枚举。
pub use error::ServerServiceError;
/// 系统资源信息服务错误枚举。
pub use error::SystemServiceError;
/// 服务器实例记录管理服务端口。
pub use instance::InstanceService;
/// 服务器进程管理服务端口。
pub use server::ServerService;
/// 系统资源信息服务端口。
pub use system::SystemService;
