use super::super::super::common::StartupMode;
use crate::models::server::ServerInstance;
use crate::services::server::manager::i18n::{manager_t1, manager_t2};
use server_installer::resolve_starter_core_key_checked as resolve_shared_starter_core_key;
use server_local_setup::startup_mode_requires_java;
use server_local_setup::ManagedConsoleEncoding;

pub(in crate::services::server::manager::runtime_start) struct LaunchContext<'a> {
    pub server: &'a ServerInstance,
    pub settings: &'a crate::models::settings::AppSettings,
    pub startup_mode: StartupMode,
    pub managed_console_encoding: ManagedConsoleEncoding,
    pub java_bin_dir_str: Option<String>,
    pub java_home_dir_str: Option<String>,
    pub startup_filename: String,
    pub starter_core_key: String,
}

impl LaunchContext<'_> {
    pub(in crate::services::server::manager::runtime_start) fn java_env(
        &self,
    ) -> Option<(&str, &str)> {
        Some((self.java_home_dir_str.as_deref()?, self.java_bin_dir_str.as_deref()?))
    }

    pub(in crate::services::server::manager::runtime_start) fn jar_path_required(
        &self,
    ) -> Result<&str, String> {
        self.server
            .jar_path()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                manager_t1(
                    "server.manager.launch_startup_path_missing",
                    self.startup_mode.as_str().to_string(),
                )
            })
    }

    pub(in crate::services::server::manager::runtime_start) fn java_path_required(
        &self,
    ) -> Result<&str, String> {
        self.server
            .java_path()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                manager_t1(
                    "server.manager.launch_java_path_missing",
                    self.startup_mode.as_str().to_string(),
                )
            })
    }

    pub(in crate::services::server::manager::runtime_start) fn java_env_required(
        &self,
    ) -> Result<(&str, &str), String> {
        self.java_env().ok_or_else(|| {
            manager_t1(
                "server.manager.launch_java_env_missing",
                self.startup_mode.as_str().to_string(),
            )
        })
    }

    pub(in crate::services::server::manager::runtime_start) fn validate_for_launch(
        &self,
    ) -> Result<(), String> {
        if self.server.local_runtime().is_none() {
            return Err(manager_t2(
                "server.runtime.config_mismatch",
                self.server.runtime_kind.clone(),
                "local",
            ));
        }

        if matches!(self.startup_mode, StartupMode::Jar | StartupMode::Starter) {
            self.jar_path_required()?;
        }

        if startup_mode_requires_java(self.startup_mode.as_str()) {
            self.java_path_required()?;
            self.java_env_required()?;
        }

        Ok(())
    }
}

pub(in crate::services::server::manager::runtime_start) fn resolve_starter_core_key(
    server: &ServerInstance,
) -> Result<String, String> {
    let resolution = resolve_shared_starter_core_key(
        server.startup_mode_str(),
        Some(&server.core_type),
        server.jar_path(),
    );
    if resolution.needs_unrecognized_error(server.startup_mode_str()) {
        return Err(manager_t1(
            "server.manager.starter_core_type_unrecognized",
            resolution.unresolved_display_hint(Some(&server.core_type)),
        ));
    }

    Ok(resolution.starter_core_key)
}

#[cfg(test)]
mod tests {
    use super::{resolve_starter_core_key, LaunchContext};
    use crate::models::server::{
        CpuPolicyConfig, DockerBackendKind, DockerCommandMode, DockerItzgRuntimeConfig,
        JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
    };
    use crate::models::settings::AppSettings;
    use crate::services::server::manager::common::StartupMode;
    use server_local_setup::ManagedConsoleEncoding;
    use std::collections::BTreeMap;

    fn test_server(startup_mode: &str, core_type: &str, jar_path: &str) -> ServerInstance {
        ServerInstance {
            id: format!("context-{}", startup_mode),
            name: "Context".to_string(),
            aliases: Vec::new(),
            core_type: core_type.to_string(),
            core_version: core_type.to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/servers/context".to_string(),
            port: 25565,
            max_memory: 2048,
            min_memory: 1024,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: jar_path.to_string(),
                startup_mode: startup_mode.to_string(),
                custom_command: None,
                java_path: "java".to_string(),
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    fn test_context<'a>(
        server: &'a ServerInstance,
        settings: &'a AppSettings,
        startup_mode: StartupMode,
        java_home_dir_str: Option<&str>,
        java_bin_dir_str: Option<&str>,
    ) -> LaunchContext<'a> {
        LaunchContext {
            server,
            settings,
            startup_mode,
            managed_console_encoding: ManagedConsoleEncoding::Utf8,
            java_bin_dir_str: java_bin_dir_str.map(str::to_string),
            java_home_dir_str: java_home_dir_str.map(str::to_string),
            startup_filename: "server.jar".to_string(),
            starter_core_key: String::new(),
        }
    }

