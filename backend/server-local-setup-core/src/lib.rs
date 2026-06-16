use std::path::{Path, PathBuf};
use std::process::Command;

use once_cell::sync::Lazy;
use regex::Regex;
use sea_lantern_runtime::{find_root_startup_file_checked, is_windows_absolute_path};
use sea_lantern_server_installer_core::{
    detect_core_key, detect_core_key_checked, find_server_jar, find_server_jar_checked,
    parse_server_core_key, CoreType,
};

static MC_VERSION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(1\.\d{1,2}(?:\.\d{1,2})?)").expect("mc version regex should compile")
});

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocalFolderInspection {
    pub startup_entry_path: Option<String>,
    pub startup_mode: Option<String>,
    pub detected_jar_path: Option<String>,
    pub inferred_core_type: Option<String>,
    pub inferred_mc_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingServerStartupSelection {
    pub startup_target_path: String,
    pub startup_mode: String,
    pub custom_command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModpackStartupSelection {
    pub startup_mode: String,
    pub custom_command: Option<String>,
    pub startup_file_path: Option<String>,
    pub selected_core_type: Option<String>,
    pub selected_mc_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarterInstallLaunchSelection {
    pub startup_entry_path: String,
    pub startup_mode: String,
    pub startup_filename: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveExistingServerStartupError {
    CustomCommandEmpty,
    ExecutableMissing(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveModpackStartupError {
    StartupFilePathMissing,
    StartupFileMissing(String),
    CustomCommandEmpty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveStarterInstallLaunchError {
    MissingScript,
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
        detect_core_key_checked(path)?
    } else if let Some(path) = detected_jar_path.as_deref() {
        detect_core_key_checked(path)?
    } else {
        detect_core_key(folder_display.as_ref())
    };
    let inferred_core_type = normalize_inferred_core_type(inferred_core_type);
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
    resolve_attach_executable_path_checked(folder)
        .ok()
        .flatten()
}

pub fn resolve_local_startup_entry_checked(
    folder: &Path,
) -> Result<Option<(String, String)>, String> {
    let inspection = inspect_local_folder_checked(folder)?;

    Ok(inspection.preferred_startup_path().map(|path| {
        (
            path.to_string(),
            inspection
                .startup_mode
                .clone()
                .unwrap_or_else(|| detect_startup_mode_from_path_like(path)),
        )
    }))
}

pub fn resolve_existing_server_requested_startup(
    requested_startup_mode: &str,
    custom_command: Option<&str>,
    executable_path: Option<&str>,
) -> Result<Option<ExistingServerStartupSelection>, ResolveExistingServerStartupError> {
    if requested_startup_mode.eq_ignore_ascii_case("custom") {
        let command = trim_optional_text(custom_command)
            .ok_or(ResolveExistingServerStartupError::CustomCommandEmpty)?;
        return Ok(Some(ExistingServerStartupSelection {
            startup_target_path: String::new(),
            startup_mode: "custom".to_string(),
            custom_command: Some(command),
        }));
    }

    if let Some(executable_path) = executable_path {
        let path = Path::new(executable_path);
        if !path.exists() {
            return Err(ResolveExistingServerStartupError::ExecutableMissing(
                executable_path.to_string(),
            ));
        }

        return Ok(Some(ExistingServerStartupSelection {
            startup_target_path: executable_path.to_string(),
            startup_mode: detect_startup_mode_from_path_like(&path.to_string_lossy()),
            custom_command: None,
        }));
    }

    Ok(None)
}

pub fn resolve_run_dir_startup_file_path(
    source_path: &Path,
    run_dir: &Path,
    startup_file_path: &str,
) -> Result<String, String> {
    let startup_path = PathBuf::from(startup_file_path);
    if startup_path.is_relative() {
        return Ok(run_dir.join(&startup_path).to_string_lossy().to_string());
    }

    if source_path.is_file() {
        let source_file_name = source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("server.jar");
        return Ok(run_dir.join(source_file_name).to_string_lossy().to_string());
    }

    if source_path.is_dir() {
        let source_norm = normalize_path_for_compare(source_path);
        let startup_norm = normalize_path_for_compare(&startup_path);
        if startup_norm.starts_with(&(source_norm.clone() + "/")) {
            if let Ok(relative) = startup_path.strip_prefix(source_path) {
                return Ok(run_dir.join(relative).to_string_lossy().to_string());
            }
        }
    }

    Err(startup_file_path.to_string())
}

pub fn resolve_starter_install_launch_selection(
    server_path: &Path,
) -> Result<StarterInstallLaunchSelection, ResolveStarterInstallLaunchError> {
    let inspection = inspect_local_folder_checked(server_path)
        .map_err(|_| ResolveStarterInstallLaunchError::MissingScript)?;
    let startup_entry_path = inspection
        .startup_entry_path
        .ok_or(ResolveStarterInstallLaunchError::MissingScript)?;
    let startup_mode = inspection
        .startup_mode
        .unwrap_or_else(|| detect_startup_mode_from_path_like(&startup_entry_path));

    if startup_mode != "bat" && startup_mode != "sh" && startup_mode != "ps1" {
        return Err(ResolveStarterInstallLaunchError::MissingScript);
    }

    let startup_filename = Path::new(&startup_entry_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or(startup_entry_path.clone());

    Ok(StarterInstallLaunchSelection {
        startup_entry_path,
        startup_mode,
        startup_filename,
    })
}

pub fn resolve_modpack_run_dir_startup_selection(
    source_path: &Path,
    run_dir: &Path,
    requested_startup_mode: &str,
    custom_command: Option<&str>,
    startup_file_path: Option<&str>,
    selected_core_type: Option<&str>,
    selected_mc_version: Option<&str>,
) -> Result<ModpackStartupSelection, ResolveModpackStartupError> {
    let resolved_startup_file_path = startup_file_path
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|raw_path| resolve_run_dir_startup_file_path(source_path, run_dir, raw_path))
        .transpose()
        .map_err(ResolveModpackStartupError::StartupFileMissing)?;

    resolve_modpack_startup_selection(
        requested_startup_mode,
        custom_command,
        resolved_startup_file_path.as_deref(),
        selected_core_type,
        selected_mc_version,
    )
}

pub fn resolve_modpack_startup_selection(
    requested_startup_mode: &str,
    custom_command: Option<&str>,
    startup_file_path: Option<&str>,
    selected_core_type: Option<&str>,
    selected_mc_version: Option<&str>,
) -> Result<ModpackStartupSelection, ResolveModpackStartupError> {
    let startup_mode = normalize_cli_startup_mode(Some(requested_startup_mode))
        .unwrap_or_else(|_| "jar".to_string());
    let requested_custom_command = trim_optional_text(custom_command);
    let startup_file_path = trim_optional_text(startup_file_path);
    let selected_core_type = trim_optional_text(selected_core_type);
    let selected_mc_version = trim_optional_text(selected_mc_version);

    let custom_command = if startup_mode == "custom" {
        requested_custom_command.or_else(|| {
            startup_file_path
                .as_ref()
                .map(|path| format_native_startup_command(path))
        })
    } else {
        requested_custom_command
    };

    let startup_file_path = if startup_mode == "custom" {
        startup_file_path
    } else {
        Some(startup_file_path.ok_or(ResolveModpackStartupError::StartupFilePathMissing)?)
    };

    if startup_mode != "custom" {
        let startup_path = startup_file_path.clone().unwrap_or_default();
        if !Path::new(&startup_path).exists() {
            return Err(ResolveModpackStartupError::StartupFileMissing(startup_path));
        }
    }

    if startup_mode == "custom" && custom_command.is_none() {
        return Err(ResolveModpackStartupError::CustomCommandEmpty);
    }

    Ok(ModpackStartupSelection {
        startup_mode,
        custom_command,
        startup_file_path,
        selected_core_type,
        selected_mc_version,
    })
}

pub fn trim_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub fn format_native_startup_command(path: &str) -> String {
    if path.contains(' ') {
        format!("\"{}\"", path)
    } else {
        path.to_string()
    }
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
        return Ok(normalize_inferred_core_type(detect_core_key_checked(path)?));
    }

    Ok(inspect_local_folder_checked(folder)?.inferred_core_type)
}

pub fn resolve_existing_server_core_type(
    explicit_core_type: Option<&str>,
    startup_mode: &str,
    startup_target_path: &str,
) -> Result<String, String> {
    if let Some(value) = explicit_core_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return normalize_core_type(Some(value));
    }

    if startup_mode.trim().eq_ignore_ascii_case("custom") {
        return Ok("Unknown".to_string());
    }

    Ok(canonical_detected_core_type(&detect_core_key(startup_target_path)))
}

pub fn resolve_local_create_core_type(
    explicit_core_type: Option<&str>,
    folder: Option<&Path>,
    jar_path: &str,
    executable_hint: Option<&str>,
) -> Result<String, String> {
    if let Some(value) = explicit_core_type {
        return normalize_core_type(Some(value));
    }

    if let Some(folder) = folder {
        if let Some(inferred) = infer_core_type_from_local_inputs_checked(folder, executable_hint)?
        {
            return Ok(inferred);
        }
    }

    Ok(canonical_detected_core_type(&detect_core_key(jar_path)))
}

pub fn resolve_local_attach_core_type(
    explicit_core_type: Option<&str>,
    inspected_core_type: Option<&str>,
    folder: &Path,
    executable_hint: Option<&str>,
) -> Option<String> {
    if let Some(value) = explicit_core_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return normalize_core_type(Some(value)).ok();
    }

    if let Some(value) = inspected_core_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(value.to_string());
    }

    infer_core_type_from_local_inputs_checked(folder, executable_hint)
        .ok()
        .flatten()
}

pub fn resolve_local_create_mc_version(
    explicit_mc_version: Option<String>,
    jar_path: &str,
    resolved_name: &str,
    resolved_entry_path: Option<&str>,
    folder_path: Option<&Path>,
    executable_hint: Option<&str>,
) -> Result<String, String> {
    explicit_mc_version
        .or(infer_local_create_mc_version_checked(
            jar_path,
            resolved_name,
            resolved_entry_path,
            folder_path,
            executable_hint,
        )?)
        .ok_or_else(|| "cli.server_setup.local.create_missing_mc_version".to_string())
}

pub fn resolve_local_attach_mc_version(
    explicit_mc_version: Option<String>,
    folder: &str,
    resolved_name: &str,
    inspected_mc_version: Option<&str>,
    folder_path: &Path,
    executable_hint: Option<&str>,
) -> Option<String> {
    explicit_mc_version
        .or_else(|| infer_mc_version_hint(&[folder, resolved_name]))
        .or_else(|| {
            inspected_mc_version
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .or_else(|| {
                    infer_mc_version_from_folder_checked(folder_path, executable_hint)
                        .ok()
                        .flatten()
                })
        })
}

pub fn resolve_docker_create_mc_version(
    explicit_mc_version: Option<String>,
    folder_path: Option<&Path>,
    folder_hint: Option<&str>,
    resolved_name: &str,
) -> Result<String, String> {
    explicit_mc_version
        .or_else(|| folder_path.and_then(|folder| infer_mc_version_from_folder(folder, None)))
        .or_else(|| folder_hint.and_then(|folder| infer_mc_version_hint(&[folder])))
        .or_else(|| infer_mc_version_hint(&[resolved_name]))
        .ok_or_else(|| "cli.server_setup.docker.create_missing_mc_version".to_string())
}

pub fn resolve_docker_create_core_type(
    explicit_core_type: Option<&str>,
    folder_path: Option<&Path>,
    default_core_type: &str,
) -> Result<String, String> {
    explicit_core_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| normalize_core_type(Some(value)))
        .transpose()?
        .or_else(|| folder_path.and_then(|folder| infer_core_type_from_local_inputs(folder, None)))
        .or_else(|| {
            folder_path.and_then(|folder| {
                folder
                    .file_name()
                    .and_then(|name| name.to_str())
                    .and_then(infer_core_type_hint)
            })
        })
        .or_else(|| normalize_core_type(Some(default_core_type)).ok())
        .ok_or_else(|| "cli.server_setup.docker.create_missing_core_type".to_string())
}

pub fn refresh_local_server_core_type(
    current_core_type: &str,
    startup_mode: &str,
    startup_target_path: Option<&str>,
) -> String {
    let fallback_core_type = normalize_core_type(Some(current_core_type))
        .unwrap_or_else(|_| current_core_type.trim().to_string());

    if startup_mode.trim().eq_ignore_ascii_case("custom") {
        return fallback_core_type;
    }

    let Some(startup_target_path) = startup_target_path
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return fallback_core_type;
    };

    let detected = canonical_detected_core_type(&detect_core_key(startup_target_path));
    if detected.is_empty() || detected.eq_ignore_ascii_case("unknown") {
        fallback_core_type
    } else {
        detected
    }
}

