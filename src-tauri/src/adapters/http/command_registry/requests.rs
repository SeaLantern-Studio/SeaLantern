use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CreateServerRequest {
    pub name: String,
    pub core_type: String,
    pub mc_version: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub port: u16,
    pub java_path: String,
    pub jar_path: String,
    pub startup_mode: String,
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
