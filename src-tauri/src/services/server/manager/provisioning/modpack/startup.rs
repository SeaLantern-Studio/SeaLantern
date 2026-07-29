use std::path::Path;

use crate::models::server::ImportModpackRequest;

use super::super::super::common::normalize_startup_mode;
use super::super::super::fs::resolve_startup_file_path;
use super::super::shared::trim_optional_string;

pub(super) struct ModpackStartupSelection {
    pub(super) startup_mode: String,
    pub(super) custom_command: Option<String>,
    pub(super) startup_file_path: Option<String>,
    pub(super) selected_core_type: Option<String>,
    pub(super) selected_mc_version: Option<String>,
}

pub(super) fn resolve_modpack_startup_selection(
    req: &ImportModpackRequest,
    source_path: &Path,
    run_dir: &Path,
) -> Result<ModpackStartupSelection, String> {
    let startup_mode = normalize_startup_mode(&req.startup_mode).to_string();
    let custom_command = trim_optional_string(req.custom_command.as_ref());
    let selected_core_type = trim_optional_string(req.core_type.as_ref());
    let selected_mc_version = trim_optional_string(req.mc_version.as_ref());

    let startup_file_path = if startup_mode == "custom" {
        None
    } else {
        let raw_path = req
            .startup_file_path
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "未提供启动文件路径".to_string())?;
        Some(resolve_startup_file_path(source_path, run_dir, raw_path)?)
    };

    let startup_path = startup_file_path.clone().unwrap_or_default();
    if startup_mode != "custom" && !Path::new(&startup_path).exists() {
        return Err(format!("启动文件不存在: {}", startup_path));
    }
    if startup_mode == "custom" && custom_command.is_none() {
        return Err("自定义启动命令不能为空".to_string());
    }

    Ok(ModpackStartupSelection {
        startup_mode,
        custom_command,
        startup_file_path,
        selected_core_type,
        selected_mc_version,
    })
}
