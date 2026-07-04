#[cfg(all(feature = "docker", feature = "plugin-builtin-runtime"))]
#[path = "obv11-client/mod.rs"]
pub(crate) mod obv11_client;

#[cfg(not(all(feature = "docker", feature = "plugin-builtin-runtime")))]
#[path = "obv11-client/mod_stub.rs"]
pub(crate) mod obv11_client;

use crate::models::plugin::{
    PluginActions, PluginAuthor, PluginDistributionClass, PluginExecutionClass, PluginInfo,
    PluginIntegrityStatus, PluginManifest, PluginPermissionProfile, PluginReviewStatus,
    PluginRuntimeKind, PluginSource, PluginState, PluginTrustLevelDisplay,
    PluginTrustedPolicySource,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(crate) const BUILTIN_PLUGIN_ID: &str = obv11_client::PLUGIN_ID;

pub(crate) fn builtin_plugins_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("plugins")
        .join("builtin")
}

pub(crate) fn builtin_settings_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("builtin")
}

pub(crate) fn builtin_plugin_infos() -> Vec<PluginInfo> {
    let root = builtin_plugins_root();
    vec![PluginInfo {
        manifest: PluginManifest {
            id: BUILTIN_PLUGIN_ID.to_string(),
            name: "SeaLantern OneBot v11 Client".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "内置 Rust OneBot v11 协议端，可通过 HTTP 或 WebSocket 暴露 SeaLantern 状态与控制 API，也可将事件直连转发到 QQ OneBot 端。"
                .to_string(),
            author: PluginAuthor {
                name: "SeaLantern".to_string(),
                email: None,
                url: None,
            },
            main: "builtin:rust".to_string(),
            license: None,
            homepage: None,
            repository: None,
            engines: None,
            permissions: Vec::new(),
            ui: None,
            events: Vec::new(),
            commands: Vec::new(),
            programs: Vec::new(),
            dependencies: Vec::new(),
            optional_dependencies: Vec::new(),
            icon: None,
            settings: Some(obv11_client::manifest_settings()),
            sidebar: None,
            locales: Some(HashMap::from([
                (
                    "zh-CN".to_string(),
                    crate::models::plugin::PluginLocaleEntry {
                        name: Some("SeaLantern OneBot v11 协议端".to_string()),
                        description: Some(
                            "提供 SeaLantern 的 OneBot v11 HTTP / WebSocket API 与 QQ 事件转发能力。"
                                .to_string(),
                        ),
                    },
                ),
                (
                    "en-US".to_string(),
                    crate::models::plugin::PluginLocaleEntry {
                        name: Some("SeaLantern OneBot v11 Client".to_string()),
                        description: Some(
                            "Builtin Rust OneBot v11 endpoint for SeaLantern status, control APIs, and QQ event forwarding."
                                .to_string(),
                        ),
                    },
                ),
            ])),
            include: Vec::new(),
            capabilities: Vec::new(),
            theme_var_map: HashMap::new(),
            presets: HashMap::new(),
            server_events: HashMap::new(),
        },
        state: PluginState::Disabled,
        path: root.join("obv11-client").to_string_lossy().to_string(),
        source: PluginSource::Builtin,
        runtime: PluginRuntimeKind::Rust,
        actions: PluginActions {
            can_toggle: true,
            can_delete: false,
            can_check_update: false,
        },
        missing_dependencies: Vec::new(),
        trust_level_display: PluginTrustLevelDisplay::Builtin,
        execution_class: PluginExecutionClass::BuiltinFull,
        review_status: PluginReviewStatus::Builtin,
        integrity_status: PluginIntegrityStatus::Bundled,
        trusted_policy_source: PluginTrustedPolicySource::Builtin,
        permission_profile: PluginPermissionProfile::BuiltinFull,
        publisher_id: Some("sealantern".to_string()),
        distribution_class: PluginDistributionClass::Builtin,
        trusted_catalog_matched: false,
        hash_matched: false,
        verified_hash: None,
        verified_signature: false,
        reviewed_at: None,
        revoked: false,
        exceeds_standard_sandbox: false,
        requires_explicit_consent: false,
    }]
}

pub(crate) fn default_settings(plugin_id: &str) -> Option<serde_json::Value> {
    match plugin_id {
        BUILTIN_PLUGIN_ID => Some(obv11_client::default_settings_json()),
        _ => None,
    }
}
