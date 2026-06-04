use serde::{Deserialize, Serialize};

use crate::models::server::ServerInstance;

use super::super::common::{format_command_for_log, StartupMode};
use super::super::startup_support::build_managed_jvm_args;
use super::launch::command_builder::{
    build_configured_command, build_direct_jar_command, find_preferred_jar_path,
};
use super::launch::context::{
    resolve_java_paths, resolve_managed_encoding, resolve_starter_installer_url, startup_filename,
    LaunchContext,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalLaunchDetail {
    pub startup_mode: String,
    pub java_path: String,
    pub launch_target: String,
    pub effective_jvm_args: Vec<String>,
    pub command_preview: String,
}

pub(crate) fn build_local_launch_detail(
    server: &ServerInstance,
    settings: &crate::models::settings::AppSettings,
) -> Result<LocalLaunchDetail, String> {
    let runtime = server
        .local_runtime()
        .ok_or_else(|| format!("当前服务器运行时暂未实现: {}", server.runtime_kind))?;

    let startup_mode = StartupMode::from_raw(&runtime.startup_mode);
    let startup_path = runtime.jar_path.as_str();
    let startup_path_obj = std::path::Path::new(startup_path);
    let managed_console_encoding = resolve_managed_encoding(startup_mode, startup_path_obj);
    let (java_bin_dir_str, java_home_dir_str) = resolve_java_paths(runtime.java_path.as_str())?;
    let starter_installer_url = resolve_starter_installer_url("inspect", server)?;

    let context = LaunchContext {
        server,
        settings,
        startup_mode,
        managed_console_encoding,
        java_bin_dir_str,
        java_home_dir_str,
        startup_filename: startup_filename(startup_path),
        starter_installer_url,
    };

    let effective_jvm_args = if matches!(startup_mode, StartupMode::Jar | StartupMode::Starter) {
        build_managed_jvm_args(server, settings, managed_console_encoding)?
    } else {
        Vec::new()
    };

    let launch_target = resolve_launch_target(server, &context);
    let command_preview = resolve_command_preview(&context)?;

    Ok(LocalLaunchDetail {
        startup_mode: runtime.startup_mode.clone(),
        java_path: runtime.java_path.clone(),
        launch_target,
        effective_jvm_args,
        command_preview,
    })
}

fn resolve_launch_target(server: &ServerInstance, context: &LaunchContext<'_>) -> String {
    if let Some(preferred_jar_path) = find_preferred_jar_path(context) {
        return preferred_jar_path;
    }

    if matches!(context.startup_mode, StartupMode::Jar | StartupMode::Starter) {
        return server.jar_path().unwrap_or_default().to_string();
    }

    if matches!(context.startup_mode, StartupMode::Custom) {
        return server.custom_command().unwrap_or_default().to_string();
    }

    context.startup_filename.clone()
}

fn resolve_command_preview(context: &LaunchContext<'_>) -> Result<String, String> {
    let command = if let Some(preferred_jar_path) = find_preferred_jar_path(context) {
        build_direct_jar_command(context, &preferred_jar_path, None)?
    } else {
        build_configured_command(context)?
    };

    Ok(format_command_for_log(&command))
}

#[cfg(test)]
mod tests {
    use super::{build_local_launch_detail, LocalLaunchDetail};
    use crate::models::server::{
        CpuPolicyConfig, CpuPolicyMode, JvmPresetConfig, LocalRuntimeConfig, ServerInstance,
        ServerRuntimeConfig,
    };
    use crate::models::settings::AppSettings;
    use tempfile::tempdir;

    fn test_settings() -> AppSettings {
        AppSettings {
            default_max_memory: 4096,
            default_min_memory: 1024,
            default_jvm_args: vec!["-Dglobal.flag=true".to_string()],
            ..AppSettings::default()
        }
    }

    fn test_server(path: String, startup_mode: &str) -> ServerInstance {
        ServerInstance {
            id: format!("detail-{}", startup_mode),
            name: "Local Detail".to_string(),
            aliases: Vec::new(),
            core_type: "paper".to_string(),
            core_version: "paper".to_string(),
            mc_version: "1.21.1".to_string(),
            path,
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "E:/servers/detail/server.jar".to_string(),
                startup_mode: startup_mode.to_string(),
                custom_command: Some("java -jar custom.jar nogui".to_string()),
                java_path: "C:/Java/bin/java.exe".to_string(),
                jvm_args: vec!["-Dserver.flag=true".to_string()],
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn build_local_launch_detail_includes_effective_jvm_args_for_jar_mode() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut server = test_server(temp_dir.path().to_string_lossy().to_string(), "jar");
        if let ServerRuntimeConfig::Local(runtime) = &mut server.runtime {
            runtime.cpu_policy = CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(2),
                explicit_set: None,
                sync_active_processor_count: true,
            };
        }

        let detail = build_local_launch_detail(&server, &test_settings())
            .expect("local launch detail should build");

        assert_eq!(detail.startup_mode, "jar");
        assert_eq!(detail.java_path, "C:/Java/bin/java.exe");
        assert_eq!(detail.launch_target, "E:/servers/detail/server.jar");
        assert!(detail
            .effective_jvm_args
            .iter()
            .any(|arg| arg == "-XX:ActiveProcessorCount=2"));
        assert!(detail.command_preview.contains("java.exe"));
        assert!(detail.command_preview.contains("-jar"));
    }

    #[test]
    fn build_local_launch_detail_uses_script_filename_for_sh_mode() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut server = test_server(temp_dir.path().to_string_lossy().to_string(), "sh");
        if let ServerRuntimeConfig::Local(runtime) = &mut server.runtime {
            runtime.jar_path = temp_dir
                .path()
                .join("start.sh")
                .to_string_lossy()
                .to_string();
        }

        let detail = build_local_launch_detail(&server, &test_settings())
            .expect("local launch detail should build");

        assert_eq!(detail.startup_mode, "sh");
        assert_eq!(detail.launch_target, "start.sh");
        assert!(detail.effective_jvm_args.is_empty());
        assert!(detail.command_preview.contains("start.sh"));
    }

    #[test]
    fn build_local_launch_detail_uses_custom_command_as_launch_target() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string(), "custom");

        let detail: LocalLaunchDetail = build_local_launch_detail(&server, &test_settings())
            .expect("local launch detail should build");

        assert_eq!(detail.startup_mode, "custom");
        assert_eq!(detail.launch_target, "java -jar custom.jar nogui");
        assert!(detail.effective_jvm_args.is_empty());
        assert!(detail.command_preview.contains("custom.jar"));
    }
}
