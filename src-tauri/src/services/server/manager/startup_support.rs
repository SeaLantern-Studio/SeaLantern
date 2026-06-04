use std::path::Path;

use crate::commands::server::config::SLStartupConfig;
use crate::models::server::ServerInstance;
use crate::models::settings::AppSettings;

use super::common::ManagedConsoleEncoding;
use super::common::StartupMode;
use super::cpu_policy;

pub(super) fn build_managed_jvm_args(
    server: &ServerInstance,
    settings: &AppSettings,
    console_encoding: ManagedConsoleEncoding,
) -> Result<Vec<String>, String> {
    let java_encoding = console_encoding.java_name();
    let default_memory = (settings.default_max_memory, settings.default_min_memory);
    let (max_mem, min_mem) = read_sl_startup_config(server, settings).unwrap_or(default_memory);
    let mut args = vec![
        format!("-Xmx{}M", max_mem),
        format!("-Xms{}M", min_mem),
        format!("-Dfile.encoding={}", java_encoding),
        format!("-Dsun.stdout.encoding={}", java_encoding),
        format!("-Dsun.stderr.encoding={}", java_encoding),
    ];

    let startup_mode = StartupMode::from_raw(server.startup_mode_str());
    let default_args = settings.default_jvm_args.clone();
    let user_args = server.jvm_args().to_vec();

    let user_already_set_apc = jvm_args_contain_active_processor_count(&default_args)
        || jvm_args_contain_active_processor_count(&user_args);

    if !user_already_set_apc {
        if let Some(arg) = cpu_policy::compute_active_processor_count_arg(server, startup_mode)? {
            args.push(arg);
        }
    }

    args.extend(default_args);
    args.extend(user_args);
    Ok(args)
}

pub(super) fn write_user_jvm_args(
    server: &ServerInstance,
    settings: &AppSettings,
    console_encoding: ManagedConsoleEncoding,
) -> Result<(), String> {
    let args = build_managed_jvm_args(server, settings, console_encoding)?;
    let user_jvm_args_path = Path::new(&server.path).join("user_jvm_args.txt");
    let content = if args.is_empty() {
        String::new()
    } else {
        format!("{}\n", args.join("\n"))
    };

    std::fs::write(&user_jvm_args_path, content)
        .map_err(|e| format!("写入 user_jvm_args.txt 失败: {}", e))
}

fn jvm_args_contain_active_processor_count(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg.starts_with("-XX:ActiveProcessorCount="))
}

