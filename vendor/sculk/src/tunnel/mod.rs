//! P2P 隧道：基于 iroh QUIC 的 TCP 端口转发层。
//!
//! [`IrohTunnel`] 是主入口，host/join 双端共用 [`TunnelConfig`] 与 [`TunnelEvent`]。

mod event;
mod iroh;
mod ticket;

pub use crate::types::{RelayUrl, SecretKey};
pub use ::iroh::EndpointId;
pub use event::{ConnectionSnapshot, HostConfig, JoinConfig, PeerId, TunnelEvent};
pub use iroh::IrohTunnel;
pub use ticket::Ticket;
