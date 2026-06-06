use std::path::{Path, PathBuf};
use std::process::Command;

use once_cell::sync::Lazy;
use regex::Regex;
use sea_lantern_runtime::{
    find_root_startup_file_checked, is_windows_absolute_path,
};
use sea_lantern_server_installer_core::{
    detect_core_type, detect_core_type_checked, find_server_jar, find_server_jar_checked,
    CoreType,
};

static MC_VERSION_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(1\.\d{1,2}(?:\.\d{1,2})?)").expect("mc version regex should compile"));

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocalFolderInspection {
    pub startup_entry_path: Option<String>,
    pub startup_mode: Option<String>,
    pub detected_jar_path: Option<String>,
    pub inferred_core_type: Option<String>,
    pub inferred_mc_version: Option<String>,
}

impl LocalFolderInspection {
    pub fn is_attachable(&self) -> bool {
        self.startup_entry_path.is_some() || self.detected_jar_path.is_some()
    }

    pub fn preferred_startup_path(&self) -> Option<&str> {
        self.startup_entry_path
            .as_deref()
            .or(self.detected_jar_path.as_deref())
    }

    pub fn describe(&self) -> String {
        format!(
            "attachable={} startup_mode={} startup_entry={} jar={} core={} mc={}",
            self.is_attachable(),
            self.startup_mode.as_deref().unwrap_or("none"),
            self.startup_entry_path.as_deref().unwrap_or("none"),
            self.detected_jar_path.as_deref().unwrap_or("none"),
            self.inferred_core_type.as_deref().unwrap_or("unknown"),
            self.inferred_mc_version.as_deref().unwrap_or("unknown")
        )
    }
}

pub fn inspect_local_folder(folder: &Path) -> LocalFolderInspection {
    inspect_local_folder_checked(folder).unwrap_or_default()
}

pub fn inspect_local_folder_checked(folder: &Path) -> Result<LocalFolderInspection, String> {
    if !folder.exists() || !folder.is_dir() {
        return Ok(LocalFolderInspection::default());
    }

    let startup_entry_path = resolve_attach_executable_path_checked(folder)?
        .map(|path| path.to_string_lossy().to_string());
    let detected_jar_path = match find_server_jar(folder) {
        Ok(path) => Some(path),
        Err(error) if error == "整合包文件夹中未找到JAR文件" => None,
        Err(error) => {
            return Err(format!("本地目录探测失败: {}", error));
        }
    };
    let startup_mode = startup_entry_path
        .as_deref()
        .map(detect_startup_mode_from_path_like)
        .or_else(|| detected_jar_path.as_ref().map(|_| "jar".to_string()));

    let folder_display = folder.to_string_lossy();
    let inferred_core_type = if let Some(path) = startup_entry_path.as_deref() {
        detect_core_type_checked(path)?
    } else if let Some(path) = detected_jar_path.as_deref() {
        detect_core_type_checked(path)?
    } else {
        detect_core_type(folder_display.as_ref())
    };
    let inferred_core_type = (!inferred_core_type.eq_ignore_ascii_case("unknown"))
        .then_some(inferred_core_type);
    let inferred_mc_version = startup_entry_path
        .as_deref()
        .and_then(|value| infer_mc_version_hint(&[value]))
        .or_else(|| {
            detected_jar_path
                .as_deref()
                .and_then(|value| infer_mc_version_hint(&[value]))
        })
        .or_else(|| infer_mc_version_hint(&[folder_display.as_ref()]));

    Ok(LocalFolderInspection {
        startup_entry_path,
        startup_mode,
        detected_jar_path,
        inferred_core_type,
        inferred_mc_version,
    })
}

pub fn resolve_attach_executable_path(folder: &Path) -> Option<PathBuf> {
    resolve_attach_executable_path_checked(folder).ok().flatten()
}

pub fn resolve_attach_executable_path_checked(folder: &Path) -> Result<Option<PathBuf>, String> {
    let preferred_scripts = [
        "start.bat",
        "start.cmd",
        "run.bat",
        "run.cmd",
        "launch.bat",
        "launch.cmd",
        "start.sh",
        "run.sh",
        "launch.sh",
        "start.ps1",
        "run.ps1",
        "launch.ps1",
    ];

    if let Some(script_path) = preferred_scripts.iter().find_map(|script| {
        let script_path = folder.join(script);
        if script_path.exists() {
            Some(script_path)
        } else {
            None
        }
    }) {
        return Ok(Some(script_path));
    }

    let root_startup = find_root_startup_file_checked(folder)?;
    Ok(root_startup.filter(|path| is_script_path(path)))
}

