pub mod copy;
pub mod core_parsing;
pub mod create;
pub mod existing;
pub mod import_metadata;
mod launch_adapter;
pub mod modpack;
pub mod run_dir;
#[path = "server_inspection/lib.rs"]
pub mod server_inspection;
pub mod startup_parsing;

pub use copy::{plan_copy, CopyInstanceError, CopyInstancePlan, CopyInstanceRequest};
pub use core_parsing::{
    inspect_core_file, inspect_core_filename, CoreFileInfo, CoreKind, CoreParseError,
};
pub use create::{plan_create, CreateInstanceError, CreateInstancePlan};
pub use existing::{plan_existing_instance, ExistingInstanceError};
pub use import_metadata::{
    apply_server_inspection, apply_server_inspection_with_options,
    inspect_and_apply_import_metadata, ImportLaunchCandidate, LaunchProfilePolicy,
    ServerInspectionProjection, ServerInspectionProjectionOptions,
};
pub use modpack::{
    plan_modpack, ModpackProvisionError, ModpackProvisionPlan, ModpackProvisionRequest,
};
pub use run_dir::{resolve_run_directory, RunDirectoryError, RunDirectoryState};
pub use server_inspection::{
    inspect_server_artifact, InspectionOptions, ServerInspectionError, ServerInspectionReport,
};
pub use startup_parsing::{
    parse_startup_script_content, parse_startup_script_file, JavaLaunch, StartupParseError,
    StartupScriptInfo, StartupScriptKind,
};
