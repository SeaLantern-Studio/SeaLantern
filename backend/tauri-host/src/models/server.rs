pub(crate) use sea_lantern_docker_core::{
    CreateDockerItzgServerRequest, DockerBackendKind, DockerCommandMode, DockerItzgRuntimeConfig,
    PublishedPort, RconConfig, VolumeMount,
};
#[allow(unused_imports)]
pub(crate) use sea_lantern_server_config_core::types::{
    CpuPolicyConfig, CpuPolicyMode, JvmPresetConfig, JvmPresetId,
};
pub(crate) use sea_lantern_server_installer_core::ParsedServerCoreInfo;
pub(crate) use sea_lantern_server_startup_scan_core::StartupScanResult;
use serde::{Deserialize, Serialize};

fn default_startup_mode() -> String {
    "jar".to_string()
}

fn default_runtime_kind() -> String {
    "local".to_string()
}

fn default_local_terminal_mode() -> LocalTerminalMode {
    LocalTerminalMode::PipeManaged
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

impl ServerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServerStatus::Stopped => "stopped",
            ServerStatus::Starting => "starting",
            ServerStatus::Running => "running",
            ServerStatus::Stopping => "stopping",
            ServerStatus::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInstance {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub core_type: String,
    pub core_version: String,
    pub mc_version: String,
    pub path: String,
    pub port: u16,
    #[serde(default)]
    pub max_memory: u32,
    #[serde(default)]
    pub min_memory: u32,
    pub created_at: u64,
    pub last_started_at: Option<u64>,
    #[serde(default = "default_runtime_kind")]
    pub runtime_kind: String,
    #[serde(default)]
    pub runtime: ServerRuntimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ServerRuntimeConfig {
    Local(LocalRuntimeConfig),
    DockerItzg(DockerItzgRuntimeConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalRuntimeConfig {
    pub jar_path: String,
    #[serde(default = "default_startup_mode")]
    pub startup_mode: String,
    #[serde(default)]
    pub custom_command: Option<String>,
    pub java_path: String,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default = "default_local_terminal_mode")]
    pub terminal_mode: LocalTerminalMode,
    #[serde(default)]
    pub cpu_policy: CpuPolicyConfig,
    #[serde(default)]
    pub jvm_preset: JvmPresetConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LocalTerminalMode {
    #[default]
    PipeManaged,
    PtyManaged,
}

impl ServerInstance {
    pub fn local_runtime(&self) -> Option<&LocalRuntimeConfig> {
        match &self.runtime {
            ServerRuntimeConfig::Local(runtime) => Some(runtime),
            ServerRuntimeConfig::DockerItzg(_) => None,
        }
    }

    pub fn local_runtime_mut(&mut self) -> Option<&mut LocalRuntimeConfig> {
        match &mut self.runtime {
            ServerRuntimeConfig::Local(runtime) => Some(runtime),
            ServerRuntimeConfig::DockerItzg(_) => None,
        }
    }

    pub fn docker_itzg_runtime(&self) -> Option<&DockerItzgRuntimeConfig> {
        match &self.runtime {
            ServerRuntimeConfig::Local(_) => None,
            ServerRuntimeConfig::DockerItzg(runtime) => Some(runtime),
        }
    }

    pub fn startup_mode_str(&self) -> &str {
        self.local_runtime()
            .map(|runtime| runtime.startup_mode.as_str())
            .unwrap_or("jar")
    }

    pub fn jar_path(&self) -> Option<&str> {
        self.local_runtime()
            .map(|runtime| runtime.jar_path.as_str())
    }

    pub fn java_path(&self) -> Option<&str> {
        self.local_runtime()
            .map(|runtime| runtime.java_path.as_str())
    }

    pub fn custom_command(&self) -> Option<&str> {
        self.local_runtime()
            .and_then(|runtime| runtime.custom_command.as_deref())
    }
}

impl Default for ServerRuntimeConfig {
    fn default() -> Self {
        Self::Local(LocalRuntimeConfig {
            jar_path: String::new(),
            startup_mode: default_startup_mode(),
            custom_command: None,
            java_path: String::new(),
            jvm_args: Vec::new(),
            terminal_mode: default_local_terminal_mode(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusInfo {
    pub id: String,
    pub status: ServerStatus,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
    #[serde(default)]
    pub detail_message: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub terminal: Option<TerminalStatusInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalStatusInfo {
    pub backend_kind: String,
    pub interactive_supported: bool,
    pub transcript_supported: bool,
    pub attach_supported: bool,
    #[serde(default)]
    pub cols: Option<u16>,
    #[serde(default)]
    pub rows: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub core_type: String,
    pub mc_version: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub port: u16,
    pub java_path: String,
    pub jar_path: String,
    #[serde(default)]
    pub server_path: Option<String>,
    #[serde(default = "default_startup_mode")]
    pub startup_mode: String,
    #[serde(default)]
    pub custom_command: Option<String>,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default = "default_local_terminal_mode")]
    pub terminal_mode: LocalTerminalMode,
    #[serde(default)]
    pub cpu_policy: CpuPolicyConfig,
    #[serde(default)]
    pub jvm_preset: JvmPresetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportServerRequest {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub jar_path: String,
    pub java_path: String,
    #[serde(default = "default_startup_mode")]
    pub startup_mode: String,
    #[serde(default)]
    pub custom_command: Option<String>,
    pub max_memory: u32,
    pub min_memory: u32,
    pub port: u16,
    pub online_mode: bool,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default = "default_local_terminal_mode")]
    pub terminal_mode: LocalTerminalMode,
    #[serde(default)]
    pub cpu_policy: CpuPolicyConfig,
    #[serde(default)]
    pub jvm_preset: JvmPresetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportModpackRequest {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub modpack_path: String,
    pub java_path: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub port: u16,
    #[serde(default = "default_startup_mode")]
    pub startup_mode: String,
    #[serde(default)]
    pub online_mode: bool,
    #[serde(default)]
    pub custom_command: Option<String>,
    #[serde(default)]
    pub run_path: String,
    #[serde(default)]
    pub startup_file_path: Option<String>,
    #[serde(default)]
    pub core_type: Option<String>,
    #[serde(default)]
    pub mc_version: Option<String>,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default = "default_local_terminal_mode")]
    pub terminal_mode: LocalTerminalMode,
    #[serde(default)]
    pub cpu_policy: CpuPolicyConfig,
    #[serde(default)]
    pub jvm_preset: JvmPresetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddExistingServerRequest {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub server_path: String,
    pub java_path: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub port: u16,
    pub startup_mode: String,
    pub executable_path: Option<String>,
    #[serde(default)]
    pub custom_command: Option<String>,
    #[serde(default)]
    pub core_type: Option<String>,
    #[serde(default)]
    pub mc_version: Option<String>,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default = "default_local_terminal_mode")]
    pub terminal_mode: LocalTerminalMode,
    #[serde(default)]
    pub cpu_policy: CpuPolicyConfig,
    #[serde(default)]
    pub jvm_preset: JvmPresetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateServerPathResult {
    pub valid: bool,
    pub message: String,
    #[serde(default)]
    pub jar_path: Option<String>,
    #[serde(default)]
    pub startup_mode: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        CpuPolicyConfig, DockerBackendKind, DockerCommandMode, DockerItzgRuntimeConfig,
        JvmPresetConfig, LocalRuntimeConfig, PublishedPort, ServerInstance, ServerRuntimeConfig,
        ServerStatus, ServerStatusInfo, VolumeMount,
    };

    fn sample_local_server() -> ServerInstance {
        ServerInstance {
            id: "server-1".to_string(),
            name: "Compat Test".to_string(),
            aliases: vec!["cache_server".to_string(), "test_server".to_string()],
            core_type: "paper".to_string(),
            core_version: "1.0.0".to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/servers/compat".to_string(),
            port: 25565,
            max_memory: 2048,
            min_memory: 1024,
            created_at: 123,
            last_started_at: Some(456),
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "E:/servers/compat/server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: Some("java -jar server.jar nogui".to_string()),
                java_path: "C:/Java/bin/java.exe".to_string(),
                jvm_args: vec!["-Xmx2G".to_string()],
                terminal_mode: crate::models::server::LocalTerminalMode::PipeManaged,
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    fn sample_docker_server() -> ServerInstance {
        let mut env = BTreeMap::new();
        env.insert("EULA".to_string(), "TRUE".to_string());

        ServerInstance {
            id: "docker-1".to_string(),
            name: "Docker Test".to_string(),
            aliases: vec!["docker_alias".to_string()],
            core_type: "paper".to_string(),
            core_version: "1.0.0".to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/servers/docker".to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 321,
            last_started_at: None,
            runtime_kind: "docker_itzg".to_string(),
            runtime: ServerRuntimeConfig::DockerItzg(DockerItzgRuntimeConfig {
                image: "itzg/minecraft-server".to_string(),
                image_tag: "java21".to_string(),
                container_name: "sea-lantern-docker-1".to_string(),
                type_value: "PAPER".to_string(),
                version: "1.21.1".to_string(),
                data_dir_mount: "E:/servers/docker".to_string(),
                published_game_port: 25565,
                env,
                extra_ports: vec![PublishedPort {
                    host_port: 25575,
                    container_port: 25575,
                    protocol: "tcp".to_string(),
                }],
                volume_mounts: vec![VolumeMount {
                    source: "E:/servers/docker/plugins".to_string(),
                    target: "/data/plugins".to_string(),
                    read_only: false,
                }],
                docker_backend_kind: DockerBackendKind::Cli,
                command_mode: DockerCommandMode::Rcon,
                rcon: None,
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn server_instance_serializes_with_runtime_kind_and_runtime() {
        let value = serde_json::to_value(sample_local_server()).unwrap();

        assert_eq!(value["id"], "server-1");
        assert_eq!(value["name"], "Compat Test");
        assert_eq!(value["aliases"][0], "cache_server");
        assert_eq!(value["port"], 25565);
        assert_eq!(value["max_memory"], 2048);
        assert_eq!(value["min_memory"], 1024);
        assert_eq!(value["runtime_kind"], "local");
        assert_eq!(value["runtime"]["kind"], "local");
        assert_eq!(value["runtime"]["jar_path"], "E:/servers/compat/server.jar");
        assert_eq!(value["runtime"]["java_path"], "C:/Java/bin/java.exe");
        assert!(value.get("jar_path").is_none());
    }

    #[test]
    fn server_instance_round_trips_local_runtime_shape() {
        let value = serde_json::to_value(sample_local_server()).unwrap();
        let server: ServerInstance = serde_json::from_value(value).unwrap();

        assert_eq!(server.runtime_kind, "local");
        assert_eq!(server.aliases, vec!["cache_server", "test_server"]);
        let runtime = server
            .local_runtime()
            .expect("round-tripped local runtime should stay local");
        assert_eq!(runtime.jar_path, "E:/servers/compat/server.jar");
        assert_eq!(runtime.startup_mode, "jar");
        assert_eq!(runtime.custom_command.as_deref(), Some("java -jar server.jar nogui"));
        assert_eq!(runtime.java_path, "C:/Java/bin/java.exe");
        assert_eq!(runtime.jvm_args, vec!["-Xmx2G"]);
    }

    #[test]
    fn server_instance_deserializes_new_local_runtime_with_runtime_defaults() {
        let server: ServerInstance = serde_json::from_str(
            r#"{
  "id": "new-local",
  "name": "New Local",
  "core_type": "paper",
  "core_version": "",
  "mc_version": "1.20.4",
  "path": "E:/servers/new-local",
  "port": 25565,
  "created_at": 1,
  "last_started_at": null,
  "runtime_kind": "local",
  "runtime": {
    "kind": "local",
    "jar_path": "E:/servers/new-local/server.jar",
    "java_path": "C:/Java/bin/java.exe"
  }
}"#,
        )
        .unwrap();

        assert_eq!(server.runtime_kind, "local");
        let runtime = server
            .local_runtime()
            .expect("new local runtime should deserialize");
        assert_eq!(runtime.jar_path, "E:/servers/new-local/server.jar");
        assert_eq!(runtime.startup_mode, "jar");
        assert_eq!(runtime.custom_command, None);
        assert_eq!(runtime.java_path, "C:/Java/bin/java.exe");
        assert!(runtime.jvm_args.is_empty());
    }

    #[test]
    fn server_instance_round_trips_docker_runtime_tagged_shape() {
        let value = serde_json::to_value(sample_docker_server()).unwrap();

        assert_eq!(value["runtime_kind"], "docker_itzg");
        assert_eq!(value["aliases"][0], "docker_alias");
        assert_eq!(value["runtime"]["kind"], "docker_itzg");
        assert_eq!(value["runtime"]["image"], "itzg/minecraft-server");
        assert_eq!(value["runtime"]["docker_backend_kind"], "cli");
        assert_eq!(value["runtime"]["command_mode"], "rcon");

        let server: ServerInstance = serde_json::from_value(value).unwrap();
        assert_eq!(server.runtime_kind, "docker_itzg");

        match server.runtime {
            ServerRuntimeConfig::DockerItzg(runtime) => {
                assert_eq!(runtime.image, "itzg/minecraft-server");
                assert_eq!(runtime.image_tag, "java21");
                assert_eq!(runtime.type_value, "PAPER");
                assert_eq!(runtime.version, "1.21.1");
                assert_eq!(runtime.env.get("EULA").map(String::as_str), Some("TRUE"));
                assert_eq!(runtime.extra_ports.len(), 1);
                assert_eq!(runtime.volume_mounts.len(), 1);
                assert_eq!(runtime.docker_backend_kind, DockerBackendKind::Cli);
                assert_eq!(runtime.command_mode, DockerCommandMode::Rcon);
            }
            ServerRuntimeConfig::Local(_) => panic!("expected docker runtime after round-trip"),
        }
    }

    #[test]
    fn server_status_info_serializes_compatibly_for_status_calls() {
        let value = serde_json::to_value(ServerStatusInfo {
            id: "server-1".to_string(),
            status: ServerStatus::Running,
            pid: Some(4321),
            uptime: Some(99),
            detail_message: Some("runtime=local/jar".to_string()),
            error_message: Some("runtime placeholder".to_string()),
            terminal: None,
        })
        .unwrap();

        assert_eq!(value["id"], "server-1");
        assert_eq!(value["status"], "Running");
        assert_eq!(value["pid"], 4321);
        assert_eq!(value["uptime"], 99);
        assert_eq!(value["detail_message"], "runtime=local/jar");
        assert_eq!(value["error_message"], "runtime placeholder");
    }

    #[test]
    fn server_instance_deserializes_legacy_shape_to_default_local_runtime() {
        let server: ServerInstance = serde_json::from_str(
            r#"{
  "id": "legacy-default",
  "name": "Legacy Default",
  "core_type": "paper",
  "core_version": "",
  "mc_version": "1.20.4",
  "path": "E:/servers/legacy-default",
  "port": 25565,
  "created_at": 1,
  "last_started_at": null
}"#,
        )
        .unwrap();

        assert_eq!(server.runtime_kind, "local");
        let runtime = server
            .local_runtime()
            .expect("default runtime should be local");
        assert_eq!(runtime.startup_mode, "jar");
        assert_eq!(runtime.jar_path, "");
        assert_eq!(runtime.java_path, "");
    }
}
