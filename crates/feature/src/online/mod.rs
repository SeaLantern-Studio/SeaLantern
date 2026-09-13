//! SeaLantern 在线联机隧道能力。
//!
//! 此模块定义应用自己的请求、状态和事件契约。底层使用的隧道库保持在私有适配器内，
//! 因而 server、桌面宿主和后续调用方不会依赖 `sculk` 的公开类型。

mod model;
mod service;

pub use model::{
    HostTunnelRequest, JoinTunnelRequest, OnlineTunnelError, TunnelConnection, TunnelErrorCategory,
    TunnelEvent, TunnelIdentity, TunnelLinkLifetime, TunnelMode, TunnelPhase, TunnelStatus,
    TunnelTicket,
};
pub use service::OnlineTunnelService;
