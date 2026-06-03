use std::path::{Path, PathBuf};

use crate::models::server::{AddExistingServerRequest, CreateServerRequest};
use crate::services::server::installer::{detect_core_type, find_server_jar, CoreType};
use crate::utils::cli::server_args::CliServerCommand;
use crate::utils::cli::server_ports::PreparedPorts;
use crate::utils::cli::server_shared::trace_cli_action;
use crate::utils::path::is_windows_absolute_path;

use super::java_support::resolve_java_path;
use super::local_folder_inspection::LocalFolderInspection;
use super::local_startup_support::{
    detect_startup_mode_from_folder, infer_local_create_startup_mode, normalize_cli_startup_mode,
    resolve_command_path_hint, resolve_custom_entry_hint_path, resolve_existing_attach_entry_path,
    resolve_existing_local_entry_path, validate_local_entry_startup_mode,
};
use super::metadata_support::{
    infer_core_type_from_local_inputs, infer_local_create_mc_version, infer_mc_version_from_folder,
    infer_mc_version_hint,
};

#[derive(Debug, Clone)]
pub(super) struct LocalDefaults<'a> {
    pub default_java_path: &'a str,
    pub default_max_memory_mb: u32,
    pub default_min_memory_mb: u32,
}

pub(super) fn build_local_create_request(
    command: &CliServerCommand,
    resolved_name: &str,
    ports: &PreparedPorts,
    defaults: &LocalDefaults<'_>,
) -> Result<CreateServerRequest, String> {
    let folder_path = validate_local_create_folder(command.folder.as_deref())?;
    let java_path = if command.java_path_prevalidated {
        command
            .java_path
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "runtime preflight 未保留已验证 Java 路径".to_string())?
    } else {
        resolve_java_path(command, defaults.default_java_path)?
    };
    let explicit_startup_mode = command
        .startup_mode
        .as_deref()
        .map(|value| normalize_cli_startup_mode(Some(value)))
        .transpose()?;
    let resolved_entry_path = command
        .entry
        .as_deref()
        .and_then(|entry| resolve_existing_local_entry_path(folder_path, entry));
    let normalized_startup_mode = explicit_startup_mode.unwrap_or_else(|| {
        infer_local_create_startup_mode(command, resolved_entry_path.as_deref())
    });
    let is_custom_startup_mode = normalized_startup_mode == "custom";

    validate_local_entry_startup_mode(
        &normalized_startup_mode,
        command.entry.as_deref(),
        resolved_entry_path.as_deref(),
    )?;

    let custom_entry_hint_path = if is_custom_startup_mode {
        resolve_custom_entry_hint_path(
            command.entry.as_deref(),
            resolved_entry_path.as_deref(),
            folder_path,
        )
    } else {
        None
    };

    let resolved_jar_path = command
        .jar_path
        .as_deref()
        .and_then(|path| resolve_command_path_hint(path, folder_path));

    let jar_path = if is_custom_startup_mode {
        resolved_jar_path
            .clone()
            .or(custom_entry_hint_path.clone())
            .unwrap_or_default()
    } else {
        resolved_entry_path
            .clone()
            .or(resolved_jar_path.clone())
            .or_else(|| command.jar_path.clone())
            .ok_or_else(|| {
                if folder_path.is_some() {
                    "local server 缺少 --jar 或 --entry；仅 --folder 不足以创建新服务器".to_string()
                } else {
                    "local server 缺少 --jar 或 --entry 或 --folder".to_string()
                }
            })?
    };

    validate_local_create_startup_path_binding(
        folder_path,
        &normalized_startup_mode,
        &jar_path,
        resolved_entry_path.as_deref(),
    )?;
    validate_local_create_startup_exists(
        &normalized_startup_mode,
        &jar_path,
        resolved_entry_path.as_deref(),
    )?;

    let executable_hint = resolved_entry_path
        .clone()
        .or_else(|| resolved_jar_path.clone())
        .or_else(|| custom_entry_hint_path.clone());

    let core_type = command
        .core_type
        .clone()
        .map(|value| normalize_core_type(Some(&value)))
        .transpose()?
        .unwrap_or_else(|| {
            folder_path
                .and_then(|folder| {
                    infer_core_type_from_local_inputs(folder, executable_hint.as_deref())
                })
                .unwrap_or_else(|| detect_core_type(&jar_path))
        });

    let mc_version = command
        .mc_version
        .clone()
        .or_else(|| {
            infer_local_create_mc_version(
                &jar_path,
                resolved_name,
                resolved_entry_path.as_deref(),
                folder_path,
                executable_hint.as_deref(),
            )
        })
        .ok_or_else(|| "local server 缺少 --mc，且无法从 --jar/名称推断版本".to_string())?;

    let server_path = folder_path
        .map(|path| path.to_string_lossy().to_string())
        .or_else(|| {
            resolve_local_create_server_path(
                &jar_path,
                resolved_entry_path.as_deref(),
                custom_entry_hint_path.as_deref(),
            )
        });

    Ok(CreateServerRequest {
        name: resolved_name.to_string(),
        aliases: command.aliases.clone(),
        core_type,
        mc_version,
        max_memory: command
            .max_memory_mb
            .unwrap_or(defaults.default_max_memory_mb),
        min_memory: command
            .min_memory_mb
            .unwrap_or(defaults.default_min_memory_mb),
        port: ports.game_port,
        java_path,
        jar_path,
        server_path,
        startup_mode: normalized_startup_mode,
        custom_command: if is_custom_startup_mode {
            command
                .entry
                .clone()
                .filter(|value| !value.trim().is_empty())
        } else {
            command
                .entry
                .clone()
                .filter(|_| resolved_entry_path.is_none())
        },
    })
}

