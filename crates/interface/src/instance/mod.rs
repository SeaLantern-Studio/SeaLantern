//! 服务器实例管理器
//!
//! 提供实例领域模型（错误分类）与宿主能力端口（服务 trait）。

/// 实例服务能力端口。
pub mod service;

/// 实例管理服务 trait。
pub use service::InstanceService;
