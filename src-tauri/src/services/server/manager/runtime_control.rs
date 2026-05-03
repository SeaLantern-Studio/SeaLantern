//! 服务器停服和状态查询流程

mod force_stop;
mod request;
mod status;
mod stop;

use crate::models::server::ServerStatusInfo;

use super::{ForceStopPreparation, ServerManager};

/// 异步请求停服
///
/// 这里会启动后台线程，避免前端调用被阻塞太久
pub(super) fn request_stop_server(manager: &ServerManager, id: &str) -> Result<(), String> {
    request::request_stop_server(manager, id)
}

/// 生成强停确认信息
pub(super) fn prepare_force_stop_server(
    manager: &ServerManager,
    id: &str,
) -> Result<ForceStopPreparation, String> {
    force_stop::prepare_force_stop_server(manager, id)
}

/// 校验确认口令并强制终止服务器进程
pub(super) fn force_stop_server(
    manager: &ServerManager,
    id: &str,
    confirmation_token: &str,
) -> Result<(), String> {
    force_stop::force_stop_server(manager, id, confirmation_token)
}

/// 发送 `stop` 命令并等待服务器退出
pub(super) fn stop_server(manager: &ServerManager, id: &str) -> Result<(), String> {
    stop::stop_server(manager, id)
}

/// 读取服务器当前运行状态
pub(super) fn get_server_status(manager: &ServerManager, id: &str) -> ServerStatusInfo {
    status::get_server_status(manager, id)
}