pub fn infer_local_create_mc_version(
    jar_path: &str,
    resolved_name: &str,
    resolved_entry_path: Option<&str>,
    folder_path: Option<&Path>,
    executable_hint: Option<&str>,
) -> Option<String> {
    infer_local_create_mc_version_checked(
        jar_path,
        resolved_name,
        resolved_entry_path,
        folder_path,
        executable_hint,
    )
    .ok()
    .flatten()
}

pub fn infer_local_create_mc_version_checked(
    jar_path: &str,
    resolved_name: &str,
    resolved_entry_path: Option<&str>,
    folder_path: Option<&Path>,
    executable_hint: Option<&str>,
) -> Result<Option<String>, String> {
    if let Some(version) = infer_mc_version_hint(&[jar_path, resolved_name]) {
        return Ok(Some(version));
    }

    if let Some(entry_path) = resolved_entry_path {
        if let Some(folder) = Path::new(entry_path).parent() {
            if let Some(version) = infer_mc_version_from_folder_checked(folder, Some(entry_path))? {
                return Ok(Some(version));
            }
        }
    }

    if let Some(folder) = folder_path {
        if let Some(version) = infer_mc_version_from_folder_checked(folder, executable_hint)? {
            return Ok(Some(version));
        }
    }

    Ok(None)
}

pub fn infer_core_type_from_local_inputs(
    folder: &Path,
    executable_path: Option<&str>,
) -> Option<String> {
    infer_core_type_from_local_inputs_checked(folder, executable_path)
        .ok()
        .flatten()
}

pub fn infer_core_type_from_local_inputs_checked(
    folder: &Path,
    executable_path: Option<&str>,
) -> Result<Option<String>, String> {
    if let Some(path) = executable_path {
        return Ok(Some(detect_core_type_checked(path)?));
    }

    Ok(inspect_local_folder_checked(folder)?.inferred_core_type)
}

pub fn infer_mc_version_from_folder(folder: &Path, executable_path: Option<&str>) -> Option<String> {
    infer_mc_version_from_folder_checked(folder, executable_path)
        .ok()
        .flatten()
}

pub fn infer_mc_version_from_folder_checked(
    folder: &Path,
    executable_path: Option<&str>,
) -> Result<Option<String>, String> {
    if let Some(path) = executable_path {
        if let Some(version) = infer_mc_version_hint(&[path]) {
            return Ok(Some(version));
        }
    }

    Ok(inspect_local_folder_checked(folder)?.inferred_mc_version)
}

pub fn infer_mc_version_hint(inputs: &[&str]) -> Option<String> {
    for input in inputs {
        if let Some(capture) = MC_VERSION_PATTERN.captures(input) {
            if let Some(value) = capture.get(1) {
                return Some(value.as_str().to_string());
            }
        }
    }
    None
}

pub fn detect_startup_mode_from_folder(folder: &Path) -> String {
    inspect_local_folder(folder)
        .startup_mode
        .unwrap_or_else(|| "jar".to_string())
}

pub fn resolve_existing_attach_entry_path(folder: &Path, entry: &str) -> Option<String> {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return None;
    }

    let direct = Path::new(trimmed);
    if direct.exists() {
        return Some(direct.to_string_lossy().to_string());
    }

    let relative_to_folder = folder.join(trimmed);
    if relative_to_folder.exists() {
        return Some(relative_to_folder.to_string_lossy().to_string());
    }

    None
}

pub fn resolve_existing_local_entry_path(folder: Option<&Path>, entry: &str) -> Option<String> {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return None;
    }

    let path = Path::new(trimmed);
    if path.exists() {
        Some(path.to_string_lossy().to_string())
    } else if let Some(folder) = folder {
        let relative_to_folder = folder.join(path);
        if relative_to_folder.exists() {
            Some(relative_to_folder.to_string_lossy().to_string())
        } else {
            None
        }
    } else {
        None
    }
}

pub fn resolve_custom_entry_hint_path(
    entry: Option<&str>,
    resolved_entry_path: Option<&str>,
    folder: Option<&Path>,
) -> Option<String> {
    if let Some(path) = resolved_entry_path {
        return Some(path.to_string());
    }

    let entry = entry?.trim();
    if entry.is_empty() {
        return None;
    }

    let tokens = shlex::split(entry)?;
    if tokens.is_empty() {
        return None;
    }

    for window in tokens.windows(2) {
        if window[0].eq_ignore_ascii_case("-jar") {
            return resolve_command_path_hint(&window[1], folder);
        }
    }

    resolve_command_path_hint(&tokens[0], folder)
}

pub fn resolve_command_path_hint(token: &str, folder: Option<&Path>) -> Option<String> {
    resolve_command_path_hint_with(token, folder, std::env::current_dir)
}