    fn docker_runtime() -> DockerItzgRuntimeConfig {
        DockerItzgRuntimeConfig {
            image: "itzg/minecraft-server".to_string(),
            image_tag: "java21".to_string(),
            container_name: "sea-test".to_string(),
            type_value: "PAPER".to_string(),
            version: "1.21.1".to_string(),
            data_dir_mount: "E:/servers/docker-runtime".to_string(),
            published_game_port: 25565,
            env: BTreeMap::new(),
            extra_ports: Vec::new(),
            volume_mounts: Vec::new(),
            docker_backend_kind: DockerBackendKind::Cli,
            command_mode: DockerCommandMode::Rcon,
            rcon: None,
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        }
    }

    #[test]
    fn resolve_starter_core_key_returns_empty_string_for_non_starter_modes() {
        let server = test_server("jar", "paper", "server.jar");

        let core_key = resolve_starter_core_key(&server).expect("jar mode should not fail");

        assert_eq!(core_key, "");
    }

    #[test]
    fn resolve_starter_core_key_prefers_normalized_server_core_type_for_starter() {
        let server = test_server(
            "starter",
            "Neoforge",
            "E:/minecraft/neoforge-26.1.0.0-alpha.1+snapshot-1-installer.jar",
        );

        let core_key = resolve_starter_core_key(&server).expect("starter core type should resolve");

        assert_eq!(core_key, "neoforge");
    }

    #[test]
    fn resolve_starter_core_key_falls_back_to_detected_launch_target() {
        let server = test_server(
            "starter",
            "   ",
            "E:/minecraft/neoforge-26.1.0.0-alpha.1+snapshot-1-installer.jar",
        );

        let core_key = resolve_starter_core_key(&server)
            .expect("starter core type should fall back to detected launch target");

        assert_eq!(core_key, "neoforge");
    }

    #[test]
    fn validate_for_launch_rejects_non_local_runtime() {
        let settings = AppSettings::default();
        let server = ServerInstance {
            id: "docker-runtime".to_string(),
            name: "Docker Runtime".to_string(),
            aliases: Vec::new(),
            core_type: "paper".to_string(),
            core_version: "paper".to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/servers/docker-runtime".to_string(),
            port: 25565,
            max_memory: 2048,
            min_memory: 1024,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "docker_itzg".to_string(),
            runtime: ServerRuntimeConfig::DockerItzg(docker_runtime()),
        };
        let context = test_context(
            &server,
            &settings,
            StartupMode::Jar,
            Some("C:/Java"),
            Some("C:/Java/bin"),
        );

        let err = context
            .validate_for_launch()
            .expect_err("non-local runtime should not pass local launch validation");

        assert!(err.contains("runtime_kind=docker_itzg"));
        assert!(err.contains("runtime.kind=local"));
    }

    #[test]
    fn validate_for_launch_rejects_missing_startup_path_for_jar_mode() {
        let settings = AppSettings::default();
        let server = test_server("jar", "paper", "   ");
        let context = test_context(
            &server,
            &settings,
            StartupMode::Jar,
            Some("C:/Java"),
            Some("C:/Java/bin"),
        );

        let err = context
            .validate_for_launch()
            .expect_err("jar mode should require a startup path");

        assert!(err.contains("jar"));
    }

    #[test]
    fn validate_for_launch_rejects_missing_java_env_for_java_modes() {
        let settings = AppSettings::default();
        let server = test_server("starter", "forge", "installer.jar");
        let context = test_context(&server, &settings, StartupMode::Starter, None, None);

        let err = context
            .validate_for_launch()
            .expect_err("starter mode should require resolved java env");

        assert!(err.contains("starter"));
    }
}
