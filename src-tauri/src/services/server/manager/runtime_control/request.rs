use super::ServerManager;
use crate::services::server::log_pipeline as server_log_pipeline;

/// 异步请求停服
///
/// 这里会启动后台线程，避免前端调用被阻塞太久
pub(super) fn request_stop_server(manager: &ServerManager, id: &str) -> Result<(), String> {
    if manager.is_stopping(id) {
        return Ok(());
    }

    manager.mark_stopping(id);
    let sid = id.to_string();
    std::thread::spawn(move || {
        let manager = crate::services::global::server_manager();
        if let Err(err) = manager.stop_server(&sid) {
            let _ = server_log_pipeline::append_sealantern_log(
                &sid,
                &format!("[Sea Lantern] 停止失败: {}", err),
            );
            manager.clear_stopping(&sid);
        }
    });

    Ok(())
}
