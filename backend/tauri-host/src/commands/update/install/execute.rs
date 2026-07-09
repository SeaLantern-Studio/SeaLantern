use std::path::PathBuf;
use std::sync::atomic::Ordering;

use crate::commands::update::common::{update_t, update_t1};
use crate::services;

use super::INSTALL_IN_PROGRESS;
#[cfg(target_os = "linux")]
use update::arch::{get_aur_helper, is_arch_linux};
use update::install_support::get_pending_update_file;
#[cfg(target_os = "windows")]
use update::install_support::{build_install_launch_plan, InstallLaunchPlan};
use update::pending::write_pending_update;
#[cfg(target_os = "windows")]
use update::windows_install;

pub(crate) async fn execute_install(file_path: String, version: String) -> Result<(), String> {
    if INSTALL_IN_PROGRESS.swap(true, Ordering::SeqCst) {
        return Err(update_t("update.install.in_progress"));
    }

    let result = (|| -> Result<(), String> {
        let path = PathBuf::from(&file_path);
        if !path.exists() {
            return Err(update_t1("update.install.file_not_found", file_path.clone()));
        }

        #[cfg(target_os = "linux")]
        {
            if is_arch_linux() {
                let helper = get_aur_helper().unwrap_or_else(|| "yay".to_string());
                return Err(update_t1("update.install.arch_linux_manual_upgrade", helper));
            }
        }

        let settings = services::global::settings_manager().get();
        if settings.close_servers_on_update {
            services::global::server_manager().stop_all_servers_checked()?;
        }

        let pending_file = get_pending_update_file();
        write_pending_update(&pending_file, &file_path, version)?;
        launch_update_installer(&path, &file_path, &pending_file)?;

        Ok(())
    })();

    if result.is_err() {
        INSTALL_IN_PROGRESS.store(false, Ordering::SeqCst);
        std::fs::remove_file(get_pending_update_file()).ok();
    }

    result
}

fn launch_update_installer(
    path: &std::path::Path,
    file_path: &str,
    pending_file: &std::path::Path,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let pending_file_path = pending_file.to_string_lossy().to_string();
        match build_install_launch_plan(path, file_path) {
            InstallLaunchPlan::ElevatedMsi { program, args } => {
                let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
                windows_install::spawn_elevated_windows_process(
                    program,
                    &arg_refs,
                    Some(file_path),
                    Some(pending_file_path.as_str()),
                )
            }
            InstallLaunchPlan::ElevatedExe { program, args } => {
                let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
                windows_install::spawn_elevated_windows_process(
                    &program,
                    &arg_refs,
                    Some(file_path),
                    Some(pending_file_path.as_str()),
                )
            }
            InstallLaunchPlan::OpenDirect => opener::open(file_path)
                .map_err(|e| update_t1("update.install.open_file_failed", e.to_string())),
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = pending_file;
        let _ = path;
        opener::open(file_path)
            .map_err(|e| update_t1("update.install.open_file_failed", e.to_string()))
    }

    #[cfg(all(target_os = "linux", not(target_os = "windows")))]
    {
        let _ = pending_file;
        let _ = path;
        opener::open(file_path)
            .map_err(|e| update_t1("update.install.open_file_failed", e.to_string()))
    }
}