fn normalize_inferred_core_type(raw: String) -> Option<String> {
    let normalized = normalize_core_type(Some(&raw)).ok()?;
    (!normalized.eq_ignore_ascii_case("unknown")).then_some(normalized)
}

fn canonical_detected_core_type(raw: &str) -> String {
    normalize_core_type(Some(raw)).unwrap_or_else(|_| raw.trim().to_string())
}

fn infer_core_type_hint(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(parsed) = parse_server_core_key(trimmed) {
        if !parsed.core_type.eq_ignore_ascii_case("unknown") {
            return Some(parsed.core_type);
        }
    }

    let detected = canonical_detected_core_type(&detect_core_key(trimmed));
    if !detected.eq_ignore_ascii_case("unknown") {
        return Some(detected);
    }

    let stripped = strip_version_suffix(trimmed);
    if stripped != trimmed {
        if let Some(normalized) = CoreType::normalize_to_api_core_key(stripped) {
            return Some(normalized);
        }

        let detected = canonical_detected_core_type(&detect_core_key(stripped));
        if !detected.eq_ignore_ascii_case("unknown") {
            return Some(detected);
        }
    }

    None
}

fn strip_version_suffix(input: &str) -> &str {
    let Some(capture) = MC_VERSION_PATTERN.captures(input) else {
        return input;
    };
    let Some(version) = capture.get(1) else {
        return input;
    };

    let prefix = input[..version.start()].trim_end_matches(['-', '_', ' ', '.']);
    if prefix.is_empty() {
        input
    } else {
        prefix
    }
}

