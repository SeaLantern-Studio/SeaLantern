use super::super::process::force_kill_process_tree;
use super::ServerManager;
use crate::services::server::log_pipeline as server_log_pipeline;

/// 发送 `stop` 命令并等待服务器退出
pub(super) fn stop_server(manager: &ServerManager, id: &str) -> Result<(), String> {
    let is_running = {
        let mut procs = manager.lock_processes()?;
        if let Some(child) = procs.get_mut(id) {
            match child.try_wait() {
                Ok(Some(_)) => {
                    procs.remove(id);
                    server_log_pipeline::shutdown_writer(id);
                    false
                }
                Ok(None) => true,
                Err(_) => {
                    procs.remove(id);
                    server_log_pipeline::shutdown_writer(id);
                    false
                }
            }
        } else {
            false
        }
    };

    if !is_running {
        manager.clear_stopping(id);
        let _ = server_log_pipeline::append_sealantern_log(id, "[Sea Lantern] 服务器未运行");
        server_log_pipeline::shutdown_writer(id);
        return Ok(());
    }

    let _ = server_log_pipeline::append_sealantern_log(id, "[Sea Lantern] 正在发送停止命令...");
    let _ = manager.send_command(id, "stop");

    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let mut procs = manager.lock_processes()?;
        if let Some(child) = procs.get_mut(id) {
            match child.try_wait() {
                Ok(Some(_)) => {
                    procs.remove(id);
                    manager.clear_stopping(id);
                    let _ = server_log_pipeline::append_sealantern_log(
                        id,
                        "[Sea Lantern] 服务器已正常停止",
                    );
                    server_log_pipeline::shutdown_writer(id);
                    return Ok(());
                }
                Ok(None) => {}
                Err(_) => {
                    procs.remove(id);
                    manager.clear_stopping(id);
                    server_log_pipeline::shutdown_writer(id);
                    return Ok(());
                }
            }
        } else {
            manager.clear_stopping(id);
            let _ = server_log_pipeline::append_sealantern_log(id, "[Sea Lantern] 服务器已停止");
            server_log_pipeline::shutdown_writer(id);
            return Ok(());
        }
    }

    let mut procs = manager.lock_processes()?;
    if let Some(mut child) = procs.remove(id) {
        let _ = force_kill_process_tree(&mut child);
        let _ =
            server_log_pipeline::append_sealantern_log(id, "[Sea Lantern] 服务器超时，已强制终止");
    }
    server_log_pipeline::shutdown_writer(id);
    manager.clear_stopping(id);
    Ok(())
}
