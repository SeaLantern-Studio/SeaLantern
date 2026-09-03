//! 服务器核心下载目录服务契约。
//!
//! 提供服务器核心类型、版本与下载链接查询的宿主能力端口，供各宿主统一消费。

pub mod service;
pub use service::ServerCatalogService;
