//! 日志相关 Tauri 命令。
//!
//! 命令仅作宿主能力适配，业务逻辑（上传到 mclo.gs）放在
//! [`sealantern_extra::mclogs`]，符合适配器端与业务端分离的设计。

/// 将当前控制台日志上传到 mclo.gs 并返回可分享链接。
///
/// 失败时使用 `String` 透传错误信息，由前端决定提示文案。
#[tauri::command(rename_all = "snake_case")]
pub async fn share_logs(content: String) -> Result<String, String> {
    sealantern_extra::mclogs::share_logs(content).await
}