pub fn infer_mc_version_from_folder(
    folder: &Path,
    executable_path: Option<&str>,
) -> Option<String> {
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

pub fn resolve_direct_custom_command(
    command_text: &str,
    folder: Option<&Path>,
) -> Option<(String, Vec<String>)> {
    let trimmed = command_text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if !custom_command_text_is_shell_safe(trimmed) {
        return None;
    }

    if let Some(program) = resolve_direct_program_only_command(trimmed, folder) {
        return Some((program, Vec::new()));
    }

    let tokens = shlex::split(trimmed)?;
    let (program_token, args) = tokens.split_first()?;
    if !custom_command_tokens_are_shell_safe(args) {
        return None;
    }

    let program = resolve_command_path_hint(program_token, folder).or_else(|| {
        if program_token.contains(['/', '\\']) || program_token.starts_with('.') {
            Some(program_token.to_string())
        } else {
            None
        }
    })?;

    Some((program, args.to_vec()))
}

fn custom_command_text_is_shell_safe(command_text: &str) -> bool {
    !command_text.contains(['|', '&', ';', '<', '>', '\n', '\r'])
        && !command_text.contains("&&")
        && !command_text.contains("||")
}

fn resolve_direct_program_only_command(command_text: &str, folder: Option<&Path>) -> Option<String> {
    if let Some(unquoted) = strip_matching_quotes(command_text) {
        if let Some(program) = resolve_command_path_hint(unquoted, folder).or_else(|| {
            let path = Path::new(unquoted);
            if path.is_absolute() || is_windows_absolute_path(unquoted) {
                Some(unquoted.to_string())
            } else {
                None
            }
        }) {
            return Some(program);
        }
    }

    resolve_command_path_hint(command_text, folder)
}

fn strip_matching_quotes(value: &str) -> Option<&str> {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return Some(&value[1..value.len() - 1]);
    }

    None
}

fn custom_command_tokens_are_shell_safe(tokens: &[String]) -> bool {
    tokens.iter().all(|token| {
        !token.contains(['|', '&', ';', '<', '>', '\n', '\r'])
            && token != "&&"
            && token != "||"
    })
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
    let folder_candidate_exists = folder.is_some_and(|folder| folder.join(path).exists());
    let is_absolute = path.is_absolute() || is_windows_absolute_path(trimmed);
    let looks_like_launch_path = is_absolute
        || trimmed.contains(['/', '\\'])
        || trimmed.starts_with('.')
        || folder_candidate_exists
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

pub fn startup_mode_is_custom(startup_mode: &str) -> bool {
    normalize_cli_startup_mode(Some(startup_mode)).unwrap_or_else(|_| "jar".to_string()) == "custom"
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedConsoleEncoding {
    Utf8,
    #[cfg(target_os = "windows")]
    Gbk,
}

impl ManagedConsoleEncoding {
    pub fn java_name(self) -> &'static str {
        match self {
            ManagedConsoleEncoding::Utf8 => "UTF-8",
            #[cfg(target_os = "windows")]
            ManagedConsoleEncoding::Gbk => "GBK",
        }
    }

    #[cfg(target_os = "windows")]
    pub fn cmd_code_page(self) -> &'static str {
        match self {
            ManagedConsoleEncoding::Utf8 => "65001",
            ManagedConsoleEncoding::Gbk => "936",
        }
    }
}

