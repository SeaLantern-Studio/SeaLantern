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

pub use copy::{CopyInstanceError, CopyInstancePlan, CopyInstanceRequest, plan_copy};
pub use core_parsing::{
    CoreFileInfo, CoreKind, CoreParseError, inspect_core_file, inspect_core_filename,
};
pub use create::{CreateInstanceError, CreateInstancePlan, plan_create};
pub use existing::{ExistingInstanceError, plan_existing_instance};
pub use import_metadata::{
    ImportLaunchCandidate, LaunchProfilePolicy, ServerInspectionProjection,
    ServerInspectionProjectionOptions, apply_server_inspection,
    apply_server_inspection_with_options, inspect_and_apply_import_metadata,
};
pub use modpack::{
    ModpackProvisionError, ModpackProvisionPlan, ModpackProvisionRequest, plan_modpack,
};
pub use run_dir::{RunDirectoryError, RunDirectoryState, resolve_run_directory};
pub use server_inspection::{
    InspectionOptions, ServerInspectionError, ServerInspectionReport, inspect_server_artifact,
};
pub use startup_parsing::{
    JavaLaunch, StartupParseError, StartupScriptInfo, StartupScriptKind,
    parse_startup_script_content, parse_startup_script_file,
};
