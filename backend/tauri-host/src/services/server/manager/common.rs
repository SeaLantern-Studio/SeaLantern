use std::path::Path;
use std::process::Command;

use crate::models::server::{ServerInstance, ServerRuntimeConfig};
use sea_lantern_server_local_setup_core::{
    decode_console_bytes as decode_shared_console_bytes, detect_startup_mode_from_path_like,
    parse_java_major_version as parse_shared_java_major_version,
    preview_command as preview_shared_command, script_bytes_prefer_utf8,
};

use super::i18n::{manager_t, manager_t1, manager_t2, manager_t3};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StartupMode {
    Starter,
    Jar,
    Bat,
    Sh,
    Ps1,
    Custom,
}

impl StartupMode {
    pub(super) fn from_raw(mode: &str) -> Self {
        match mode.to_ascii_lowercase().as_str() {
            "starter" => Self::Starter,
            "bat" => Self::Bat,
            "sh" => Self::Sh,
            "ps1" => Self::Ps1,
            "custom" => Self::Custom,
            _ => Self::Jar,
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Starter => "starter",
            Self::Jar => "jar",
            Self::Bat => "bat",
            Self::Sh => "sh",
            Self::Ps1 => "ps1",
            Self::Custom => "custom",
        }
    }

    pub(super) fn is_custom(self) -> bool {
        matches!(self, Self::Custom)
    }

    pub(super) fn is_starter(self) -> bool {
        matches!(self, Self::Starter)
    }

    #[cfg(test)]
    pub(super) fn prefers_direct_jar(self) -> bool {
        matches!(self, Self::Jar | Self::Starter)
    }

    #[cfg(target_os = "windows")]
    pub(super) fn uses_windows_script_encoding_detection(self) -> bool {
        matches!(self, Self::Bat | Self::Ps1)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ManagedConsoleEncoding {
    Utf8,
    #[cfg(target_os = "windows")]
    Gbk,
}

impl ManagedConsoleEncoding {
    pub(super) fn java_name(self) -> &'static str {
        match self {
            ManagedConsoleEncoding::Utf8 => "UTF-8",
            #[cfg(target_os = "windows")]
            ManagedConsoleEncoding::Gbk => "GBK",
        }
    }

    #[cfg(target_os = "windows")]
    pub(super) fn cmd_code_page(self) -> &'static str {
        match self {
            ManagedConsoleEncoding::Utf8 => "65001",
            ManagedConsoleEncoding::Gbk => "936",
        }
    }
}

/// 验证服务器名称，避免路径和系统保留名带来的问题。
pub(super) fn validate_server_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(manager_t("server.manager.name_empty"));
    }
    if trimmed.len() > 64 {
        return Err(manager_t("server.manager.name_too_long"));
    }

    let forbidden_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];
    for c in forbidden_chars {
        if trimmed.contains(c) {
            return Err(manager_t1("server.manager.name_invalid_char", c.to_string()));
        }
    }

    if trimmed.starts_with('.') || trimmed.ends_with('.') || trimmed.ends_with(' ') {
        return Err(manager_t("server.manager.name_invalid_edge_chars"));
    }

    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let upper = trimmed.to_uppercase();
    for r in reserved {
        if upper == r || upper.starts_with(&format!("{}.", r)) {
            return Err(manager_t1("server.manager.name_reserved", r.to_string()));
        }
    }

    Ok(trimmed.to_string())
}

