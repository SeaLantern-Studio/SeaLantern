#![forbid(unsafe_code)]

//! 面向宿主适配器的服务器应用服务。
//!
//! 此 crate 负责定义上层服务器操作的稳定入口和观测边界；底层进程、实例和解析
//! 能力仍由 `sealantern-core` 等基础 crate 提供。

pub mod observability;
pub mod rpc;
