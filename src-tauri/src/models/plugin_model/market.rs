use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdateInfo {
    pub plugin_id: String,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: Option<String>,
    pub changelog: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPluginInfo {
    pub id: String,
    pub name: serde_json::Value,
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default = "default_empty_value")]
    pub description: serde_json::Value,
    #[serde(default)]
    pub author: MarketAuthorInfo,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub download_url: Option<String>,
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub download_type: Option<String>,
    #[serde(default)]
    pub release_asset: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default, skip_deserializing)]
    pub _path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketAuthorInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
}

fn default_empty_value() -> serde_json::Value {
    serde_json::Value::String(String::new())
}
