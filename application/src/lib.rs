//! `sealantern-application` 应用层 crate。
//!
//! 提供与具体 RPC 传输无关的领域能力组装：错误类型、可观测性事件，
//! 以及高层服务接口与其实现集合。
#![forbid(unsafe_code)]

/// 应用层错误类型。
pub mod error;
/// 可观测性（日志/指标）事件与常量。
pub mod observability;
/// 插件安全策略和运行状态服务。
pub mod plugin;
/// 应用层业务能力端口。
pub mod port;
/// 应用层服务实现。
pub mod service;
/// 服务实现与注册集合。
pub mod services;

pub use port::*;