pub(super) fn build_local_attach_request(
    command: &CliServerCommand,
    resolved_name: &str,
    ports: &PreparedPorts,
    folder: &str,
    defaults: &LocalDefaults<'_>,
    inspection: &LocalFolderInspection,
) -> Result<AddExistingServerRequest, String> {
    let java_path = if command.java_path_prevalidated {
        command
            .java_path
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "runtime preflight 未保留已验证 Java 路径".to_string())?
    } else {
        resolve_java_path(command, defaults.default_java_path)?
    };
    let folder_path = Path::new(folder);
    if !folder_path.exists() || !folder_path.is_dir() {
        return Err(format!("--folder 指定目录不存在或不是文件夹: {}", folder));
    }

    let resolved_entry_path = command
        .entry
        .as_deref()
        .and_then(|entry| resolve_existing_attach_entry_path(folder_path, entry));
    let startup_mode = command
        .startup_mode
        .as_deref()
        .map(|value| normalize_cli_startup_mode(Some(value)))
        .transpose()?
        .unwrap_or_else(|| {
            inspection
                .startup_mode
                .clone()
                .unwrap_or_else(|| detect_startup_mode_from_folder(folder_path))
        });
    let is_custom_startup_mode = startup_mode == "custom";

    validate_local_entry_startup_mode(
        &startup_mode,
        command.entry.as_deref(),
        resolved_entry_path.as_deref(),
    )?;

    let executable_path = resolved_entry_path
        .clone()
        .filter(|_| !is_custom_startup_mode)
        .or_else(|| command.jar_path.clone().filter(|_| !is_custom_startup_mode))
        .or_else(|| {
            if is_custom_startup_mode {
                None
            } else {
                inspection.preferred_startup_path().map(str::to_string)
            }
        });
    let executable_hint = executable_path
        .clone()
        .or_else(|| inspection.detected_jar_path.clone())
        .or_else(|| find_server_jar(folder_path).ok());

    Ok(AddExistingServerRequest {
        name: resolved_name.to_string(),
        aliases: command.aliases.clone(),
        server_path: folder.to_string(),
        java_path,
        max_memory: command
            .max_memory_mb
            .unwrap_or(defaults.default_max_memory_mb),
        min_memory: command
            .min_memory_mb
            .unwrap_or(defaults.default_min_memory_mb),
        port: ports.game_port,
        startup_mode,
        executable_path: executable_path.clone(),
        custom_command: if is_custom_startup_mode {
            command
                .entry
                .clone()
                .filter(|value| !value.trim().is_empty())
        } else {
            command
                .entry
                .clone()
                .filter(|_| resolved_entry_path.is_none())
        },
        core_type: command.core_type.clone().or_else(|| {
            inspection.inferred_core_type.clone().or_else(|| {
                infer_core_type_from_local_inputs(folder_path, executable_hint.as_deref())
            })
        }),
        mc_version: command
            .mc_version
            .clone()
            .or_else(|| infer_mc_version_hint(&[folder, resolved_name]))
            .or_else(|| {
                inspection.inferred_mc_version.clone().or_else(|| {
                    infer_mc_version_from_folder(folder_path, executable_hint.as_deref())
                })
            }),
    })
}

