use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct m_PluginInfo {
    pub m_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub file_name: String,
    pub file_size: u64,
    pub enabled: bool,
    pub main_class: String,
    pub has_config_folder: bool,
    pub config_files: Vec<m_PluginConfigFile>,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct m_PluginConfigFile {
    pub file_name: String,
    pub content: String,
    pub file_type: String,
    pub file_path: String,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct m_PluginConfig {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub authors: Option<Vec<String>>,
    pub main: Option<String>,
}
