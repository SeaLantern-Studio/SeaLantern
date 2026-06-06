#[cfg(target_os = "windows")]
use super::super::super::common::build_windows_cmd_command;
use super::super::super::startup_support;
use super::context::LaunchContext;
use super::script_launch_support;
use crate::services::server::manager::common::StartupMode;
use sea_lantern_server_local_setup_core::{
    resolve_direct_jar_launch_target, resolve_local_preferred_jar_path,
};
use std::path::Path;
use std::process::Command;

/// 优先寻找可直接运行的 JAR 文件
pub(crate) fn find_preferred_jar_path(context: &LaunchContext<'_>) -> Option<String> {
    resolve_local_preferred_jar_path(
        context.startup_mode.as_str(),
        context.server.jar_path(),
        Path::new(&context.server.path),
    )
}

/// 构建直接运行 JAR 的命令
pub(crate) fn build_direct_jar_command(
    context: &LaunchContext<'_>,
    jar_path: &str,
    installer_url: Option<&str>,
) -> Result<Command, String> {
    let launch_target = resolve_direct_jar_launch_target(&context.server.path, jar_path);

    let mut java_cmd = Command::new(
        context
            .server
            .java_path()
            .expect("local runtime launch requires java_path"),
    );
    for arg in startup_support::build_managed_jvm_args(
        context.server,
        context.settings,
        context.managed_console_encoding,
    )? {
        java_cmd.arg(arg);
    }
    java_cmd.arg("-jar");
    java_cmd.arg(launch_target);
    java_cmd.arg("nogui");
    if let Some(url) = installer_url {
        java_cmd.arg("--installer");
        java_cmd.arg(url);
    }
    Ok(java_cmd)
}

/// 按启动方式构建最终命令
pub(crate) fn build_configured_command(context: &LaunchContext<'_>) -> Result<Command, String> {
    match context.startup_mode {
        StartupMode::Custom => build_custom_command(context),
        StartupMode::Bat => build_bat_command(context),
        StartupMode::Sh => build_sh_command(context),
        StartupMode::Ps1 => build_ps1_command(context),
        StartupMode::Starter => {
            let installer_url = context
                .starter_installer_url
                .as_deref()
                .ok_or_else(|| "Starter 安装器下载链接为空".to_string())?;
            build_direct_jar_command(
                context,
                context
                    .server
                    .jar_path()
                    .expect("starter launch requires jar_path"),
                Some(installer_url),
            )
        }
        StartupMode::Jar => build_direct_jar_command(
            context,
            context
                .server
                .jar_path()
                .expect("jar launch requires jar_path"),
            None,
        ),
    }
}

/// 构建自定义命令启动方式
fn build_custom_command(context: &LaunchContext<'_>) -> Result<Command, String> {
    let custom_command = context
        .server
        .custom_command()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "自定义启动命令为空".to_string())?;

    #[cfg(target_os = "windows")]
    {
        let mut custom_cmd = build_windows_cmd_command(custom_command);
        script_launch_support::apply_java_process_env(
            &mut custom_cmd,
            &context.java_home_dir_str,
            &context.java_bin_dir_str,
        );
        Ok(custom_cmd)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut custom_cmd = Command::new("sh");
        custom_cmd.arg("-c");
        custom_cmd.arg(custom_command);
        script_launch_support::apply_java_process_env(
            &mut custom_cmd,
            &context.java_home_dir_str,
            &context.java_bin_dir_str,
        );
        Ok(custom_cmd)
    }
}

/// 构建 BAT 启动命令
fn build_bat_command(context: &LaunchContext<'_>) -> Result<Command, String> {
    script_launch_support::prepare_script_startup(context)?;

    #[cfg(target_os = "windows")]
    {
        let bat_cmd = script_launch_support::build_windows_bat_command(
            &context.startup_filename,
            context.managed_console_encoding,
            &context.java_home_dir_str,
            &context.java_bin_dir_str,
        );
        Ok(bat_cmd)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("BAT 启动方式仅支持 Windows".to_string())
    }
}

