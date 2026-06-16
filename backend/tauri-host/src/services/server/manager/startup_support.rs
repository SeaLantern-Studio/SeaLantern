use std::path::Path;

use crate::models::server::ServerInstance;
use crate::models::settings::AppSettings;
use sea_lantern_server_config_core::startup::read_server_startup_config_document;
use sea_lantern_server_config_core::types::ServerStartupConfigDocument;
use sea_lantern_server_config_core::{
    build_managed_jvm_args_from_input, resolve_effective_startup_config_from_document,
    EffectiveStartupConfig as SharedEffectiveStartupConfig, ManagedJvmBuildInput,
    StartupResolutionDefaults, StartupRuntimeDefaults,
};
use sea_lantern_server_local_setup_core::ManagedConsoleEncoding;

use super::common::StartupMode;
use super::cpu_policy;
use super::i18n::manager_t1;

pub(crate) type EffectiveStartupConfig = SharedEffectiveStartupConfig;

fn runtime_defaults(server: &ServerInstance) -> StartupRuntimeDefaults {
    let runtime_jvm_args = server
        .local_runtime()
        .map(|runtime| runtime.jvm_args.clone())
        .or_else(|| {
            server
                .docker_itzg_runtime()
                .map(|runtime| runtime.jvm_args.clone())
        })
        .unwrap_or_default();
    let runtime_cpu_policy = server
        .local_runtime()
        .map(|runtime| runtime.cpu_policy.clone())
        .or_else(|| {
            server
                .docker_itzg_runtime()
                .map(|runtime| runtime.cpu_policy.clone())
        })
        .unwrap_or_default();
    let runtime_jvm_preset = server
        .local_runtime()
        .map(|runtime| runtime.jvm_preset.clone())
        .or_else(|| {
            server
                .docker_itzg_runtime()
                .map(|runtime| runtime.jvm_preset.clone())
        })
        .unwrap_or_default();

    StartupRuntimeDefaults {
        max_memory: server.max_memory,
        min_memory: server.min_memory,
        jvm_args: runtime_jvm_args,
        cpu_policy: runtime_cpu_policy,
        jvm_preset: runtime_jvm_preset,
    }
}

fn resolution_defaults(settings: &AppSettings) -> StartupResolutionDefaults {
    StartupResolutionDefaults {
        default_max_memory: settings.default_max_memory,
        default_min_memory: settings.default_min_memory,
    }
}

pub(crate) fn resolve_effective_startup_config_checked(
    server: &ServerInstance,
    settings: &AppSettings,
) -> Result<EffectiveStartupConfig, String> {
    let startup = read_startup_document_checked(server)?;

    Ok(resolve_effective_startup_config_from_document(
        &startup,
        &runtime_defaults(server),
        &resolution_defaults(settings),
    ))
}

#[cfg(test)]
pub(crate) fn resolve_effective_startup_config(
    server: &ServerInstance,
    settings: &AppSettings,
) -> EffectiveStartupConfig {
    resolve_effective_startup_config_checked(server, settings).unwrap_or_else(|_| {
        resolve_effective_startup_config_from_document(
            &ServerStartupConfigDocument::default(),
            &runtime_defaults(server),
            &resolution_defaults(settings),
        )
    })
}

pub(super) fn build_managed_jvm_args(
    server: &ServerInstance,
    settings: &AppSettings,
    console_encoding: ManagedConsoleEncoding,
) -> Result<Vec<String>, String> {
    let java_encoding = console_encoding.java_name();
    let effective = resolve_effective_startup_config_checked(server, settings)?;
    let startup_mode = StartupMode::from_raw(server.startup_mode_str());
    let active_processor_count_arg =
        cpu_policy::compute_active_processor_count_arg(server, settings, startup_mode)?;

    Ok(build_managed_jvm_args_from_input(ManagedJvmBuildInput {
        effective,
        java_encoding: java_encoding.to_string(),
        default_jvm_args: settings.default_jvm_args.clone(),
        active_processor_count_arg,
    }))
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
        .map_err(|e| manager_t1("server.manager.user_jvm_args_write_failed", e.to_string()))
}

pub(crate) fn read_startup_document_checked(
    server: &ServerInstance,
) -> Result<ServerStartupConfigDocument, String> {
    read_server_startup_config_document(&server.path)
}

#[cfg(test)]
mod tests {
    use super::{
        build_managed_jvm_args, read_startup_document_checked, resolve_effective_startup_config,
        resolve_effective_startup_config_checked, write_user_jvm_args,
    };
    use crate::models::server::{
        CpuPolicyConfig, CpuPolicyMode, JvmPresetConfig, JvmPresetId, LocalRuntimeConfig,
        ServerInstance, ServerRuntimeConfig,
    };
    use crate::models::settings::AppSettings;
    use sea_lantern_server_local_setup_core::ManagedConsoleEncoding;
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
                terminal_mode: crate::models::server::LocalTerminalMode::PipeManaged,
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
    fn build_managed_jvm_args_prefers_instance_config_memory_and_preserves_arg_order() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        let config_dir = temp_dir.path().join("SeaLantern");
        std::fs::create_dir_all(&config_dir).expect("config dir should be created");
        std::fs::write(config_dir.join("config.toml"), "max_memory = 3072\nmin_memory = 1536\n")
            .expect("config.toml should be written");

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
    fn effective_startup_config_prefers_instance_values_for_runtime_overrides() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut server = test_server(temp_dir.path().to_string_lossy().to_string());
        if let ServerRuntimeConfig::Local(runtime) = &mut server.runtime {
            runtime.jvm_args = vec!["-Druntime.flag=true".to_string()];
            runtime.cpu_policy = CpuPolicyConfig {
                mode: CpuPolicyMode::Explicit,
                count: None,
                explicit_set: Some("4,6".to_string()),
                sync_active_processor_count: true,
            };
            runtime.jvm_preset = JvmPresetConfig { preset: JvmPresetId::ThroughputBasic };
        }

