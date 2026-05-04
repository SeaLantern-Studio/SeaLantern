use std::path::Path;

use crate::models::server::ImportModpackRequest;

use super::super::super::common::current_timestamp_secs;
use super::super::super::fs::{upsert_run_path_mapping, RunPathServerMapping};
use super::startup::ModpackStartupSelection;

pub(super) fn save_modpack_run_mapping(
    data_dir: &str,
    id: &str,
    server_name: &str,
    req: &ImportModpackRequest,
    run_dir: &Path,
    startup: &ModpackStartupSelection,
) -> Result<(), String> {
    upsert_run_path_mapping(
        data_dir,
        RunPathServerMapping {
            run_path: run_dir.to_string_lossy().to_string(),
            server_id: id.to_string(),
            server_name: server_name.to_string(),
            startup_mode: startup.startup_mode.clone(),
            startup_file_path: startup.startup_file_path.clone(),
            custom_command: startup.custom_command.clone(),
            source_modpack_path: req.modpack_path.clone(),
            updated_at: current_timestamp_secs(),
        },
    )
}
