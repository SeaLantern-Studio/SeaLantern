//! 在线隧道服务契约。
//!
//! 定义隧道启动请求、运行状态与事件等模型，以及开启 / 加入隧道的宿主能力端口。

mod model;
mod service;

pub use model::{
    OnlineTunnelConnection, OnlineTunnelEvent, OnlineTunnelHostRequest, OnlineTunnelJoinRequest,
    OnlineTunnelMode, OnlineTunnelStatus,
};
pub use service::OnlineTunnelService;