fn read_sl_startup_config(server: &ServerInstance, settings: &AppSettings) -> Option<(u32, u32)> {
    let sl_path = Path::new(&server.path).join("SL.json");
    if !sl_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&sl_path).ok()?;
    let config: SLStartupConfig = serde_json::from_str(&content).ok()?;
    match (config.max_memory, config.min_memory) {
        (Some(max), Some(min)) => Some((max, min)),
        (Some(max), None) => Some((max, settings.default_min_memory)),
        (None, Some(min)) => Some((settings.default_max_memory, min)),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{build_managed_jvm_args, write_user_jvm_args};
    use crate::models::server::{
        CpuPolicyConfig, CpuPolicyMode, JvmPresetConfig, LocalRuntimeConfig, ServerInstance,
        ServerRuntimeConfig,
    };
    use crate::models::settings::AppSettings;
    use crate::services::server::manager::common::ManagedConsoleEncoding;
    use tempfile::tempdir;

    fn test_server(path: String) -> ServerInstance {
        ServerInstance {
            id: "startup-support".to_string(),
            name: "Startup Support".to_string(),
            aliases: Vec::new(),
            core_type: "fabric".to_string(),
            core_version: "fabric".to_string(),
            mc_version: "1.20.1".to_string(),
            path,
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "java".to_string(),
                jvm_args: vec!["-Dserver.flag=true".to_string()],
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    fn test_server_with_cpu_policy(
        path: String,
        startup_mode: &str,
        cpu_policy: CpuPolicyConfig,
    ) -> ServerInstance {
        let mut server = test_server(path);
        if let ServerRuntimeConfig::Local(runtime) = &mut server.runtime {
            runtime.startup_mode = startup_mode.to_string();
            runtime.cpu_policy = cpu_policy;
        }
        server
    }

    fn test_settings() -> AppSettings {
        AppSettings {
            default_max_memory: 8192,
            default_min_memory: 1024,
            default_jvm_args: vec!["-Dglobal.flag=true".to_string(), "-XX:+UseG1GC".to_string()],
            ..AppSettings::default()
        }
    }

    #[test]
    fn build_managed_jvm_args_prefers_sl_json_memory_and_preserves_arg_order() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        std::fs::write(temp_dir.path().join("SL.json"), r#"{"max_memory":3072,"min_memory":1536}"#)
            .expect("SL.json should be written");

        let args = build_managed_jvm_args(&server, &test_settings(), ManagedConsoleEncoding::Utf8)
            .expect("managed args should build");

        assert_eq!(
            args,
            vec![
                "-Xmx3072M".to_string(),
                "-Xms1536M".to_string(),
                "-Dfile.encoding=UTF-8".to_string(),
                "-Dsun.stdout.encoding=UTF-8".to_string(),
                "-Dsun.stderr.encoding=UTF-8".to_string(),
                "-Dglobal.flag=true".to_string(),
                "-XX:+UseG1GC".to_string(),
                "-Dserver.flag=true".to_string(),
            ]
        );
    }

    #[test]
    fn write_user_jvm_args_uses_default_min_memory_when_sl_json_omits_it() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        std::fs::write(temp_dir.path().join("SL.json"), r#"{"max_memory":6144}"#)
            .expect("SL.json should be written");

        write_user_jvm_args(&server, &test_settings(), ManagedConsoleEncoding::Utf8)
            .expect("user_jvm_args.txt should be written");

        let content = std::fs::read_to_string(temp_dir.path().join("user_jvm_args.txt"))
            .expect("user_jvm_args.txt should exist");

        assert_eq!(
            content,
            concat!(
                "-Xmx6144M\n",
                "-Xms1024M\n",
                "-Dfile.encoding=UTF-8\n",
                "-Dsun.stdout.encoding=UTF-8\n",
                "-Dsun.stderr.encoding=UTF-8\n",
                "-Dglobal.flag=true\n",
                "-XX:+UseG1GC\n",
                "-Dserver.flag=true\n"
            )
        );
    }

    #[test]
    fn build_managed_jvm_args_injects_active_processor_count_before_default_args() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server_with_cpu_policy(
            temp_dir.path().to_string_lossy().to_string(),
            "jar",
            CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(2),
                explicit_set: None,
                sync_active_processor_count: true,
            },
        );

        let args = build_managed_jvm_args(&server, &test_settings(), ManagedConsoleEncoding::Utf8)
            .expect("managed args should build");

        assert_eq!(args[5], "-XX:ActiveProcessorCount=2");
        assert_eq!(args[6], "-Dglobal.flag=true");
    }

    #[test]
    fn build_managed_jvm_args_skips_active_processor_count_when_user_already_provided() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut server = test_server_with_cpu_policy(
            temp_dir.path().to_string_lossy().to_string(),
            "starter",
            CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(2),
                explicit_set: None,
                sync_active_processor_count: true,
            },
        );
        if let ServerRuntimeConfig::Local(runtime) = &mut server.runtime {
            runtime.jvm_args = vec!["-XX:ActiveProcessorCount=6".to_string()];
        }

        let args = build_managed_jvm_args(&server, &test_settings(), ManagedConsoleEncoding::Utf8)
            .expect("managed args should build");

        assert_eq!(
            args.iter()
                .filter(|arg| arg.starts_with("-XX:ActiveProcessorCount="))
                .count(),
            1
        );
        assert!(args.iter().any(|arg| arg == "-XX:ActiveProcessorCount=6"));
    }

    #[test]
    fn build_managed_jvm_args_rejects_cpu_policy_for_unsupported_modes() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server_with_cpu_policy(
            temp_dir.path().to_string_lossy().to_string(),
            "bat",
            CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(2),
                explicit_set: None,
                sync_active_processor_count: true,
            },
        );

        let err = build_managed_jvm_args(&server, &test_settings(), ManagedConsoleEncoding::Utf8)
            .expect_err("unsupported mode should fail");
        assert!(err.contains("暂不支持 CPU policy"));
    }
}
