#[cfg(target_os = "windows")]
use super::super::super::common::{build_windows_cmd_command, escape_cmd_arg};
use super::super::super::{common::detect_java_major_version, startup_support};
use super::context::LaunchContext;
#[cfg(target_os = "windows")]
use crate::services::server::manager::common::ManagedConsoleEncoding;
use std::process::Command;

pub(super) fn prepare_script_startup(context: &LaunchContext<'_>) -> Result<(), String> {
    ensure_script_java_compat(
        context
            .server
            .java_path()
            .expect("script launch requires java_path"),
    )?;
    startup_support::write_user_jvm_args(
        context.server,
        context.settings,
        context.managed_console_encoding,
    )
}

pub(super) fn apply_script_process_env(cmd: &mut Command, context: &LaunchContext<'_>) {
    apply_java_process_env(cmd, &context.java_home_dir_str, &context.java_bin_dir_str);
}

#[cfg(target_os = "windows")]
pub(super) fn build_windows_cmd_env_prefix(
    java_home_dir_str: &str,
    java_bin_dir_str: &str,
) -> String {
    format!(
        "set \"JAVA_HOME={}\" & set \"PATH={};%PATH%\"",
        escape_cmd_arg(java_home_dir_str),
        escape_cmd_arg(java_bin_dir_str)
    )
}

#[cfg(target_os = "windows")]
pub(super) fn build_windows_bat_command(
    startup_filename: &str,
    managed_console_encoding: ManagedConsoleEncoding,
    java_home_dir_str: &str,
    java_bin_dir_str: &str,
) -> Command {
    build_windows_cmd_command(&build_windows_bat_command_text(
        startup_filename,
        managed_console_encoding,
        java_home_dir_str,
        java_bin_dir_str,
    ))
}

#[cfg(target_os = "windows")]
fn build_windows_bat_command_text(
    startup_filename: &str,
    managed_console_encoding: ManagedConsoleEncoding,
    java_home_dir_str: &str,
    java_bin_dir_str: &str,
) -> String {
    format!(
        "chcp {}>nul & {} & call \"{}\" nogui",
        managed_console_encoding.cmd_code_page(),
        build_windows_cmd_env_prefix(java_home_dir_str, java_bin_dir_str),
        escape_cmd_arg(startup_filename)
    )
}

pub(super) fn apply_java_process_env(
    cmd: &mut Command,
    java_home_dir_str: &str,
    java_bin_dir_str: &str,
) {
    cmd.env("JAVA_HOME", java_home_dir_str);
    cmd.env("PATH", build_script_path_value(java_bin_dir_str));
}

fn ensure_script_java_compat(java_path: &str) -> Result<(), String> {
    ensure_supported_script_java_major_version(detect_java_major_version(java_path))
}

fn ensure_supported_script_java_major_version(major_version: Option<u32>) -> Result<(), String> {
    if let Some(major_version) = major_version {
        if major_version < 9 {
            return Err(format!(
                "当前 Java 版本 {} 不支持 @user_jvm_args.txt 参数文件语法，请改用 Java 9+（NeoForge 建议 Java 21）",
                major_version
            ));
        }
    }

    Ok(())
}

fn build_script_path_value(java_bin_dir_str: &str) -> String {
    build_prefixed_path_value(
        java_bin_dir_str,
        &std::env::var("PATH").unwrap_or_default(),
        path_separator(),
    )
}

fn build_prefixed_path_value(
    java_bin_dir_str: &str,
    existing_path: &str,
    separator: &str,
) -> String {
    if existing_path.is_empty() {
        java_bin_dir_str.to_string()
    } else {
        format!("{}{}{}", java_bin_dir_str, separator, existing_path)
    }
}

#[cfg(target_os = "windows")]
fn path_separator() -> &'static str {
    ";"
}

#[cfg(not(target_os = "windows"))]
fn path_separator() -> &'static str {
    ":"
}