pub(super) fn ensure_server_identity_available(
    existing: &[ServerInstance],
    candidate_name: &str,
    candidate_aliases: &[String],
    candidate_path: &str,
    candidate_container_name: Option<&str>,
) -> Result<(), String> {
    let candidate_name_lower = candidate_name.trim().to_ascii_lowercase();
    let candidate_path_normalized = normalize_server_identity_path(candidate_path);
    let candidate_container_lower = candidate_container_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase());

    let alias_pairs = candidate_aliases
        .iter()
        .map(|alias| (alias, alias.trim().to_ascii_lowercase()))
        .filter(|(_, lowered)| !lowered.is_empty())
        .collect::<Vec<_>>();

    for server in existing {
        if server.name.trim().to_ascii_lowercase() == candidate_name_lower {
            return Err(manager_t2(
                "server.manager.name_conflict_existing",
                server.name.clone(),
                server.id.clone(),
            ));
        }

        if normalize_server_identity_path(&server.path) == candidate_path_normalized {
            return Err(manager_t3(
                "server.manager.path_conflict_existing",
                server.path.clone(),
                server.id.clone(),
                server.name.clone(),
            ));
        }

        for (alias, alias_lower) in &alias_pairs {
            if server.name.trim().to_ascii_lowercase() == *alias_lower {
                return Err(manager_t3(
                    "server.manager.alias_conflict_with_name",
                    alias.to_string(),
                    server.name.clone(),
                    server.id.clone(),
                ));
            }
            if server
                .aliases
                .iter()
                .any(|existing_alias| existing_alias.trim().to_ascii_lowercase() == *alias_lower)
            {
                return Err(manager_t3(
                    "server.manager.alias_conflict_existing_alias",
                    alias.to_string(),
                    server.name.clone(),
                    server.id.clone(),
                ));
            }
        }

        if server.aliases.iter().any(|existing_alias| {
            existing_alias.trim().to_ascii_lowercase() == candidate_name_lower
        }) {
            return Err(manager_t3(
                "server.manager.name_conflict_with_alias",
                candidate_name.to_string(),
                server.name.clone(),
                server.id.clone(),
            ));
        }

        if let Some(candidate_container_lower) = candidate_container_lower.as_deref() {
            if server_container_name_lower(server).as_deref() == Some(candidate_container_lower) {
                return Err(manager_t3(
                    "server.manager.docker_container_conflict",
                    candidate_container_name.unwrap_or_default().to_string(),
                    server.id.clone(),
                    server.name.clone(),
                ));
            }
        }
    }

    Ok(())
}

fn normalize_server_identity_path(path: &str) -> String {
    path.trim().replace('\\', "/").to_ascii_lowercase()
}

fn server_container_name_lower(server: &ServerInstance) -> Option<String> {
    match &server.runtime {
        ServerRuntimeConfig::DockerItzg(runtime) => {
            let trimmed = runtime.container_name.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_ascii_lowercase())
            }
        }
        ServerRuntimeConfig::Local(_) => None,
    }
}

#[cfg(target_os = "windows")]
pub(super) fn build_windows_cmd_command(command_text: &str) -> Command {
    use std::os::windows::process::CommandExt;

    let mut cmd = Command::new("cmd");
    cmd.arg("/d");
    cmd.arg("/c");
    cmd.raw_arg(command_text);
    cmd
}

pub(super) fn current_timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(super) fn current_timestamp_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

pub(super) fn get_data_dir_checked() -> Result<String, String> {
    crate::utils::path::get_or_create_app_data_dir_checked()
        .map_err(|e| manager_t1("server.manager.data_dir_resolve_failed", e.to_string()))
}

pub(super) fn normalize_startup_mode(mode: &str) -> &str {
    StartupMode::from_raw(mode).as_str()
}

pub(super) fn detect_startup_mode_from_path(path: &Path) -> String {
    detect_startup_mode_from_path_like(&path.to_string_lossy())
}

pub(super) fn resolve_managed_console_encoding(
    startup_mode: StartupMode,
    startup_path: &Path,
) -> ManagedConsoleEncoding {
    #[cfg(target_os = "windows")]
    {
        if startup_mode.uses_windows_script_encoding_detection() {
            return detect_windows_batch_encoding(startup_path);
        }
    }

    let _ = startup_mode;
    let _ = startup_path;
    ManagedConsoleEncoding::Utf8
}

#[cfg(target_os = "windows")]
fn detect_windows_batch_encoding(startup_path: &Path) -> ManagedConsoleEncoding {
    let bytes = match std::fs::read(startup_path) {
        Ok(bytes) => bytes,
        Err(_) => return ManagedConsoleEncoding::Utf8,
    };

    if script_bytes_prefer_utf8(&bytes) {
        ManagedConsoleEncoding::Utf8
    } else {
        ManagedConsoleEncoding::Gbk
    }
}

