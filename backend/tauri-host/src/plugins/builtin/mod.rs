use crate::models::plugin::{
    PluginActions, PluginAuthor, PluginDistributionClass, PluginExecutionClass, PluginInfo,
    PluginIntegrityStatus, PluginManifest, PluginPermissionProfile, PluginReviewStatus,
    PluginRuntimeKind, PluginSettingField, PluginSource, PluginState, PluginTrustLevelDisplay,
    PluginTrustedPolicySource,
};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const BUILTIN_PLUGIN_ID: &str = "sea-lantern-builtin-demo";

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
            name: "SeaLantern Builtin Demo".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "内置 Rust 插件接入骨架示例，用于验证内置插件可见、可启停、可保存设置。"
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
            settings: Some(vec![PluginSettingField {
                key: "banner_text".to_string(),
                label: "Banner Text".to_string(),
                field_type: "string".to_string(),
                display: None,
                default: Some(json!("Hello from builtin Rust plugin")),
                description: Some("用于验证内置插件设置写入流程。".to_string()),
                options: None,
                rows: None,
                maxlength: Some(120),
            }]),
            sidebar: None,
            locales: Some(HashMap::from([
                (
                    "zh-CN".to_string(),
                    crate::models::plugin::PluginLocaleEntry {
                        name: Some("SeaLantern 内置示例插件".to_string()),
                        description: Some(
                            "用于验证内置 Rust 插件接入插件管理页的第一版实现。".to_string(),
                        ),
                    },
                ),
                (
                    "en-US".to_string(),
                    crate::models::plugin::PluginLocaleEntry {
                        name: Some("SeaLantern Builtin Demo Plugin".to_string()),
                        description: Some(
                            "First-pass builtin Rust plugin integration skeleton.".to_string(),
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
        path: root.join(BUILTIN_PLUGIN_ID).to_string_lossy().to_string(),
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
        BUILTIN_PLUGIN_ID => Some(json!({
            "banner_text": "Hello from builtin Rust plugin"
        })),
        _ => None,
    }
}
