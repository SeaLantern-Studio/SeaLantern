use sea_lantern_server_local_setup_core::canonical_core_type;
use server_flavor_core::normalize_core_key as normalize_published_core_key;

#[cfg(test)]
use server_flavor_core::{
    resolve_profile_from_parts, ServerEdition, ServerFlavorProfile, ServerRole,
};

pub(crate) fn normalize_core_key_for_profile(input: &str) -> Option<String> {
    normalize_published_core_key(input)
        .map(|core_key| match core_key {
            "arclight_forge" => "arclight-forge".to_string(),
            "arclight_neoforge" => "arclight-neoforge".to_string(),
            other => other.to_string(),
        })
        .or_else(|| {
            let normalized = canonical_core_type(input);
            (normalized != input.trim()).then_some(normalized)
        })
}

pub(crate) fn normalize_core_key_for_api(input: &str) -> Option<String> {
    normalize_core_key_for_profile(input)
}

pub(crate) fn canonical_server_core_type(input: &str) -> String {
    canonical_core_type(input)
}

pub(crate) fn normalize_user_core_type(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("--core 不能为空".to_string());
    }

    Ok(canonical_server_core_type(trimmed))
}

#[cfg(test)]
pub(crate) fn canonical_detected_core_type(input: &str) -> String {
    canonical_server_core_type(input)
}

#[cfg(test)]
use crate::models::server::ServerInstance;

#[cfg(test)]
fn resolve_server_flavor(server: &ServerInstance) -> ServerFlavorProfile {
    let core_key = normalize_core_key_for_profile(&server.core_type);
    resolve_profile_from_parts(
        core_key.as_deref(),
        Some(server.runtime_kind.as_str()),
        Some(server.startup_mode_str()),
        None,
        false,
    )
}

#[cfg(test)]
fn extension_dir_for_kind(
    server: &ServerInstance,
    kind: server_flavor_core::ServerExtensionKind,
) -> Option<&'static str> {
    sea_lantern_server_plugin_core::resolve_extension_relative_dir(
        &server.core_type,
        &server.runtime_kind,
        server.startup_mode_str(),
        kind,
    )
}

