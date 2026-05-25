//! sculk：面向 Minecraft 联机的 P2P 隧道库。
//!
//! 基于 [`iroh`](https://iroh.computer) 提供端到端加密的 QUIC 连接，
//! 封装了 host/join 双端流程、票据编码、事件流与自动重连能力。
//!
//! # Overview
//!
//! - [`tunnel::IrohTunnel`]：创建 host 或 join 隧道。
//! - [`tunnel::Ticket`]：`sculk://` 连接票据（可序列化分享）。
//! - [`tunnel::HostConfig`] / [`tunnel::JoinConfig`]：分端配置。
//! - [`tunnel::TunnelEvent`]：运行时状态与错误事件。
//!
//! # Examples
//!
//! Host 端：
//!
//! ```no_run
//! use sculk::tunnel::{IrohTunnel, HostConfig};
//!
//! # async fn demo() -> sculk::Result<()> {
//! let (_tunnel, ticket, mut events) =
//!     IrohTunnel::host(25565, None, None, HostConfig::default()).await?;
//! println!("share ticket: {ticket}");
//!
//! while let Some(event) = events.recv().await {
//!     println!("{event:?}");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Join 端：
//!
//! ```no_run
//! use sculk::tunnel::{IrohTunnel, Ticket, JoinConfig};
//!
//! # async fn demo() -> sculk::Result<()> {
//! let ticket: Ticket = "sculk://<endpoint-id>".parse()?;
//! let (_tunnel, mut events) = IrohTunnel::join(&ticket, 30000, JoinConfig::default()).await?;
//!
//! while let Some(event) = events.recv().await {
//!     println!("{event:?}");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Notes
//!
//! - `HostConfig::max_players` 按唯一 `EndpointId` 计数。
//! - 密码是应用层校验，不替代传输层加密。
//! - `join` 侧是否自动重连由 `JoinConfig::max_retries` 控制。

#[cfg(feature = "clipboard")]
pub mod clipboard;
pub mod error;
#[cfg(feature = "persist")]
pub mod persist;
pub mod tunnel;
pub mod types;

pub use error::{Result, SculkError};
pub use types::{RelayUrl, SecretKey};

/// Minecraft 服务端标准端口。
pub const DEFAULT_MC_PORT: u16 = 25565;

/// join 端本地入站监听端口默认值。
pub const DEFAULT_INLET_PORT: u16 = 30000;