/// 构建 SH 启动命令
fn build_sh_command(context: &LaunchContext<'_>) -> Result<Command, String> {
    script_launch_support::prepare_script_startup(context)?;
    let mut sh_cmd = Command::new("sh");
    sh_cmd.arg(&context.startup_filename);
    sh_cmd.arg("nogui");
    script_launch_support::apply_script_process_env(&mut sh_cmd, context);
    Ok(sh_cmd)
}

/// 构建 PowerShell 启动命令
fn build_ps1_command(context: &LaunchContext<'_>) -> Result<Command, String> {
    script_launch_support::prepare_script_startup(context)?;
    #[cfg(target_os = "windows")]
    {
        let mut ps_cmd = Command::new("powershell");
        ps_cmd.arg("-NoProfile");
        ps_cmd.arg("-NonInteractive");
        ps_cmd.arg("-ExecutionPolicy");
        ps_cmd.arg("Bypass");
        ps_cmd.arg("-File");
        ps_cmd.arg(&context.startup_filename);
        ps_cmd.arg("nogui");
        script_launch_support::apply_script_process_env(&mut ps_cmd, context);
        Ok(ps_cmd)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("PS1 启动方式仅支持 Windows".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_custom_command, build_ps1_command, build_sh_command,
    };
    use crate::models::server::{
        CpuPolicyConfig, JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
    };
    use crate::models::settings::AppSettings;
    use crate::services::server::manager::common::{ManagedConsoleEncoding, StartupMode};
    use crate::services::server::manager::runtime_start::launch::context::LaunchContext;
    use sea_lantern_server_local_setup_core::resolve_direct_jar_launch_target;
    use std::path::Path;
    use std::process::Command;
    use tempfile::TempDir;

    fn collect_envs(command: &Command) -> Vec<(String, Option<String>)> {
        command
            .get_envs()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().to_string(),
                    value.map(|value| value.to_string_lossy().to_string()),
                )
            })
            .collect()
    }

    fn fake_java_probe_command() -> String {
        #[cfg(target_os = "windows")]
        {
            std::env::var("ComSpec").unwrap_or_else(|_| "cmd".to_string())
        }

        #[cfg(not(target_os = "windows"))]
        {
            "sh".to_string()
        }
    }

    fn test_settings() -> AppSettings {
        AppSettings {
            default_max_memory: 4096,
            default_min_memory: 1024,
            default_jvm_args: vec!["-Dlaunch.test=true".to_string()],
            ..AppSettings::default()
        }
    }

    fn test_server(
        server_dir: &Path,
        startup_mode: &str,
        custom_command: Option<&str>,
    ) -> ServerInstance {
        ServerInstance {
            id: format!("launch-{}", startup_mode),
            name: format!("Launch {}", startup_mode),
            aliases: Vec::new(),
            core_type: "paper".to_string(),
            core_version: "paper".to_string(),
            mc_version: "1.21.1".to_string(),
            path: server_dir.to_string_lossy().to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 1024,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: server_dir.join("server.jar").to_string_lossy().to_string(),
                startup_mode: startup_mode.to_string(),
                custom_command: custom_command.map(str::to_string),
                java_path: fake_java_probe_command(),
                jvm_args: vec!["-Dserver.test=true".to_string()],
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    fn test_launch_context<'a>(
        server: &'a ServerInstance,
        settings: &'a AppSettings,
        startup_mode: StartupMode,
        startup_filename: &str,
    ) -> LaunchContext<'a> {
        LaunchContext {
            server,
            settings,
            startup_mode,
            managed_console_encoding: ManagedConsoleEncoding::Utf8,
            java_bin_dir_str: "C:/Java/JDK 21/bin".to_string(),
            java_home_dir_str: "C:/Java/JDK 21".to_string(),
            startup_filename: startup_filename.to_string(),
            starter_installer_url: None,
        }
    }

    fn assert_java_process_env_is_injected(command: &Command) {
        let envs = collect_envs(command);

        assert!(envs.iter().any(|(key, value)| {
            key == "JAVA_HOME" && value.as_deref() == Some("C:/Java/JDK 21")
        }));
        assert!(envs.iter().any(|(key, value)| {
            key == "PATH"
                && value
                    .as_deref()
                    .is_some_and(|value| value.starts_with("C:/Java/JDK 21/bin"))
        }));
    }

    #[test]
    fn custom_mode_does_not_prefer_direct_jar() {
        assert!(!StartupMode::Custom.prefers_direct_jar());
    }

    #[test]
    fn script_modes_still_prefer_direct_jar() {
        assert!(StartupMode::Bat.prefers_direct_jar());
        assert!(StartupMode::Sh.prefers_direct_jar());
        assert!(StartupMode::Ps1.prefers_direct_jar());
    }

    #[test]
    fn custom_command_keeps_user_shell_text_and_injects_java_via_process_envs() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let settings = test_settings();
        let server = test_server(temp_dir.path(), "custom", Some("echo launch ready"));
        let context = test_launch_context(&server, &settings, StartupMode::Custom, "ignored.bat");

        let command = build_custom_command(&context).expect("custom command should build");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        #[cfg(target_os = "windows")]
        {
            assert_eq!(command.get_program().to_string_lossy(), "cmd");
            assert_eq!(args, vec!["/d", "/c", "echo launch ready"]);
        }

        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(command.get_program().to_string_lossy(), "sh");
            assert_eq!(args, vec!["-c", "echo launch ready"]);
        }

        assert_java_process_env_is_injected(&command);
    }

    #[test]
    fn sh_command_prepares_user_jvm_args_and_injects_java_via_process_envs() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let settings = test_settings();
        let server = test_server(temp_dir.path(), "sh", None);
        let context = test_launch_context(&server, &settings, StartupMode::Sh, "start.sh");

        let command = build_sh_command(&context).expect("sh command should build");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        let user_jvm_args_path = temp_dir.path().join("user_jvm_args.txt");

        assert_eq!(command.get_program().to_string_lossy(), "sh");
        assert_eq!(args, vec!["start.sh", "nogui"]);
        assert_java_process_env_is_injected(&command);
        assert!(user_jvm_args_path.exists());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn ps1_command_uses_process_env_injection_instead_of_inline_cmd_prefix() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let settings = test_settings();
        let server = test_server(temp_dir.path(), "ps1", None);
        let context = test_launch_context(&server, &settings, StartupMode::Ps1, "start.ps1");

        let command = build_ps1_command(&context).expect("ps1 command should build");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert_eq!(command.get_program().to_string_lossy(), "powershell");
        assert_eq!(
            args,
            vec![
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                "start.ps1",
                "nogui",
            ]
        );
        assert!(args
            .iter()
            .all(|arg| !arg.contains("JAVA_HOME") && !arg.contains("PATH=")));
        assert_java_process_env_is_injected(&command);
    }

    #[test]
    fn direct_jar_launch_uses_filename_when_jar_lives_under_server_path() {
        let target = resolve_direct_jar_launch_target(
            "E:/servers/fabric-1.20.1",
            "E:/servers/fabric-1.20.1/server.jar",
        );

        assert_eq!(target, "server.jar");
    }

    #[test]
    fn direct_jar_launch_keeps_full_path_when_jar_is_outside_server_path() {
        let target = resolve_direct_jar_launch_target(
            "E:/servers/fabric-1.20.1",
            "E:/srv/shared/server.jar",
        );

        assert_eq!(target.replace('\\', "/"), "E:/srv/shared/server.jar");
    }

    #[test]
    fn direct_jar_launch_keeps_full_path_when_jar_is_in_nested_subdirectory() {
        let target = resolve_direct_jar_launch_target(
            "E:/servers/fabric-1.20.1",
            "E:/servers/fabric-1.20.1/libraries/server.jar",
        );

        assert_eq!(target.replace('\\', "/"), "E:/servers/fabric-1.20.1/libraries/server.jar");
    }
}
