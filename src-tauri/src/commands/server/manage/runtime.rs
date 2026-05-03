use super::common::{manager, ForceStopPreparationResponse, ServerStartFallbackEvent};
use crate::models::server::{ServerInstance, ServerStatusInfo};
use tauri::Emitter;

/// 生成强制停止确认信息
pub(super) fn prepare_force_stop_server(
    id: String,
) -> Result<ForceStopPreparationResponse, String> {
    let preparation = manager().prepare_force_stop_server(&id)?;
    Ok(ForceStopPreparationResponse {
        token: preparation.token,
        expires_at: preparation.expires_at,
    })
}

/// 执行强制停止
pub(super) fn force_stop_server(id: String, confirmation_token: String) -> Result<(), String> {
    manager().force_stop_server(&id, &confirmation_token)
}

/// 启动服务器，并在需要时向前端发送启动回退事件
pub(super) fn start_server(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let report = manager().start_server(&id)?;
    if let Some(fallback) = report.fallback {
        let _ = app.emit(
            "server-start-fallback",
            ServerStartFallbackEvent {
                server_id: report.server_id,
                server_name: report.server_name,
                from_mode: fallback.from_mode,
                to_mode: fallback.to_mode,
                reason: fallback.reason,
            },
        );
    }
    Ok(())
}

/// 请求停止服务器
pub(super) fn stop_server(id: String) -> Result<(), String> {
    manager().request_stop_server(&id)
}

/// 发送控制台命令
pub(super) fn send_command(id: String, command: String) -> Result<(), String> {
    manager().send_command(&id, &command)
}

/// 读取服务器列表
pub(super) fn get_server_list() -> Vec<ServerInstance> {
    manager().get_server_list()
}

/// 读取服务器状态
///
/// 发现错误时会顺手发系统通知和前端事件
pub(super) fn get_server_status(app: tauri::AppHandle, id: String) -> ServerStatusInfo {
    let status = manager().get_server_status(&id);

    if let Some(error_msg) = &status.error_message {
        use tauri_plugin_notification::NotificationExt;

        let server_name = manager()
            .get_server_list()
            .iter()
            .find(|server| server.id == id)
            .map(|server| server.name.clone())
            .unwrap_or_else(|| id.clone());

        let _ = app
            .notification()
            .builder()
            .title("Sea Lantern - 服务器错误")
            .body(format!("服务器「{}」{}", server_name, error_msg))
            .show();

        let _ = app.emit("server-error", ());
    }

    status
}

/// 删除服务器
pub(super) fn delete_server(id: String) -> Result<(), String> {
    manager().delete_server(&id)
}

/// 读取服务器日志
pub(super) fn get_server_logs(id: String, since: usize, max_lines: Option<usize>) -> Vec<String> {
    crate::services::server::log_pipeline::get_logs(&id, since, max_lines)
}

pub(super) fn update_server_name(id: String, name: String) -> Result<(), String> {
    manager().update_server_name(&id, &name)
}

pub(super) fn update_server_path(
    id: String,
    new_path: String,
    new_jar_path: Option<String>,
    new_startup_mode: Option<String>,
) -> Result<ServerInstance, String> {
    manager().update_server_path(
        &id,
        &new_path,
        new_jar_path.as_deref(),
        new_startup_mode.as_deref(),
    )
}
