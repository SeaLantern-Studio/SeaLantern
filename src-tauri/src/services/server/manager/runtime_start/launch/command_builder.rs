use super::super::super::common::detect_java_major_version;
#[cfg(target_os = "windows")]
use super::super::super::common::escape_cmd_arg;
use super::context::LaunchContext;
use crate::services::server::installer;
use std::path::Path;
use std::process::Command;

/// 优先寻找可直接运行的 JAR 文件
pub(super) fn find_preferred_jar_path(context: &LaunchContext<'_>) -> Option<String> {
    let startup_path_obj = Path::new(&context.server.jar_path);
    let jar_preferred_mode = matches!(context.startup_mode, "bat" | "sh" | "ps1" | "custom");

    if !jar_preferred_mode {
        return None;
    }

    if startup_path_obj
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("jar"))
        .unwrap_or(false)
    {
        Some(context.server.jar_path.clone())
    } else {
        installer::find_server_jar(Path::new(&context.server.path)).ok()
    }
}

/// 检查脚本启动方式是否支持参数文件语法
fn ensure_script_java_compat(java_path: &str) -> Result<(), String> {
    if let Some(major_version) = detect_java_major_version(java_path) {
        if major_version < 9 {
            return Err(format!(
                "当前 Java 版本 {} 不支持 @user_jvm_args.txt 参数文件语法，请改用 Java 9+（NeoForge 建议 Java 21）",
                major_version
            ));
        }
    }
    Ok(())
}

/// 把 Java bin 目录加到 PATH 前面
fn extend_path(java_bin_dir_str: &str, separator: &str) -> String {
    let existing_path = std::env::var("PATH").unwrap_or_default();
    if existing_path.is_empty() {
        java_bin_dir_str.to_string()
    } else {
        format!("{}{}{}", java_bin_dir_str, separator, existing_path)
    }
}

/// 构建直接运行 JAR 的命令
pub(super) fn build_direct_jar_command(
    context: &LaunchContext<'_>,
    jar_path: &str,
    installer_url: Option<&str>,
) -> Command {
    let jar_path_obj = Path::new(jar_path);
    let launch_target = jar_path_obj
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| jar_path.to_string());

    let mut java_cmd = Command::new(&context.server.java_path);
    for arg in context.manager.build_managed_jvm_args(
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

/// 按启动方式构建最终命令
pub(super) fn build_configured_command(context: &LaunchContext<'_>) -> Result<Command, String> {
    match context.startup_mode {
        "custom" => build_custom_command(context),
        "bat" => build_bat_command(context),
        "sh" => build_sh_command(context),
        "ps1" => build_ps1_command(context),
        "starter" => {
            let installer_url = context
                .starter_installer_url
                .as_deref()
                .ok_or_else(|| "Starter 安装器下载链接为空".to_string())?;
            Ok(build_direct_jar_command(context, &context.server.jar_path, Some(installer_url)))
        }
        "jar" => Ok(build_direct_jar_command(context, &context.server.jar_path, None)),
        _ => Ok(build_direct_jar_command(context, &context.server.jar_path, None)),
    }
}

/// 构建自定义命令启动方式
fn build_custom_command(context: &LaunchContext<'_>) -> Result<Command, String> {
    let custom_command = context
        .server
        .custom_command
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "自定义启动命令为空".to_string())?;

    #[cfg(target_os = "windows")]
    {
        let mut custom_cmd = Command::new("cmd");
        custom_cmd.arg("/d");
        custom_cmd.arg("/c");
        custom_cmd.arg(custom_command);
        custom_cmd.env("JAVA_HOME", &context.java_home_dir_str);
        custom_cmd.env("PATH", extend_path(&context.java_bin_dir_str, ";"));
        Ok(custom_cmd)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut custom_cmd = Command::new("sh");
        custom_cmd.arg("-c");
        custom_cmd.arg(custom_command);
        custom_cmd.env("JAVA_HOME", &context.java_home_dir_str);
        custom_cmd.env("PATH", extend_path(&context.java_bin_dir_str, ":"));
        Ok(custom_cmd)
    }
}

/// 生成脚本启动会读取的 JVM 参数文件
fn write_user_jvm_args(context: &LaunchContext<'_>) -> Result<(), String> {
    context.manager.write_user_jvm_args(
        context.server,
        context.settings,
        context.managed_console_encoding,
    )
}

/// 构建 BAT 启动命令
fn build_bat_command(context: &LaunchContext<'_>) -> Result<Command, String> {
    ensure_script_java_compat(&context.server.java_path)?;
    write_user_jvm_args(context)?;

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        let mut bat_cmd = Command::new("cmd");
        let code_page = context.managed_console_encoding.cmd_code_page();
        let launch_command = format!(
            "chcp {}>nul & set \"JAVA_HOME={}\" & set \"PATH={};%PATH%\" & call \"{}\" nogui",
            code_page,
            escape_cmd_arg(&context.java_home_dir_str),
            escape_cmd_arg(&context.java_bin_dir_str),
            escape_cmd_arg(&context.startup_filename)
        );
        bat_cmd.arg("/d");
        bat_cmd.arg("/c");
        bat_cmd.raw_arg(&launch_command);
        Ok(bat_cmd)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("BAT 启动方式仅支持 Windows".to_string())
    }
}

/// 构建 SH 启动命令
fn build_sh_command(context: &LaunchContext<'_>) -> Result<Command, String> {
    ensure_script_java_compat(&context.server.java_path)?;
    write_user_jvm_args(context)?;
    let mut sh_cmd = Command::new("sh");
    sh_cmd.arg(&context.startup_filename);
    sh_cmd.arg("nogui");
    sh_cmd.env("JAVA_HOME", &context.java_home_dir_str);
    sh_cmd.env("PATH", extend_path(&context.java_bin_dir_str, ":"));
    Ok(sh_cmd)
}

/// 构建 PowerShell 启动命令
fn build_ps1_command(context: &LaunchContext<'_>) -> Result<Command, String> {
    ensure_script_java_compat(&context.server.java_path)?;
    write_user_jvm_args(context)?;
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
        ps_cmd.env("JAVA_HOME", &context.java_home_dir_str);
        ps_cmd.env("PATH", extend_path(&context.java_bin_dir_str, ";"));
        Ok(ps_cmd)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("PS1 启动方式仅支持 Windows".to_string())
    }
}
