use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "linux")]
use crate::commands::update_arch::{get_aur_helper, is_arch_linux};
use crate::commands::update_types::PendingUpdate;
use crate::commands::update_version::compare_versions;

/// 安装进度标志
#[allow(dead_code)]
pub static INSTALL_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// 获取更新缓存目录
#[allow(dead_code)]
pub fn get_update_cache_dir() -> PathBuf {
    let cache_dir = dirs_next::cache_dir().unwrap_or_else(std::env::temp_dir);
    cache_dir.join("com.fpsz.sea-lantern").join("updates")
}

/// 获取待更新文件路径
#[allow(dead_code)]
pub fn get_pending_update_file() -> PathBuf {
    get_update_cache_dir().join("pending_update.json")
}

/// 执行更新安装
#[allow(dead_code)]
pub async fn execute_install(file_path: String, version: String) -> Result<(), String> {
    if INSTALL_IN_PROGRESS.swap(true, Ordering::SeqCst) {
        return Err("Install is already in progress".to_string());
    }

    let result = (|| -> Result<(), String> {
        let path = PathBuf::from(&file_path);
        if !path.exists() {
            return Err(format!("Update file not found: {}", file_path));
        }

        // Arch Linux 特殊处理
        #[cfg(target_os = "linux")]
        {
            if is_arch_linux() {
                let helper = get_aur_helper().unwrap_or_else(|| "yay".to_string());
                return Err(format!(
                    "您使用的是 Arch Linux\n\
                     请使用包管理器更新 SeaLantern：\n\
                     {} -S sealantern\n\
                     \n\
                     或使用其他 AUR 助手",
                    helper
                ));
            }
        }

        let pending_file = get_pending_update_file();
        if let Some(parent) = pending_file.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create pending update directory: {}", e))?;
        }

        let pending = PendingUpdate { file_path: file_path.clone(), version };

        let json = serde_json::to_string(&pending)
            .map_err(|e| format!("Failed to serialize pending update: {}", e))?;

        std::fs::write(&pending_file, json)
            .map_err(|e| format!("Failed to write pending update file: {}", e))?;

        #[cfg(target_os = "windows")]
        {
            let pending_file_path = pending_file.to_string_lossy().to_string();
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            match extension.as_deref() {
                Some("msi") => {
                    windows::spawn_elevated_windows_process(
                        "msiexec.exe",
                        &["/i", &file_path, "/passive", "/norestart"],
                        Some(&file_path),
                        Some(pending_file_path.as_str()),
                    )?;
                }
                Some("exe") => {
                    windows::spawn_elevated_windows_process(
                        &file_path,
                        &["/S", "/norestart"],
                        Some(&file_path),
                        Some(pending_file_path.as_str()),
                    )?;
                }
                _ => {
                    opener::open(&file_path)
                        .map_err(|e| format!("Failed to open update file: {}", e))?;
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            opener::open(&file_path).map_err(|e| format!("Failed to open update file: {}", e))?;
        }

        #[cfg(target_os = "linux")]
        {
            opener::open(&file_path).map_err(|e| format!("Failed to open update file: {}", e))?;
        }

        Ok(())
    })();

    if result.is_err() {
        INSTALL_IN_PROGRESS.store(false, Ordering::SeqCst);
        std::fs::remove_file(get_pending_update_file()).ok();
    }

    result
}

/// 检查待更新状态
#[allow(dead_code)]
pub async fn check_pending_update() -> Result<Option<PendingUpdate>, String> {
    let pending_file = get_pending_update_file();

    if !pending_file.exists() {
        return Ok(None);
    }

    let json = std::fs::read_to_string(&pending_file)
        .map_err(|e| format!("Failed to read pending update file: {}", e))?;

    let pending: PendingUpdate = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse pending update: {}", e))?;

    let path = PathBuf::from(&pending.file_path);
    if !path.exists() {
        std::fs::remove_file(&pending_file).ok();
        return Ok(None);
    }

    let current_version = env!("CARGO_PKG_VERSION");
    if !compare_versions(current_version, &pending.version) {
        std::fs::remove_file(&pending_file).ok();
        return Ok(None);
    }

    Ok(Some(pending))
}

/// 清除待更新状态
#[allow(dead_code)]
pub async fn clear_pending_update() -> Result<(), String> {
    let pending_file = get_pending_update_file();
    if pending_file.exists() {
        std::fs::remove_file(&pending_file)
            .map_err(|e| format!("Failed to remove pending update file: {}", e))?;
    }
    Ok(())
}

/// Windows 平台特定实现
#[cfg(target_os = "windows")]
mod windows {

    /// 转义 PowerShell 单引号
    #[allow(dead_code)]
    pub fn escape_powershell_single_quoted(value: &str) -> String {
        value.replace('\'', "''")
    }

    /// 构建隐藏的 PowerShell 命令
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if stderr.is_empty() {
                "Administrator permission was denied or installer failed to launch".to_string()
            } else {
                format!(
                    "Administrator permission was denied or installer failed to launch: {}",
                    stderr
                )
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
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

/// 非 Windows 平台的占位实现
#[cfg(not(target_os = "windows"))]
mod windows {

    #[allow(dead_code)]
    pub fn spawn_elevated_windows_process(
        _file_path: &str,
        _args: &[&str],
        _cleanup_file_path: Option<&str>,
        _pending_file_path: Option<&str>,
    ) -> Result<(), String> {
        Err("Windows-specific function called on non-Windows platform".to_string())
    }
}
