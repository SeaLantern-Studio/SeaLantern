use super::command_builder::{
    build_configured_command, build_direct_jar_command, build_starter_install_command,
    find_preferred_jar_path,
};
use super::context::LaunchContext;
use crate::models::server::ServerInstance;
use crate::services::server::log_pipeline as server_log_pipeline;
use crate::services::server::manager::common::StartupMode;
use crate::services::server::manager::cpu_policy;
use crate::services::server::manager::i18n::{manager_t, manager_t1, manager_t2, manager_t3};
use sea_lantern_server_local_setup_core::inspect_local_folder_checked;
use sea_lantern_server_local_setup_core::{
    build_primary_jar_fallback_info, format_fallback_chain_error, format_launch_fallback_log,
    format_primary_jar_early_exit_reason, format_primary_jar_probe_error_reason,
    format_primary_jar_spawn_error_reason,
};
use std::process::{Command, Stdio};

struct CompletedCommandOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

pub(in crate::services::server::manager::runtime_start) struct LaunchPlan {
    pub child: std::process::Child,
    pub fallback_info: Option<super::super::super::StartFallbackInfo>,
}

pub(in crate::services::server::manager::runtime_start) fn launch_server_process(
    id: &str,
    context: LaunchContext<'_>,
) -> Result<LaunchPlan, String> {
    if matches!(context.startup_mode, StartupMode::Starter) {
        let child = install_and_launch_starter(id, &context)?;
        return Ok(LaunchPlan { child, fallback_info: None });
    }

    let configured_mode = context.startup_mode.as_str().to_string();
    let preferred_jar_path = find_preferred_jar_path(&context);
    let mut fallback_info: Option<super::super::super::StartFallbackInfo> = None;

    let child = if let Some(jar_path) = preferred_jar_path {
        let preferred_phase = manager_t("server.manager.launch_phase_preferred_direct_jar");
        let fallback_phase = manager_t("server.manager.launch_phase_fallback_configured");
        match spawn_command(
            id,
            context.server,
            build_direct_jar_command(&context, &jar_path, None)?,
            &preferred_phase,
            StartupMode::Jar,
        ) {
            Ok(mut primary_child) => {
                const PRIMARY_LAUNCH_PROBE_DELAY_MS: u64 = 800;
                std::thread::sleep(std::time::Duration::from_millis(PRIMARY_LAUNCH_PROBE_DELAY_MS));
                match primary_child.try_wait() {
                    Ok(None) => primary_child,
                    Ok(Some(status)) => {
                        let fallback = build_primary_jar_fallback_info(
                            &configured_mode,
                            format_primary_jar_early_exit_reason(&status.to_string()),
                        );
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            &format_launch_fallback_log(&fallback.reason, &fallback.to_mode),
                        );
                        let fallback_cmd = build_configured_command(&context)?;
                        let fallback_child = spawn_command(
                            id,
                            context.server,
                            fallback_cmd,
                            &fallback_phase,
                            context.startup_mode,
                        )?;
                        fallback_info = Some(super::super::super::StartFallbackInfo {
                            from_mode: fallback.from_mode,
                            to_mode: fallback.to_mode,
                            reason: fallback.reason,
                        });
                        fallback_child
                    }
                    Err(error) => {
                        let fallback = build_primary_jar_fallback_info(
                            &configured_mode,
                            format_primary_jar_probe_error_reason(&error.to_string()),
                        );
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            &format_launch_fallback_log(&fallback.reason, &fallback.to_mode),
                        );
                        let fallback_cmd = build_configured_command(&context)?;
                        let fallback_child = spawn_command(
                            id,
                            context.server,
                            fallback_cmd,
                            &fallback_phase,
                            context.startup_mode,
                        )?;
                        fallback_info = Some(super::super::super::StartFallbackInfo {
                            from_mode: fallback.from_mode,
                            to_mode: fallback.to_mode,
                            reason: fallback.reason,
                        });
                        fallback_child
                    }
                }
            }
            Err(primary_error) => {
                let fallback = build_primary_jar_fallback_info(
                    &configured_mode,
                    format_primary_jar_spawn_error_reason(&primary_error),
                );
                let _ = server_log_pipeline::append_sealantern_log(
                    id,
                    &format_launch_fallback_log(&fallback.reason, &fallback.to_mode),
                );
                let fallback_cmd = build_configured_command(&context)?;
                let fallback_child = spawn_command(
                    id,
                    context.server,
                    fallback_cmd,
                    &fallback_phase,
                    context.startup_mode,
                )
                .map_err(|fallback_error| {
                    format_fallback_chain_error(&fallback.reason, &fallback_error)
                })?;
                fallback_info = Some(super::super::super::StartFallbackInfo {
                    from_mode: fallback.from_mode,
                    to_mode: fallback.to_mode,
                    reason: fallback.reason,
                });
                fallback_child
            }
        }
    } else {
        let command = build_configured_command(&context)?;
        let configured_phase = manager_t("server.manager.launch_phase_configured");
        spawn_command(id, context.server, command, &configured_phase, context.startup_mode)?
    };

    Ok(LaunchPlan { child, fallback_info })
}

