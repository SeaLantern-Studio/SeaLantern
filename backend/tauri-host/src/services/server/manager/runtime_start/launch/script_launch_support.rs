#[cfg(target_os = "windows")]
use super::super::super::common::build_windows_cmd_command;
use super::super::super::startup_support;
use super::context::LaunchContext;
#[cfg(target_os = "windows")]
use sea_lantern_server_local_setup_core::build_windows_bat_command_text as build_shared_windows_bat_command_text;
#[cfg(target_os = "windows")]
use sea_lantern_server_local_setup_core::build_windows_bat_command_text_without_java as build_shared_windows_bat_command_text_without_java;
#[cfg(target_os = "windows")]
use sea_lantern_server_local_setup_core::ManagedConsoleEncoding;
use sea_lantern_server_local_setup_core::{
    build_java_launch_path_value as build_shared_java_launch_path_value,
    detect_java_major_version as detect_shared_java_major_version,
    ensure_supported_script_java_major_version, startup_mode_requires_java,
};
use std::process::Command;

pub(super) fn prepare_script_startup(context: &LaunchContext<'_>) -> Result<(), String> {
    if !startup_mode_requires_java(context.startup_mode.as_str()) {
        return Ok(());
    }

    let java_path = context.java_path_required()?;
    ensure_supported_script_java_major_version(detect_shared_java_major_version(java_path))?;
    startup_support::write_user_jvm_args(
        context.server,
        context.settings,
        context.managed_console_encoding,
    )
}

pub(super) fn apply_script_process_env(cmd: &mut Command, context: &LaunchContext<'_>) {
    if let Some((java_home_dir_str, java_bin_dir_str)) = context.java_env() {
        apply_java_process_env(cmd, java_home_dir_str, java_bin_dir_str);
    }
}

#[cfg(target_os = "windows")]
pub(super) fn build_windows_bat_command(
    startup_filename: &str,
    managed_console_encoding: ManagedConsoleEncoding,
    java_home_dir_str: Option<&str>,
    java_bin_dir_str: Option<&str>,
) -> Command {
    let command_text = match (java_home_dir_str, java_bin_dir_str) {
        (Some(java_home), Some(java_bin))
            if !java_home.trim().is_empty() && !java_bin.trim().is_empty() =>
        {
            build_shared_windows_bat_command_text(
                startup_filename,
                managed_console_encoding.cmd_code_page(),
                java_home,
                java_bin,
            )
        }
        _ => build_shared_windows_bat_command_text_without_java(
            startup_filename,
            managed_console_encoding.cmd_code_page(),
        ),
    };

    build_windows_cmd_command(&command_text)
}

pub(super) fn apply_java_process_env(
    cmd: &mut Command,
    java_home_dir_str: &str,
    java_bin_dir_str: &str,
) {
    cmd.env("JAVA_HOME", java_home_dir_str);
    cmd.env(
        "PATH",
        build_shared_java_launch_path_value(
            java_bin_dir_str,
            &std::env::var("PATH").unwrap_or_default(),
        ),
    );
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "windows")]
    use super::build_windows_bat_command;
    use super::{apply_java_process_env, ensure_supported_script_java_major_version};
    use sea_lantern_server_local_setup_core::prepend_path_entry;
    #[cfg(target_os = "windows")]
    use sea_lantern_server_local_setup_core::{
        build_windows_java_env_prefix, ManagedConsoleEncoding,
    };
    use std::process::Command;

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
    fn prepend_path_entry_keeps_java_bin_first() {
        let path = prepend_path_entry("E:/java/bin", "C:/Windows/System32", ";");

        assert_eq!(path, "E:/java/bin;C:/Windows/System32");
    }

    #[test]
    fn prepend_path_entry_omits_separator_when_existing_path_is_empty() {
        let path = prepend_path_entry("E:/java/bin", "", ";");

        assert_eq!(path, "E:/java/bin");
    }

    #[test]
    fn process_env_injection_sets_java_home_and_puts_java_bin_first_in_path() {
        let mut command = Command::new("cmd");

        apply_java_process_env(&mut command, "E:/java", "E:/java/bin");

        let envs = collect_envs(&command);

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
    fn windows_bat_env_prefix_sets_java_home_before_path_update() {
        let prefix = build_windows_java_env_prefix("C:/Java/JDK 21", "C:/Java/JDK 21/bin");

        assert_eq!(
            prefix,
            "set \"JAVA_HOME=C:/Java/JDK 21\" & set \"PATH=C:/Java/JDK 21/bin;%PATH%\""
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_bat_command_inlines_shared_java_env_before_calling_script() {
        let cmd = build_windows_bat_command(
            "start &(1)%2.bat",
            ManagedConsoleEncoding::Utf8,
            Some("C:/Java/JDK 21"),
            Some("C:/Java/JDK 21/bin"),
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
    fn windows_bat_command_keeps_java_env_in_command_text_instead_of_process_envs() {
        let java_home = "D:/Java/Zulu 21";
        let java_bin = "D:/Java/Zulu 21/bin";
        let cmd = build_windows_bat_command(
            "launch.bat",
            ManagedConsoleEncoding::Gbk,
            Some(java_home),
            Some(java_bin),
        );

        let args = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        let envs = collect_envs(&cmd);

        assert!(envs.is_empty(), "bat launch should keep env injection inline in cmd text");
        assert_eq!(
            args[2],
            format!(
                "chcp 936>nul & {} & call \"launch.bat\" nogui",
                build_windows_java_env_prefix(java_home, java_bin)
            )
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_bat_command_reuses_shared_env_prefix_verbatim() {
        let java_home = "D:/Java/Zulu 21";
        let java_bin = "D:/Java/Zulu 21/bin";
        let cmd = build_windows_bat_command(
            "launch.bat",
            ManagedConsoleEncoding::Gbk,
            Some(java_home),
            Some(java_bin),
        );

        let args = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        let expected = format!(
            "chcp 936>nul & {} & call \"launch.bat\" nogui",
            build_windows_java_env_prefix(java_home, java_bin)
        );

        assert_eq!(args, vec!["/d".to_string(), "/c".to_string(), expected]);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_bat_command_can_skip_java_env_injection() {
        let cmd = build_windows_bat_command("launch.bat", ManagedConsoleEncoding::Utf8, None, None);

        let args = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        let envs = collect_envs(&cmd);

        assert!(envs.is_empty());
        assert_eq!(
            args,
            vec![
                "/d".to_string(),
                "/c".to_string(),
                "chcp 65001>nul & call \"launch.bat\" nogui".to_string(),
            ]
        );
    }
}
