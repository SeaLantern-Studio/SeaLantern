//! core crate 统一错误类型。

use std::path::PathBuf;

use thiserror::Error;

/// 装箱的错误源类型，用于保留原始错误链。
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// 持久化层错误。
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PersistError {
    /// 系统数据目录不可用。
    #[error("cannot determine system data directory")]
    SystemDataDirUnavailable,
    /// 路径 I/O 错误。
    #[error("{op} `{path}` failed: {source}")]
    PathIo {
        op: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// 密钥文件长度非法。
    #[error("invalid key file length: expected {expected} bytes, got {actual} bytes")]
    InvalidKeyLength { expected: usize, actual: usize },
    /// profile 解析失败。
    #[cfg(feature = "persist")]
    #[error("failed to parse profile `{path}`: {source}")]
    ProfileParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    /// profile 序列化失败。
    #[cfg(feature = "persist")]
    #[error(transparent)]
    ProfileSerialize(#[from] toml::ser::Error),
    /// relay URL 解析失败。
    #[error("invalid relay URL: {0}")]
    RelayUrlParse(String),
}

/// 票据解析错误。
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TicketError {
    /// URL 解析失败。
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    /// 协议不匹配。
    #[error("invalid scheme: expected \"{expected}\", got \"{actual}\"")]
    InvalidScheme {
        expected: &'static str,
        actual: String,
    },
    /// 缺少 endpoint id。
    #[error("missing endpoint id in ticket URL")]
    MissingEndpointId,
    /// endpoint id 非法。
    #[error("invalid endpoint id: {0}")]
    EndpointIdParse(String),
    /// relay URL 非法。
    #[error("invalid relay URL: {0}")]
    RelayUrlParse(String),
}

/// 隧道运行时错误。
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TunnelError {
    /// 锁中毒。
    #[error("mutex poisoned: {name}")]
    MutexPoisoned { name: &'static str },
    /// 绑定 host endpoint 失败。
    #[error("bind host endpoint failed")]
    BindHostEndpoint(#[source] BoxError),
    /// 绑定 join endpoint 失败。
    #[error("bind join endpoint failed")]
    BindJoinEndpoint(#[source] BoxError),
    /// 绑定本地 TCP 监听失败。
    #[error("bind local tcp listener failed")]
    BindLocalListener(#[source] BoxError),
    /// host accept 失败。
    #[error("accept host connection failed")]
    AcceptHostConnection(#[source] BoxError),
    /// QUIC 双向流 accept 失败。
    #[error("accept QUIC bi stream failed")]
    AcceptQuicBiStream(#[source] BoxError),
    /// 本地 TCP 客户端 accept 失败。
    #[error("accept local tcp client failed")]
    AcceptLocalTcpClient(#[source] BoxError),
    /// 与 host 建连失败。
    #[error("connect host endpoint failed")]
    ConnectHostEndpoint(#[source] BoxError),
    /// 初次连接重试耗尽。
    #[error("initial connection failed after {attempts} attempts")]
    InitialConnectionExhausted { attempts: u32 },
    /// 打开认证流失败。
    #[error("open auth stream failed")]
    OpenAuthStream(#[source] BoxError),
    /// 接收认证流失败。
    #[error("accept auth stream failed")]
    AcceptAuthStream(#[source] BoxError),
    /// 读取认证结果失败。
    #[error("read auth result failed")]
    ReadAuthResult(#[source] BoxError),
    /// 读取认证负载失败。
    #[error("read auth payload failed")]
    ReadAuthPayload(#[source] BoxError),
    /// 写入认证负载失败。
    #[error("write auth payload failed")]
    WriteAuthPayload(#[source] BoxError),
    /// 写入认证拒绝失败。
    #[error("write auth rejected failed")]
    WriteAuthRejected(#[source] BoxError),
    /// 写入认证决策失败。
    #[error("write auth decision failed")]
    WriteAuthDecision(#[source] BoxError),
    /// 结束认证流失败。
    #[error("finish auth stream failed")]
    FinishAuthStream(#[source] BoxError),
    /// 认证被 host 拒绝。
    #[error("auth rejected by host")]
    AuthRejectedByHost,
    /// 桥接 tcp->quic 失败。
    #[error("bridge tcp->quic failed")]
    BridgeTcpToQuic(#[source] BoxError),
    /// 桥接 quic->tcp 失败。
    #[error("bridge quic->tcp failed")]
    BridgeQuicToTcp(#[source] BoxError),
}

/// sculk core 顶层错误。
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SculkError {
    /// 持久化错误。
    #[error(transparent)]
    Persist(#[from] PersistError),
    /// 票据错误。
    #[error(transparent)]
    Ticket(#[from] TicketError),
    /// 隧道错误。
    #[error(transparent)]
    Tunnel(#[from] TunnelError),
}

impl TunnelError {
    /// 构造锁中毒错误。
    pub fn mutex_poisoned(name: &'static str) -> Self {
        Self::MutexPoisoned { name }
    }
}

/// core crate 统一 `Result` 别名。
pub type Result<T> = std::result::Result<T, SculkError>;
