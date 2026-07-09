use std::path::Path;

use crate::models::server::ImportModpackRequest;
use crate::services::server::manager::provisioning::i18n::{provisioning_t, provisioning_t1};
use server_local_setup::{
    resolve_modpack_run_dir_startup_selection as resolve_shared_modpack_run_dir_startup_selection,
    ModpackStartupSelection, ResolveModpackStartupError,
};

pub(super) fn resolve_modpack_startup_selection(
    req: &ImportModpackRequest,
    source_path: &Path,
    run_dir: &Path,
) -> Result<ModpackStartupSelection, String> {
    resolve_shared_modpack_run_dir_startup_selection(
        source_path,
        run_dir,
        &req.startup_mode,
        req.custom_command.as_deref(),
        req.startup_file_path.as_deref(),
        req.core_type.as_deref(),
        req.mc_version.as_deref(),
    )
    .map_err(|error| match error {
        ResolveModpackStartupError::StartupFilePathMissing => {
            provisioning_t("server.provisioning.startup_file_path_missing")
        }
        ResolveModpackStartupError::StartupFileMissing(path) => {
            provisioning_t1("server.provisioning.startup_file_missing", path)
        }
        ResolveModpackStartupError::CustomCommandEmpty => {
            provisioning_t("server.provisioning.custom_command_empty")
        }
    })
}
