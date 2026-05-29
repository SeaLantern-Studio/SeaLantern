use crate::models::server::{ServerStatus, ServerStatusInfo};

use super::super::common::current_timestamp_secs;
use super::ServerManager;
use crate::services::server::runtime;

/// 读取服务器当前运行状态
pub(super) fn get_server_status(manager: &ServerManager, id: &str) -> ServerStatusInfo {
    let server = manager.find_server_clone_optional(id).ok().flatten();

    let Some(server) = server else {
        return ServerStatusInfo {
            id: id.to_string(),
            status: ServerStatus::Stopped,
            pid: None,
            uptime: None,
            detail_message: None,
            error_message: Some(format!("未找到服务器: {}", id)),
        };
    };

    let resolved_runtime = match runtime::resolve_runtime(&server) {
        Ok(runtime) => runtime,
        Err(err) => {
            return ServerStatusInfo {
                id: id.to_string(),
                status: ServerStatus::Error,
                pid: None,
                uptime: None,
                detail_message: None,
                error_message: Some(err),
            };
        }
    };

    let snapshot = match resolved_runtime.status_with_manager(manager, &server) {
        Ok(snapshot) => snapshot,
        Err(err) => {
            return ServerStatusInfo {
                id: id.to_string(),
                status: ServerStatus::Error,
                pid: None,
                uptime: None,
                detail_message: None,
                error_message: Some(err),
            };
        }
    };

    let uptime = server
        .last_started_at
        .and_then(|started_at| current_timestamp_secs().checked_sub(started_at));

    ServerStatusInfo {
        id: id.to_string(),
        status: snapshot.status,
        pid: snapshot.pid,
        uptime,
        detail_message: snapshot.detail_message,
        error_message: snapshot.error_message,
    }
}
