use super::super::common::{current_timestamp_millis, current_timestamp_secs};
use super::super::process::force_kill_process_tree;
use super::{ForceStopPreparation, ServerManager};
use crate::services::server::log_pipeline as server_log_pipeline;

/// 生成强停确认信息
pub(super) fn prepare_force_stop_server(
    manager: &ServerManager,
    id: &str,
) -> Result<ForceStopPreparation, String> {
    let expires_at = current_timestamp_secs().saturating_add(15);
    let token = format!("{}-{}", id, current_timestamp_millis());

    let mut pending = manager
        .pending_force_stop_tokens
        .lock()
        .map_err(|_| "pending_force_stop_tokens lock poisoned".to_string())?;
    pending.insert(id.to_string(), (token.clone(), expires_at));
    drop(pending);

    let _ = server_log_pipeline::append_sealantern_log(
        id,
        "[Sea Lantern] 已创建强制关停确认，会在确认后执行",
    );

    Ok(ForceStopPreparation { token, expires_at })
}

/// 校验确认口令并强制终止服务器进程
pub(super) fn force_stop_server(
    manager: &ServerManager,
    id: &str,
    confirmation_token: &str,
) -> Result<(), String> {
    let mut pending = manager
        .pending_force_stop_tokens
        .lock()
        .map_err(|_| "pending_force_stop_tokens lock poisoned".to_string())?;

    let Some((expected_token, expires_at)) = pending.get(id).cloned() else {
        return Err("缺少强制关停确认，请重新发起操作".to_string());
    };

    let now = current_timestamp_secs();
    if now > expires_at {
        pending.remove(id);
        return Err("强制关停确认已过期，请重新发起操作".to_string());
    }

    if expected_token != confirmation_token {
        return Err("强制关停确认无效，已拒绝执行".to_string());
    }

    pending.remove(id);
    drop(pending);

    manager.mark_stopping(id);

    let mut procs = manager.lock_processes()?;
    let Some(mut child) = procs.remove(id) else {
        manager.clear_stopping(id);
        let _ = server_log_pipeline::append_sealantern_log(id, "[Sea Lantern] 服务器未运行");
        server_log_pipeline::shutdown_writer(id);
        return Ok(());
    };

    let kill_result = force_kill_process_tree(&mut child);
    drop(procs);

    manager.clear_starting(id);
    manager.clear_stopping(id);

    match kill_result {
        Ok(_) => {
            let _ = server_log_pipeline::append_sealantern_log(
                id,
                "[Sea Lantern] 已按用户确认强制终止服务器进程",
            );
            server_log_pipeline::shutdown_writer(id);
            Ok(())
        }
        Err(err) => {
            let message = format!("强制终止服务器失败: {}", err);
            let _ = server_log_pipeline::append_sealantern_log(
                id,
                &format!("[Sea Lantern] {}", message),
            );
            server_log_pipeline::shutdown_writer(id);
            Err(message)
        }
    }
}
