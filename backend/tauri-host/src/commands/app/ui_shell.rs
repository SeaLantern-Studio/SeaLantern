use crate::services::ui_shell::{self, UiShellStatus};
use serde::Deserialize;
use serde::Serialize;
use tauri::AppHandle;

#[derive(Debug, Clone, Deserialize)]
pub struct SetUiShellRequest {
    pub shell_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReportUiShellRuntimeRequest {
    pub shell_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RestartAppResponse {
    pub restarted: bool,
}

#[tauri::command]
pub fn get_ui_shell_status() -> Result<UiShellStatus, String> {
    Ok(ui_shell::get_status())
}

#[tauri::command]
pub fn set_ui_shell(request: SetUiShellRequest) -> Result<UiShellStatus, String> {
    ui_shell::set_shell(&request.shell_id)
}

#[tauri::command]
pub fn report_ui_shell_runtime(
    request: ReportUiShellRuntimeRequest,
) -> Result<UiShellStatus, String> {
    ui_shell::report_runtime(&request.shell_id)
}

#[tauri::command]
pub fn restart_app(app: AppHandle) -> Result<RestartAppResponse, String> {
    crate::runtime::restart_desktop_app(&app)
}