fn resolve_command_path_hint_with<F>(
    token: &str,
    folder: Option<&Path>,
    current_dir: F,
) -> Option<String>
where
    F: FnOnce() -> std::io::Result<PathBuf>,
{
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }

    let path = Path::new(trimmed);
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    let is_absolute = path.is_absolute() || is_windows_absolute_path(trimmed);
    let looks_like_launch_path = is_absolute
        || trimmed.contains(['/', '\\'])
        || trimmed.starts_with('.')
        || matches!(extension.as_deref(), Some("jar" | "bat" | "sh" | "ps1" | "cmd"));

    if !looks_like_launch_path {
        return None;
    }

    if is_absolute {
        Some(trimmed.to_string())
    } else if let Some(folder) = folder {
        Some(folder.join(path).to_string_lossy().to_string())
    } else {
        current_dir()
            .ok()
            .map(|current_dir| current_dir.join(path).to_string_lossy().to_string())
    }
}

pub fn infer_local_create_startup_mode(
    has_entry: bool,
    resolved_entry_path: Option<&str>,
) -> String {
    if let Some(entry_path) = resolved_entry_path {
        return detect_startup_mode_from_path_like(entry_path);
    }
    if has_entry {
        return "custom".to_string();
    }
    "jar".to_string()
}

pub fn resolve_local_preferred_jar_path(
    startup_mode: &str,
    configured_startup_path: Option<&str>,
    server_path: &Path,
) -> Option<String> {
    resolve_local_preferred_jar_path_checked(startup_mode, configured_startup_path, server_path)
        .ok()
        .flatten()
}

pub fn resolve_local_preferred_jar_path_checked(
    startup_mode: &str,
    configured_startup_path: Option<&str>,
    server_path: &Path,
) -> Result<Option<String>, String> {
    if !startup_mode_prefers_direct_jar(startup_mode) {
        return Ok(None);
    }

    let Some(configured_startup_path) = configured_startup_path else {
        return Ok(None);
    };
    let configured_startup_path = configured_startup_path.trim();
    if configured_startup_path.is_empty() {
        return Ok(None);
    }

    let startup_path_obj = Path::new(configured_startup_path);
    if startup_path_obj
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("jar"))
        .unwrap_or(false)
    {
        Ok(Some(configured_startup_path.to_string()))
    } else {
        find_server_jar_checked(server_path).map(Some)
    }
}

pub fn resolve_direct_jar_launch_target(server_path: &str, jar_path: &str) -> String {
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

pub fn resolve_local_launch_target(
    startup_mode: &str,
    preferred_jar_path: Option<&str>,
    configured_jar_path: Option<&str>,
    custom_command: Option<&str>,
    startup_filename: &str,
) -> String {
    if let Some(preferred_jar_path) = preferred_jar_path {
        return preferred_jar_path.to_string();
    }

    match normalize_cli_startup_mode(Some(startup_mode))
        .unwrap_or_else(|_| "jar".to_string())
        .as_str()
    {
        "jar" | "starter" => configured_jar_path.unwrap_or_default().to_string(),
        "custom" => custom_command.unwrap_or_default().to_string(),
        _ => startup_filename.to_string(),
    }
}

pub fn validate_local_entry_startup_mode(
    startup_mode: &str,
    entry: Option<&str>,
    resolved_entry_path: Option<&str>,
) -> Result<(), String> {
    let Some(entry) = entry.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    if startup_mode == "custom" {
        return Ok(());
    }

    let Some(entry_path) = resolved_entry_path else {
        return Err(format!(
            "--startup {} 需要可解析的启动文件路径；当前 --entry 更像命令文本，请改用 --startup custom 或提供实际脚本/JAR 路径",
            startup_mode
        ));
    };

    let detected_mode = detect_startup_mode_from_path_like(entry_path);
    if detected_mode != startup_mode {
        return Err(format!(
            "--startup {} 与 --entry={} 的文件类型不匹配，检测到的是 {}",
            startup_mode, entry, detected_mode
        ));
    }

    Ok(())
}

pub fn detect_startup_mode_from_path_like(path: &str) -> String {
    let extension = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "bat" | "cmd" => "bat".to_string(),
        "sh" => "sh".to_string(),
        "ps1" => "ps1".to_string(),
        _ => "jar".to_string(),
    }
}

pub fn normalize_cli_startup_mode(value: Option<&str>) -> Result<String, String> {
    let raw = value.unwrap_or("jar").trim().to_ascii_lowercase();
    match raw.as_str() {
        "jar" | "bat" | "sh" | "ps1" | "starter" | "custom" => Ok(raw),
        _ => Err(format!("不支持的 startup mode: {}", raw)),
    }
}

pub fn validate_local_create_folder(folder: Option<&str>) -> Result<Option<&Path>, String> {
    let Some(folder) = folder.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    let folder_path = Path::new(folder);
    if folder_path.exists() && !folder_path.is_dir() {
        return Err(format!("--folder 指定目录不存在或不是文件夹: {}", folder));
    }

    Ok(Some(folder_path))
}