#[cfg(test)]
mod tests {
    use super::{
        apply_java_process_env, build_prefixed_path_value,
        ensure_supported_script_java_major_version,
    };
    use std::process::Command;

    #[cfg(target_os = "windows")]
    use super::{build_windows_bat_command, build_windows_cmd_env_prefix};
    #[cfg(target_os = "windows")]
    use crate::services::server::manager::common::ManagedConsoleEncoding;

    #[test]
    fn script_launch_rejects_java_8_for_args_file_mode() {
        let err = ensure_supported_script_java_major_version(Some(8))
            .expect_err("java 8 should be rejected for script launch args files");

        assert!(err.contains("Java 版本 8"));
        assert!(err.contains("Java 9+"));
    }

    #[test]
    fn script_launch_allows_unknown_or_modern_java_versions() {
        ensure_supported_script_java_major_version(None)
            .expect("unknown java version should not block launch");
        ensure_supported_script_java_major_version(Some(21))
            .expect("modern java version should be allowed");
    }

    #[test]
    fn build_prefixed_path_value_keeps_java_bin_first() {
        let path = build_prefixed_path_value("E:/java/bin", "C:/Windows/System32", ";");

        assert_eq!(path, "E:/java/bin;C:/Windows/System32");
    }

    #[test]
    fn build_prefixed_path_value_omits_separator_when_existing_path_is_empty() {
        let path = build_prefixed_path_value("E:/java/bin", "", ";");

        assert_eq!(path, "E:/java/bin");
    }

    #[test]
    fn apply_java_process_env_sets_java_home_and_prefixed_path() {
        let mut command = Command::new("cmd");

        apply_java_process_env(&mut command, "E:/java", "E:/java/bin");

        let envs = command
            .get_envs()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().to_string(),
                    value.map(|value| value.to_string_lossy().to_string()),
                )
            })
            .collect::<Vec<_>>();

        assert!(envs
            .iter()
            .any(|(key, value)| { key == "JAVA_HOME" && value.as_deref() == Some("E:/java") }));
        assert!(envs.iter().any(|(key, value)| {
            key == "PATH"
                && value
                    .as_deref()
                    .is_some_and(|value| value.starts_with("E:/java/bin"))
        }));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn build_windows_cmd_env_prefix_sets_java_home_and_path_in_order() {
        let prefix = build_windows_cmd_env_prefix("C:/Java/JDK 21", "C:/Java/JDK 21/bin");

        assert_eq!(
            prefix,
            "set \"JAVA_HOME=C:/Java/JDK 21\" & set \"PATH=C:/Java/JDK 21/bin;%PATH%\""
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn build_windows_bat_command_keeps_java_env_and_call_escaping_in_support_layer() {
        let cmd = build_windows_bat_command(
            "start &(1)%2.bat",
            ManagedConsoleEncoding::Utf8,
            "C:/Java/JDK 21",
            "C:/Java/JDK 21/bin",
        );

        let args = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert_eq!(args.len(), 3);
        assert_eq!(args[0], "/d");
        assert_eq!(args[1], "/c");
        assert_eq!(
            args[2],
            "chcp 65001>nul & set \"JAVA_HOME=C:/Java/JDK 21\" & set \"PATH=C:/Java/JDK 21/bin;%PATH%\" & call \"start ^&^(1^)%%2.bat\" nogui"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn build_windows_bat_command_reuses_shared_env_prefix_and_selected_code_page() {
        let java_home = "D:/Java/Zulu 21";
        let java_bin = "D:/Java/Zulu 21/bin";
        let cmd = build_windows_bat_command(
            "launch.bat",
            ManagedConsoleEncoding::Gbk,
            java_home,
            java_bin,
        );

        let args = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        let expected = format!(
            "chcp 936>nul & {} & call \"launch.bat\" nogui",
            build_windows_cmd_env_prefix(java_home, java_bin)
        );

        assert_eq!(args, vec!["/d".to_string(), "/c".to_string(), expected]);
    }
}
