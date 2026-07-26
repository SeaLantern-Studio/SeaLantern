use serde::{Deserialize, Serialize};

/// The only plugin API revision accepted by the first app-plugin engine.
pub const PLUGIN_API_VERSION: u32 = 2;

/// A capability exposed by the first app-plugin API revision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PluginPermission {
    Log,
    Storage,
}

impl PluginPermission {
    /// Parses a capability name from the manifest's JSON representation.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "log" => Some(Self::Log),
            "storage" => Some(Self::Storage),
            _ => None,
        }
    }
}

/// Strict v2 metadata required before a plugin script may be evaluated.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginManifest {
    pub api_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub main: String,
    #[serde(default)]
    pub permissions: Vec<PluginPermission>,
}

#[cfg(test)]
mod tests {
    use super::{PluginManifest, PluginPermission, PLUGIN_API_VERSION};

    #[test]
    fn v2_manifest_uses_camel_case_api_version() {
        let manifest: PluginManifest = serde_json::from_str(
            r#"{
                "apiVersion": 2,
                "id": "example.plugin",
                "name": "Example Plugin",
                "version": "1.0.0",
                "main": "main.lua",
                "permissions": ["log", "storage"]
            }"#,
        )
        .expect("v2 manifest should deserialize");

        assert_eq!(manifest.api_version, PLUGIN_API_VERSION);
        assert_eq!(manifest.permissions, vec![PluginPermission::Log, PluginPermission::Storage]);
    }

    #[test]
    fn manifest_rejects_unknown_top_level_fields() {
        let error = serde_json::from_str::<PluginManifest>(
            r#"{
                "apiVersion": 2,
                "id": "example.plugin",
                "name": "Example Plugin",
                "version": "1.0.0",
                "main": "main.lua",
                "legacyField": true
            }"#,
        )
        .expect_err("v2 manifest must not accept legacy fields");

        assert!(error.to_string().contains("legacyField"));
    }

    #[test]
    fn permission_enum_rejects_unknown_values() {
        assert!(serde_json::from_str::<PluginPermission>(r#""network""#).is_err());
    }
}
