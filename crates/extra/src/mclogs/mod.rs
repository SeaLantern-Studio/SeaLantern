//! mclo.gs 日志分享服务。
//!
//! 将控制台/服务器日志上传到 mclo.gs 换取一个可分享的只读链接，
//! 便于在社区、Issue、群里贴日志求助，而无需直接粘贴大段文本。
//!
//! 参考：<https://mclo.gs/doc/api>

mod client;
mod payload;

pub use client::share_logs;
