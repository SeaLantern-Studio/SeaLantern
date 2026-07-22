use std::ffi::OsStr;
use std::process::Command;

use crate::observability;

use super::PlatformError;

/// 已启动或已提交给系统授权对话框的提权请求。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElevationLaunch {
    /// Unix `sudo` 已启动，进程 ID 属于 sudo 包装进程。
    Started { process_id: u32 },
    /// Windows 或 macOS 已将请求交给系统授权界面。
    Requested,
}

/// 返回当前进程是否以管理员或 root 权限运行。
pub fn is_elevated() -> Result<bool, PlatformError> {
    let result = imp::is_elevated();
    result.inspect_err(|error| observability::platform_operation_failed("check elevation", error))
}

/// 请求操作系统以提升后的权限启动指定程序。
///
/// Unix 平台启动 `sudo -- <program> <args>`。Windows 和 macOS 会显示操作系统原生
/// 授权对话框，调用方应在请求后结束当前工作流而不是等待子进程。
pub fn request_elevation(
    program: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<ElevationLaunch, PlatformError> {
    let args = args
        .into_iter()
        .map(|arg| arg.as_ref().to_owned())
        .collect::<Vec<_>>();
    let result = imp::request_elevation(program.as_ref(), &args);
    result.inspect_err(|error| observability::platform_operation_failed("request elevation", error))
}

#[cfg(target_os = "windows")]
mod imp {
    use super::*;

    pub fn is_elevated() -> Result<bool, PlatformError> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)",
            ])
            .output()
            .map_err(|source| PlatformError::Command { operation: "check Windows elevation", source })?;
        if !output.status.success() {
            return Err(PlatformError::InvalidCommandOutput {
                operation: "check Windows elevation",
            });
        }
        match String::from_utf8_lossy(&output.stdout).trim() {
            "True" => Ok(true),
            "False" => Ok(false),
            _ => Err(PlatformError::InvalidCommandOutput { operation: "check Windows elevation" }),
        }
    }

    pub fn request_elevation(
        program: &OsStr,
        args: &[std::ffi::OsString],
    ) -> Result<ElevationLaunch, PlatformError> {
        let program = powershell_quote(&program.to_string_lossy());
        let arguments = args
            .iter()
            .map(|arg| format!("'{}'", powershell_quote(&arg.to_string_lossy())))
            .collect::<Vec<_>>()
            .join(", ");
        let command = format!(
            "$ErrorActionPreference = 'Stop'; Start-Process -FilePath '{program}' -ArgumentList @({arguments}) -Verb RunAs"
        );
        let status = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &command])
            .status()
            .map_err(|source| PlatformError::Command {
                operation: "request Windows elevation",
                source,
            })?;
        if status.success() {
            Ok(ElevationLaunch::Requested)
        } else {
            Err(PlatformError::InvalidCommandOutput { operation: "request Windows elevation" })
        }
    }

    pub(super) fn powershell_quote(value: &str) -> String {
        value.replace('\'', "''")
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::*;

    pub fn is_elevated() -> Result<bool, PlatformError> {
        super::unix_is_elevated()
    }

    pub fn request_elevation(
        program: &OsStr,
        args: &[std::ffi::OsString],
    ) -> Result<ElevationLaunch, PlatformError> {
        let command = shell_command(program, args);
        let script = format!(
            "do shell script {} with administrator privileges",
            applescript_quote(&command)
        );
        let status = Command::new("osascript")
            .args(["-e", &script])
            .status()
            .map_err(|source| PlatformError::Command {
                operation: "request macOS elevation",
                source,
            })?;
        if status.success() {
            Ok(ElevationLaunch::Requested)
        } else {
            Err(PlatformError::InvalidCommandOutput { operation: "request macOS elevation" })
        }
    }

    fn applescript_quote(value: &str) -> String {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('\"', "\\\""))
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod imp {
    use super::*;

    pub fn is_elevated() -> Result<bool, PlatformError> {
        super::unix_is_elevated()
    }

    pub fn request_elevation(
        program: &OsStr,
        args: &[std::ffi::OsString],
    ) -> Result<ElevationLaunch, PlatformError> {
        let child = Command::new("sudo")
            .arg("--")
            .arg(program)
            .args(args)
            .spawn()
            .map_err(|source| PlatformError::Command { operation: "start sudo", source })?;
        Ok(ElevationLaunch::Started { process_id: child.id() })
    }
}

#[cfg(unix)]
fn unix_is_elevated() -> Result<bool, PlatformError> {
    let output =
        Command::new("id")
            .arg("-u")
            .output()
            .map_err(|source| PlatformError::Command {
                operation: "check Unix elevation",
                source,
            })?;
    if !output.status.success() {
        return Err(PlatformError::InvalidCommandOutput { operation: "check Unix elevation" });
    }
    match String::from_utf8_lossy(&output.stdout).trim() {
        "0" => Ok(true),
        value if value.parse::<u32>().is_ok() => Ok(false),
        _ => Err(PlatformError::InvalidCommandOutput { operation: "check Unix elevation" }),
    }
}

#[cfg(not(any(windows, unix)))]
mod imp {
    use super::*;

    pub fn is_elevated() -> Result<bool, PlatformError> {
        Err(PlatformError::Unsupported { operation: "check elevation" })
    }

    pub fn request_elevation(
        _program: &OsStr,
        _args: &[std::ffi::OsString],
    ) -> Result<ElevationLaunch, PlatformError> {
        Err(PlatformError::Unsupported { operation: "request elevation" })
    }
}

#[cfg(target_os = "macos")]
fn shell_command(program: &OsStr, args: &[std::ffi::OsString]) -> String {
    std::iter::once(program)
        .chain(args.iter().map(|arg| arg.as_os_str()))
        .map(|value| format!("'{}'", value.to_string_lossy().replace('\'', "'\\''")))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elevation_check_returns_a_boolean_or_contextual_error() {
        match is_elevated() {
            Ok(_) => {}
            Err(error) => assert!(error.to_string().contains("elevation")),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn powershell_quotes_single_quotes() {
        assert_eq!(imp::powershell_quote("C:\\O'Brien"), "C:\\O''Brien");
    }
}
