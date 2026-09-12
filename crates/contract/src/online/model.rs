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

/// 在线隧道的生命周期阶段。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OnlineTunnelPhase {
    /// 空闲，未持有隧道。
    Idle,
    /// 正在建立隧道。
    Starting,
    /// 隧道已建立并运行。
    Active,
    /// 正在关闭隧道。
    Stopping,
}

/// 在线隧道错误的产品级分类。
///
/// 分类稳定且可序列化，供前端按类别给出可操作的提示；底层细节不跨传输面。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OnlineTunnelErrorCategory {
    /// 票据格式非法或协议版本不受支持。
    InvalidJoinUri,
    /// 端点或中继配置不可用。
    InvalidEndpoint,
    /// Host 拒绝了访问凭证。
    AuthorizationDenied,
    /// 无法连通远端 Host 或其中继路径。
    HostUnreachable,
    /// 本地目标服务（Minecraft 服务端）不可达。
    TargetUnavailable,
    /// 本地监听地址无法绑定。
    LocalPortUnavailable,
    /// 持久化身份无法安全读写。
    IdentityUnavailable,
    /// 生命周期操作与当前状态冲突。
    OperationConflict,
    /// 触达连接数或工作上限。
    ResourceLimit,
    /// 调用方提供的配置非法。
    InvalidConfiguration,
    /// 无明确产品动作可恢复的内部错误。
    Internal,
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
    /// 隧道是否处于活动状态（等价于 `phase == active`）。
    pub active: bool,
    /// 当前生命周期阶段。
    pub phase: OnlineTunnelPhase,
    /// 当前运行角色；隧道未启动时为空。
    pub mode: Option<OnlineTunnelMode>,
    /// 仅 Host 隧道存在，供主机分享给对端使用。
    pub ticket: Option<String>,
    /// 当前已建立的对端连接列表。
    pub connections: Vec<OnlineTunnelConnection>,
    /// 当前生命周期内最近一次结构化错误分类。
    pub last_error: Option<OnlineTunnelErrorCategory>,
}

/// 应用层的在线隧道事件。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OnlineTunnelEvent {
    /// 隧道已启动（由宿主在 host / join 成功后发出）。
    Started {
        /// 启动后的运行角色。
        mode: OnlineTunnelMode,
    },
    /// 隧道已停止（由宿主在 stop 成功后、停止事件转发前发出）。
    Stopped {
        /// 停止前的运行角色。
        mode: OnlineTunnelMode,
    },
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
    /// 访问令牌已轮换，Host 应重新获取 Join URI。
    TokenRotated,
    /// 隧道操作失败。
    Error {
        /// 错误分类。
        category: OnlineTunnelErrorCategory,
        /// 错误描述。
        message: String,
    },
    /// 底层中继提供方上报的消息。
    ProviderMessage {
        /// 提供方消息内容。
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_events_use_snake_case_kind_and_mode() {
        let started =
            serde_json::to_value(OnlineTunnelEvent::Started { mode: OnlineTunnelMode::Host })
                .expect("started event must serialize");
        assert_eq!(started["kind"], "started");
        assert_eq!(started["mode"], "host");

        let stopped =
            serde_json::to_value(OnlineTunnelEvent::Stopped { mode: OnlineTunnelMode::Join })
                .expect("stopped event must serialize");
        assert_eq!(stopped["kind"], "stopped");
        assert_eq!(stopped["mode"], "join");
    }
}
