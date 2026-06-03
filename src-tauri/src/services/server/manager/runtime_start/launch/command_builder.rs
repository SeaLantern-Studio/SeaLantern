#[cfg(target_os = "windows")]
use super::super::super::common::build_windows_cmd_command;
use super::super::super::startup_support;
use super::context::LaunchContext;
use super::script_launch_support;
use crate::services::server::installer;
use crate::services::server::manager::common::StartupMode;
use std::path::Path;
use std::process::Command;

/// 优先寻找可直接运行的 JAR 文件
pub(super) fn find_preferred_jar_path(context: &LaunchContext<'_>) -> Option<String> {
    let startup_path_obj = Path::new(context.server.jar_path()?);
    let jar_preferred_mode = context.startup_mode.prefers_direct_jar();

    if !jar_preferred_mode {
        return None;
    }

    if startup_path_obj
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("jar"))
        .unwrap_or(false)
    {
        Some(context.server.jar_path()?.to_string())
    } else {
        installer::find_server_jar(Path::new(&context.server.path)).ok()
    }
}

/// 构建直接运行 JAR 的命令
pub(super) fn build_direct_jar_command(
    context: &LaunchContext<'_>,
    jar_path: &str,
    installer_url: Option<&str>,
) -> Command {
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
    ) {
        java_cmd.arg(arg);
    }
    java_cmd.arg("-jar");
    java_cmd.arg(launch_target);
    java_cmd.arg("nogui");
    if let Some(url) = installer_url {
        java_cmd.arg("--installer");
        java_cmd.arg(url);
    }
    java_cmd
}

fn resolve_direct_jar_launch_target(server_path: &str, jar_path: &str) -> String {
    let jar_path_obj = Path::new(jar_path);
    let server_path_obj = Path::new(server_path);

    if jar_path_obj.parent() == Some(server_path_obj) {
        return jar_path_obj
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| jar_path.to_string());
    }

    jar_path.to_string()
}

/// 按启动方式构建最终命令
pub(super) fn build_configured_command(context: &LaunchContext<'_>) -> Result<Command, String> {
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
            Ok(build_direct_jar_command(
                context,
                context
                    .server
                    .jar_path()
                    .expect("starter launch requires jar_path"),
                Some(installer_url),
            ))
        }
        StartupMode::Jar => Ok(build_direct_jar_command(
            context,
            context
                .server
                .jar_path()
                .expect("jar launch requires jar_path"),
            None,
        )),
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
    use super::resolve_direct_jar_launch_target;
    use crate::services::server::manager::common::StartupMode;

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
