use serde::{Deserialize, Serialize};

fn default_startup_mode() -> String {
    "jar".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInstance {
    pub id: String,
    pub name: String,
    pub core_type: String,
    pub core_version: String,
    pub mc_version: String,
    pub path: String,
    pub jar_path: String,
    #[serde(default = "default_startup_mode")]
    pub startup_mode: String,
    #[serde(default)]
    pub custom_command: Option<String>,
    pub java_path: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub jvm_args: Vec<String>,
    pub port: u16,
    pub created_at: u64,
    pub last_started_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusInfo {
    pub id: String,
    pub status: ServerStatus,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub core_type: String,
    pub mc_version: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub port: u16,
    pub java_path: String,
    pub jar_path: String,
    #[serde(default = "default_startup_mode")]
    pub startup_mode: String,
    #[serde(default)]
    pub custom_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportServerRequest {
    pub name: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportModpackRequest {
    pub name: String,
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
    pub use_software_data_dir: bool,
    #[serde(default)]
    pub startup_file_path: Option<String>,
    #[serde(default)]
    pub core_type: Option<String>,
    #[serde(default)]
    pub mc_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddExistingServerRequest {
    pub name: String,
    pub server_path: String,
    pub java_path: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub port: u16,
    pub startup_mode: String,
    pub executable_path: Option<String>,
    #[serde(default)]
    pub custom_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedServerCoreInfo {
    pub core_type: String,
    pub main_class: Option<String>,
    pub jar_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupScanResult {
    pub parsed_core: ParsedServerCoreInfo,
    pub candidates: Vec<StartupCandidateItem>,
    pub detected_core_type_key: Option<String>,
    pub core_type_options: Vec<String>,
    pub mc_version_options: Vec<String>,
    pub detected_mc_version: Option<String>,
    pub mc_version_detection_failed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupCandidateItem {
    pub id: String,
    pub mode: String,
    pub label: String,
    pub detail: String,
    pub path: String,
    pub recommended: u8,
}
