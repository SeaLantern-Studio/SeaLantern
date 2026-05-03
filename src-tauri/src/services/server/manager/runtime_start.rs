mod launch;
mod preload;

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::common::normalize_startup_mode;
use super::{ServerManager, StartFallbackInfo, StartServerReport};
use crate::services::server::log_pipeline as server_log_pipeline;

pub(super) fn start_server(
    manager: &ServerManager,
    id: &str,
) -> Result<StartServerReport, String> {
    let server = {
        let servers = manager.lock_servers()?;
        servers
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| format!("未找到服务器: {}", id))?
            .clone()
    };

    println!(
        "准备启动服务器: id={}, name={}, startup_mode={}, startup_path={}, java_path={}",
        server.id, server.name, server.startup_mode, server.jar_path, server.java_path
    );

    {
        let mut procs = manager.lock_processes()?;
        if let Some(child) = procs.get_mut(id) {
            match child.try_wait() {
                Ok(Some(_)) => {
                    procs.remove(id);
                    server_log_pipeline::shutdown_writer(id);
                }
                Ok(None) => return Err(format!("服务器已在运行中: {}", id)),
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

    let startup_mode = normalize_startup_mode(&server.startup_mode);
    let startup_path_obj = std::path::Path::new(&server.jar_path);
    let managed_console_encoding =
        launch::context::resolve_managed_encoding(startup_mode, startup_path_obj);
    let (java_bin_dir_str, java_home_dir_str) =
        launch::context::resolve_java_paths(&server.java_path)?;
    let startup_filename = launch::context::startup_filename(&server.jar_path);
    let starter_installer_url = launch::context::resolve_starter_installer_url(id, &server)?;
    let launch_context = launch::context::LaunchContext {
        manager,
        server: &server,
        settings: &settings,
        startup_mode,
        managed_console_encoding,
        java_bin_dir_str,
        java_home_dir_str,
        startup_filename,
        starter_installer_url,
    };

    server_log_pipeline::init_db(Path::new(&server.path))?;
    let launch_plan = launch::runner::launch_server_process(id, launch_context)?;

    let mut child = launch_plan.child;
    let fallback_info: Option<StartFallbackInfo> = launch_plan.fallback_info;

    println!("Java进程已启动，PID: {:?}", child.id());

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    manager.lock_processes()?.insert(id.to_string(), child);
    manager.mark_starting(id);

    {
        let mut servers = manager.lock_servers()?;
        if let Some(s) = servers.iter_mut().find(|s| s.id == id) {
            s.last_started_at = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| format!("获取当前时间失败: {}", e))?
                    .as_secs(),
            );
        }
    }
    manager.save()?;
    let _ = server_log_pipeline::append_sealantern_log(id, "[Sea Lantern] 服务器启动中...");

    if let Some(stdout) = stdout {
        server_log_pipeline::spawn_server_output_reader(id.to_string(), stdout);
    }
    if let Some(stderr) = stderr {
        server_log_pipeline::spawn_server_output_reader(id.to_string(), stderr);
    }

    Ok(StartServerReport {
        server_id: server.id.clone(),
        server_name: server.name.clone(),
        fallback: fallback_info,
    })
}