pub(super) fn detect_java_major_version(java_path: &str) -> Option<u32> {
    let output = Command::new(java_path).arg("-version").output().ok()?;
    let text = if output.stderr.is_empty() {
        decode_console_bytes(&output.stdout)
    } else {
        decode_console_bytes(&output.stderr)
    };

    for line in text.lines() {
        if line.contains("version") {
            let mut segments = line.split('"');
            let _ = segments.next();
            if let Some(version_text) = segments.next() {
                return parse_shared_java_major_version(version_text);
            }
        }
    }

    None
}

pub(super) fn format_command_for_log(command: &Command) -> String {
    preview_shared_command(command)
}

pub(super) fn decode_console_bytes(bytes: &[u8]) -> String {
    decode_shared_console_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{lock_env, EnvGuard};
    #[cfg(target_os = "windows")]
    use tempfile::TempDir;

    #[cfg(target_os = "windows")]
    #[test]
    fn build_windows_cmd_command_keeps_shell_command_intact() {
        let command_text =
            "\"C:\\Program Files\\Java\\jdk-21\\bin\\java.exe\" -jar paper.jar nogui";
        let cmd = build_windows_cmd_command(command_text);

        let args = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert_eq!(args, vec!["/d", "/c", command_text]);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn build_windows_cmd_command_executes_quoted_program_path() {
        let temp_dir = TempDir::new().unwrap();

        let script_path = temp_dir.path().join("echo args.cmd");
        std::fs::write(&script_path, "@echo off\r\necho %1\r\n").unwrap();

        let command_text = format!("\"{}\" ok", script_path.display());
        let output = build_windows_cmd_command(&command_text).output().unwrap();

        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
    }

    #[test]
    fn format_command_for_log_escapes_nested_quotes_for_display_only() {
        let mut cmd = Command::new("cmd");
        cmd.arg("/c");
        cmd.arg("\"C:\\Program Files\\Java\\jdk-21\\bin\\java.exe\" -jar paper.jar nogui");

        let formatted = format_command_for_log(&cmd);

        assert_eq!(
            formatted,
            "cmd /c '\"C:\\Program Files\\Java\\jdk-21\\bin\\java.exe\" -jar paper.jar nogui'"
        );
    }

    #[test]
    fn startup_mode_parsing_and_flags_match_runtime_expectations() {
        assert_eq!(StartupMode::from_raw("CUSTOM"), StartupMode::Custom);
        assert_eq!(StartupMode::from_raw("unknown"), StartupMode::Jar);
        assert!(StartupMode::Jar.prefers_direct_jar());
        assert!(StartupMode::Starter.prefers_direct_jar());
        assert!(!StartupMode::Bat.prefers_direct_jar());
        assert!(!StartupMode::Sh.prefers_direct_jar());
        assert!(!StartupMode::Custom.prefers_direct_jar());
        assert!(StartupMode::Starter.is_starter());
        assert!(StartupMode::Custom.is_custom());
    }

    #[test]
    fn get_data_dir_checked_surfaces_app_data_dir_creation_failures() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let blocked_root = temp_dir.path().join("blocked-root");
        std::fs::write(&blocked_root, b"not a directory")
            .expect("file-backed app data root should exist");
        let blocked_path = blocked_root.join("nested");
        let _env_lock = lock_env();
        let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &blocked_path.to_string_lossy());

        let error = match get_data_dir_checked() {
            Err(error) => error,
            Ok(path) => {
                panic!("app data dir failure should not be silently downgraded, got path: {}", path)
            }
        };

        assert!(
            error.contains("Failed to resolve server manager data directory")
                || error.contains("无法解析服务器管理数据目录"),
            "unexpected error: {}",
            error
        );
        assert!(error.contains("blocked-root"), "unexpected error: {}", error);
    }
}
