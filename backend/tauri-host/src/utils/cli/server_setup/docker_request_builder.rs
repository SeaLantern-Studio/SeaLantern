use std::collections::BTreeMap;
use std::path::Path;

use super::docker_mounts::{parse_docker_volume_mounts, parse_extra_published_ports};
use super::docker_paths::{resolve_docker_data_dir, sanitize_container_name};
use super::{DockerCreateDefaults, DEFAULT_DOCKER_IMAGE, DEFAULT_DOCKER_IMAGE_TAG};
use crate::models::server::{
    CpuPolicyConfig, CreateDockerItzgServerRequest, DockerItzgRuntimeConfig, JvmPresetConfig,
};
use crate::services::global::i18n_service;
use crate::utils::cli::server_args::CliServerCommand;
use crate::utils::cli::server_docker::{build_docker_command_transport, default_docker_env};
use crate::utils::cli::server_ports::PreparedPorts;
use crate::utils::cli::server_shared::trace_cli_action;
use sea_lantern_docker_core::{
    format_memory_env_value, parse_command_mode, parse_docker_backend, resolve_docker_image_and_tag,
};
use sea_lantern_server_installer_core::CoreType;
use sea_lantern_server_local_setup_core::{
    infer_core_type_from_local_inputs, infer_mc_version_from_folder, infer_mc_version_hint,
    normalize_core_type,
};

fn cli_docker_t(key: &str) -> String {
    i18n_service().t(key)
}

pub(crate) fn resolve_requested_docker_image(
    command: &CliServerCommand,
) -> Result<(String, String), String> {
    resolve_docker_image_and_tag(
        command.image.as_deref(),
        command.image_tag.as_deref(),
        DEFAULT_DOCKER_IMAGE,
        DEFAULT_DOCKER_IMAGE_TAG,
    )
}

pub(crate) fn build_docker_create_request(
    command: &CliServerCommand,
    resolved_name: &str,
    ports: &PreparedPorts,
    defaults: DockerCreateDefaults,
) -> Result<CreateDockerItzgServerRequest, String> {
    let docker_folder_path = command.folder.as_deref().map(Path::new);
    let mc_version = command
        .mc_version
        .clone()
        .or_else(|| {
            docker_folder_path.and_then(|folder| infer_mc_version_from_folder(folder, None))
        })
        .or_else(|| {
            command
                .folder
                .as_deref()
                .and_then(|folder| infer_mc_version_hint(&[folder]))
        })
        .or_else(|| infer_mc_version_hint(&[resolved_name]))
        .ok_or_else(|| cli_docker_t("cli.server_setup.docker.create_missing_mc_version"))?;
    let core_type = command
        .core_type
        .clone()
        .or_else(|| {
            docker_folder_path.and_then(|folder| infer_core_type_from_local_inputs(folder, None))
        })
        .map(|value| normalize_core_type(Some(&value)))
        .transpose()?
        .unwrap_or_else(|| "paper".to_string());
    let api_core = CoreType::normalize_to_api_core_key(&core_type).unwrap_or(core_type.clone());
    let data_dir = resolve_docker_data_dir(command, resolved_name)?;
    let (image, image_tag) = resolve_requested_docker_image(command)?;
    let container_name = command
        .container_name
        .clone()
        .unwrap_or_else(|| format!("sealantern-{}", sanitize_container_name(resolved_name)));
    let docker_backend_kind = parse_docker_backend(command.docker_backend.as_deref())?;
    let command_mode = parse_command_mode(command.command_mode.as_deref())?;
    let mut env = default_docker_env();
    apply_docker_memory_env(command, &mut env, defaults);
    apply_custom_docker_env(command, &mut env);
    let volume_mounts = parse_docker_volume_mounts(&command.docker_mounts)?;
    let mut reserved_ports = vec![ports.game_port];
    if let Some(web_port) = ports.web_port {
        reserved_ports.push(web_port);
    }
    let (mut extra_ports, rcon) =
        build_docker_command_transport(&command_mode, &container_name, &reserved_ports, &mut env)?;
    reserved_ports.extend(extra_ports.iter().map(|port| port.host_port));
    let published_ports = parse_extra_published_ports(&command.docker_publishes, &reserved_ports)?;
    extra_ports.extend(published_ports);

    trace_cli_action(
        "docker_request_built",
        &format!(
            "name={} image={}:{} container={} backend={} command_mode={} data_dir={} game_port= {}",
            resolved_name,
            image,
            image_tag,
            container_name,
            docker_backend_kind.as_str(),
            command_mode.as_str(),
            data_dir,
            ports.game_port
        ),
    );

    Ok(CreateDockerItzgServerRequest {
        name: resolved_name.to_string(),
        aliases: command.aliases.clone(),
        core_type,
        mc_version: mc_version.clone(),
        port: ports.game_port,
        runtime: DockerItzgRuntimeConfig {
            image,
            image_tag,
            container_name,
            type_value: api_core.to_ascii_uppercase().replace('-', "_"),
            version: mc_version,
            data_dir_mount: data_dir,
            published_game_port: ports.game_port,
            env,
            extra_ports,
            volume_mounts,
            docker_backend_kind,
            command_mode,
            rcon,
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        },
    })
}

fn apply_docker_memory_env(
    command: &CliServerCommand,
    env: &mut BTreeMap<String, String>,
    defaults: DockerCreateDefaults,
) {
    let max_memory_mb = command
        .max_memory_mb
        .unwrap_or(defaults.default_max_memory_mb);
    let min_memory_mb = command
        .min_memory_mb
        .unwrap_or(defaults.default_min_memory_mb);

    env.insert("MEMORY".to_string(), format_memory_env_value(max_memory_mb));
    env.insert("MAX_MEMORY".to_string(), format_memory_env_value(max_memory_mb));
    env.insert("INIT_MEMORY".to_string(), format_memory_env_value(min_memory_mb));
}

fn apply_custom_docker_env(command: &CliServerCommand, env: &mut BTreeMap<String, String>) {
    for (key, value) in &command.docker_env {
        env.insert(key.clone(), value.clone());
    }
}
