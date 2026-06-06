use crate::services::server::log_pipeline as server_log_pipeline;
#[cfg(target_os = "windows")]
use crate::services::server::manager::common::build_windows_cmd_command;
use crate::services::server::manager::i18n::{manager_t, manager_t1};

pub(super) fn run_preload_script(id: &str, server_path: &str) {
    #[cfg(target_os = "windows")]
    {
        let preload_script = std::path::Path::new(server_path).join("preload.bat");
        if preload_script.exists() {
            println!(
                "{}",
                manager_t1(
                    "server.manager.preload_script_found",
                    preload_script.display().to_string(),
                )
            );
            let _ = server_log_pipeline::append_sealantern_log(
                id,
                &manager_t("server.manager.preload_start_log"),
            );

            let preload_script_name = preload_script
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("preload.bat");
            let launch_command = format!("call \"{}\"", preload_script_name.replace('"', "\"\""));

            let mut cmd = build_windows_cmd_command(&launch_command);
            cmd.current_dir(server_path);

            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);

            match cmd.output() {
                Ok(result) => {
                    if result.status.success() {
                        println!("{}", manager_t("server.manager.preload_bat_success"));
                        if !result.stdout.is_empty() {
                            let output = String::from_utf8_lossy(&result.stdout);
                            for line in output.lines() {
                                let _ = server_log_pipeline::append_sealantern_log(
                                    id,
                                    &format!("[preload] {}", line),
                                );
                            }
                        }
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            &manager_t("server.manager.preload_success_log"),
                        );
                    } else {
                        let error = String::from_utf8_lossy(&result.stderr);
                        println!(
                            "{}",
                            manager_t1("server.manager.preload_bat_failed", error.to_string())
                        );
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            &manager_t1(
                                "server.manager.preload_exec_failed_log",
                                error.to_string(),
                            ),
                        );
                    }
                }
                Err(e) => {
                    let error_msg =
                        manager_t1("server.manager.preload_bat_exec_failed", e.to_string());
                    println!("{}", error_msg);
                    let _ = server_log_pipeline::append_sealantern_log(
                        id,
                        &manager_t1("server.manager.preload_log_line", error_msg),
                    );
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let preload_script = std::path::Path::new(server_path).join("preload.sh");
        if preload_script.exists() {
            println!(
                "{}",
                manager_t1(
                    "server.manager.preload_script_found",
                    preload_script.display().to_string(),
                )
            );
            let _ = server_log_pipeline::append_sealantern_log(
                id,
                &manager_t("server.manager.preload_start_log"),
            );

            match std::process::Command::new("sh")
                .arg(&preload_script)
                .current_dir(server_path)
                .output()
            {
                Ok(result) => {
                    if result.status.success() {
                        println!("{}", manager_t("server.manager.preload_sh_success"));
                        if !result.stdout.is_empty() {
                            let output = String::from_utf8_lossy(&result.stdout);
                            for line in output.lines() {
                                let _ = server_log_pipeline::append_sealantern_log(
                                    id,
                                    &format!("[preload] {}", line),
                                );
                            }
                        }
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            &manager_t("server.manager.preload_success_log"),
                        );
                    } else {
                        let error = String::from_utf8_lossy(&result.stderr);
                        println!(
                            "{}",
                            manager_t1("server.manager.preload_sh_failed", error.to_string())
                        );
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            &manager_t1(
                                "server.manager.preload_exec_failed_log",
                                error.to_string(),
                            ),
                        );
                    }
                }
                Err(e) => {
                    let error_msg =
                        manager_t1("server.manager.preload_sh_exec_failed", e.to_string());
                    println!("{}", error_msg);
                    let _ = server_log_pipeline::append_sealantern_log(
                        id,
                        &manager_t1("server.manager.preload_log_line", error_msg),
                    );
                }
            }
        }
    }
}
