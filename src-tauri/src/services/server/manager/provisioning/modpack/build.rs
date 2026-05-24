use std::path::Path;

use crate::models::server::{ImportModpackRequest, ServerInstance};

use super::startup::ModpackStartupSelection;
use crate::services::server::installer;

pub(super) fn build_modpack_server_instance(
    id: String,
    server_name: String,
    req: ImportModpackRequest,
    run_dir: &Path,
    startup: ModpackStartupSelection,
    port: u16,
) -> ServerInstance {
    let startup_path = startup.startup_file_path.clone().unwrap_or_default();
    let detected_core_type = if startup.startup_mode == "custom" {
        "modpack".to_string()
    } else {
        installer::detect_core_type(&startup_path)
    };
    let core_type = startup.selected_core_type.unwrap_or(detected_core_type);
    let mc_version = startup
        .selected_mc_version
        .unwrap_or_else(|| "unknown".to_string());

    ServerInstance::new(
        id,
        server_name,
        core_type,
        String::new(),
        mc_version,
        run_dir.to_string_lossy().to_string(),
        startup_path,
        startup.startup_mode,
        startup.custom_command,
        req.java_path,
        Vec::new(),
        port,
        super::super::super::common::current_timestamp_secs(),
        None,
    )
}