pub fn startup_mode_is_starter(startup_mode: &str) -> bool {
    normalize_cli_startup_mode(Some(startup_mode)).unwrap_or_else(|_| "jar".to_string())
        == "starter"
}

pub fn startup_mode_uses_windows_script_encoding_detection(startup_mode: &str) -> bool {
    matches!(
        normalize_cli_startup_mode(Some(startup_mode))
            .unwrap_or_else(|_| "jar".to_string())
            .as_str(),
        "bat" | "ps1"
    )
}

pub fn windows_script_prefers_utf8(startup_mode: &str, startup_path: &Path) -> bool {
    if !startup_mode_uses_windows_script_encoding_detection(startup_mode) {
        return true;
    }

    let bytes = match std::fs::read(startup_path) {
        Ok(bytes) => bytes,
        Err(_) => return true,
    };

    script_bytes_prefer_utf8(&bytes)
}

pub fn resolve_managed_console_encoding(
    startup_mode: &str,
    startup_path: &Path,
) -> ManagedConsoleEncoding {
    if startup_mode_is_custom(startup_mode) {
        return ManagedConsoleEncoding::Utf8;
    }

    #[cfg(target_os = "windows")]
    {
        if startup_mode_uses_windows_script_encoding_detection(startup_mode) {
            return if windows_script_prefers_utf8(startup_mode, startup_path) {
                ManagedConsoleEncoding::Utf8
            } else {
                ManagedConsoleEncoding::Gbk
            };
        }
    }

    ManagedConsoleEncoding::Utf8
}

