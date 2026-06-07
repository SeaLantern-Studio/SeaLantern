use std::path::Path;

use crate::models::server::ImportModpackRequest;
use crate::services::server::manager::provisioning::i18n::{provisioning_t, provisioning_t1};

use super::super::super::common::StartupMode;
use super::super::super::fs::resolve_startup_file_path;

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
    let startup_mode = StartupMode::from_raw(&req.startup_mode);
    let requested_custom_command = trim_optional_string(req.custom_command.as_ref());
    let selected_core_type = trim_optional_string(req.core_type.as_ref());
    let selected_mc_version = trim_optional_string(req.mc_version.as_ref());

    let resolved_startup_file_path = req
        .startup_file_path
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|raw_path| resolve_startup_file_path(source_path, run_dir, raw_path))
        .transpose()?;

    let custom_command = if startup_mode.is_custom() {
        requested_custom_command.or_else(|| {
            resolved_startup_file_path
                .as_ref()
                .map(|path| format_native_startup_command(path))
        })
    } else {
        requested_custom_command
    };

    let startup_file_path = if startup_mode.is_custom() {
        resolved_startup_file_path
    } else {
        let raw_path = req
            .startup_file_path
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| provisioning_t("server.provisioning.startup_file_path_missing"))?;
        Some(resolve_startup_file_path(source_path, run_dir, raw_path)?)
    };

    let startup_path = startup_file_path.clone().unwrap_or_default();
    if !startup_mode.is_custom() && !Path::new(&startup_path).exists() {
        return Err(provisioning_t1("server.provisioning.startup_file_missing", startup_path));
    }
    if startup_mode.is_custom() && custom_command.is_none() {
        return Err(provisioning_t("server.provisioning.custom_command_empty"));
    }

    Ok(ModpackStartupSelection {
        startup_mode: startup_mode.as_str().to_string(),
        custom_command,
        startup_file_path,
        selected_core_type,
        selected_mc_version,
    })
}

fn trim_optional_string(value: Option<&String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn format_native_startup_command(path: &str) -> String {
    if path.contains(' ') {
        format!("\"{}\"", path)
    } else {
        path.to_string()
    }
}
