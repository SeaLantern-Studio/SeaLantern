use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCommand {
    pub id: String,
    pub name: String,
    pub command: String,
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
    pub java_path: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub jvm_args: Vec<String>,
    pub port: u16,
    pub created_at: u64,
    pub last_started_at: Option<u64>,
    pub commands: Vec<ServerCommand>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportServerRequest {
    pub name: String,
    pub jar_path: String,
    pub java_path: String,
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
}