pub fn validate_local_create_startup_path_binding(
    folder_path: Option<&Path>,
    startup_mode: &str,
    jar_path: &str,
    resolved_entry_path: Option<&str>,
) -> Result<(), String> {
    let Some(folder_path) = folder_path else {
        return Ok(());
    };

    if startup_mode == "custom" {
        return Ok(());
    }

    let startup_path = resolved_entry_path.unwrap_or(jar_path).trim();
    if startup_path.is_empty() {
        return Ok(());
    }

    let startup_path_obj = Path::new(startup_path);
    let startup_parent = startup_path_obj.parent().ok_or_else(|| {
        format!(
            "--folder={} 下创建本地服务器时，启动文件必须位于该目录根下",
            folder_path.display()
        )
    })?;

    if !paths_refer_to_same_location(startup_parent, folder_path) {
        return Err(format!(
            "--folder={} 下创建本地服务器时，--jar/--entry 的启动文件必须位于该目录根下；当前路径为 {}",
            folder_path.display(),
            startup_path
        ));
    }

    Ok(())
}

pub fn validate_local_create_startup_exists(
    startup_mode: &str,
    jar_path: &str,
    resolved_entry_path: Option<&str>,
) -> Result<(), String> {
    if startup_mode == "custom" {
        return Ok(());
    }

    let startup_path = resolved_entry_path.unwrap_or(jar_path).trim();
    if startup_path.is_empty() {
        return Err("本地服务器缺少可用的启动文件路径".to_string());
    }

    let startup_path_obj = Path::new(startup_path);
    if startup_path_obj.exists() {
        return Ok(());
    }

    Err(format!(
        "本地服务器启动文件不存在，请先准备好对应 JAR/脚本后再创建: {}",
        startup_path
    ))
}

fn paths_refer_to_same_location(left: &Path, right: &Path) -> bool {
    normalize_path_for_compare(left) == normalize_path_for_compare(right)
}

fn normalize_path_for_compare(path: &Path) -> String {
    let absolute = if path.is_absolute() || is_windows_absolute_path(&path.to_string_lossy()) {
        path.to_path_buf()
    } else if let Ok(current_dir) = std::env::current_dir() {
        current_dir.join(path)
    } else {
        path.to_path_buf()
    };

    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }

    let normalized = normalized
        .to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string();

    if cfg!(windows) {
        normalized.to_ascii_lowercase()
    } else {
        normalized
    }
}

pub fn resolve_local_create_server_path(
    jar_path: &str,
    resolved_entry_path: Option<&str>,
    custom_entry_hint_path: Option<&str>,
) -> Option<String> {
    resolved_entry_path
        .and_then(path_parent_string)
        .or_else(|| path_parent_string(jar_path))
        .or_else(|| custom_entry_hint_path.and_then(path_parent_string))
}

pub fn resolve_java_paths(java_path: &str) -> Result<(String, String), String> {
    let java_path_obj = Path::new(java_path);
    let java_bin_dir = java_path_obj
        .parent()
        .ok_or_else(|| format!("Java 路径无效，缺少 bin 目录: {}", java_path))?;
    let java_home_dir = java_bin_dir.parent().unwrap_or(java_bin_dir);

    Ok((
        java_bin_dir.to_string_lossy().to_string(),
        java_home_dir.to_string_lossy().to_string(),
    ))
}

pub fn startup_filename(startup_path: &str) -> String {
    Path::new(startup_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| startup_path.to_string())
}