fn install_and_launch_starter(
    id: &str,
    context: &LaunchContext<'_>,
) -> Result<std::process::Child, String> {
    let install_phase = manager_t("server.manager.launch_phase_starter_install");
    let install_result = spawn_command_and_capture(
        id,
        context.server,
        build_starter_install_command(context)?,
        &install_phase,
    )?;

    append_captured_output_logs(id, &install_phase, &install_result);

    if !install_result.status.success() {
        return Err(manager_t2(
            "server.manager.starter_install_failed_exit",
            install_result
                .status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "terminated".to_string()),
            context.server.path.clone(),
        ));
    }

    let inspection = inspect_local_folder_checked(std::path::Path::new(&context.server.path))?;
    let startup_entry = inspection.startup_entry_path.ok_or_else(|| {
        manager_t1("server.manager.starter_install_missing_script", context.server.path.clone())
    })?;
    let startup_mode = inspection
        .startup_mode
        .as_deref()
        .map(StartupMode::from_raw)
        .unwrap_or(StartupMode::Jar);
    let startup_filename = std::path::Path::new(&startup_entry)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or(startup_entry.clone());

    let launch_phase = manager_t("server.manager.launch_phase_starter_script");
    let command = match startup_mode {
        StartupMode::Bat => {
            #[cfg(target_os = "windows")]
            {
                super::script_launch_support::build_windows_bat_command(
                    &startup_filename,
                    context.managed_console_encoding,
                    &context.java_home_dir_str,
                    &context.java_bin_dir_str,
                )
            }
            #[cfg(not(target_os = "windows"))]
            {
                return Err(manager_t("server.manager.launch_bat_only_windows"));
            }
        }
        StartupMode::Sh => {
            let mut sh_cmd = std::process::Command::new("sh");
            sh_cmd.arg(&startup_filename);
            sh_cmd.arg("nogui");
            super::script_launch_support::apply_java_process_env(
                &mut sh_cmd,
                &context.java_home_dir_str,
                &context.java_bin_dir_str,
            );
            sh_cmd
        }
        StartupMode::Ps1 => {
            #[cfg(target_os = "windows")]
            {
                let mut ps_cmd = std::process::Command::new("powershell");
                ps_cmd.arg("-NoProfile");
                ps_cmd.arg("-NonInteractive");
                ps_cmd.arg("-ExecutionPolicy");
                ps_cmd.arg("Bypass");
                ps_cmd.arg("-File");
                ps_cmd.arg(&startup_filename);
                ps_cmd.arg("nogui");
                super::script_launch_support::apply_java_process_env(
                    &mut ps_cmd,
                    &context.java_home_dir_str,
                    &context.java_bin_dir_str,
                );
                ps_cmd
            }
            #[cfg(not(target_os = "windows"))]
            {
                return Err(manager_t("server.manager.launch_ps1_only_windows"));
            }
        }
        _ => {
            return Err(manager_t1(
                "server.manager.starter_install_missing_script",
                context.server.path.clone(),
            ));
        }
    };

    spawn_command(id, context.server, command, &launch_phase, startup_mode)
}

