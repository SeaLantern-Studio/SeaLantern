//! 协议无关的 RPC 操作边界。
//!
//! 传输适配器负责将 Tauri、HTTP 或其他请求转换为此模块的方法调用；此处不依赖
//! 任一具体传输协议。

pub mod methods;
pub mod service;
