use crate::models::server::{ServerStatus, ServerStatusInfo};

use super::super::common::current_timestamp_secs;
use super::ServerManager;
use crate::services::server::log_pipeline as server_log_pipeline;

/// 读取服务器当前运行状态
pub(super) fn get_server_status(manager: &ServerManager, id: &str) -> ServerStatusInfo {
    let mut exit_code: Option<i32> = None;
    let mut error_message: Option<String> = None;

    let is_running = manager
        .lock_processes()
        .ok()
        .map(|mut procs| {
            if let Some(child) = procs.get_mut(id) {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        exit_code = status.code();
                        match &exit_code {
                            Some(0) => {
                                let _ = server_log_pipeline::append_sealantern_log(
                                    id,
                                    "[Sea Lantern] 服务器已正常退出",
                                );
                            }
                            Some(code) => {
                                error_message = Some(format!("服务器异常退出 (退出码：{})", code));
                                let _ = server_log_pipeline::append_sealantern_log(
                                    id,
                                    &format!("[Sea Lantern] 服务器异常退出 (退出码：{})", code),
                                );
                            }
                            None => {
                                error_message = Some("服务器被强制终止".to_string());
                                let _ = server_log_pipeline::append_sealantern_log(
                                    id,
                                    "[Sea Lantern] 服务器被强制终止",
                                );
                            }
                        }

                        procs.remove(id);
                        server_log_pipeline::shutdown_writer(id);
                        manager.clear_starting(id);
                        false
                    }
                    Ok(None) => true,
                    Err(_) => {
                        error_message = Some("获取服务器状态失败".to_string());
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            "[Sea Lantern] 获取服务器状态失败",
                        );
                        procs.remove(id);
                        server_log_pipeline::shutdown_writer(id);
                        manager.clear_starting(id);
                        false
                    }
                }
            } else {
                false
            }
        })
        .unwrap_or(false);

    let pid = if is_running {
        manager
            .lock_processes()
            .ok()
            .and_then(|mut procs| procs.get_mut(id).map(|child| child.id()))
    } else {
        None
    };

    let uptime = manager
        .lock_servers()
        .ok()
        .and_then(|servers| {
            servers
                .iter()
                .find(|s| s.id == id)
                .and_then(|s| s.last_started_at)
        })
        .and_then(|started_at| current_timestamp_secs().checked_sub(started_at));

    ServerStatusInfo {
        id: id.to_string(),
        status: if manager.is_stopping(id) {
            ServerStatus::Stopping
        } else if is_running && manager.is_starting(id) {
            ServerStatus::Starting
        } else if is_running {
            ServerStatus::Running
        } else {
            ServerStatus::Stopped
        },
        pid,
        uptime,
        error_message,
    }
}
