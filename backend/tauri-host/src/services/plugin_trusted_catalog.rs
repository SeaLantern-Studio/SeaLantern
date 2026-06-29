use crate::hardcode_data::app_files::PLUGIN_INSTALL_METADATA_FILE_NAME;
use crate::models::plugin::{
    PluginDistributionClass, PluginEnableBlockReason, PluginEnableGrantScope,
    PluginEnableResult, PluginExecutionClass, PluginInfo, PluginIntegrityStatus, PluginManifest,
    PluginPermissionProfile, PluginReviewStatus, PluginTrustLevelDisplay,
    PluginTrustedPolicySource,
};
use crate::plugins::runtime::permissions::normalize_permissions;
use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

const TRUSTED_CATALOG_JSON: &str = include_str!("../../../../shared/plugin-trusted-catalog.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedCatalogSnapshot {
    pub version: u32,
    pub catalog_id: String,
    pub generated_at: String,
    pub expires_at: String,
    pub issuer: String,
    #[serde(default)]
    pub plugins: Vec<TrustedCatalogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedCatalogEntry {
    pub plugin_id: String,
    pub version: String,
    pub archive_sha256: String,
    pub publisher_id: String,
    pub review_class: String,
    pub permission_profile: String,
    #[serde(default)]
    pub permission_ceiling: Vec<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub reviewed_at: Option<String>,
    #[serde(default)]
    pub revoked: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginInstallMetadata {
    #[serde(default)]
    pub distribution_class: Option<PluginDistributionClass>,
    #[serde(default)]
    pub archive_sha256: Option<String>,
    #[serde(default)]
    pub installed_tree_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistedPluginEnableGrant {
    pub plugin_id: String,
    pub version: String,
    pub hash: Option<String>,
    pub permissions_fingerprint: String,
    pub grant_scope: PluginEnableGrantScope,
    pub granted_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct PluginTrustAssessment {
    pub trust_level_display: PluginTrustLevelDisplay,
    pub execution_class: PluginExecutionClass,
    pub review_status: PluginReviewStatus,
    pub integrity_status: PluginIntegrityStatus,
    pub trusted_policy_source: PluginTrustedPolicySource,
    pub permission_profile: PluginPermissionProfile,
    pub publisher_id: Option<String>,
    pub trusted_catalog_matched: bool,
    pub hash_matched: bool,
    pub verified_hash: Option<String>,
    pub verified_signature: bool,
    pub reviewed_at: Option<String>,
    pub revoked: bool,
    pub exceeds_standard_sandbox: bool,
    pub requires_explicit_consent: bool,
}

static TRUSTED_CATALOG: Lazy<TrustedCatalogSnapshot> = Lazy::new(|| {
    serde_json::from_str(TRUSTED_CATALOG_JSON)
        .expect("shared/plugin-trusted-catalog.json must stay valid")
});

pub fn bundled_snapshot() -> &'static TrustedCatalogSnapshot {
    &TRUSTED_CATALOG
}

pub fn install_metadata_path(plugin_dir: &Path) -> std::path::PathBuf {
    plugin_dir.join(PLUGIN_INSTALL_METADATA_FILE_NAME)
}

pub fn read_install_metadata(plugin_dir: &Path) -> Option<PluginInstallMetadata> {
    let path = install_metadata_path(plugin_dir);
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn write_install_metadata(
    plugin_dir: &Path,
    metadata: &PluginInstallMetadata,
) -> Result<(), String> {
    let path = install_metadata_path(plugin_dir);
    let json = serde_json::to_string_pretty(metadata)
        .map_err(|error| format!("Failed to serialize plugin install metadata: {}", error))?;
    std::fs::write(&path, json).map_err(|error| {
        format!(
            "Failed to write plugin install metadata '{}': {}",
            path.display(),
            error
        )
    })
}

fn should_ignore_tree_hash_entry(relative_path: &str) -> bool {
    matches!(
        relative_path,
        PLUGIN_INSTALL_METADATA_FILE_NAME | "settings.json"
    )
}

pub fn compute_plugin_tree_sha256(plugin_dir: &Path) -> Result<String, String> {
    fn collect_files(
        root: &Path,
        current: &Path,
        entries: &mut Vec<(String, Vec<u8>)>,
    ) -> Result<(), String> {
        let mut children = std::fs::read_dir(current)
            .map_err(|error| format!("Failed to read plugin directory '{}': {}", current.display(), error))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Failed to read plugin directory entry: {}", error))?;
        children.sort_by_key(|entry| entry.file_name());

        for entry in children {
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
                format!("Failed to inspect plugin entry '{}': {}", path.display(), error)
            })?;
            if metadata.file_type().is_symlink() {
                return Err(format!("Plugin entry '{}' must not be a symlink", path.display()));
            }

            let relative = path
                .strip_prefix(root)
                .map_err(|error| format!("Failed to normalize plugin entry '{}': {}", path.display(), error))?
                .to_string_lossy()
                .replace('\\', "/");

            if metadata.is_dir() {
                collect_files(root, &path, entries)?;
                continue;
            }

            if should_ignore_tree_hash_entry(&relative) {
                continue;
            }

            let content = std::fs::read(&path)
                .map_err(|error| format!("Failed to read plugin entry '{}': {}", path.display(), error))?;
            entries.push((relative, content));
        }

        Ok(())
    }

    let mut entries = Vec::new();
    collect_files(plugin_dir, plugin_dir, &mut entries)?;

    let mut hasher = Sha256::new();
    for (relative, content) in entries {
        hasher.update(relative.as_bytes());
        hasher.update([0u8]);
        hasher.update((content.len() as u64).to_le_bytes());
        hasher.update(content);
        hasher.update([0xff]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn apply_runtime_integrity_state(
    mut plugin: PluginInfo,
    plugin_dir: &Path,
    metadata: &PluginInstallMetadata,
) -> Result<PluginInfo, String> {
    let Some(expected_tree_hash) = metadata.installed_tree_sha256.as_deref() else {
        return Ok(plugin);
    };

    if !(plugin.hash_matched || matches!(plugin.trust_level_display, PluginTrustLevelDisplay::Trusted)) {
        return Ok(plugin);
    }

    let current_tree_hash = compute_plugin_tree_sha256(plugin_dir)?;
    if current_tree_hash.eq_ignore_ascii_case(expected_tree_hash) {
        return Ok(plugin);
    }

    plugin.trust_level_display = PluginTrustLevelDisplay::Unreviewed;
    plugin.execution_class = PluginExecutionClass::Sandboxed;
    if !plugin.revoked {
        plugin.review_status = PluginReviewStatus::Unreviewed;
    }
    plugin.integrity_status = PluginIntegrityStatus::Mismatch;
    plugin.permission_profile = PluginPermissionProfile::Unreviewed;
    plugin.hash_matched = false;
    plugin.verified_hash = None;
    plugin.requires_explicit_consent = true;
    Ok(plugin)
}

pub fn enable_grants_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("plugin_enable_grants.json")
}

pub fn load_enable_grants(data_dir: &Path) -> Result<Vec<PersistedPluginEnableGrant>, String> {
    let path = enable_grants_path(data_dir);
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(format!(
                "Failed to read plugin enable grants '{}': {}",
                path.display(),
                error
            ));
        }
    };

    serde_json::from_str(&content).map_err(|error| {
        format!(
            "Failed to parse plugin enable grants '{}': {}",
            path.display(),
            error
        )
    })
}

pub fn save_enable_grants(
    data_dir: &Path,
    grants: &[PersistedPluginEnableGrant],
) -> Result<(), String> {
    let path = enable_grants_path(data_dir);
    let content = serde_json::to_string_pretty(grants)
        .map_err(|error| format!("Failed to serialize plugin enable grants: {}", error))?;
    std::fs::write(&path, content).map_err(|error| {
        format!(
            "Failed to write plugin enable grants '{}': {}",
            path.display(),
            error
        )
    })
}

pub fn permissions_fingerprint(permissions: &[String]) -> String {
    normalize_permissions(permissions.to_vec()).join("|")
}

pub fn required_grant_scope(plugin: &PluginInfo) -> Option<PluginEnableGrantScope> {
    match plugin.trust_level_display {
        PluginTrustLevelDisplay::Builtin => None,
        PluginTrustLevelDisplay::Trusted => Some(PluginEnableGrantScope::Hash),
        PluginTrustLevelDisplay::StandardSandbox => {
            if plugin.requires_explicit_consent {
                Some(PluginEnableGrantScope::Version)
            } else {
                None
            }
        }
        PluginTrustLevelDisplay::Unreviewed => Some(PluginEnableGrantScope::Version),
    }
}

pub fn grant_scope_covers(
    provided: PluginEnableGrantScope,
    required: PluginEnableGrantScope,
) -> bool {
    matches!(
        (provided, required),
        (PluginEnableGrantScope::Once, PluginEnableGrantScope::Once)
            | (PluginEnableGrantScope::Version, PluginEnableGrantScope::Version)
            | (PluginEnableGrantScope::Hash, PluginEnableGrantScope::Version)
            | (PluginEnableGrantScope::Hash, PluginEnableGrantScope::Hash)
    )
}

pub fn evaluate_enable_requirement(
    plugin: &PluginInfo,
    grants: &[PersistedPluginEnableGrant],
) -> PluginEnableResult {
    if plugin.revoked || matches!(plugin.review_status, PluginReviewStatus::Revoked) {
        return PluginEnableResult {
            success: false,
            disabled_plugins: Vec::new(),
            confirmation_required: true,
            block_reason: Some(PluginEnableBlockReason::Revoked),
            plugin: Some(plugin.clone()),
            grant_scope: Some(PluginEnableGrantScope::Version),
            message: Some("Plugin was revoked from its previous trusted policy state".to_string()),
        };
    }

    let Some(required_scope) = required_grant_scope(plugin) else {
        return PluginEnableResult {
            success: true,
            disabled_plugins: Vec::new(),
            confirmation_required: false,
            block_reason: None,
            plugin: Some(plugin.clone()),
            grant_scope: None,
            message: None,
        };
    };

    let expected_fingerprint = permissions_fingerprint(&plugin.manifest.permissions);
    let grant = grants.iter().find(|grant| grant.plugin_id == plugin.manifest.id);
    let satisfied = match grant {
        Some(grant) if grant.permissions_fingerprint == expected_fingerprint => {
            match required_scope {
                PluginEnableGrantScope::Once => false,
                PluginEnableGrantScope::Version => {
                    grant_scope_covers(
                        grant.grant_scope.clone(),
                        PluginEnableGrantScope::Version,
                    ) && grant.version == plugin.manifest.version
                }
                PluginEnableGrantScope::Hash => {
                    matches!(grant.grant_scope, PluginEnableGrantScope::Hash)
                        && grant.version == plugin.manifest.version
                        && grant.hash == plugin.verified_hash
                        && plugin.verified_hash.is_some()
                }
            }
        }
        _ => false,
    };

    if satisfied {
        return PluginEnableResult {
            success: true,
            disabled_plugins: Vec::new(),
            confirmation_required: false,
            block_reason: None,
            plugin: Some(plugin.clone()),
            grant_scope: Some(required_scope),
            message: None,
        };
    }

    PluginEnableResult {
        success: false,
        disabled_plugins: Vec::new(),
        confirmation_required: true,
        block_reason: Some(PluginEnableBlockReason::UserConfirmationRequired),
        plugin: Some(plugin.clone()),
        grant_scope: Some(required_scope),
        message: None,
    }
}

pub fn upsert_enable_grant(
    data_dir: &Path,
    plugin: &PluginInfo,
    grant_scope: PluginEnableGrantScope,
) -> Result<(), String> {
    let mut grants = load_enable_grants(data_dir)?;
    grants.retain(|grant| grant.plugin_id != plugin.manifest.id);
    grants.push(PersistedPluginEnableGrant {
        plugin_id: plugin.manifest.id.clone(),
        version: plugin.manifest.version.clone(),
        hash: plugin.verified_hash.clone(),
        permissions_fingerprint: permissions_fingerprint(&plugin.manifest.permissions),
        grant_scope,
        granted_at: chrono::Utc::now().to_rfc3339(),
    });
    save_enable_grants(data_dir, &grants)
}

pub fn assess_plugin(
    manifest: &PluginManifest,
    distribution_class: PluginDistributionClass,
    archive_sha256: Option<&str>,
) -> PluginTrustAssessment {
    let normalized_permissions = normalize_permissions(manifest.permissions.clone());
    let exceeds_standard_sandbox =
        crate::hardcode_data::plugin_permissions::exceeds_standard_sandbox_ceiling(
            &normalized_permissions,
        );
    let requests_trusted_capabilities =
        crate::hardcode_data::plugin_permissions::requests_trusted_capabilities(
            &normalized_permissions,
        );
    let requires_explicit_consent =
        crate::hardcode_data::plugin_permissions::requires_explicit_consent(
            &normalized_permissions,
        );
    let is_standard_distribution = matches!(
        distribution_class,
        PluginDistributionClass::Market | PluginDistributionClass::StandardPackage
    );

    let base_permission_profile = if requests_trusted_capabilities || exceeds_standard_sandbox {
        PluginPermissionProfile::Unreviewed
    } else if requires_explicit_consent {
        PluginPermissionProfile::SandboxedExtended
    } else {
        PluginPermissionProfile::SandboxedNormal
    };

    let mut assessment = PluginTrustAssessment {
        trust_level_display: if is_standard_distribution
            && !exceeds_standard_sandbox
            && !requests_trusted_capabilities
        {
            PluginTrustLevelDisplay::StandardSandbox
        } else {
            PluginTrustLevelDisplay::Unreviewed
        },
        execution_class: PluginExecutionClass::Sandboxed,
        review_status: PluginReviewStatus::Unreviewed,
        integrity_status: if archive_sha256.is_some() {
            PluginIntegrityStatus::Unsigned
        } else {
            PluginIntegrityStatus::Unknown
        },
        trusted_policy_source: PluginTrustedPolicySource::None,
        permission_profile: base_permission_profile,
        publisher_id: None,
        trusted_catalog_matched: false,
        hash_matched: false,
        verified_hash: None,
        verified_signature: false,
        reviewed_at: None,
        revoked: false,
        exceeds_standard_sandbox,
        requires_explicit_consent,
    };

    if !requests_trusted_capabilities {
        return assessment;
    }

    assessment.trust_level_display = PluginTrustLevelDisplay::Unreviewed;
    assessment.execution_class = PluginExecutionClass::Sandboxed;
    assessment.review_status = PluginReviewStatus::Unreviewed;
    assessment.permission_profile = PluginPermissionProfile::Unreviewed;
    assessment.trusted_policy_source = PluginTrustedPolicySource::BundledSnapshot;

    let Some(archive_sha256) = archive_sha256 else {
        assessment.integrity_status = PluginIntegrityStatus::Unknown;
        return assessment;
    };

    let entry = bundled_snapshot()
        .plugins
        .iter()
        .find(|entry| entry.plugin_id == manifest.id && entry.version == manifest.version);

    let Some(entry) = entry else {
        assessment.integrity_status = PluginIntegrityStatus::Mismatch;
        return assessment;
    };

    assessment.publisher_id = Some(entry.publisher_id.clone());
    assessment.reviewed_at = entry.reviewed_at.clone();
    assessment.trusted_catalog_matched = true;

    if entry.revoked || entry.review_class == "revoked" {
        assessment.review_status = PluginReviewStatus::Revoked;
        assessment.integrity_status = PluginIntegrityStatus::Mismatch;
        assessment.revoked = true;
        return assessment;
    }

    if !entry.archive_sha256.eq_ignore_ascii_case(archive_sha256) {
        assessment.integrity_status = PluginIntegrityStatus::Mismatch;
        return assessment;
    }

    let allowed_permissions: HashSet<String> = entry
        .permission_ceiling
        .iter()
        .map(|permission| crate::hardcode_data::plugin_permissions::normalize_permission_id(permission))
        .collect();
    let within_ceiling = normalized_permissions
        .iter()
        .all(|permission| allowed_permissions.contains(permission));

    if !within_ceiling {
        assessment.integrity_status = PluginIntegrityStatus::Mismatch;
        return assessment;
    }

    assessment.trust_level_display = PluginTrustLevelDisplay::Trusted;
    assessment.execution_class = PluginExecutionClass::TrustedFull;
    assessment.review_status = PluginReviewStatus::SealanternReviewed;
    assessment.integrity_status = PluginIntegrityStatus::VerifiedHash;
    assessment.permission_profile = PluginPermissionProfile::TrustedFull;
    assessment.hash_matched = true;
    assessment.verified_hash = Some(entry.archive_sha256.clone());
    assessment
}

#[cfg(test)]
mod tests {
    use super::{apply_runtime_integrity_state, assess_plugin, compute_plugin_tree_sha256, grant_scope_covers, PluginInstallMetadata};
    use crate::models::plugin::{
        PluginAuthor, PluginDistributionClass, PluginExecutionClass, PluginEnableGrantScope,
        PluginInfo, PluginIntegrityStatus, PluginManifest, PluginPermissionProfile,
        PluginReviewStatus, PluginSource, PluginTrustedPolicySource, PluginTrustLevelDisplay,
        PluginRuntimeKind, PluginActions, PluginDistributionClass as DistributionClass,
    };
    use std::collections::HashMap;

    fn manifest_with_permissions(id: &str, version: &str, permissions: Vec<&str>) -> PluginManifest {
        PluginManifest {
            id: id.to_string(),
            name: id.to_string(),
            version: version.to_string(),
            description: "test".to_string(),
            author: PluginAuthor {
                name: "SeaLantern".to_string(),
                email: None,
                url: None,
            },
            main: "main.lua".to_string(),
            license: None,
            homepage: None,
            repository: None,
            engines: None,
            permissions: permissions.into_iter().map(|item| item.to_string()).collect(),
            ui: None,
            events: Vec::new(),
            commands: Vec::new(),
            programs: Vec::new(),
            dependencies: Vec::new(),
            optional_dependencies: Vec::new(),
            icon: None,
            settings: None,
            sidebar: None,
            locales: None,
            include: Vec::new(),
            capabilities: Vec::new(),
            theme_var_map: HashMap::new(),
            presets: HashMap::new(),
            server_events: HashMap::new(),
        }
    }

    #[test]
    fn standard_market_plugin_stays_standard_sandbox() {
        let manifest = manifest_with_permissions("demo.standard", "1.0.0", vec!["log", "fs.data"]);

        let assessment = assess_plugin(&manifest, PluginDistributionClass::Market, None);

        assert_eq!(assessment.trust_level_display, PluginTrustLevelDisplay::StandardSandbox);
        assert_eq!(assessment.execution_class, PluginExecutionClass::Sandboxed);
        assert_eq!(assessment.review_status, PluginReviewStatus::Unreviewed);
        assert_eq!(assessment.permission_profile, PluginPermissionProfile::SandboxedNormal);
    }

    #[test]
    fn trusted_only_permission_without_catalog_match_stays_unreviewed() {
        let manifest = manifest_with_permissions(
            "demo.trusted",
            "1.0.0",
            vec!["execute_program", "process.exec"],
        );

        let assessment = assess_plugin(
            &manifest,
            PluginDistributionClass::Market,
            Some("deadbeef"),
        );

        assert_eq!(assessment.trust_level_display, PluginTrustLevelDisplay::Unreviewed);
        assert_eq!(assessment.execution_class, PluginExecutionClass::Sandboxed);
        assert_eq!(assessment.review_status, PluginReviewStatus::Unreviewed);
        assert_eq!(assessment.integrity_status, PluginIntegrityStatus::Mismatch);
        assert_eq!(assessment.permission_profile, PluginPermissionProfile::Unreviewed);
    }

    #[test]
    fn hash_scope_covers_version_scope() {
        assert!(grant_scope_covers(
            PluginEnableGrantScope::Hash,
            PluginEnableGrantScope::Version
        ));
        assert!(grant_scope_covers(
            PluginEnableGrantScope::Version,
            PluginEnableGrantScope::Version
        ));
        assert!(!grant_scope_covers(
            PluginEnableGrantScope::Version,
            PluginEnableGrantScope::Hash
        ));
    }

    #[test]
    fn runtime_tree_hash_mismatch_downgrades_trusted_plugin() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let plugin_dir = temp_dir.path().join("trusted-plugin");
        std::fs::create_dir_all(&plugin_dir).expect("plugin dir should exist");
        std::fs::write(plugin_dir.join("main.lua"), b"print('hello')\n")
            .expect("main.lua should exist");

        let stored_tree_hash =
            compute_plugin_tree_sha256(&plugin_dir).expect("tree hash should be computed");

        std::fs::write(plugin_dir.join("main.lua"), b"print('modified')\n")
            .expect("main.lua should be rewritable");

        let plugin = PluginInfo {
            manifest: manifest_with_permissions("trusted.plugin", "1.0.0", vec!["process.exec"]),
            state: crate::models::plugin::PluginState::Loaded,
            path: plugin_dir.to_string_lossy().to_string(),
            source: PluginSource::Local,
            runtime: PluginRuntimeKind::Lua,
            actions: PluginActions {
                can_toggle: true,
                can_delete: true,
                can_check_update: true,
            },
            missing_dependencies: Vec::new(),
            trust_level_display: PluginTrustLevelDisplay::Trusted,
            execution_class: PluginExecutionClass::TrustedFull,
            review_status: PluginReviewStatus::SealanternReviewed,
            integrity_status: PluginIntegrityStatus::VerifiedHash,
            trusted_policy_source: PluginTrustedPolicySource::BundledSnapshot,
            permission_profile: PluginPermissionProfile::TrustedFull,
            publisher_id: Some("sealantern".to_string()),
            distribution_class: DistributionClass::Market,
            trusted_catalog_matched: true,
            hash_matched: true,
            verified_hash: Some("deadbeef".to_string()),
            verified_signature: false,
            reviewed_at: Some("2026-06-29T00:00:00Z".to_string()),
            revoked: false,
            exceeds_standard_sandbox: false,
            requires_explicit_consent: false,
        };

        let downgraded = apply_runtime_integrity_state(
            plugin,
            &plugin_dir,
            &PluginInstallMetadata {
                distribution_class: Some(DistributionClass::Market),
                archive_sha256: Some("deadbeef".to_string()),
                installed_tree_sha256: Some(stored_tree_hash),
            },
        )
        .expect("mismatch downgrade should succeed");

        assert_eq!(downgraded.trust_level_display, PluginTrustLevelDisplay::Unreviewed);
        assert_eq!(downgraded.execution_class, PluginExecutionClass::Sandboxed);
        assert_eq!(downgraded.review_status, PluginReviewStatus::Unreviewed);
        assert_eq!(downgraded.integrity_status, PluginIntegrityStatus::Mismatch);
        assert_eq!(downgraded.permission_profile, PluginPermissionProfile::Unreviewed);
        assert!(!downgraded.hash_matched);
        assert_eq!(downgraded.verified_hash, None);
        assert!(downgraded.requires_explicit_consent);
    }
}
