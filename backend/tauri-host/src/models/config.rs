use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub description: String,
    pub value_type: String,
    pub default_value: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProperties {
    pub entries: Vec<ConfigEntry>,
    pub raw: HashMap<String, String>,
}
