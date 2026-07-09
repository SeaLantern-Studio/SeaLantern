use std::path::PathBuf;

use crate::models::server::ServerInstance;
use crate::utils::path::validate_file_name_only;
use server_flavor_core::ServerExtensionKind;
use server_plugin::{ensure_extension_target_dir, resolve_extension_relative_dir};

pub(crate) fn ensure_plugin_target_dir_for_server(
    server: &ServerInstance,
) -> Result<PathBuf, String> {
    ensure_extension_target_dir(
        &server.path,
        &server.core_type,
        &server.runtime_kind,
        server.startup_mode_str(),
        ServerExtensionKind::Plugin,
    )
}

pub(crate) fn plugin_relative_dir_for_server(
    server: &ServerInstance,
) -> Result<&'static str, String> {
    resolve_extension_relative_dir(
        &server.core_type,
        &server.runtime_kind,
        server.startup_mode_str(),
        ServerExtensionKind::Plugin,
    )
    .ok_or_else(|| {
        format!("Server core '{}' does not support plugin-style extensions", server.core_type)
    })
}

pub(crate) fn validate_plugin_file_name(file_name: &str) -> Result<String, String> {
    let normalized = if file_name.ends_with(".jar.disabled") {
        file_name.replace(".disabled", "")
    } else if file_name.ends_with(".jar") {
        file_name.to_string()
    } else {
        format!("{}.jar", file_name)
    };
    let safe_name = validate_file_name_only(&normalized)?;
    Ok(safe_name.to_string())
}