        let config_dir = temp_dir.path().join("SeaLantern");
        std::fs::create_dir_all(&config_dir).expect("config dir should be created");
        std::fs::write(
            config_dir.join("config.toml"),
            concat!(
                "max_memory = 3072\n",
                "min_memory = 1536\n",
                "jvm_args = [\"-Dinstance.flag=true\"]\n",
                "[cpu_policy]\n",
                "mode = \"count\"\n",
                "count = 2\n",
                "sync_active_processor_count = true\n",
                "[jvm_preset]\n",
                "preset = \"aikar_g1\"\n"
            ),
        )
        .expect("config.toml should be written");

        let effective = resolve_effective_startup_config(&server, &test_settings());

        assert_eq!(effective.max_memory, 3072);
        assert_eq!(effective.min_memory, 1536);
        assert_eq!(effective.jvm_args, vec!["-Dinstance.flag=true"]);
        assert_eq!(effective.cpu_policy.mode, CpuPolicyMode::Count);
        assert_eq!(effective.cpu_policy.count, Some(2));
        assert_eq!(effective.jvm_preset.preset, JvmPresetId::AikarG1);
    }

    #[test]
    fn effective_startup_config_falls_back_to_runtime_when_instance_values_missing() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut server = test_server(temp_dir.path().to_string_lossy().to_string());
        if let ServerRuntimeConfig::Local(runtime) = &mut server.runtime {
            runtime.jvm_args = vec!["-Druntime.flag=true".to_string()];
            runtime.cpu_policy = CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(3),
                explicit_set: None,
                sync_active_processor_count: true,
            };
            runtime.jvm_preset = JvmPresetConfig {
                preset: JvmPresetId::PaperRecommendedLite,
            };
        }

        let effective = resolve_effective_startup_config(&server, &test_settings());

        assert_eq!(effective.max_memory, 4096);
        assert_eq!(effective.min_memory, 2048);
        assert_eq!(effective.jvm_args, vec!["-Druntime.flag=true"]);
        assert_eq!(effective.cpu_policy.mode, CpuPolicyMode::Count);
        assert_eq!(effective.cpu_policy.count, Some(3));
        assert_eq!(effective.jvm_preset.preset, JvmPresetId::PaperRecommendedLite);
    }

    #[test]
    fn write_user_jvm_args_uses_default_min_memory_when_instance_config_omits_it() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        let config_dir = temp_dir.path().join("SeaLantern");
        std::fs::create_dir_all(&config_dir).expect("config dir should be created");
        std::fs::write(config_dir.join("config.toml"), "max_memory = 6144\n")
            .expect("config.toml should be written");

        write_user_jvm_args(&server, &test_settings(), ManagedConsoleEncoding::Utf8)
            .expect("user_jvm_args.txt should be written");

        let content = std::fs::read_to_string(temp_dir.path().join("user_jvm_args.txt"))
            .expect("user_jvm_args.txt should exist");

        assert_eq!(
            content,
            concat!(
                "-Xmx6144M\n",
                "-Xms2048M\n",
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
        assert!(err.contains("鏆備笉鏀寔 CPU policy"));
    }

    #[test]
    fn read_startup_document_checked_surfaces_invalid_instance_config() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        let config_dir = temp_dir.path().join("SeaLantern");
        std::fs::create_dir_all(&config_dir).expect("config dir should be created");
        std::fs::write(config_dir.join("config.toml"), "max_memory = [\n")
            .expect("broken config.toml should be written");

        let err = read_startup_document_checked(&server)
            .expect_err("invalid config should surface an explicit error");

        assert!(err.contains("瑙ｆ瀽瀹炰緥閰嶇疆澶辫触"));
    }

    #[test]
    fn resolve_effective_startup_config_checked_surfaces_invalid_instance_config() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        let config_dir = temp_dir.path().join("SeaLantern");
        std::fs::create_dir_all(&config_dir).expect("config dir should be created");
        std::fs::write(config_dir.join("config.toml"), "max_memory = [\n")
            .expect("broken config.toml should be written");

        let err = resolve_effective_startup_config_checked(&server, &test_settings())
            .expect_err("invalid config should not silently downgrade to runtime defaults");

        assert!(err.contains("瑙ｆ瀽瀹炰緥閰嶇疆澶辫触"));
    }

    #[test]
    fn build_managed_jvm_args_surfaces_invalid_instance_config() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        let config_dir = temp_dir.path().join("SeaLantern");
        std::fs::create_dir_all(&config_dir).expect("config dir should be created");
        std::fs::write(config_dir.join("config.toml"), "max_memory = [\n")
            .expect("broken config.toml should be written");

        let err = build_managed_jvm_args(&server, &test_settings(), ManagedConsoleEncoding::Utf8)
            .expect_err("invalid config should abort managed JVM arg synthesis");

        assert!(err.contains("瑙ｆ瀽瀹炰緥閰嶇疆澶辫触"));
    }
}