pub(super) fn trace_local_create_request(resolved_name: &str, request: &CreateServerRequest) {
    trace_cli_action(
        "local_create_request_built",
        &format!(
            "name={} server_path={} jar_path={} startup_mode={} core={} mc={} game_port={}",
            resolved_name,
            request.server_path.as_deref().unwrap_or("<auto>"),
            request.jar_path,
            request.startup_mode,
            request.core_type,
            request.mc_version,
            request.port
        ),
    );
}

pub(super) fn trace_local_attach_request(
    resolved_name: &str,
    request: &AddExistingServerRequest,
    ports: &PreparedPorts,
) {
    trace_cli_action(
        "local_attach_request_built",
        &format!(
            "name={} folder={} startup_mode={} executable_path={} custom_command={} game_port={}",
            resolved_name,
            request.server_path,
            request.startup_mode,
            request
                .executable_path
                .as_deref()
                .unwrap_or("<auto-detect>"),
            request.custom_command.as_deref().unwrap_or("<none>"),
            ports.game_port
        ),
    );
}

fn validate_local_create_folder(folder: Option<&str>) -> Result<Option<&Path>, String> {
    let Some(folder) = folder.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    let folder_path = Path::new(folder);
    if folder_path.exists() && !folder_path.is_dir() {
        return Err(format!("--folder 指定目录不存在或不是文件夹: {}", folder));
    }

    Ok(Some(folder_path))
}

