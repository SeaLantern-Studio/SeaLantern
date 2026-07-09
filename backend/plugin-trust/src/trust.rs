use crate::permissions::{
    exceeds_standard_sandbox_ceiling, normalize_permission_id, normalize_permissions,
    requests_trusted_capabilities, requires_explicit_consent,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const PLUGIN_INSTALL_METADATA_FILE_NAME: &str = ".sealantern-plugin-install.json";

const TRUSTED_CATALOG_JSON: &str = include_str!("../../../shared/plugin-trusted-catalog.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Signed or bundled snapshot describing reviewed plugin artifacts.
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
/// One trusted-catalog entry for a specific plugin version and archive hash.
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// Origin class used when assessing trust and consent rules.
pub enum PluginDistributionClass {
    Builtin,
    Market,
    StandardPackage,
    ManualImport,
    LocalDirectory,
    TrustedCatalog,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
/// Persistence scope granted when a user confirms plugin enablement.
pub enum PluginEnableGrantScope {
    Once,
    #[default]
    Version,
    Hash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
/// User-facing trust badge shown for a plugin.
pub enum PluginTrustLevelDisplay {
    Builtin,
    Trusted,
    #[default]
    StandardSandbox,
    Unreviewed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
/// Runtime execution class derived from trust assessment.
pub enum PluginExecutionClass {
    BuiltinFull,
    TrustedFull,
    #[default]
    Sandboxed,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
/// Review status assigned by SeaLantern trust policy.
pub enum PluginReviewStatus {
    Builtin,
    SealanternReviewed,
    #[default]
    Unreviewed,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
/// Integrity status derived from hashes or bundled provenance.
pub enum PluginIntegrityStatus {
    Bundled,
    VerifiedHash,
    VerifiedSignature,
    Unsigned,
    Mismatch,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
/// Source of the policy decision that upgraded a plugin's trust state.
pub enum PluginTrustedPolicySource {
    Builtin,
    BundledSnapshot,
    RemoteSignedCatalog,
    LocalAttestation,
    #[default]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
/// Permission profile inferred from requested capabilities and trust policy.
pub enum PluginPermissionProfile {
    BuiltinFull,
    TrustedFull,
    #[default]
    SandboxedNormal,
    SandboxedExtended,
    Unreviewed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// Reason why a plugin cannot be enabled without additional user action.
pub enum PluginEnableBlockReason {
    UserConfirmationRequired,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Minimal manifest fields needed for trust assessment.
pub struct PluginManifestInput {
    pub id: String,
    pub version: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Metadata persisted alongside an installed plugin directory.
pub struct PluginInstallMetadata {
    #[serde(default)]
    pub distribution_class: Option<PluginDistributionClass>,
    #[serde(default)]
    pub archive_sha256: Option<String>,
    #[serde(default)]
    pub installed_tree_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Persisted record of a user's approval to enable a plugin.
pub struct PersistedPluginEnableGrant {
    pub plugin_id: String,
    pub version: String,
    pub hash: Option<String>,
    pub permissions_fingerprint: String,
    pub grant_scope: PluginEnableGrantScope,
    pub granted_at: String,
}

#[derive(Debug, Clone, Default)]
/// Full trust-assessment result produced during installation or refresh.
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

#[derive(Debug, Clone)]
/// Runtime trust state after installation metadata and integrity checks are applied.
pub struct PluginRuntimeTrustState {
    pub trust_level_display: PluginTrustLevelDisplay,
    pub execution_class: PluginExecutionClass,
    pub review_status: PluginReviewStatus,
    pub integrity_status: PluginIntegrityStatus,
    pub permission_profile: PluginPermissionProfile,
    pub hash_matched: bool,
    pub verified_hash: Option<String>,
    pub revoked: bool,
    pub requires_explicit_consent: bool,
}

#[derive(Debug, Clone)]
/// Input required to decide whether plugin enablement needs user confirmation.
pub struct PluginEnableContext {
    pub plugin_id: String,
    pub version: String,
    pub permissions: Vec<String>,
    pub trust_level_display: PluginTrustLevelDisplay,
    pub review_status: PluginReviewStatus,
    pub revoked: bool,
    pub verified_hash: Option<String>,
    pub requires_explicit_consent: bool,
}

#[derive(Debug, Clone, Default)]
/// Outcome of evaluating whether a plugin may be enabled immediately.
pub struct PluginEnableRequirement {
    pub success: bool,
    pub confirmation_required: bool,
    pub block_reason: Option<PluginEnableBlockReason>,
    pub grant_scope: Option<PluginEnableGrantScope>,
    pub message: Option<String>,
}

static TRUSTED_CATALOG: Lazy<TrustedCatalogSnapshot> = Lazy::new(|| {
    serde_json::from_str(TRUSTED_CATALOG_JSON)
        .expect("shared/plugin-trusted-catalog.json must stay valid")
});

/// Returns the bundled trusted-catalog snapshot shipped with the application.
pub fn bundled_snapshot() -> &'static TrustedCatalogSnapshot {
    &TRUSTED_CATALOG
}

/// Returns the install metadata path inside a plugin directory.
pub fn install_metadata_path(plugin_dir: &Path) -> PathBuf {
    plugin_dir.join(PLUGIN_INSTALL_METADATA_FILE_NAME)
}

/// Reads persisted install metadata if it exists and is valid JSON.
pub fn read_install_metadata(plugin_dir: &Path) -> Option<PluginInstallMetadata> {
    let path = install_metadata_path(plugin_dir);
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Writes install metadata next to the plugin directory contents.
pub fn write_install_metadata(
    plugin_dir: &Path,
    metadata: &PluginInstallMetadata,
) -> Result<(), String> {
    let path = install_metadata_path(plugin_dir);
    let json = serde_json::to_string_pretty(metadata)
        .map_err(|error| format!("Failed to serialize plugin install metadata: {}", error))?;
    std::fs::write(&path, json).map_err(|error| {
        format!("Failed to write plugin install metadata '{}': {}", path.display(), error)
    })
}

/// Computes a stable tree hash for all plugin files that participate in integrity checks.
pub fn compute_plugin_tree_sha256(plugin_dir: &Path) -> Result<String, String> {
    fn collect_files(
        root: &Path,
        current: &Path,
        entries: &mut Vec<(String, Vec<u8>)>,
    ) -> Result<(), String> {
        let mut children = std::fs::read_dir(current)
            .map_err(|error| {
                format!("Failed to read plugin directory '{}': {}", current.display(), error)
            })?
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
                .map_err(|error| {
                    format!("Failed to normalize plugin entry '{}': {}", path.display(), error)
                })?
                .to_string_lossy()
                .replace('\\', "/");

            if metadata.is_dir() {
                collect_files(root, &path, entries)?;
                continue;
            }

            if should_ignore_tree_hash_entry(&relative) {
                continue;
            }

            let content = std::fs::read(&path).map_err(|error| {
                format!("Failed to read plugin entry '{}': {}", path.display(), error)
            })?;
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

/// Downgrades runtime trust when the installed plugin tree no longer matches the recorded hash.
pub fn apply_runtime_integrity_state(
    mut state: PluginRuntimeTrustState,
    plugin_dir: &Path,
    metadata: &PluginInstallMetadata,
) -> Result<PluginRuntimeTrustState, String> {
    let Some(expected_tree_hash) = metadata.installed_tree_sha256.as_deref() else {
        return Ok(state);
    };

    if !(state.hash_matched
        || matches!(state.trust_level_display, PluginTrustLevelDisplay::Trusted))
    {
        return Ok(state);
    }

    let current_tree_hash = compute_plugin_tree_sha256(plugin_dir)?;
    if current_tree_hash.eq_ignore_ascii_case(expected_tree_hash) {
        return Ok(state);
    }

    state.trust_level_display = PluginTrustLevelDisplay::Unreviewed;
    state.execution_class = PluginExecutionClass::Sandboxed;
    if !state.revoked {
        state.review_status = PluginReviewStatus::Unreviewed;
    }
    state.integrity_status = PluginIntegrityStatus::Mismatch;
    state.permission_profile = PluginPermissionProfile::Unreviewed;
    state.hash_matched = false;
    state.verified_hash = None;
    state.requires_explicit_consent = true;
    Ok(state)
}

/// Returns the persisted enable-grant file path under the application data directory.
pub fn enable_grants_path(data_dir: &Path) -> PathBuf {
    data_dir.join("plugin_enable_grants.json")
}

/// Loads persisted plugin enable grants.
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
        format!("Failed to parse plugin enable grants '{}': {}", path.display(), error)
    })
}

/// Saves persisted plugin enable grants.
pub fn save_enable_grants(
    data_dir: &Path,
    grants: &[PersistedPluginEnableGrant],
) -> Result<(), String> {
    let path = enable_grants_path(data_dir);
    let content = serde_json::to_string_pretty(grants)
        .map_err(|error| format!("Failed to serialize plugin enable grants: {}", error))?;
    std::fs::write(&path, content).map_err(|error| {
        format!("Failed to write plugin enable grants '{}': {}", path.display(), error)
    })
}

/// Builds a stable fingerprint for a permission set after normalization.
pub fn permissions_fingerprint(permissions: &[String]) -> String {
    normalize_permissions(permissions.to_vec()).join("|")
}

/// Returns the consent scope required to enable a plugin with the given trust state.
pub fn required_grant_scope(plugin: &PluginEnableContext) -> Option<PluginEnableGrantScope> {
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

/// Returns whether a persisted grant scope satisfies the currently required scope.
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

/// Evaluates whether a plugin can be enabled immediately or needs renewed consent.
pub fn evaluate_enable_requirement(
    plugin: &PluginEnableContext,
    grants: &[PersistedPluginEnableGrant],
) -> PluginEnableRequirement {
    if plugin.revoked || matches!(plugin.review_status, PluginReviewStatus::Revoked) {
        return PluginEnableRequirement {
            success: false,
            confirmation_required: true,
            block_reason: Some(PluginEnableBlockReason::Revoked),
            grant_scope: Some(PluginEnableGrantScope::Version),
            message: Some("Plugin was revoked from its previous trusted policy state".to_string()),
        };
    }

    let Some(required_scope) = required_grant_scope(plugin) else {
        return PluginEnableRequirement {
            success: true,
            confirmation_required: false,
            block_reason: None,
            grant_scope: None,
            message: None,
        };
    };

    let expected_fingerprint = permissions_fingerprint(&plugin.permissions);
    let grant = grants
        .iter()
        .find(|grant| grant.plugin_id == plugin.plugin_id);
    let satisfied = match grant {
        Some(grant) if grant.permissions_fingerprint == expected_fingerprint => {
            match required_scope {
                PluginEnableGrantScope::Once => false,
                PluginEnableGrantScope::Version => {
                    grant_scope_covers(grant.grant_scope.clone(), PluginEnableGrantScope::Version)
                        && grant.version == plugin.version
                }
                PluginEnableGrantScope::Hash => {
                    matches!(grant.grant_scope, PluginEnableGrantScope::Hash)
                        && grant.version == plugin.version
                        && grant.hash == plugin.verified_hash
                        && plugin.verified_hash.is_some()
                }
            }
        }
        _ => false,
    };

    if satisfied {
        return PluginEnableRequirement {
            success: true,
            confirmation_required: false,
            block_reason: None,
            grant_scope: Some(required_scope),
            message: None,
        };
    }

    PluginEnableRequirement {
        success: false,
        confirmation_required: true,
        block_reason: Some(PluginEnableBlockReason::UserConfirmationRequired),
        grant_scope: Some(required_scope),
        message: None,
    }
}

/// Replaces the persisted enable grant for a plugin.
pub fn upsert_enable_grant(
    data_dir: &Path,
    plugin: &PluginEnableContext,
    grant_scope: PluginEnableGrantScope,
) -> Result<(), String> {
    let mut grants = load_enable_grants(data_dir)?;
    grants.retain(|grant| grant.plugin_id != plugin.plugin_id);
    grants.push(PersistedPluginEnableGrant {
        plugin_id: plugin.plugin_id.clone(),
        version: plugin.version.clone(),
        hash: plugin.verified_hash.clone(),
        permissions_fingerprint: permissions_fingerprint(&plugin.permissions),
        grant_scope,
        granted_at: chrono::Utc::now().to_rfc3339(),
    });
    save_enable_grants(data_dir, &grants)
}

/// Assesses trust, integrity, and consent requirements for a plugin manifest.
pub fn assess_plugin(
    manifest: &PluginManifestInput,
    distribution_class: PluginDistributionClass,
    archive_sha256: Option<&str>,
) -> PluginTrustAssessment {
    let normalized_permissions = normalize_permissions(manifest.permissions.clone());
    let exceeds_standard_sandbox = exceeds_standard_sandbox_ceiling(&normalized_permissions);
    let requests_trusted = requests_trusted_capabilities(&normalized_permissions);
    let requires_consent = requires_explicit_consent(&normalized_permissions);
    let is_standard_distribution = matches!(
        distribution_class,
        PluginDistributionClass::Market | PluginDistributionClass::StandardPackage
    );

    let base_permission_profile = if requests_trusted || exceeds_standard_sandbox {
        PluginPermissionProfile::Unreviewed
    } else if requires_consent {
        PluginPermissionProfile::SandboxedExtended
    } else {
        PluginPermissionProfile::SandboxedNormal
    };

    let mut assessment = PluginTrustAssessment {
        trust_level_display: if is_standard_distribution
            && !exceeds_standard_sandbox
            && !requests_trusted
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
        requires_explicit_consent: requires_consent,
    };

    if !requests_trusted {
        return assessment;
    }

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
        .map(|permission| normalize_permission_id(permission))
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

fn should_ignore_tree_hash_entry(relative_path: &str) -> bool {
    matches!(relative_path, PLUGIN_INSTALL_METADATA_FILE_NAME | "settings.json")
}

#[cfg(test)]
mod tests {
    use super::{
        apply_runtime_integrity_state, assess_plugin, compute_plugin_tree_sha256,
        evaluate_enable_requirement, grant_scope_covers, PluginDistributionClass,
        PluginEnableContext, PluginEnableGrantScope, PluginExecutionClass, PluginInstallMetadata,
        PluginIntegrityStatus, PluginManifestInput, PluginPermissionProfile, PluginReviewStatus,
        PluginRuntimeTrustState, PluginTrustLevelDisplay,
    };

    fn manifest_with_permissions(
        id: &str,
        version: &str,
        permissions: Vec<&str>,
    ) -> PluginManifestInput {
        PluginManifestInput {
            id: id.to_string(),
            version: version.to_string(),
            permissions: permissions.into_iter().map(str::to_string).collect(),
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
        let assessment =
            assess_plugin(&manifest, PluginDistributionClass::Market, Some("deadbeef"));

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

        let downgraded = apply_runtime_integrity_state(
            PluginRuntimeTrustState {
                trust_level_display: PluginTrustLevelDisplay::Trusted,
                execution_class: PluginExecutionClass::TrustedFull,
                review_status: PluginReviewStatus::SealanternReviewed,
                integrity_status: PluginIntegrityStatus::VerifiedHash,
                permission_profile: PluginPermissionProfile::TrustedFull,
                hash_matched: true,
                verified_hash: Some("deadbeef".to_string()),
                revoked: false,
                requires_explicit_consent: false,
            },
            &plugin_dir,
            &PluginInstallMetadata {
                distribution_class: Some(PluginDistributionClass::Market),
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

    #[test]
    fn trusted_plugins_require_hash_scope() {
        let context = PluginEnableContext {
            plugin_id: "trusted.plugin".to_string(),
            version: "1.0.0".to_string(),
            permissions: vec!["process.exec".to_string()],
            trust_level_display: PluginTrustLevelDisplay::Trusted,
            review_status: PluginReviewStatus::SealanternReviewed,
            revoked: false,
            verified_hash: Some("abc123".to_string()),
            requires_explicit_consent: false,
        };

        let requirement = evaluate_enable_requirement(&context, &[]);
        assert!(!requirement.success);
        assert!(requirement.confirmation_required);
        assert_eq!(requirement.grant_scope, Some(PluginEnableGrantScope::Hash));
    }
}
