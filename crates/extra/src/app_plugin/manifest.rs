use serde::{Deserialize, Serialize};

use sealantern_core::app_plugin::ScopeBinding;

/// The only plugin API revision accepted by the first app-plugin engine.
pub const PLUGIN_API_VERSION: u32 = 2;

/// API v2 manifest 中声明的一项能力。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginCapability {
    pub id: String,
    #[serde(default)]
    pub scope: Option<ScopeBinding>,
}

impl PluginCapability {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), scope: None }
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
    pub capabilities: Vec<PluginCapability>,
}

#[cfg(test)]
mod tests {
    use super::{PluginCapability, PluginManifest, PLUGIN_API_VERSION};

    #[test]
    fn v2_manifest_uses_camel_case_api_version() {
        let manifest: PluginManifest = serde_json::from_str(
            r#"{
                "apiVersion": 2,
                "id": "example.plugin",
                "name": "Example Plugin",
                "version": "1.0.0",
                "main": "main.lua",
                "capabilities": [{"id": "plugin.log.emit"}, {"id": "plugin.storage.read"}]
            }"#,
        )
        .expect("v2 manifest should deserialize");

        assert_eq!(manifest.api_version, PLUGIN_API_VERSION);
        assert_eq!(
            manifest.capabilities,
            vec![
                PluginCapability::new("plugin.log.emit"),
                PluginCapability::new("plugin.storage.read"),
            ]
        );
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
    fn capability_rejects_unknown_fields() {
        assert!(serde_json::from_str::<PluginCapability>(
            r#"{"id":"plugin.log.emit","legacy":true}"#
        )
        .is_err());
    }
}