fn validate_local_create_startup_path_binding(
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

fn validate_local_create_startup_exists(
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

fn resolve_local_create_server_path(
    jar_path: &str,
    resolved_entry_path: Option<&str>,
    custom_entry_hint_path: Option<&str>,
) -> Option<String> {
    resolved_entry_path
        .and_then(path_parent_string)
        .or_else(|| path_parent_string(jar_path))
        .or_else(|| custom_entry_hint_path.and_then(path_parent_string))
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

fn normalize_core_type(value: Option<&str>) -> Result<String, String> {
    let raw = value.unwrap_or("paper").trim();
    if raw.is_empty() {
        return Err("--core 不能为空".to_string());
    }
    Ok(CoreType::normalize_to_api_core_key(raw).unwrap_or_else(|| raw.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{build_local_attach_request, build_local_create_request, LocalDefaults};
    use crate::utils::cli::server_args::CliServerCommand;
    use crate::utils::cli::server_ports::PreparedPorts;
    use crate::utils::cli::server_setup::local_folder_inspection::inspect_local_folder;
    use tempfile::tempdir;

    fn sample_defaults<'a>() -> LocalDefaults<'a> {
        LocalDefaults {
            default_java_path: "C:/Java/bin/java.exe",
            default_max_memory_mb: 4096,
            default_min_memory_mb: 2048,
        }
    }

    #[test]
    fn build_local_attach_request_can_infer_core_from_folder_name_without_jar() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("paper-prod-1.21.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::write(folder.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

        let command = CliServerCommand {
            folder: Some(folder.to_string_lossy().to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let build = build_local_attach_request(
            &command,
            "paper-prod",
            &ports,
            folder.to_string_lossy().as_ref(),
            &sample_defaults(),
            &inspect_local_folder(&folder),
        )
        .expect("local attach request should build");

        assert_eq!(build.startup_mode, "sh");
        assert_eq!(build.core_type.as_deref(), Some("Paper"));
        assert_eq!(build.mc_version.as_deref(), Some("1.21.1"));
    }

    #[test]
    fn build_local_attach_request_prefers_inspected_startup_entry_when_cli_is_implicit() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("fabric-1.20.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        let script_path = folder.join("start.ps1");
        std::fs::write(&script_path, b"Write-Host start\n").expect("script should write");

        let command = CliServerCommand {
            folder: Some(folder.to_string_lossy().to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let build = build_local_attach_request(
            &command,
            "fabric-1.20.1",
            &ports,
            folder.to_string_lossy().as_ref(),
            &sample_defaults(),
            &inspect_local_folder(&folder),
        )
        .expect("local attach should use inspected startup entry");

        assert_eq!(build.startup_mode, "ps1");
        assert_eq!(build.executable_path.as_deref(), Some(script_path.to_string_lossy().as_ref()));
    }

    #[test]
    fn build_local_create_request_uses_defaults_and_infers_metadata() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let jar_path = temp_dir
            .path()
            .join("fabric-server-mc.1.20.1-loader.0.15.11.jar");
        std::fs::write(&jar_path, b"placeholder").expect("jar placeholder should write");

        let command = CliServerCommand {
            jar_path: Some(jar_path.to_string_lossy().to_string()),
            java_path_prevalidated: true,
            java_path: Some("C:/validated/java/bin/java.exe".to_string()),
            aliases: vec!["fabric_latest".to_string()],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25570, web_port: None };

        let request =
            build_local_create_request(&command, "fabric-1.20.1", &ports, &sample_defaults())
                .expect("local create request should build");

        assert_eq!(request.name, "fabric-1.20.1");
        assert_eq!(request.aliases, vec!["fabric_latest"]);
        assert_eq!(request.core_type, "Fabric");
        assert_eq!(request.mc_version, "1.20.1");
        assert_eq!(request.port, 25570);
        assert_eq!(request.max_memory, 4096);
        assert_eq!(request.min_memory, 2048);
        assert_eq!(request.java_path, "C:/validated/java/bin/java.exe");
        assert_eq!(request.startup_mode, "jar");
        assert_eq!(request.custom_command, None);
    }

    #[test]
    fn build_local_create_request_treats_existing_entry_script_as_startup_path() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let script_path = temp_dir.path().join("start.bat");
        std::fs::write(&script_path, b"@echo off\r\n").unwrap();

        let command = CliServerCommand {
            entry: Some(script_path.to_string_lossy().to_string()),
            mc_version: Some("1.20.1".to_string()),
            core_type: Some("fabric".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_local_create_request(&command, "fabric-script-entry", &ports, &sample_defaults())
                .expect("local create should accept entry script as startup path");

        assert_eq!(request.startup_mode, "bat");
        assert_eq!(request.jar_path, script_path.to_string_lossy().to_string());
        assert_eq!(
            request.server_path.as_deref(),
            Some(temp_dir.path().to_string_lossy().as_ref())
        );
        assert_eq!(request.custom_command, None);
    }

    #[test]
    fn build_local_create_request_infers_metadata_from_script_entry_folder() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let script_path = temp_dir.path().join("start.bat");
        std::fs::write(&script_path, b"@echo off\r\n").unwrap();
        std::fs::write(
            temp_dir
                .path()
                .join("fabric-server-mc.1.20.1-loader.0.15.11.jar"),
            b"placeholder",
        )
        .unwrap();

        let command = CliServerCommand {
            entry: Some(script_path.to_string_lossy().to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_local_create_request(&command, "cache-server", &ports, &sample_defaults())
                .expect("local create should infer metadata from script folder");

        assert_eq!(request.startup_mode, "bat");
        assert_eq!(request.core_type, "Fabric");
        assert_eq!(request.mc_version, "1.20.1");
        assert_eq!(request.jar_path, script_path.to_string_lossy().to_string());
        assert_eq!(
            request.server_path.as_deref(),
            Some(temp_dir.path().to_string_lossy().as_ref())
        );
    }

    #[test]
    fn build_local_create_request_infers_mc_version_for_custom_script_entry_path() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let script_path = temp_dir.path().join("launch.ps1");
        std::fs::write(&script_path, b"Write-Host start\n").unwrap();
        std::fs::write(temp_dir.path().join("paper-1.21.1-31.jar"), b"placeholder").unwrap();

        let command = CliServerCommand {
            startup_mode: Some("custom".to_string()),
            entry: Some(script_path.to_string_lossy().to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_local_create_request(&command, "paper-cache", &ports, &sample_defaults())
                .expect("custom script entry should still infer mc version from sibling jar");

        assert_eq!(request.startup_mode, "custom");
        assert_eq!(request.core_type, "Paper");
        assert_eq!(request.mc_version, "1.21.1");
        assert_eq!(
            request.server_path.as_deref(),
            Some(temp_dir.path().to_string_lossy().as_ref())
        );
        assert_eq!(request.custom_command.as_deref(), Some(script_path.to_string_lossy().as_ref()));
    }

    #[test]
    fn build_local_create_request_treats_non_path_entry_as_custom_command() {
        let command = CliServerCommand {
            entry: Some("java -Xmx4G -Xms4G -jar server.jar nogui".to_string()),
            jar_path: Some("E:/servers/fabric/server.jar".to_string()),
            mc_version: Some("1.20.1".to_string()),
            core_type: Some("fabric".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_local_create_request(&command, "fabric-custom-entry", &ports, &sample_defaults())
                .expect("local create should accept custom command entry");

        assert_eq!(request.startup_mode, "custom");
        assert_eq!(request.jar_path, "E:/servers/fabric/server.jar");
        assert_eq!(request.server_path.as_deref(), Some("E:/servers/fabric"));
        assert_eq!(
            request.custom_command.as_deref(),
            Some("java -Xmx4G -Xms4G -jar server.jar nogui")
        );
    }

    #[test]
    fn build_local_create_request_allows_custom_entry_without_jar_path() {
        let command = CliServerCommand {
            entry: Some("java -Xmx4G -Xms4G -jar server.jar nogui".to_string()),
            mc_version: Some("1.20.1".to_string()),
            core_type: Some("fabric".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_local_create_request(&command, "fabric-custom-only", &ports, &sample_defaults())
                .expect("local create should allow custom command without jar path");

        assert_eq!(request.startup_mode, "custom");
        assert!(request.jar_path.replace('\\', "/").ends_with("/server.jar"));
        let expected = std::env::current_dir()
            .expect("current dir should resolve")
            .to_string_lossy()
            .replace('\\', "/");
        assert_eq!(
            request
                .server_path
                .as_deref()
                .map(|path| path.replace('\\', "/")),
            Some(expected)
        );
        assert_eq!(
            request.custom_command.as_deref(),
            Some("java -Xmx4G -Xms4G -jar server.jar nogui")
        );
    }

    #[test]
    fn build_local_create_request_preserves_custom_startup_with_existing_entry_script() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let script_path = temp_dir.path().join("start.bat");
        std::fs::write(&script_path, b"@echo off\r\n").unwrap();

        let command = CliServerCommand {
            startup_mode: Some("custom".to_string()),
            entry: Some(script_path.to_string_lossy().to_string()),
            jar_path: Some("E:/servers/fabric/server.jar".to_string()),
            mc_version: Some("1.20.1".to_string()),
            core_type: Some("fabric".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request = build_local_create_request(
            &command,
            "fabric-custom-script-path",
            &ports,
            &sample_defaults(),
        )
        .expect("explicit custom mode should keep script path as custom command");

        assert_eq!(request.startup_mode, "custom");
        assert_eq!(request.jar_path, "E:/servers/fabric/server.jar");
        assert_eq!(
            request.server_path.as_deref(),
            Some(temp_dir.path().to_string_lossy().as_ref())
        );
        assert_eq!(request.custom_command.as_deref(), Some(script_path.to_string_lossy().as_ref()));
    }

    #[test]
    fn build_local_create_request_uses_folder_as_target_dir_for_relative_jar() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("fresh-fabric-1.20.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::write(folder.join("fabric-server-mc.1.20.1-loader.0.15.11.jar"), b"placeholder")
            .expect("relative jar placeholder should write");

        let command = CliServerCommand {
            folder: Some(folder.to_string_lossy().to_string()),
            jar_path: Some("fabric-server-mc.1.20.1-loader.0.15.11.jar".to_string()),
            mc_version: Some("1.20.1".to_string()),
            core_type: Some("fabric".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_local_create_request(&command, "fresh-fabric-1.20.1", &ports, &sample_defaults())
                .expect("folder-backed local create should build");

        assert_eq!(
            request.jar_path,
            folder
                .join("fabric-server-mc.1.20.1-loader.0.15.11.jar")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(request.server_path.as_deref(), Some(folder.to_string_lossy().as_ref()));
    }

    #[test]
    fn build_local_create_request_rejects_missing_startup_file_before_create() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("fresh-fabric-1.20.1");
        std::fs::create_dir_all(&folder).expect("folder should create");

        let command = CliServerCommand {
            folder: Some(folder.to_string_lossy().to_string()),
            jar_path: Some("fabric-server-mc.1.20.1-loader.0.15.11.jar".to_string()),
            mc_version: Some("1.20.1".to_string()),
            core_type: Some("fabric".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let err =
            build_local_create_request(&command, "fresh-fabric-1.20.1", &ports, &sample_defaults())
                .expect_err("missing startup file should be rejected before create");

        assert!(err.contains("启动文件不存在"));
    }

    #[test]
    fn build_local_create_request_rejects_folder_create_with_external_jar() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("fresh-fabric-1.20.1");
        let external = temp_dir.path().join("elsewhere").join("server.jar");

        let command = CliServerCommand {
            folder: Some(folder.to_string_lossy().to_string()),
            jar_path: Some(external.to_string_lossy().to_string()),
            mc_version: Some("1.20.1".to_string()),
            core_type: Some("fabric".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let err =
            build_local_create_request(&command, "fresh-fabric-1.20.1", &ports, &sample_defaults())
                .expect_err("folder-backed create should reject external jar path");

        assert!(err.contains("启动文件必须位于该目录根下"));
    }

    #[test]
    fn build_local_attach_request_keeps_script_detection_when_folder_has_start_bat() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(temp_dir.path().join("start.bat"), b"@echo off\r\n").unwrap();
        std::fs::write(
            temp_dir
                .path()
                .join("fabric-server-mc.1.20.1-loader.0.15.11.jar"),
            b"placeholder",
        )
        .unwrap();

        let command = CliServerCommand {
            folder: Some(temp_dir.path().to_string_lossy().to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let build = build_local_attach_request(
            &command,
            "fabric-scripted",
            &ports,
            temp_dir.path().to_string_lossy().as_ref(),
            &sample_defaults(),
            &inspect_local_folder(temp_dir.path()),
        )
        .expect("local attach request should build");

        assert_eq!(build.startup_mode, "bat");
        assert_eq!(
            build.executable_path.as_deref(),
            Some(temp_dir.path().join("start.bat").to_string_lossy().as_ref())
        );
        assert_eq!(build.core_type.as_deref(), Some("Fabric"));
        assert_eq!(build.mc_version.as_deref(), Some("1.20.1"));
    }

    #[test]
    fn build_local_attach_request_prefers_custom_named_root_script_when_no_known_name_exists() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let script_path = temp_dir.path().join("fabric-prod.ps1");
        std::fs::write(&script_path, b"Write-Host start\n").unwrap();
        std::fs::write(temp_dir.path().join("paper-1.21.1-31.jar"), b"placeholder").unwrap();

        let command = CliServerCommand {
            folder: Some(temp_dir.path().to_string_lossy().to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let build = build_local_attach_request(
            &command,
            "paper-prod",
            &ports,
            temp_dir.path().to_string_lossy().as_ref(),
            &sample_defaults(),
            &inspect_local_folder(temp_dir.path()),
        )
        .expect("custom named root script should still attach as existing server");

        assert_eq!(build.startup_mode, "ps1");
        assert_eq!(build.executable_path.as_deref(), Some(script_path.to_string_lossy().as_ref()));
        assert_eq!(build.core_type.as_deref(), Some("Paper"));
        assert_eq!(build.mc_version.as_deref(), Some("1.21.1"));
    }

    #[test]
    fn build_local_attach_request_resolves_relative_entry_inside_folder() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let script_path = temp_dir.path().join("start.ps1");
        std::fs::write(&script_path, b"Write-Host start\n").unwrap();

        let command = CliServerCommand {
            folder: Some(temp_dir.path().to_string_lossy().to_string()),
            entry: Some("start.ps1".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let build = build_local_attach_request(
            &command,
            "folder-entry",
            &ports,
            temp_dir.path().to_string_lossy().as_ref(),
            &sample_defaults(),
            &inspect_local_folder(temp_dir.path()),
        )
        .expect("relative entry path should resolve");

        assert_eq!(build.executable_path, Some(script_path.to_string_lossy().to_string()));
        assert_eq!(build.custom_command, None);
    }

    #[test]
    fn build_local_attach_request_preserves_custom_command_without_spaces_when_not_a_path() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(temp_dir.path().join("server.jar"), b"placeholder").unwrap();

        let command = CliServerCommand {
            folder: Some(temp_dir.path().to_string_lossy().to_string()),
            startup_mode: Some("custom".to_string()),
            entry: Some("launch-custom".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let build = build_local_attach_request(
            &command,
            "folder-custom",
            &ports,
            temp_dir.path().to_string_lossy().as_ref(),
            &sample_defaults(),
            &inspect_local_folder(temp_dir.path()),
        )
        .expect("custom entry should stay as command text");

        assert_eq!(build.startup_mode, "custom");
        assert_eq!(build.executable_path, None);
        assert_eq!(build.custom_command.as_deref(), Some("launch-custom"));
    }

    #[test]
    fn build_local_attach_request_preserves_custom_command_when_entry_points_to_existing_script() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let script_path = temp_dir.path().join("start.ps1");
        std::fs::write(&script_path, b"Write-Host start\n").unwrap();

        let command = CliServerCommand {
            folder: Some(temp_dir.path().to_string_lossy().to_string()),
            startup_mode: Some("custom".to_string()),
            entry: Some(script_path.to_string_lossy().to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let build = build_local_attach_request(
            &command,
            "folder-custom-script",
            &ports,
            temp_dir.path().to_string_lossy().as_ref(),
            &sample_defaults(),
            &inspect_local_folder(temp_dir.path()),
        )
        .expect(
            "explicit custom mode should not silently convert existing script into executable path",
        );

        assert_eq!(build.startup_mode, "custom");
        assert_eq!(build.executable_path, None);
        assert_eq!(build.custom_command.as_deref(), Some(script_path.to_string_lossy().as_ref()));
    }
}
