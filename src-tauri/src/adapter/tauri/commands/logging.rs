//! 日志相关 Tauri 命令。
//!
//! 命令仅作宿主能力适配，业务逻辑（上传到 mclo.gs）经应用装配层的
//! [`ConsoleService`] 提供，符合适配器端与业务端分离的设计。

use sealantern_application::port::ConsoleService;
use sealantern_application::services::AppServices;
use sealantern_contract::ConsoleServiceError;
use tauri::State;

/// 将当前控制台日志上传到 mclo.gs 并返回可分享链接。
///
/// 失败时使用契约错误 [`ConsoleServiceError`] 返回，由前端决定提示文案。
#[tauri::command(rename_all = "snake_case")]
pub async fn share_logs(
    services: State<'_, AppServices>,
    content: String,
) -> Result<String, ConsoleServiceError> {
    services.console().share_logs(&content).await
}