fn spawn_command(
    id: &str,
    server: &ServerInstance,
    mut cmd: Command,
    phase: &str,
    startup_mode: StartupMode,
) -> Result<std::process::Child, String> {
    let command_for_log = super::super::super::common::format_command_for_log(&cmd);
    let _ = server_log_pipeline::append_sealantern_log(
        id,
        &manager_t2("server.manager.launch_command_log", phase.to_string(), command_for_log),
    );

    cmd.current_dir(&server.path);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.stdin(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let child = cmd.spawn().map_err(|e| {
        manager_t3(
            "server.manager.launch_spawn_failed",
            id.to_string(),
            server.path.clone(),
            e.to_string(),
        )
    })?;

    let settings = crate::services::global::settings_manager().get();
    apply_cpu_policy_after_spawn(id, server, &settings, startup_mode, child.id())?;
    Ok(child)
}

fn spawn_command_and_capture(
    id: &str,
    server: &ServerInstance,
    mut cmd: Command,
    phase: &str,
) -> Result<CompletedCommandOutput, String> {
    let command_for_log = super::super::super::common::format_command_for_log(&cmd);
    let _ = server_log_pipeline::append_sealantern_log(
        id,
        &manager_t2("server.manager.launch_command_log", phase.to_string(), command_for_log),
    );

    cmd.current_dir(&server.path);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.stdin(Stdio::null());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd.output().map_err(|e| {
        manager_t3(
            "server.manager.launch_spawn_failed",
            id.to_string(),
            server.path.clone(),
            e.to_string(),
        )
    })?;

    Ok(CompletedCommandOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn append_captured_output_logs(server_id: &str, phase: &str, output: &CompletedCommandOutput) {
    append_output_block(server_id, phase, "stdout", &output.stdout);
    append_output_block(server_id, phase, "stderr", &output.stderr);
}

fn append_output_block(server_id: &str, phase: &str, stream: &str, content: &str) {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return;
    }

    for line in format_output_block_lines(phase, stream, trimmed) {
        let _ = server_log_pipeline::append_sealantern_log(server_id, &line);
    }
}

fn format_output_block_lines(phase: &str, stream: &str, content: &str) -> Vec<String> {
    content
        .lines()
        .map(|line| format!("[Sea Lantern] {} {}: {}", phase, stream, line.trim_end()))
        .collect()
}

fn apply_cpu_policy_after_spawn(
    id: &str,
    server: &ServerInstance,
    settings: &crate::models::settings::AppSettings,
    startup_mode: StartupMode,
    pid: u32,
) -> Result<(), String> {
    if !cpu_policy::mode_supports_local_cpu_policy(startup_mode) {
        return Ok(());
    }

    let Some(resolved) = cpu_policy::resolve_local_cpu_policy(server, settings)? else {
        return Ok(());
    };
    let policy = cpu_policy::local_cpu_policy_checked(server, settings)?;

    cpu_policy::apply_cpu_affinity_to_pid(pid, &resolved).map_err(|e| {
        manager_t2("server.manager.launch_cpu_affinity_apply_failed", pid.to_string(), e)
    })?;

    let _ = server_log_pipeline::append_sealantern_log(
        id,
        &manager_t3(
            "server.manager.launch_cpu_affinity_applied_log",
            policy.mode.as_str().to_string(),
            resolved.cpuset_display.clone(),
            pid.to_string(),
        ),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::format_output_block_lines;
    use sea_lantern_server_local_setup_core::{
        build_primary_jar_fallback_info, format_fallback_chain_error, format_launch_fallback_log,
        format_primary_jar_early_exit_reason, format_primary_jar_probe_error_reason,
        format_primary_jar_spawn_error_reason,
    };

    #[test]
    fn format_output_block_lines_preserves_each_installer_output_line() {
        let lines = format_output_block_lines(
            "先执行安装器",
            "stdout",
            "usage: installer --help\r\nsecond line",
        );

        assert_eq!(
            lines,
            vec![
                "[Sea Lantern] 先执行安装器 stdout: usage: installer --help".to_string(),
                "[Sea Lantern] 先执行安装器 stdout: second line".to_string(),
            ]
        );
    }

    #[test]
    fn primary_jar_fallback_reason_chain_is_stable() {
        let early_exit = format_primary_jar_early_exit_reason("exit status: 1");
        let probe_error = format_primary_jar_probe_error_reason("io error");
        let spawn_error = format_primary_jar_spawn_error_reason("spawn failed");
        let fallback = build_primary_jar_fallback_info("sh", early_exit.clone());

        assert_eq!(early_exit, "JAR 直启进程过早退出: exit status: 1");
        assert_eq!(probe_error, "JAR 直启状态检查失败: io error");
        assert_eq!(spawn_error, "JAR 直启失败: spawn failed");
        assert_eq!(fallback.from_mode, "jar");
        assert_eq!(fallback.to_mode, "sh");
        assert_eq!(
            format_launch_fallback_log(&fallback.reason, &fallback.to_mode),
            "[Sea Lantern] JAR 直启进程过早退出: exit status: 1，回退到 sh 启动"
        );
        assert_eq!(
            format_fallback_chain_error(&spawn_error, "fallback failed"),
            "JAR 直启失败: spawn failed；回退也失败：fallback failed"
        );
    }
}