#[cfg(test)]
pub(crate) fn profile_summary(server: &ServerInstance) -> (ServerEdition, ServerRole) {
    let profile = resolve_server_flavor(server);
    (profile.edition, profile.server_role)
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_detected_core_type, canonical_server_core_type, extension_dir_for_kind,
        normalize_core_key_for_api, normalize_core_key_for_profile, normalize_user_core_type,
        profile_summary, resolve_server_flavor,
    };
    use crate::models::server::{
        CpuPolicyConfig, JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
    };
    use server_flavor_core::{ServerEdition, ServerExtensionKind, ServerFlavorKind, ServerRole};

    #[derive(Debug, serde::Deserialize)]
    struct SharedServerCoreTaxonomyDocument {
        entries: Vec<SharedServerCoreTaxonomyEntry>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SharedServerCoreTaxonomyEntry {
        key: String,
        #[serde(default)]
        supports_plugin_extensions: bool,
        #[serde(default)]
        aliases: Vec<SharedServerCoreTaxonomyAlias>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct SharedServerCoreTaxonomyAlias {
        value: String,
    }

    fn test_server(core_type: &str, startup_mode: &str) -> ServerInstance {
        ServerInstance {
            id: format!("test-{core_type}"),
            name: "Test".to_string(),
            aliases: Vec::new(),
            core_type: core_type.to_string(),
            core_version: core_type.to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/servers/test".to_string(),
            port: 25565,
            max_memory: 2048,
            min_memory: 1024,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "server.jar".to_string(),
                startup_mode: startup_mode.to_string(),
                custom_command: None,
                java_path: "java".to_string(),
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn normalizes_legacy_core_aliases_for_profile() {
        assert_eq!(normalize_core_key_for_profile("Nukkitx").as_deref(), Some("nukkit"));
        assert_eq!(normalize_core_key_for_profile("bedrock").as_deref(), Some("bds"));
        assert_eq!(canonical_server_core_type("Paper"), "paper");
        assert_eq!(canonical_server_core_type("AllayMC"), "allay");
        assert_eq!(
            normalize_core_key_for_api("Arclight-Neoforge").as_deref(),
            Some("arclight-neoforge")
        );
    }

    #[test]
    fn reuses_shared_core_normalization_before_local_bridge_aliases() {
        assert_eq!(normalize_core_key_for_profile("Waterfall").as_deref(), Some("waterfall"));
        assert_eq!(normalize_core_key_for_profile("Folia").as_deref(), Some("folia"));
        assert_eq!(normalize_core_key_for_profile("bedrock").as_deref(), Some("bds"));
        assert_eq!(
            normalize_core_key_for_profile("Arclight-Forge").as_deref(),
            Some("arclight-forge")
        );
        assert_eq!(normalize_core_key_for_profile("Leaf").as_deref(), Some("leaves"));
    }

    #[test]
    fn resolves_bedrock_and_proxy_profiles() {
        let bedrock = resolve_server_flavor(&test_server("AllayMC", "jar"));
        assert_eq!(bedrock.flavor_kind, ServerFlavorKind::BedrockLike);
        assert_eq!(bedrock.edition, ServerEdition::Bedrock);

        let proxy = resolve_server_flavor(&test_server("BungeeCord", "jar"));
        assert_eq!(proxy.flavor_kind, ServerFlavorKind::ProxyLike);
        assert_eq!(proxy.server_role, ServerRole::Proxy);
    }

    #[test]
    fn resolves_extension_directories_from_flavor() {
        let paper = test_server("Paper", "jar");
        assert_eq!(extension_dir_for_kind(&paper, ServerExtensionKind::Plugin), Some("plugins"));
        assert_eq!(extension_dir_for_kind(&paper, ServerExtensionKind::Mod), None);

        let fabric = test_server("Fabric", "jar");
        assert_eq!(extension_dir_for_kind(&fabric, ServerExtensionKind::Mod), Some("mods"));

        let bds = test_server("bedrock", "custom");
        assert_eq!(
            extension_dir_for_kind(&bds, ServerExtensionKind::Addon),
            Some("behavior_packs")
        );
    }

    #[test]
    fn exposes_stable_profile_summary() {
        let (edition, role) = profile_summary(&test_server("Paper", "jar"));
        assert_eq!(edition, ServerEdition::Java);
        assert_eq!(role, ServerRole::GameServer);
    }

    #[test]
    fn normalize_user_core_type_rejects_blank_and_canonicalizes_aliases() {
        assert!(normalize_user_core_type("  ").is_err());
        assert_eq!(normalize_user_core_type("Leaf").as_deref(), Ok("leaves"));
        assert_eq!(normalize_user_core_type("bedrock").as_deref(), Ok("bds"));
    }

    #[test]
    fn canonical_detected_core_type_normalizes_detection_aliases() {
        assert_eq!(canonical_detected_core_type("nukkitx"), "nukkit");
        assert_eq!(canonical_detected_core_type("spongeforge"), "forge");
    }

    #[test]
    fn shared_taxonomy_metadata_stays_aligned_with_backend_flavor_contract() {
        let taxonomy: SharedServerCoreTaxonomyDocument =
            serde_json::from_str(include_str!("../../../../../shared/server-core-taxonomy.json"))
                .expect("shared server core taxonomy should be valid json");

        for entry in taxonomy.entries {
            assert_eq!(canonical_server_core_type(&entry.key), entry.key, "canonical key drift");

            let plugin_supported = extension_dir_for_kind(
                &test_server(&entry.key, "jar"),
                ServerExtensionKind::Plugin,
            )
            .is_some();
            assert_eq!(
                plugin_supported, entry.supports_plugin_extensions,
                "plugin support drift for {}",
                entry.key
            );

            for alias in entry.aliases {
                let normalized = normalize_core_key_for_profile(&alias.value)
                    .unwrap_or_else(|| canonical_server_core_type(&alias.value));
                assert_eq!(normalized, entry.key, "alias drift for {}", alias.value);
            }
        }
    }
}
