use super::command_builder::{
    build_configured_command, build_direct_jar_command, find_preferred_jar_path,
};
use super::context::LaunchContext;
use crate::models::server::ServerInstance;
use crate::services::server::log_pipeline as server_log_pipeline;
use crate::services::server::manager::common::StartupMode;
use crate::services::server::manager::cpu_policy;
use sea_lantern_server_local_setup_core::{
    build_primary_jar_fallback_info, format_fallback_chain_error,
    format_launch_fallback_log, format_primary_jar_early_exit_reason,
    format_primary_jar_probe_error_reason, format_primary_jar_spawn_error_reason,
};
use std::process::{Command, Stdio};

pub(in crate::services::server::manager::runtime_start) struct LaunchPlan {
    pub child: std::process::Child,
    pub fallback_info: Option<super::super::super::StartFallbackInfo>,
}

pub(in crate::services::server::manager::runtime_start) fn launch_server_process(
    id: &str,
    context: LaunchContext<'_>,
) -> Result<LaunchPlan, String> {
    let configured_mode = context.startup_mode.as_str().to_string();
    let preferred_jar_path = find_preferred_jar_path(&context);
    let mut fallback_info: Option<super::super::super::StartFallbackInfo> = None;

    let child = if let Some(jar_path) = preferred_jar_path {
        match spawn_command(
            id,
            context.server,
            build_direct_jar_command(&context, &jar_path, None)?,
            "优先 JAR 直启",
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
                            "回退脚本/配置模式",
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
                            "回退脚本/配置模式",
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
                    "回退脚本/配置模式",
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
        spawn_command(id, context.server, command, "配置模式", context.startup_mode)?
    };

    Ok(LaunchPlan { child, fallback_info })
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
        &format!("[Sea Lantern] {}启动命令: {}", phase, command_for_log),
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

    let child = cmd
        .spawn()
        .map_err(|e| format!("启动失败（id={}, path={}）: {}", id, server.path, e))?;

    let settings = crate::services::global::settings_manager().get();
    apply_cpu_policy_after_spawn(id, server, &settings, startup_mode, child.id())?;
    Ok(child)
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

    cpu_policy::apply_cpu_affinity_to_pid(pid, &resolved)
        .map_err(|e| format!("启动成功但应用 CPU affinity 失败（pid={}）: {}", pid, e))?;

    let _ = server_log_pipeline::append_sealantern_log(
        id,
        &format!(
            "[Sea Lantern] CPU affinity applied: mode={} cpuset={} pid={}",
            cpu_policy::local_cpu_policy(server, settings).mode.as_str(),
            resolved.cpuset_display,
            pid
        ),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use sea_lantern_server_local_setup_core::{
        build_primary_jar_fallback_info, format_fallback_chain_error,
        format_launch_fallback_log, format_primary_jar_early_exit_reason,
        format_primary_jar_probe_error_reason, format_primary_jar_spawn_error_reason,
    };

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
