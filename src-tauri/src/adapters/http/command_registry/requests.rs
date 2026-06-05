use serde::Deserialize;
use std::collections::HashMap;

use crate::models::server::{CpuPolicyConfig, JvmPresetConfig};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CreateServerRequest {
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
    pub startup_mode: String,
    #[serde(default)]
    pub custom_command: Option<String>,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub cpu_policy: CpuPolicyConfig,
    #[serde(default)]
    pub jvm_preset: JvmPresetConfig,
}

#[cfg(test)]
mod tests {
    use super::CreateServerRequest;

    #[test]
    fn create_server_request_accepts_extended_optional_fields() {
        let req: CreateServerRequest = serde_json::from_str(
            r#"{
  "name": "custom-local",
  "aliases": ["cache_server", "test_server"],
  "coreType": "Fabric",
  "mcVersion": "1.20.1",
  "maxMemory": 4096,
  "minMemory": 2048,
  "port": 25565,
  "javaPath": "C:/Java/bin/java.exe",
  "jarPath": "E:/srv/server.jar",
  "serverPath": "E:/srv",
  "startupMode": "custom",
  "customCommand": "java -Xmx4G -Xms4G -jar server.jar nogui"
}"#,
        )
        .expect("extended create request should deserialize");

        assert_eq!(req.aliases, vec!["cache_server", "test_server"]);
        assert_eq!(req.server_path.as_deref(), Some("E:/srv"));
        assert_eq!(req.custom_command.as_deref(), Some("java -Xmx4G -Xms4G -jar server.jar nogui"));
        assert!(req.jvm_args.is_empty());
    }

    #[test]
    fn create_server_request_defaults_extended_fields_for_legacy_payloads() {
        let req: CreateServerRequest = serde_json::from_str(
            r#"{
  "name": "legacy-local",
  "coreType": "Paper",
  "mcVersion": "1.21.1",
  "maxMemory": 4096,
  "minMemory": 2048,
  "port": 25565,
  "javaPath": "C:/Java/bin/java.exe",
  "jarPath": "E:/srv/server.jar",
  "startupMode": "jar"
}"#,
        )
        .expect("legacy create request should still deserialize");

        assert!(req.aliases.is_empty());
        assert_eq!(req.server_path, None);
        assert_eq!(req.custom_command, None);
        assert!(req.jvm_args.is_empty());
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ImportServerRequest {
    pub name: String,
    pub jar_path: String,
    pub startup_mode: String,
    pub java_path: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub port: u16,
    pub online_mode: bool,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub cpu_policy: CpuPolicyConfig,
    #[serde(default)]
    pub jvm_preset: JvmPresetConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ImportModpackRequest {
    pub name: String,
    pub modpack_path: String,
    pub java_path: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub port: u16,
    pub startup_mode: String,
    pub online_mode: bool,
    pub custom_command: Option<String>,
    pub run_path: String,
    pub startup_file_path: Option<String>,
    pub core_type: Option<String>,
    pub mc_version: Option<String>,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub cpu_policy: CpuPolicyConfig,
    #[serde(default)]
    pub jvm_preset: JvmPresetConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SendCommandRequest {
    pub id: String,
    pub command: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GetServerStatusRequest {
    pub id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ServerIdRequest {
    pub id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GetLogsRequest {
    pub id: String,
    pub since: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UpdateNameRequest {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UpdateJavaPathRequest {
    pub id: String,
    pub java_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScanStartupCandidatesRequest {
    pub source_path: String,
    pub source_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ParseServerCoreTypeRequest {
    pub source_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CollectCopyConflictsRequest {
    pub source_dir: String,
    pub target_dir: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CopyDirectoryContentsRequest {
    pub source_dir: String,
    pub target_dir: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AddExistingServerRequest {
    pub name: String,
    pub server_path: String,
    pub java_path: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub port: u16,
    pub startup_mode: String,
    pub executable_path: Option<String>,
    pub custom_command: Option<String>,
    pub core_type: Option<String>,
    pub mc_version: Option<String>,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub cpu_policy: CpuPolicyConfig,
    #[serde(default)]
    pub jvm_preset: JvmPresetConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TunnelHostRequest {
    pub port: u16,
    pub password: Option<String>,
    pub max_players: Option<u32>,
    pub relay_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TunnelJoinRequest {
    pub ticket: String,
    pub local_port: u16,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReadConfigRequest {
    pub server_path: String,
    pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WriteConfigRequest {
    pub server_path: String,
    pub path: String,
    pub values: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WriteServerPropertiesRequest {
    pub server_path: String,
    pub values: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WriteServerPropertiesSourceRequest {
    pub server_path: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ParseServerPropertiesSourceRequest {
    pub source: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PreviewServerPropertiesWriteRequest {
    pub server_path: String,
    pub values: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PreviewServerPropertiesWriteFromSourceRequest {
    pub source: String,
    pub values: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ValidateJavaPathRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReadServerPropertiesRequest {
    pub server_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ServerPathRequest {
    pub server_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PlayerActionRequest {
    pub server_id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BanPlayerRequest {
    pub server_id: String,
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct KickPlayerRequest {
    pub server_id: String,
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ExportLogsRequest {
    pub logs: Vec<String>,
    pub save_path: String,
}