pub fn ensure_supported_script_java_major_version(major_version: Option<u32>) -> Result<(), String> {
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

pub fn parse_java_major_version(raw_version: &str) -> Option<u32> {
    let version = raw_version.trim().trim_matches('"');
    let mut parts = version.split('.');
    let first = parts.next()?.parse::<u32>().ok()?;
    if first == 1 {
        parts.next()?.parse::<u32>().ok()
    } else {
        Some(first)
    }
}

pub fn build_java_launch_path_value(java_bin_dir_str: &str, existing_path: &str) -> String {
    prepend_path_entry(java_bin_dir_str, existing_path, path_separator())
}

fn format_command_preview(
    program: &str,
    args: &[String],
    env: &[(String, String)],
) -> String {
    let mut parts = Vec::new();

    if !env.is_empty() {
        let env_parts = env
            .iter()
            .map(|(key, value)| format!("{}={}", key, quote_command_fragment(value)))
            .collect::<Vec<_>>();
        parts.push(format!("env {{{}}}", env_parts.join(", ")));
    }

    parts.push(quote_command_fragment(program));
    parts.extend(args.iter().map(|arg| quote_command_fragment(arg)));
    parts.join(" ")
}

pub fn preview_command(command: &Command) -> String {
    let env = command
        .get_envs()
        .filter_map(|(key, value)| {
            value.map(|value| (key.to_string_lossy().to_string(), value.to_string_lossy().to_string()))
        })
        .collect::<Vec<_>>();
    let args = command
        .get_args()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    format_command_preview(&command.get_program().to_string_lossy(), &args, &env)
}

pub fn decode_console_bytes(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_string();
    }

    #[cfg(target_os = "windows")]
    {
        let (decoded, _, _) = encoding_rs::GBK.decode(bytes);
        decoded.into_owned()
    }
    #[cfg(not(target_os = "windows"))]
    {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

pub fn script_bytes_prefer_utf8(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xEF, 0xBB, 0xBF]) || std::str::from_utf8(bytes).is_ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalLaunchFallbackInfo {
    pub from_mode: String,
    pub to_mode: String,
    pub reason: String,
}

pub fn format_primary_jar_early_exit_reason(status: &str) -> String {
    format!("JAR 直启进程过早退出: {}", status)
}

pub fn format_primary_jar_probe_error_reason(error: &str) -> String {
    format!("JAR 直启状态检查失败: {}", error)
}

pub fn format_primary_jar_spawn_error_reason(error: &str) -> String {
    format!("JAR 直启失败: {}", error)
}

pub fn build_primary_jar_fallback_info(
    configured_mode: &str,
    reason: String,
) -> LocalLaunchFallbackInfo {
    LocalLaunchFallbackInfo {
        from_mode: "jar".to_string(),
        to_mode: configured_mode.to_string(),
        reason,
    }
}

pub fn format_launch_fallback_log(reason: &str, configured_mode: &str) -> String {
    format!("[Sea Lantern] {}，回退到 {} 启动", reason, configured_mode)
}

pub fn format_fallback_chain_error(reason: &str, fallback_error: &str) -> String {
    format!("{}；回退也失败：{}", reason, fallback_error)
}

pub fn prepend_path_entry(path_entry: &str, existing_path: &str, separator: &str) -> String {
    if existing_path.is_empty() {
        path_entry.to_string()
    } else {
        format!("{}{}{}", path_entry, separator, existing_path)
    }
}

#[cfg(target_os = "windows")]
pub fn build_windows_java_env_prefix(java_home_dir_str: &str, java_bin_dir_str: &str) -> String {
    format!(
        "set \"JAVA_HOME={}\" & set \"PATH={};%PATH%\"",
        escape_windows_cmd_arg(java_home_dir_str),
        escape_windows_cmd_arg(java_bin_dir_str)
    )
}

#[cfg(target_os = "windows")]
pub fn build_windows_bat_command_text(
    startup_filename: &str,
    cmd_code_page: &str,
    java_home_dir_str: &str,
    java_bin_dir_str: &str,
) -> String {
    format!(
        "chcp {}>nul & {} & call \"{}\" nogui",
        cmd_code_page,
        build_windows_java_env_prefix(java_home_dir_str, java_bin_dir_str),
        escape_windows_cmd_arg(startup_filename)
    )
}

#[cfg(target_os = "windows")]
fn escape_windows_cmd_arg(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '^' => out.push_str("^^"),
            '&' => out.push_str("^&"),
            '|' => out.push_str("^|"),
            '<' => out.push_str("^<"),
            '>' => out.push_str("^>"),
            '(' => out.push_str("^("),
            ')' => out.push_str("^)"),
            '%' => out.push_str("%%"),
            '"' => out.push_str("\"\""),
            other => out.push(other),
        }
    }
    out
}

#[cfg(target_os = "windows")]
fn path_separator() -> &'static str {
    ";"
}

#[cfg(not(target_os = "windows"))]
fn path_separator() -> &'static str {
    ":"
}

fn quote_command_fragment(value: &str) -> String {
    let requires_quotes = value.is_empty()
        || value.chars().any(|ch| ch.is_whitespace())
        || value.contains('"')
        || value.contains('\'')
        || value.contains(';')
        || value.contains('&')
        || value.contains('|');

    if !requires_quotes {
        return value.to_string();
    }

    if value.contains('"') && !value.contains('\'') {
        return format!("'{}'", value);
    }

    format!("\"{}\"", value.replace('"', "\\\""))
}

fn path_parent_string(path: &str) -> Option<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    Path::new(trimmed)
        .parent()
        .map(|parent| parent.to_string_lossy().to_string())
        .filter(|parent| !parent.trim().is_empty())
}

pub fn normalize_core_type(value: Option<&str>) -> Result<String, String> {
    let raw = value.unwrap_or("paper").trim();
    if raw.is_empty() {
        return Err("--core 不能为空".to_string());
    }
    Ok(CoreType::normalize_to_api_core_key(raw).unwrap_or_else(|| raw.to_string()))
}

fn is_script_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref(),
        Some("bat" | "cmd" | "sh" | "ps1")
    )
}

