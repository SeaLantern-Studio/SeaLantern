//! 服务器进程契约模型。
//!
//! 定义宿主消费的服务器进程状态快照等模型，全部可序列化，供跨传输面传递。

use serde::Serialize;

/// 服务器进程运行状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerState {
    /// 正在启动（进程已拉起，尚未就绪）。
    Starting,
    /// 运行中。
    Running,
    /// 正在优雅停止。
    Stopping,
    /// 已停止。
    Stopped,
}

/// 服务器进程状态快照（宿主消费的契约模型）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerSnapshot {
    /// 实例标识。
    pub instance_id: String,
    /// 运行状态。
    pub state: ServerState,
    /// 进程 ID；未运行时为 `None`。
    pub pid: Option<u32>,
    /// 本次运行的启动时长（秒）；未运行时为 `None`。
    pub uptime_secs: Option<u64>,
    /// 异常退出信息；正常状态为 `None`。
    pub error_message: Option<String>,
}