pub fn local_cpu_policy_supported_startup_mode(startup_mode: &str) -> bool {
    matches!(
        normalize_cli_startup_mode(Some(startup_mode))
            .unwrap_or_else(|_| "jar".to_string())
            .as_str(),
        "jar" | "starter"
    )
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

pub fn paths_equal(left: &Path, right: &Path) -> bool {
    normalize_path_for_compare(left) == normalize_path_for_compare(right)
}

pub fn path_is_child_of(candidate: &Path, parent: &Path) -> bool {
    let candidate_norm = normalize_path_for_compare(candidate);
    let parent_norm = normalize_path_for_compare(parent);

    candidate_norm.starts_with(&(parent_norm + "/"))
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
    if java_path.trim().is_empty() {
        return Ok((String::new(), String::new()));
    }

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

pub fn ensure_supported_script_java_major_version(
    major_version: Option<u32>,
) -> Result<(), String> {
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

pub fn detect_java_major_version(java_path: &str) -> Option<u32> {
    let output = Command::new(java_path).arg("-version").output().ok()?;
    let text = if output.stderr.is_empty() {
        decode_console_bytes(&output.stdout)
    } else {
        decode_console_bytes(&output.stderr)
    };

    parse_java_major_version_output(&text)
}

pub fn build_java_launch_path_value(java_bin_dir_str: &str, existing_path: &str) -> String {
    prepend_path_entry(java_bin_dir_str, existing_path, path_separator())
}

fn format_command_preview(program: &str, args: &[String], env: &[(String, String)]) -> String {
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
            value.map(|value| {
                (key.to_string_lossy().to_string(), value.to_string_lossy().to_string())
            })
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

fn parse_java_major_version_output(text: &str) -> Option<u32> {
    for line in text.lines() {
        if line.contains("version") {
            let mut segments = line.split('"');
            let _ = segments.next();
            if let Some(version_text) = segments.next() {
                return parse_java_major_version(version_text);
            }
        }
    }

    None
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

pub fn canonical_core_type(input: &str) -> String {
    let trimmed = input.trim();
    CoreType::normalize_to_api_core_key(trimmed).unwrap_or_else(|| trimmed.to_string())
}

pub fn normalize_core_type(value: Option<&str>) -> Result<String, String> {
    let raw = value.unwrap_or("paper").trim();
    if raw.is_empty() {
        return Err("--core 不能为空".to_string());
    }

    Ok(canonical_core_type(raw))
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
        "jar" | "starter"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_java_launch_path_value, canonical_core_type, decode_console_bytes,
        detect_startup_mode_from_path_like, ensure_supported_script_java_major_version,
        format_command_preview, format_fallback_chain_error, format_launch_fallback_log,
        format_native_startup_command, format_primary_jar_early_exit_reason,
        format_primary_jar_probe_error_reason, format_primary_jar_spawn_error_reason,
        infer_core_type_from_local_inputs_checked, infer_mc_version_from_folder_checked,
        inspect_local_folder, inspect_local_folder_checked,
        local_cpu_policy_supported_startup_mode, normalize_core_type, normalize_path_for_compare,
        parse_java_major_version, parse_java_major_version_output, path_is_child_of,
        path_parent_string, paths_equal, prepend_path_entry, preview_command,
        refresh_local_server_core_type, resolve_attach_executable_path_checked,
        resolve_command_path_hint_with, resolve_direct_custom_command,
        resolve_direct_jar_launch_target,
        resolve_docker_create_core_type, resolve_docker_create_mc_version,
        resolve_existing_server_core_type, resolve_existing_server_requested_startup,
        resolve_java_paths, resolve_local_attach_core_type, resolve_local_attach_mc_version,
        resolve_local_create_core_type, resolve_local_create_mc_version,
        resolve_local_create_server_path, resolve_local_launch_target,
        resolve_local_preferred_jar_path, resolve_local_preferred_jar_path_checked,
        resolve_local_startup_entry_checked, resolve_modpack_run_dir_startup_selection,
        resolve_modpack_startup_selection, resolve_run_dir_startup_file_path,
        resolve_starter_install_launch_selection, script_bytes_prefer_utf8, startup_filename,
        startup_mode_is_custom, startup_mode_is_starter,
        startup_mode_uses_windows_script_encoding_detection, trim_optional_text,
        validate_local_create_folder, validate_local_create_startup_exists,
        validate_local_create_startup_path_binding, windows_script_prefers_utf8,
        ExistingServerStartupSelection, ModpackStartupSelection, ResolveExistingServerStartupError,
        ResolveModpackStartupError, ResolveStarterInstallLaunchError,
        StarterInstallLaunchSelection,
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
    fn paths_equal_normalizes_relative_segments_and_case() {
        assert!(paths_equal(
            Path::new("E:/servers/./paper/../paper"),
            Path::new("e:/servers/paper/")
        ));
    }

    #[test]
    fn path_is_child_of_requires_real_child_path() {
        assert!(path_is_child_of(
            Path::new("E:/servers/source/run/server"),
            Path::new("E:/servers/source")
        ));
        assert!(!path_is_child_of(
            Path::new("E:/servers/source"),
            Path::new("E:/servers/source")
        ));
        assert!(!path_is_child_of(
            Path::new("E:/servers/source-sibling"),
            Path::new("E:/servers/source")
        ));
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
        assert_eq!(canonical_core_type("AllayMC"), "allay");
        assert_eq!(canonical_core_type("Arclight-Neoforge"), "arclight-neoforge");

        let normalized = normalize_core_type(Some("fabric")).expect("fabric should normalize");
        assert_eq!(normalized, "fabric");

        let waterfall = normalize_core_type(Some("Waterfall")).expect("waterfall should normalize");
        assert_eq!(waterfall, "waterfall");

        let bedrock = normalize_core_type(Some("bedrock-dedicated-server"))
            .expect("bedrock alias should normalize");
        assert_eq!(bedrock, "bds");

        let legacy_leaf = normalize_core_type(Some("Leaf")).expect("leaf alias should normalize");
        assert_eq!(legacy_leaf, "leaves");

        let legacy_nukkitx =
            normalize_core_type(Some("nukkitx")).expect("nukkitx alias should normalize");
        assert_eq!(legacy_nukkitx, "nukkit");

        let legacy_spongeforge =
            normalize_core_type(Some("spongeforge")).expect("spongeforge alias should normalize");
        assert_eq!(legacy_spongeforge, "forge");
    }

    #[test]
    fn resolve_existing_server_core_type_preserves_custom_unknown_without_explicit_core() {
        let core_type = resolve_existing_server_core_type(None, "custom", "E:/servers/run.bat")
            .expect("custom attach should resolve");

        assert_eq!(core_type, "Unknown");
    }

    #[test]
    fn resolve_existing_server_core_type_canonicalizes_explicit_aliases() {
        let core_type =
            resolve_existing_server_core_type(Some("Leaf"), "jar", "E:/servers/server.jar")
                .expect("explicit core type should normalize");

        assert_eq!(core_type, "leaves");
    }

    #[test]
    fn resolve_existing_server_requested_startup_handles_custom_and_explicit_executable() {
        let custom =
            resolve_existing_server_requested_startup("custom", Some("  launch-paper  "), None)
                .expect("custom startup should resolve");

        assert_eq!(
            custom,
            Some(ExistingServerStartupSelection {
                startup_target_path: String::new(),
                startup_mode: "custom".to_string(),
                custom_command: Some("launch-paper".to_string()),
            })
        );

        let dir = tempdir().expect("temp dir should exist");
        let script_path = dir.path().join("start.cmd");
        std::fs::write(&script_path, b"@echo off\r\n").expect("script should write");

        let explicit = resolve_existing_server_requested_startup(
            "jar",
            None,
            Some(script_path.to_string_lossy().as_ref()),
        )
        .expect("explicit executable should resolve")
        .expect("selection should exist");

        assert_eq!(explicit.startup_target_path, script_path.to_string_lossy());
        assert_eq!(explicit.startup_mode, "bat");
        assert_eq!(explicit.custom_command, None);
    }

    #[test]
    fn resolve_existing_server_requested_startup_reports_validation_errors() {
        let custom_err = resolve_existing_server_requested_startup("custom", Some("   "), None)
            .expect_err("blank custom command should fail");
        assert_eq!(custom_err, ResolveExistingServerStartupError::CustomCommandEmpty);

        let missing = "E:/missing/start.sh";
        let executable_err = resolve_existing_server_requested_startup("jar", None, Some(missing))
            .expect_err("missing executable should fail");
        assert_eq!(
            executable_err,
            ResolveExistingServerStartupError::ExecutableMissing(missing.to_string())
        );
    }

    #[test]
    fn resolve_local_create_core_type_prefers_folder_inference_before_jar_fallback() {
        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("paper-prod-1.21.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        let script_path = folder.join("start.sh");
        std::fs::write(&script_path, b"#!/bin/sh\n").expect("script should write");

        let core_type = resolve_local_create_core_type(
            None,
            Some(&folder),
            "E:/servers/mystery-server.jar",
            Some(script_path.to_string_lossy().as_ref()),
        )
        .expect("local create core type should resolve");

        assert_eq!(core_type, "paper");
    }

    #[test]
    fn resolve_local_attach_core_type_reuses_checked_folder_inference() {
        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("paper-prod-1.21.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::write(folder.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

        let core_type = resolve_local_attach_core_type(None, None, &folder, None);

        assert_eq!(core_type.as_deref(), Some("paper"));
    }

    #[test]
    fn resolve_local_attach_core_type_preserves_attach_fallback_semantics() {
        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("mystery-server");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::create_dir(folder.join("server.jar"))
            .expect("directory-backed jar path should create");

        let core_type = resolve_local_attach_core_type(None, None, &folder, None);

        assert_eq!(core_type, None);
    }

    #[test]
    fn resolve_local_startup_entry_checked_prefers_script_and_keeps_detected_mode() {
        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("paper-prod-1.21.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        let script_path = folder.join("start.sh");
        std::fs::write(&script_path, b"#!/bin/sh\n").expect("script should write");

        let resolved = resolve_local_startup_entry_checked(&folder)
            .expect("startup entry resolution should succeed")
            .expect("startup entry should exist");

        assert_eq!(resolved.0, script_path.to_string_lossy().to_string());
        assert_eq!(resolved.1, "sh");
    }

    #[test]
    fn resolve_run_dir_startup_file_path_maps_relative_and_source_relative_paths() {
        let dir = tempdir().expect("temp dir should exist");
        let source_dir = dir.path().join("pack");
        let run_dir = dir.path().join("run");
        std::fs::create_dir_all(source_dir.join("bin")).expect("source dir should create");
        std::fs::create_dir_all(&run_dir).expect("run dir should create");

        let relative = resolve_run_dir_startup_file_path(&source_dir, &run_dir, "start.sh")
            .expect("relative startup path should map into run dir");
        let absolute = resolve_run_dir_startup_file_path(
            &source_dir,
            &run_dir,
            source_dir
                .join("bin")
                .join("start.sh")
                .to_string_lossy()
                .as_ref(),
        )
        .expect("source-relative absolute path should map into run dir");

        assert_eq!(relative, run_dir.join("start.sh").to_string_lossy().to_string());
        assert_eq!(
            absolute,
            run_dir
                .join("bin")
                .join("start.sh")
                .to_string_lossy()
                .to_string()
        );
    }

    #[test]
    fn resolve_modpack_startup_selection_derives_custom_command_from_startup_path() {
        let dir = tempdir().expect("temp dir should exist");
        let startup_path = dir.path().join("Paper 1.21").join("start.sh");
        std::fs::create_dir_all(startup_path.parent().expect("parent should exist"))
            .expect("parent dir should create");
        std::fs::write(&startup_path, b"#!/bin/sh\n").expect("startup file should exist");

        let resolved = resolve_modpack_startup_selection(
            "custom",
            None,
            Some(startup_path.to_string_lossy().as_ref()),
            Some(" AllayMC "),
            Some(" 1.21.1 "),
        )
        .expect("custom modpack startup should resolve");

        assert_eq!(
            resolved,
            ModpackStartupSelection {
                startup_mode: "custom".to_string(),
                custom_command: Some(format!("\"{}\"", startup_path.to_string_lossy())),
                startup_file_path: Some(startup_path.to_string_lossy().to_string()),
                selected_core_type: Some("AllayMC".to_string()),
                selected_mc_version: Some("1.21.1".to_string()),
            }
        );
    }

    #[test]
    fn resolve_modpack_run_dir_startup_selection_maps_relative_path_before_resolution() {
        let dir = tempdir().expect("temp dir should exist");
        let source_dir = dir.path().join("pack");
        let run_dir = dir.path().join("run");
        std::fs::create_dir_all(&source_dir).expect("source dir should create");
        std::fs::create_dir_all(&run_dir).expect("run dir should create");
        let startup_path = run_dir.join("start.sh");
        std::fs::write(&startup_path, b"#!/bin/sh\n").expect("startup file should exist");

        let resolved = resolve_modpack_run_dir_startup_selection(
            &source_dir,
            &run_dir,
            "jar",
            None,
            Some("start.sh"),
            Some("paper"),
            Some("1.21.1"),
        )
        .expect("run-dir startup selection should resolve");

        assert_eq!(resolved.startup_mode, "jar");
        assert_eq!(
            resolved.startup_file_path.as_deref(),
            Some(startup_path.to_string_lossy().as_ref())
        );
        assert_eq!(resolved.selected_core_type.as_deref(), Some("paper"));
        assert_eq!(resolved.selected_mc_version.as_deref(), Some("1.21.1"));
    }

    #[test]
    fn resolve_modpack_startup_selection_requires_non_custom_startup_file_and_existing_path() {
        let missing = resolve_modpack_startup_selection("jar", None, None, None, None)
            .expect_err("non-custom startup should require startup file path");
        assert_eq!(missing, ResolveModpackStartupError::StartupFilePathMissing);

        let nonexistent = resolve_modpack_startup_selection(
            "jar",
            None,
            Some("E:/missing/server.jar"),
            None,
            None,
        )
        .expect_err("nonexistent startup path should fail");
        assert_eq!(
            nonexistent,
            ResolveModpackStartupError::StartupFileMissing("E:/missing/server.jar".to_string())
        );
    }

    #[test]
    fn resolve_modpack_startup_selection_requires_custom_command_when_no_path_fallback_exists() {
        let err = resolve_modpack_startup_selection("custom", Some("   "), None, None, None)
            .expect_err("blank custom command without path fallback should fail");
        assert_eq!(err, ResolveModpackStartupError::CustomCommandEmpty);
    }

    #[test]
    fn trim_optional_text_discards_blank_values() {
        assert_eq!(trim_optional_text(Some("  paper  ")), Some("paper".to_string()));
        assert_eq!(trim_optional_text(Some("   ")), None);
        assert_eq!(trim_optional_text(None), None);
    }

    #[test]
    fn local_cpu_policy_supported_startup_mode_only_allows_jar_like_modes() {
        assert!(local_cpu_policy_supported_startup_mode("jar"));
        assert!(local_cpu_policy_supported_startup_mode("starter"));
        assert!(!local_cpu_policy_supported_startup_mode("bat"));
        assert!(!local_cpu_policy_supported_startup_mode("custom"));
    }

    #[test]
    fn startup_mode_predicates_reuse_normalized_mode_values() {
        assert!(startup_mode_is_custom("CUSTOM"));
        assert!(!startup_mode_is_custom("jar"));
        assert!(startup_mode_is_starter("starter"));
        assert!(startup_mode_is_starter("STARTER"));
        assert!(!startup_mode_is_starter("sh"));
        assert!(startup_mode_uses_windows_script_encoding_detection("bat"));
        assert!(startup_mode_uses_windows_script_encoding_detection("PS1"));
        assert!(!startup_mode_uses_windows_script_encoding_detection("sh"));
    }

    #[test]
    fn windows_script_prefers_utf8_only_for_supported_script_modes() {
        let dir = tempdir().expect("temp dir should exist");
        let batch = dir.path().join("start.bat");
        std::fs::write(&batch, "你好".as_bytes()).expect("utf8 script should write");

        assert!(windows_script_prefers_utf8("bat", &batch));
        assert!(windows_script_prefers_utf8("jar", &batch));

        let gbk = dir.path().join("start.ps1");
        std::fs::write(&gbk, &[0xC4, 0xE3, 0xBA, 0xC3]).expect("gbk-like bytes should write");
        assert!(!windows_script_prefers_utf8("ps1", &gbk));
    }

    #[test]
    fn resolve_starter_install_launch_selection_requires_supported_script_entry() {
        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("starter-paper");
        std::fs::create_dir_all(&folder).expect("folder should create");
        let script_path = folder.join("start.ps1");
        std::fs::write(&script_path, b"Write-Host start\n").expect("script should write");

        let resolved = resolve_starter_install_launch_selection(&folder)
            .expect("starter launch selection should resolve");

        assert_eq!(
            resolved,
            StarterInstallLaunchSelection {
                startup_entry_path: script_path.to_string_lossy().to_string(),
                startup_mode: "ps1".to_string(),
                startup_filename: "start.ps1".to_string(),
            }
        );

        let empty = dir.path().join("empty-starter");
        std::fs::create_dir_all(&empty).expect("empty folder should create");
        let err = resolve_starter_install_launch_selection(&empty)
            .expect_err("missing starter script should fail");
        assert_eq!(err, ResolveStarterInstallLaunchError::MissingScript);
    }

    #[test]
    fn format_native_startup_command_quotes_paths_with_spaces() {
        assert_eq!(
            format_native_startup_command("E:/Servers/Paper 1.21/start.bat"),
            "\"E:/Servers/Paper 1.21/start.bat\""
        );
        assert_eq!(
            format_native_startup_command("E:/Servers/Paper/start.bat"),
            "E:/Servers/Paper/start.bat"
        );
    }

    #[test]
    fn refresh_local_server_core_type_prefers_detected_non_unknown_value() {
        let refreshed =
            refresh_local_server_core_type("paper", "jar", Some("E:/servers/nukkit.jar"));

        assert_eq!(refreshed, "nukkit");
    }

    #[test]
    fn refresh_local_server_core_type_preserves_current_for_custom_or_unknown_detection() {
        assert_eq!(
            refresh_local_server_core_type("Leaf", "custom", Some("E:/servers/run.bat")),
            "leaves"
        );
        assert_eq!(
            refresh_local_server_core_type("paper", "jar", Some("E:/servers/server.jar")),
            "paper"
        );
    }

    #[test]
    fn resolve_local_create_mc_version_prefers_explicit_and_checked_inference() {
        assert_eq!(
            resolve_local_create_mc_version(
                Some("1.20.1".to_string()),
                "E:/servers/server.jar",
                "fabric-cache",
                None,
                None,
                None,
            )
            .expect("explicit mc version should win"),
            "1.20.1"
        );

        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("paper-1.21.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        let entry_path = folder.join("start.sh");
        std::fs::write(&entry_path, b"#!/bin/sh\n").expect("entry should write");

        assert_eq!(
            resolve_local_create_mc_version(
                None,
                "server.jar",
                "paper-cache",
                Some(entry_path.to_string_lossy().as_ref()),
                Some(&folder),
                None,
            )
            .expect("checked inference should resolve mc version"),
            "1.21.1"
        );
    }

    #[test]
    fn resolve_local_attach_mc_version_reuses_hint_and_swallowed_checked_fallback() {
        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("paper-1.21.1");
        std::fs::create_dir_all(&folder).expect("folder should create");

        assert_eq!(
            resolve_local_attach_mc_version(
                None,
                folder.to_string_lossy().as_ref(),
                "paper-prod",
                None,
                &folder,
                None,
            )
            .as_deref(),
            Some("1.21.1")
        );

        let broken = dir.path().join("broken-server");
        std::fs::create_dir_all(&broken).expect("broken folder should create");
        std::fs::create_dir(broken.join("server.jar"))
            .expect("directory-backed jar path should create");

        assert_eq!(
            resolve_local_attach_mc_version(
                None,
                broken.to_string_lossy().as_ref(),
                "mystery-server",
                None,
                &broken,
                None,
            ),
            None
        );
    }

    #[test]
    fn resolve_docker_create_mc_version_prefers_explicit_then_folder_then_name_hint() {
        assert_eq!(
            resolve_docker_create_mc_version(
                Some("latest".to_string()),
                None,
                None,
                "paper-docker",
            )
            .expect("explicit mc version should win"),
            "latest"
        );

        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("fabric-1.20.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::write(folder.join("fabric-server.jar"), b"placeholder").expect("jar should write");

        assert_eq!(
            resolve_docker_create_mc_version(None, Some(&folder), None, "docker-server")
                .expect("folder inference should resolve mc version"),
            "1.20.1"
        );

        assert_eq!(
            resolve_docker_create_mc_version(None, None, None, "paper-1.21.1")
                .expect("name hint should resolve mc version"),
            "1.21.1"
        );
    }

    #[test]
    fn resolve_docker_create_core_type_prefers_explicit_then_folder_then_default() {
        assert_eq!(
            resolve_docker_create_core_type(Some("bedrock"), None, "paper")
                .expect("explicit core type should normalize"),
            "bds"
        );

        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("allay-1.21.1");
        std::fs::create_dir_all(&folder).expect("folder should create");

        assert_eq!(
            resolve_docker_create_core_type(None, Some(&folder), "paper")
                .expect("folder inference should resolve core type"),
            "allay"
        );

        assert_eq!(
            resolve_docker_create_core_type(None, None, "paper")
                .expect("default core type should be used"),
            "paper"
        );
    }

    #[test]
    fn resolve_java_paths_extracts_bin_and_home() {
        let (bin_dir, home_dir) =
            resolve_java_paths("C:/Java/JDK 21/bin/java.exe").expect("java path should resolve");

        assert_eq!(bin_dir, "C:/Java/JDK 21/bin");
        assert_eq!(home_dir, "C:/Java/JDK 21");
    }

    #[test]
    fn resolve_java_paths_accepts_blank_java_path_as_empty_pair() {
        assert_eq!(
            resolve_java_paths("   ").expect("blank java path should resolve"),
            (String::new(), String::new())
        );
    }

    #[test]
    fn startup_filename_prefers_last_path_segment() {
        assert_eq!(startup_filename("E:/servers/paper/start.sh"), "start.sh");
        assert_eq!(startup_filename("server.jar"), "server.jar");
    }

    #[test]
    fn detect_startup_mode_from_path_like_treats_cmd_as_bat() {
        assert_eq!(detect_startup_mode_from_path_like("E:/servers/paper/start.cmd"), "bat");
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
        assert_eq!(inspection.inferred_core_type.as_deref(), Some("paper"));
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

        let error = resolve_attach_executable_path_checked(&missing).expect_err(
            "checked attach executable resolution should surface startup scan failures",
        );

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
    fn detect_java_major_version_reads_quoted_version_output() {
        let output = "openjdk version \"21.0.3\" 2024-04-16\nOpenJDK Runtime Environment";

        assert_eq!(parse_java_major_version_output(output), Some(21));
    }

    #[test]
    fn detect_java_major_version_handles_legacy_version_output() {
        let output = "java version \"1.8.0_412\"\nJava(TM) SE Runtime Environment";

        assert_eq!(parse_java_major_version_output(output), Some(8));
    }

    #[test]
    fn detect_java_major_version_returns_none_without_quoted_version_line() {
        let output = "openjdk full version 21.0.3 without quotes";

        assert_eq!(parse_java_major_version_output(output), None);
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

        assert_eq!(preview_command(&command), "env {JAVA_HOME=C:/Java} java -jar server.jar");
    }

    #[test]
    fn resolve_local_preferred_jar_path_prefers_configured_jar_for_jar_mode() {
        let resolved = resolve_local_preferred_jar_path(
            "jar",
            Some("E:/servers/paper/server.jar"),
            Path::new("E:/servers/paper"),
        );

        assert_eq!(resolved.as_deref(), Some("E:/servers/paper/server.jar"));
    }

    #[test]
    fn resolve_local_preferred_jar_path_ignores_script_mode_even_when_jar_exists() {
        let resolved = resolve_local_preferred_jar_path(
            "sh",
            Some("E:/servers/paper/server.jar"),
            Path::new("E:/servers/paper"),
        );

        assert_eq!(resolved, None);
    }

    #[test]
    fn resolve_local_preferred_jar_path_checked_surfaces_server_scan_failures_for_jar_mode() {
        let dir = tempdir().expect("temp dir should exist");
        let folder = dir.path().join("paper-server");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::create_dir(folder.join("server.jar"))
            .expect("directory-backed jar path should create");

        let error = resolve_local_preferred_jar_path_checked(
            "jar",
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
            "jar",
            Some(folder.join("start.sh").to_string_lossy().as_ref()),
            &folder,
        );

        assert_eq!(resolved, None);
    }

    #[test]
    fn resolve_command_path_hint_rejects_relative_path_when_current_dir_is_unavailable() {
        let resolved = resolve_command_path_hint_with("server.jar", None, || {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "current dir unavailable"))
        });

        assert_eq!(resolved, None);
    }

    #[test]
    fn resolve_direct_custom_command_accepts_direct_program_with_args() {
        let dir = tempdir().expect("temp dir should exist");
        let server_dir = dir.path().join("pumpkin-server");
        std::fs::create_dir_all(&server_dir).expect("server dir should create");
        let exe_path = server_dir.join("pumpkin-X64-Windows.exe");
        std::fs::write(&exe_path, b"pumpkin").expect("exe placeholder should write");

        let resolved = resolve_direct_custom_command(
            "pumpkin-X64-Windows.exe --no-gui",
            Some(&server_dir),
        )
        .expect("direct custom command should resolve");

        assert_eq!(resolved.0, exe_path.to_string_lossy().to_string());
        assert_eq!(resolved.1, vec!["--no-gui".to_string()]);
    }

    #[test]
    fn resolve_direct_custom_command_rejects_shell_operators() {
        let resolved = resolve_direct_custom_command("pumpkin.exe && echo ready", None);

        assert_eq!(resolved, None);
    }

    #[test]
    fn resolve_direct_custom_command_rejects_plain_shell_builtins() {
        let resolved = resolve_direct_custom_command("echo launch ready", None);

        assert_eq!(resolved, None);
    }

    #[test]
    fn resolve_direct_custom_command_accepts_absolute_windows_path_without_quotes() {
        let resolved = resolve_direct_custom_command(
            "C:\\Users\\miaom\\Downloads\\cache\\test2\\pumpkin-X64-Windows.exe",
            None,
        )
        .expect("absolute windows path should resolve as direct program");

        assert_eq!(
            resolved.0,
            "C:\\Users\\miaom\\Downloads\\cache\\test2\\pumpkin-X64-Windows.exe"
        );
        assert!(resolved.1.is_empty());
    }

    #[test]
    fn resolve_direct_custom_command_accepts_quoted_absolute_windows_path_without_args() {
        let resolved = resolve_direct_custom_command(
            "\"C:\\Users\\miaom\\Downloads\\cache\\test2\\pumpkin-X64-Windows.exe\"",
            None,
        )
        .expect("quoted absolute windows path should resolve as direct program");

        assert_eq!(
            resolved.0,
            "C:\\Users\\miaom\\Downloads\\cache\\test2\\pumpkin-X64-Windows.exe"
        );
        assert!(resolved.1.is_empty());
    }

    #[test]
    fn resolve_direct_jar_launch_target_uses_filename_for_root_jar() {
        let target = resolve_direct_jar_launch_target(
            "E:/servers/fabric-1.20.1",
            "E:/servers/fabric-1.20.1/server.jar",
        );

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
