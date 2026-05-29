use super::ServerManager;
use crate::services::server::runtime;

/// 发送 `stop` 命令并等待服务器退出
pub(super) fn stop_server(manager: &ServerManager, id: &str) -> Result<(), String> {
    let server = manager.find_server_clone(id)?;

    let resolved_runtime = runtime::resolve_runtime(&server)?;
    resolved_runtime.stop_with_manager(manager, &server)
}
