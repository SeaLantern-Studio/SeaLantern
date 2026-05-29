use super::ServerManager;
use crate::services::server::runtime;

/// 异步请求停服
///
/// 这里会启动后台线程，避免前端调用被阻塞太久
pub(super) fn request_stop_server(manager: &ServerManager, id: &str) -> Result<(), String> {
    let server = manager.find_server_clone(id)?;

    let resolved_runtime = runtime::resolve_runtime(&server)?;
    resolved_runtime.request_stop_with_manager(manager, &server)
}
