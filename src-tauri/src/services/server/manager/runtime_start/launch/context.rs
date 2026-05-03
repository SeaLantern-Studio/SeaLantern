use super::super::super::common::{
    resolve_managed_console_encoding, ManagedConsoleEncoding,
};
use crate::models::server::ServerInstance;
use crate::services::server::installer;
use crate::services::server::log_pipeline as server_log_pipeline;
use std::path::Path;

pub(in crate::services::server::manager::runtime_start) struct LaunchContext<'a> {
    pub manager: &'a super::super::super::ServerManager,
    pub server: &'a ServerInstance,
    pub settings: &'a crate::models::settings::AppSettings,
    pub startup_mode: &'a str,
    pub managed_console_encoding: ManagedConsoleEncoding,
    pub java_bin_dir_str: String,
    pub java_home_dir_str: String,
    pub startup_filename: String,
    pub starter_installer_url: Option<String>,
}

pub(in crate::services::server::manager::runtime_start) fn resolve_managed_encoding(
    startup_mode: &str,
    startup_path: &Path,
) -> ManagedConsoleEncoding {
    if startup_mode == "custom" {
        ManagedConsoleEncoding::Utf8
    } else {
        resolve_managed_console_encoding(startup_mode, startup_path)
    }
}

pub(in crate::services::server::manager::runtime_start) fn resolve_java_paths(
    java_path: &str,
) -> Result<(String, String), String> {
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

pub(in crate::services::server::manager::runtime_start) fn startup_filename(
    startup_path: &str,
) -> String {
    Path::new(startup_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| startup_path.to_string())
}

pub(in crate::services::server::manager::runtime_start) fn resolve_starter_installer_url(
    id: &str,
    server: &ServerInstance,
) -> Result<Option<String>, String> {
    let startup_mode = super::super::super::common::normalize_startup_mode(&server.startup_mode);
    if startup_mode != "starter" {
        return Ok(None);
    }

    let detected_core_type = installer::detect_core_type(&server.jar_path);
    let core_key = installer::CoreType::normalize_to_api_core_key(&server.core_type)
        .or_else(|| installer::CoreType::normalize_to_api_core_key(&detected_core_type))
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
