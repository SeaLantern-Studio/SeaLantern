#[cfg(target_os = "windows")]
mod imp {
    /// 转义 PowerShell 单引号
    #[allow(dead_code)] // Windows 调用
    pub fn escape_powershell_single_quoted(value: &str) -> String {
        value.replace('\'', "''")
    }

    fn decode_utf16(bytes: &[u8], little_endian: bool) -> String {
        let utf16 = bytes
            .chunks_exact(2)
            .map(|chunk| {
                if little_endian {
                    u16::from_le_bytes([chunk[0], chunk[1]])
                } else {
                    u16::from_be_bytes([chunk[0], chunk[1]])
                }
            })
            .collect::<Vec<u16>>();
        String::from_utf16_lossy(&utf16).trim().to_string()
    }

    fn decode_powershell_output(bytes: &[u8]) -> String {
        if bytes.is_empty() {
            return String::new();
        }

        if bytes.len() >= 2 {
            if bytes.starts_with(&[0xFF, 0xFE]) {
                return decode_utf16(&bytes[2..], true);
            }
            if bytes.starts_with(&[0xFE, 0xFF]) {
                return decode_utf16(&bytes[2..], false);
            }
        }

        if bytes.len().is_multiple_of(2) && bytes.len() >= 4 {
            let even_zeros = bytes.iter().step_by(2).filter(|b| **b == 0).count();
            let odd_zeros = bytes.iter().skip(1).step_by(2).filter(|b| **b == 0).count();
            let even_count = bytes.len() / 2;
            let odd_count = bytes.len() / 2;
            let even_ratio = even_zeros as f32 / even_count as f32;
            let odd_ratio = odd_zeros as f32 / odd_count as f32;

            if odd_ratio >= 0.6 && even_ratio <= 0.2 {
                return decode_utf16(bytes, true);
            }
            if even_ratio >= 0.6 && odd_ratio <= 0.2 {
                return decode_utf16(bytes, false);
            }
        }

        String::from_utf8_lossy(bytes).trim().to_string()
    }

    /// 构建隐藏的 PowerShell 命令
    #[allow(dead_code)] // Windows 调用
    pub fn build_hidden_powershell_command(command: &str) -> std::process::Command {
        let mut process = std::process::Command::new("powershell");
        process.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-Command",
            command,
        ]);

        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        process.creation_flags(CREATE_NO_WINDOW);

        process
    }

    /// 启动更新重启监视器
    #[allow(dead_code)] // Windows 调用
    pub fn spawn_update_relaunch_watcher(
        installer_pid: u32,
        relaunch_exe: &str,
        cleanup_file_path: Option<&str>,
        pending_file_path: Option<&str>,
    ) -> Result<(), String> {
        let relaunch_exe_escaped = escape_powershell_single_quoted(relaunch_exe);
        let cleanup_file_script = cleanup_file_path
            .map(escape_powershell_single_quoted)
            .map(|path| {
                format!(
                    "if (Test-Path '{path}') {{ Remove-Item -Path '{path}' -Force -ErrorAction SilentlyContinue }}; "
                )
            })
            .unwrap_or_default();
        let cleanup_pending_script = pending_file_path
            .map(escape_powershell_single_quoted)
            .map(|path| {
                format!(
                    "if (Test-Path '{path}') {{ Remove-Item -Path '{path}' -Force -ErrorAction SilentlyContinue }}; "
                )
            })
            .unwrap_or_default();
        let watcher_command = format!(
            "$ErrorActionPreference = 'SilentlyContinue'; \
             try {{ \
               $installer = [System.Diagnostics.Process]::GetProcessById({installer_pid}); \
               if ($installer) {{ \
                 $installer.WaitForExit(); \
                 if ($installer.ExitCode -eq 0) {{ \
                   {cleanup_file_script}\
                   {cleanup_pending_script}\
                   Start-Sleep -Milliseconds 700; \
                   Start-Process -FilePath '{relaunch_exe_escaped}' \
                 }} \
               }} \
             }} catch {{}}"
        );

        build_hidden_powershell_command(&watcher_command)
            .spawn()
            .map_err(|e| format!("Failed to watch installer process: {}", e))?;

        Ok(())
    }

    /// 以提升权限启动 Windows 进程
    #[allow(dead_code)] // Windows 调用
    pub fn spawn_elevated_windows_process(
        file_path: &str,
        args: &[&str],
        cleanup_file_path: Option<&str>,
        pending_file_path: Option<&str>,
    ) -> Result<(), String> {
        let file_path_escaped = escape_powershell_single_quoted(file_path);
        let argument_list = args
            .iter()
            .map(|arg| format!("'{}'", escape_powershell_single_quoted(arg)))
            .collect::<Vec<String>>()
            .join(", ");

        let launch_command = format!(
            "$ErrorActionPreference = 'Stop'; \
             $installer = Start-Process -FilePath '{file_path_escaped}' -ArgumentList @({argument_list}) -Verb RunAs -PassThru; \
             if (-not $installer) {{ throw 'Installer process was not created.' }}; \
             Write-Output $installer.Id"
        );

        let output = build_hidden_powershell_command(&launch_command)
            .output()
            .map_err(|e| format!("Failed to request administrator privileges: {}", e))?;

        if !output.status.success() {
            let stderr = decode_powershell_output(&output.stderr);
            return Err(if stderr.is_empty() {
                "Administrator permission was denied or installer failed to launch".to_string()
            } else {
                format!(
                    "Administrator permission was denied or installer failed to launch: {}",
                    stderr
                )
            });
        }

        let stdout = decode_powershell_output(&output.stdout);
        let installer_pid = stdout
            .lines()
            .rev()
            .find_map(|line| line.trim().parse::<u32>().ok())
            .ok_or_else(|| "Installer started but process id was not returned".to_string())?;

        if let Some(relaunch_exe) = std::env::current_exe()
            .ok()
            .and_then(|path| path.to_str().map(|value| value.to_string()))
        {
            spawn_update_relaunch_watcher(
                installer_pid,
                &relaunch_exe,
                cleanup_file_path,
                pending_file_path,
            )?;
        }

        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    #[allow(dead_code)] // 跨平台占位
    pub fn spawn_elevated_windows_process(
        _file_path: &str,
        _args: &[&str],
        _cleanup_file_path: Option<&str>,
        _pending_file_path: Option<&str>,
    ) -> Result<(), String> {
        Err("Windows-specific function called on non-Windows platform".to_string())
    }
}

pub(super) use imp::spawn_elevated_windows_process;
