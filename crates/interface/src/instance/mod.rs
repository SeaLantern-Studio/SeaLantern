//! 服务器实例管理器
//!
//! 提供实例领域模型（错误分类）与宿主能力端口（服务 trait）。

/// 实例领域模型与错误分类。
pub mod model;
/// 实例管理服务能力端口。
pub mod service;

/// 实例管理错误枚举。
pub use model::InstanceServiceError;
/// 实例管理服务 trait。
pub use service::InstanceService;
