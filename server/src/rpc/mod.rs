//! 协议无关的 RPC 操作边界。
//!
//! 传输适配器负责将 Tauri、HTTP 或其他请求转换为此模块的方法调用；此处不依赖
//! 任一具体传输协议。

mod access;
pub mod axum;
mod context;
mod contract;
mod error;
mod lifecycle;
mod method_name;
mod response;

pub mod methods;
pub mod service;

pub use access::{RpcAccess, RpcPermission};
pub use context::{RpcContext, RpcRequest, RpcRequestId, RpcTransport};
pub use contract::{dispatch, RpcMethod};
pub use error::{RpcError, RpcErrorCode, RpcResult};
pub use lifecycle::{RpcCancellationToken, RpcDeadline};
pub use method_name::RpcMethodName;
pub use response::RpcResponse;