fn startup_mode_prefers_direct_jar(startup_mode: &str) -> bool {
    matches!(
        normalize_cli_startup_mode(Some(startup_mode))
            .unwrap_or_else(|_| "jar".to_string())
            .as_str(),
        "bat" | "sh" | "ps1"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        detect_startup_mode_from_path_like, inspect_local_folder,
        infer_core_type_from_local_inputs_checked, infer_mc_version_from_folder_checked,
        inspect_local_folder_checked,
        normalize_core_type, normalize_path_for_compare, path_parent_string,
        build_java_launch_path_value, ensure_supported_script_java_major_version,
        decode_console_bytes, format_command_preview, format_fallback_chain_error,
        format_launch_fallback_log, format_primary_jar_early_exit_reason,
        format_primary_jar_probe_error_reason, format_primary_jar_spawn_error_reason,
        parse_java_major_version, resolve_command_path_hint_with, script_bytes_prefer_utf8,
        preview_command,
        prepend_path_entry, resolve_direct_jar_launch_target, resolve_local_create_server_path,
        resolve_attach_executable_path_checked, resolve_local_launch_target,
        resolve_local_preferred_jar_path,
        resolve_local_preferred_jar_path_checked,
        resolve_java_paths, startup_filename, validate_local_create_folder,
        validate_local_create_startup_exists,
        validate_local_create_startup_path_binding,
    };
    use std::path::Path;
    use std::process::Command;
    use tempfile::tempdir;

    #[test]
    fn validate_local_create_folder_accepts_missing_folder_path() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let missing = temp_dir.path().join("fresh-paper");
        let missing_display = missing.to_string_lossy().to_string();

        let resolved = validate_local_create_folder(Some(missing_display.as_str()))
            .expect("missing create folder should still be accepted");

        assert_eq!(resolved, Some(missing.as_path()));
    }

    #[test]
    fn validate_local_create_startup_path_binding_rejects_external_file() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("paper");
        let external = temp_dir.path().join("elsewhere").join("server.jar");

        let err = validate_local_create_startup_path_binding(
            Some(&folder),
            "jar",
            external.to_string_lossy().as_ref(),
            None,
        )
        .expect_err("external startup path should be rejected");

        assert!(err.contains("启动文件必须位于该目录根下"));
    }

    #[test]
    fn validate_local_create_startup_exists_rejects_missing_file() {
        let err = validate_local_create_startup_exists("jar", "E:/missing/server.jar", None)
            .expect_err("missing startup file should fail");

        assert!(err.contains("启动文件不存在"));
    }

    #[test]
    fn normalize_path_for_compare_collapses_relative_segments() {
        let normalized = normalize_path_for_compare(Path::new("E:/servers/./paper/../paper"));
        assert_eq!(normalized, "e:/servers/paper");
    }

    #[test]
    fn resolve_local_create_server_path_prefers_entry_parent() {
        let resolved = resolve_local_create_server_path(
            "E:/servers/paper/server.jar",
            Some("E:/servers/paper/start.bat"),
            None,
        );

        assert_eq!(resolved.as_deref(), Some("E:/servers/paper"));
    }

    #[test]
    fn path_parent_string_rejects_blank_value() {
        assert_eq!(path_parent_string("  "), None);
    }

    #[test]
    fn normalize_core_type_maps_known_aliases() {
        let normalized = normalize_core_type(Some("fabric")).expect("fabric should normalize");
        assert_eq!(normalized, "fabric");
    }

    #[test]
    fn resolve_java_paths_extracts_bin_and_home() {
        let (bin_dir, home_dir) =
            resolve_java_paths("C:/Java/JDK 21/bin/java.exe").expect("java path should resolve");

        assert_eq!(bin_dir, "C:/Java/JDK 21/bin");
        assert_eq!(home_dir, "C:/Java/JDK 21");
    }

    #[test]
    fn startup_filename_prefers_last_path_segment() {
        assert_eq!(startup_filename("E:/servers/paper/start.sh"), "start.sh");
        assert_eq!(startup_filename("server.jar"), "server.jar");
    }

    #[test]
    fn detect_startup_mode_from_path_like_treats_cmd_as_bat() {
        assert_eq!(
            detect_startup_mode_from_path_like("E:/servers/paper/start.cmd"),
            "bat"
        );
    }

    #[test]
    fn inspect_local_folder_prefers_cmd_script_as_attachable_entry() {
        let dir = tempdir().expect("temp dir should exist");
        let cmd_path = dir.path().join("start.cmd");
        std::fs::write(&cmd_path, "@echo off\n").expect("cmd script should write");

        let inspection = inspect_local_folder(dir.path());

        assert_eq!(inspection.startup_mode.as_deref(), Some("bat"));
        assert_eq!(
            inspection.startup_entry_path.as_deref(),
            Some(cmd_path.to_string_lossy().as_ref())
        );
        assert!(inspection.is_attachable());
    }

    #[test]
    fn inspect_local_folder_checked_matches_attachable_folder_shape() {
        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("paper-prod-1.21.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::write(folder.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

        let inspection = inspect_local_folder_checked(&folder)
            .expect("checked inspection should succeed for readable folder");

        assert!(inspection.is_attachable());
        assert_eq!(inspection.startup_mode.as_deref(), Some("sh"));
        assert_eq!(inspection.inferred_core_type.as_deref(), Some("Paper"));
    }

    #[test]
    fn infer_checked_metadata_surfaces_folder_scan_failures() {
        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("paper-prod-1.21.1");
        std::fs::create_dir_all(folder.join("server.jar"))
            .expect("directory-backed jar path should create");

        let core_error = infer_core_type_from_local_inputs_checked(&folder, None)
            .expect_err("checked core inference should surface inspection failures");
        let version_error = infer_mc_version_from_folder_checked(&folder, None)
            .expect_err("checked version inference should surface inspection failures");

        assert!(core_error.contains("本地目录探测失败"));
        assert!(core_error.contains("server.jar"));
        assert!(version_error.contains("本地目录探测失败"));
        assert!(version_error.contains("server.jar"));
    }

    #[test]
    fn infer_checked_core_type_surfaces_script_neighbor_jar_scan_failures() {
        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("mystery-server");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::create_dir(folder.join("server.jar"))
            .expect("directory-backed jar path should create");
        let script_path = folder.join("launch.sh");
        std::fs::write(&script_path, b"#!/bin/sh\n").expect("script should write");

        let error = infer_core_type_from_local_inputs_checked(
            &folder,
            Some(script_path.to_string_lossy().as_ref()),
        )
        .expect_err("checked core inference should surface script neighbor jar scan failures");

        assert!(error.contains("server.jar"));
    }

    #[test]
    fn resolve_attach_executable_path_checked_surfaces_startup_scan_failures() {
        let dir = tempdir().expect("temp dir should exist");
        let missing = dir.path().join("missing");

        let error = resolve_attach_executable_path_checked(&missing)
            .expect_err("checked attach executable resolution should surface startup scan failures");

        assert!(error.contains("读取启动目录失败"), "unexpected error: {}", error);
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
    fn parse_java_major_version_supports_legacy_and_modern_formats() {
        assert_eq!(parse_java_major_version("1.8.0_412"), Some(8));
        assert_eq!(parse_java_major_version("17.0.11"), Some(17));
        assert_eq!(parse_java_major_version("\"21.0.3\""), Some(21));
    }

    #[test]
    fn parse_java_major_version_rejects_unparseable_input() {
        assert_eq!(parse_java_major_version(""), None);
        assert_eq!(parse_java_major_version("not-a-version"), None);
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
    fn build_java_launch_path_value_keeps_java_bin_first() {
        let path = build_java_launch_path_value("E:/java/bin", "C:/Windows/System32");

        assert!(path.starts_with("E:/java/bin"));
    }

    #[test]
    fn format_command_preview_escapes_nested_quotes_and_env_values() {
        let formatted = format_command_preview(
            "cmd",
            &[
                "/c".to_string(),
                "\"C:\\Program Files\\Java\\jdk-21\\bin\\java.exe\" -jar paper.jar nogui"
                    .to_string(),
            ],
            &[("JAVA_HOME".to_string(), "C:/Program Files/Java/JDK 21".to_string())],
        );

        assert_eq!(
            formatted,
            "env {JAVA_HOME=\"C:/Program Files/Java/JDK 21\"} cmd /c '\"C:\\Program Files\\Java\\jdk-21\\bin\\java.exe\" -jar paper.jar nogui'"
        );
    }

    #[test]
    fn local_launch_fallback_messages_are_stable() {
        assert_eq!(
            format_primary_jar_early_exit_reason("exit status: 1"),
            "JAR 直启进程过早退出: exit status: 1"
        );
        assert_eq!(
            format_primary_jar_probe_error_reason("io error"),
            "JAR 直启状态检查失败: io error"
        );
        assert_eq!(
            format_primary_jar_spawn_error_reason("spawn failed"),
            "JAR 直启失败: spawn failed"
        );
        assert_eq!(
            format_launch_fallback_log("JAR 直启失败: spawn failed", "sh"),
            "[Sea Lantern] JAR 直启失败: spawn failed，回退到 sh 启动"
        );
        assert_eq!(
            format_fallback_chain_error("JAR 直启失败: spawn failed", "fallback failed"),
            "JAR 直启失败: spawn failed；回退也失败：fallback failed"
        );
    }

    #[test]
    fn decode_console_bytes_keeps_utf8_text() {
        assert_eq!(decode_console_bytes("hello".as_bytes()), "hello");
    }

    #[test]
    fn script_bytes_prefer_utf8_accepts_bom_and_valid_utf8() {
        assert!(script_bytes_prefer_utf8(&[0xEF, 0xBB, 0xBF, b'h', b'i']));
        assert!(script_bytes_prefer_utf8("你好".as_bytes()));
    }

    #[test]
    fn script_bytes_prefer_utf8_rejects_non_utf8_bytes() {
        assert!(!script_bytes_prefer_utf8(&[0xC4, 0xE3, 0xBA, 0xC3]));
    }

    #[test]
    fn preview_command_formats_program_args_and_env() {
        let mut command = Command::new("java");
        command.env("JAVA_HOME", "C:/Java");
        command.arg("-jar");
        command.arg("server.jar");

        assert_eq!(
            preview_command(&command),
            "env {JAVA_HOME=C:/Java} java -jar server.jar"
        );
    }

    #[test]
    fn resolve_local_preferred_jar_path_prefers_configured_jar_for_script_modes() {
        let resolved = resolve_local_preferred_jar_path(
            "sh",
            Some("E:/servers/paper/server.jar"),
            Path::new("E:/servers/paper"),
        );

        assert_eq!(resolved.as_deref(), Some("E:/servers/paper/server.jar"));
    }

    #[test]
    fn resolve_local_preferred_jar_path_ignores_non_script_modes() {
        let resolved = resolve_local_preferred_jar_path(
            "custom",
            Some("E:/servers/paper/server.jar"),
            Path::new("E:/servers/paper"),
        );

        assert_eq!(resolved, None);
    }

    #[test]
    fn resolve_local_preferred_jar_path_checked_surfaces_server_scan_failures() {
        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("paper-server");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::create_dir(folder.join("server.jar"))
            .expect("directory-backed jar path should create");

        let error = resolve_local_preferred_jar_path_checked(
            "sh",
            Some(folder.join("start.sh").to_string_lossy().as_ref()),
            &folder,
        )
        .expect_err("checked preferred jar resolution should surface server scan failures");

        assert!(error.contains("server.jar"), "unexpected error: {}", error);
    }

    #[test]
    fn resolve_local_preferred_jar_path_legacy_wrapper_still_downgrades_scan_failures() {
        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("paper-server");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::create_dir(folder.join("server.jar"))
            .expect("directory-backed jar path should create");

        let resolved = resolve_local_preferred_jar_path(
            "sh",
            Some(folder.join("start.sh").to_string_lossy().as_ref()),
            &folder,
        );

        assert_eq!(resolved, None);
    }

    #[test]
    fn resolve_command_path_hint_rejects_relative_path_when_current_dir_is_unavailable() {
        let resolved = resolve_command_path_hint_with("server.jar", None, || {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "current dir unavailable",
            ))
        });

        assert_eq!(resolved, None);
    }

    #[test]
    fn resolve_direct_jar_launch_target_uses_filename_for_root_jar() {
        let target =
            resolve_direct_jar_launch_target("E:/servers/fabric-1.20.1", "E:/servers/fabric-1.20.1/server.jar");

        assert_eq!(target, "server.jar");
    }

    #[test]
    fn resolve_direct_jar_launch_target_keeps_nested_or_external_path() {
        let nested = resolve_direct_jar_launch_target(
            "E:/servers/fabric-1.20.1",
            "E:/servers/fabric-1.20.1/libraries/server.jar",
        );
        let external = resolve_direct_jar_launch_target(
            "E:/servers/fabric-1.20.1",
            "E:/srv/shared/server.jar",
        );

        assert_eq!(nested.replace('\\', "/"), "E:/servers/fabric-1.20.1/libraries/server.jar");
        assert_eq!(external.replace('\\', "/"), "E:/srv/shared/server.jar");
    }

    #[test]
    fn resolve_local_launch_target_matches_mode_shape() {
        assert_eq!(
            resolve_local_launch_target(
                "jar",
                None,
                Some("E:/servers/paper/server.jar"),
                Some("java -jar custom.jar nogui"),
                "start.sh",
            ),
            "E:/servers/paper/server.jar"
        );
        assert_eq!(
            resolve_local_launch_target(
                "custom",
                None,
                Some("E:/servers/paper/server.jar"),
                Some("java -jar custom.jar nogui"),
                "start.sh",
            ),
            "java -jar custom.jar nogui"
        );
        assert_eq!(
            resolve_local_launch_target(
                "sh",
                Some("E:/servers/paper/found.jar"),
                Some("E:/servers/paper/server.jar"),
                Some("java -jar custom.jar nogui"),
                "start.sh",
            ),
            "E:/servers/paper/found.jar"
        );
    }
}
