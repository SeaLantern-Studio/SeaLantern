use std::path::{Path, PathBuf};

use crate::utils::cli::cli_env::{
    configured_servers_container_root, has_docker_host_path_mapping, is_container_like_environment,
};
use crate::utils::cli::server_args::CliServerCommand;
use crate::utils::cli::server_docker::sanitize_name_like;
use crate::utils::path::strip_path_prefix_for_compare;

pub(crate) fn resolve_docker_data_dir(
    command: &CliServerCommand,
    resolved_name: &str,
) -> Result<String, String> {
    if let Some(explicit_data_dir) = command.data_dir.clone().or_else(|| command.folder.clone()) {
        return Ok(explicit_data_dir);
    }

    if is_container_like_environment() && !has_docker_host_path_mapping() {
        return Err(
            "当前 Sea Lantern 运行在容器可见路径下，且未配置 SEALANTERN_SERVERS_CONTAINER_ROOT / SEALANTERN_SERVERS_HOST_ROOT；请显式传入 --data-dir，或配置宿主路径映射"
                .to_string(),
        );
    }

    Ok(default_docker_server_dir(resolved_name))
}

pub(crate) fn map_container_visible_path_to_docker_host_path(path: &Path) -> Option<String> {
    let host_root = std::env::var("SEALANTERN_SERVERS_HOST_ROOT").ok()?;
    let container_root = std::env::var("SEALANTERN_SERVERS_CONTAINER_ROOT").ok()?;

    let host_root = PathBuf::from(host_root);
    let container_root = PathBuf::from(container_root);
    let relative = strip_path_prefix_for_compare(path, &container_root)?;

    if relative.is_empty() {
        return Some(host_root.to_string_lossy().to_string());
    }

    Some(host_root.join(relative).to_string_lossy().to_string())
}

pub(crate) fn sanitize_container_name(name: &str) -> String {
    sanitize_name_like(name)
}

fn default_docker_server_dir(name: &str) -> String {
    let path = default_container_visible_server_dir(name);
    map_container_visible_path_to_docker_host_path(&path)
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

fn default_container_visible_server_dir(name: &str) -> PathBuf {
    let mut path = configured_servers_container_root().unwrap_or_else(|| {
        let mut app_data_dir = PathBuf::from(crate::utils::path::get_or_create_app_data_dir());
        app_data_dir.push("servers");
        app_data_dir
    });
    path.push(sanitize_container_name(name));
    path
}
