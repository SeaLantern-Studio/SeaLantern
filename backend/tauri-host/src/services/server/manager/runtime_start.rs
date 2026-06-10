mod launch;
mod local_launch_detail;
mod preload;

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::common::StartupMode;
use super::i18n::{manager_t, manager_t1, manager_t5};
use super::ServerManager;
use crate::services::server::log_pipeline as server_log_pipeline;
use crate::services::server::runtime::{
    RuntimeProcessHandle, RuntimeStartRequest, RuntimeStartResult,
};
use sea_lantern_server_local_setup_core::{
    resolve_java_paths, resolve_managed_console_encoding, startup_filename,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalLaunchDetail {
    pub startup_mode: String,
    pub java_path: String,
    pub launch_target: String,
    pub effective_max_memory: u32,
    pub effective_min_memory: u32,
    pub effective_cpu_policy_mode: String,
    pub effective_jvm_preset: String,
    pub effective_jvm_args: Vec<String>,
    pub command_preview: String,
}

pub(crate) fn get_local_launch_detail(
    server: &crate::models::server::ServerInstance,
    settings: &crate::models::settings::AppSettings,
) -> Result<LocalLaunchDetail, String> {
    local_launch_detail::build_local_launch_detail(server, settings)
}

pub(crate) fn start_local_runtime(
    manager: &ServerManager,
    request: RuntimeStartRequest<'_>,
) -> Result<RuntimeStartResult, String> {
    let id = request.server_id;
    let server = request.server;

    let startup_mode_raw = server.startup_mode_str().to_string();
    let startup_path = server.jar_path().unwrap_or_default().to_string();
    let java_path = server.java_path().unwrap_or_default().to_string();

    println!(
        "{}",
        manager_t5(
            "server.manager.start_prepare_details",
            server.id.clone(),
            server.name.clone(),
            startup_mode_raw.clone(),
            startup_path.clone(),
            java_path.clone(),
        )
    );

    {
        let mut procs = manager.lock_processes()?;
        if let Some(child) = procs.get_mut(id) {
            match child.try_wait() {
                Ok(Some(_)) => {
                    procs.remove(id);
                    server_log_pipeline::shutdown_writer(id);
                }
                Ok(None) => {
                    return Err(manager_t1("server.manager.server_already_running", id.to_string()))
                }
                Err(_) => {
                    procs.remove(id);
                    server_log_pipeline::shutdown_writer(id);
                }
            }
        }
    }

    let settings = manager.get_app_settings();
    if settings.auto_accept_eula {
        let eula = std::path::Path::new(&server.path).join("eula.txt");
        let _ = std::fs::write(&eula, "# Auto-accepted by Sea Lantern\neula=true\n");
    }

    preload::run_preload_script(id, &server.path);

    let startup_mode = StartupMode::from_raw(server.startup_mode_str());
    let startup_path_obj = std::path::Path::new(startup_path.as_str());
    let managed_console_encoding =
        resolve_managed_console_encoding(startup_mode.as_str(), startup_path_obj);
    let (java_bin_dir_str, java_home_dir_str) = resolve_java_paths(java_path.as_str())?;
    let startup_filename = startup_filename(startup_path.as_str());
    let starter_core_key = launch::context::resolve_starter_core_key(server)?;
    let launch_context = launch::context::LaunchContext {
        server,
        settings: &settings,
        startup_mode,
        managed_console_encoding,
        java_bin_dir_str,
        java_home_dir_str,
        startup_filename,
        starter_core_key,
    };

    server_log_pipeline::init_db(Path::new(&server.path))?;
    let launch_plan = launch::runner::launch_server_process(id, launch_context)?;

    let _ = server_log_pipeline::append_sealantern_log(
        id,
        &manager_t("server.manager.server_starting_log"),
    );

    Ok(RuntimeStartResult {
        process_handle: Some(RuntimeProcessHandle::LocalChild(launch_plan.child)),
        fallback: launch_plan.fallback_info,
    })
}
