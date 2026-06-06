use super::super::super::common::{
    resolve_managed_console_encoding, ManagedConsoleEncoding, StartupMode,
};
use crate::models::server::ServerInstance;
use crate::services::server::log_pipeline as server_log_pipeline;
use sea_lantern_server_local_setup_core::{
    resolve_java_paths as resolve_shared_java_paths,
    startup_filename as resolve_shared_startup_filename,
};
use sea_lantern_server_installer_core::{detect_core_type, CoreType};
use std::path::Path;

pub(in crate::services::server::manager::runtime_start) struct LaunchContext<'a> {
    pub server: &'a ServerInstance,
    pub settings: &'a crate::models::settings::AppSettings,
    pub startup_mode: StartupMode,
    pub managed_console_encoding: ManagedConsoleEncoding,
    pub java_bin_dir_str: String,
    pub java_home_dir_str: String,
    pub startup_filename: String,
    pub starter_installer_url: Option<String>,
}

pub(in crate::services::server::manager::runtime_start) fn resolve_managed_encoding(
    startup_mode: StartupMode,
    startup_path: &Path,
) -> ManagedConsoleEncoding {
    if startup_mode.is_custom() {
        ManagedConsoleEncoding::Utf8
    } else {
        resolve_managed_console_encoding(startup_mode, startup_path)
    }
}

pub(in crate::services::server::manager::runtime_start) fn resolve_java_paths(
    java_path: &str,
) -> Result<(String, String), String> {
    resolve_shared_java_paths(java_path)
}

pub(in crate::services::server::manager::runtime_start) fn startup_filename(
    startup_path: &str,
) -> String {
    resolve_shared_startup_filename(startup_path)
}

pub(in crate::services::server::manager::runtime_start) fn resolve_starter_installer_url(
    id: &str,
    server: &ServerInstance,
) -> Result<Option<String>, String> {
    let startup_mode = StartupMode::from_raw(server.startup_mode_str());
    if !startup_mode.is_starter() {
        return Ok(None);
    }

    let detected_core_type = detect_core_type(server.jar_path().unwrap_or_default());
    let core_key = CoreType::normalize_to_api_core_key(&server.core_type)
        .or_else(|| CoreType::normalize_to_api_core_key(&detected_core_type))
        .ok_or_else(|| {
            format!(
                "无法识别 Starter 核心类型：{}",
                if server.core_type.trim().is_empty() {
                    detected_core_type
                } else {
                    server.core_type.clone()
                }
            )
        })?;

    let mc_version = server.mc_version.trim();
    if mc_version.is_empty() || mc_version.eq_ignore_ascii_case("unknown") {
        return Err("Starter 启动需要 MC 版本，请在步骤三中选择后再创建服务器".to_string());
    }

    let (installer_url, installer_sha256) =
        crate::services::download::starter_installer_links::fetch_starter_installer_url(
            &core_key, mc_version,
        )?;
    if let Some(sha256) = installer_sha256 {
        let _ = server_log_pipeline::append_sealantern_log(
            id,
            &format!(
                "[Sea Lantern] Starter 安装器: core={}, version={}, sha256={}",
                core_key, mc_version, sha256
            ),
        );
    }

    Ok(Some(installer_url))
}
