//! 在线隧道契约模型。
//!
//! 定义隧道启动请求、运行角色、对端连接快照、状态与事件等模型，
//! 全部可序列化，供跨传输面传递。

use serde::{Deserialize, Serialize};

/// 以 Host 角色开启隧道的请求。
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OnlineTunnelHostRequest {
    /// 对外暴露给对端的 Minecraft 端口。
    pub minecraft_port: u16,
    /// 可选的对端访问密码。
    pub password: Option<String>,
    /// 可选的并发玩家数上限。
    pub max_players: Option<u32>,
    /// 可选的中继服务器地址；为空时使用默认中继。
    pub relay_url: Option<String>,
    /// 可选的稳定 32 字节主机身份密钥，永远不会包含在响应中。
    pub identity: Option<Vec<u8>>,
}

/// 以 Join 角色加入已有隧道的请求。
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OnlineTunnelJoinRequest {
    /// 主机分享的隧道票据。
    pub ticket: String,
    /// 本地需要暴露到隧道内的端口。
    pub local_port: u16,
    /// 可选的对端访问密码。
    pub password: Option<String>,
    /// 可选的连接失败最大重试次数。
    pub max_retries: Option<u32>,
}

/// 在线隧道的运行角色。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OnlineTunnelMode {
    /// 作为主机开启隧道。
    Host,
    /// 作为对端加入隧道。
    Join,
}

/// 已建立隧道中单个对端的运行时快照。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OnlineTunnelConnection {
    /// 对端唯一标识。
    pub remote_id: String,
    /// 是否经过中继服务器转发。
    pub is_relay: bool,
    /// 最近一次往返时延（毫秒）。
    pub rtt_ms: u64,
    /// 累计发送字节。
    pub tx_bytes: u64,
    /// 累计接收字节。
    pub rx_bytes: u64,
    /// 对端当前是否在线。
    pub alive: bool,
    /// 该连接已存在时长（毫秒）。
    pub elapsed_ms: u64,
}

/// 当前在线隧道状态快照。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OnlineTunnelStatus {
    /// 隧道是否处于活动状态。
    pub active: bool,
    /// 当前运行角色；隧道未启动时为空。
    pub mode: Option<OnlineTunnelMode>,
    /// 仅 Host 隧道存在，供主机分享给对端使用。
    pub ticket: Option<String>,
    /// 当前已建立的对端连接列表。
    pub connections: Vec<OnlineTunnelConnection>,
}

/// 应用层的在线隧道事件。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OnlineTunnelEvent {
    /// 有玩家加入隧道。
    PlayerJoined {
        /// 加入玩家的标识。
        remote_id: String,
    },
    /// 有玩家离开隧道。
    PlayerLeft {
        /// 离开玩家的标识。
        remote_id: String,
        /// 离开原因。
        reason: String,
    },
    /// 隧道已连通。
    Connected,
    /// 隧道断开。
    Disconnected {
        /// 断开原因。
        reason: String,
    },
    /// 对端的数据路径发生变化（如改经中继转发）。
    PathChanged {
        /// 路径变化对端的标识。
        remote_id: String,
        /// 是否改经中继服务器转发。
        is_relay: bool,
        /// 新路径的往返时延（毫秒）。
        rtt_ms: u64,
    },
    /// 正在重连。
    Reconnecting {
        /// 当前重连尝试次数。
        attempt: u32,
    },
    /// 重连成功。
    Reconnected,
    /// 对端身份认证失败。
    AuthenticationFailed {
        /// 认证失败对端的标识。
        remote_id: String,
    },
    /// 对端被拒绝接入。
    PlayerRejected {
        /// 被拒绝对端的标识。
        remote_id: String,
        /// 拒绝原因。
        reason: String,
    },
    /// 隧道操作失败。
    Error {
        /// 错误描述。
        message: String,
    },
    /// 底层中继提供方上报的消息。
    ProviderMessage {
        /// 提供方消息内容。
        message: String,
    },
}
