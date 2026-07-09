use crate::models::plugin::{
    PluginDistributionClass, PluginEnableBlockReason, PluginEnableGrantScope, PluginEnableResult,
    PluginExecutionClass, PluginInfo, PluginIntegrityStatus, PluginManifest,
    PluginPermissionProfile, PluginReviewStatus, PluginTrustLevelDisplay,
    PluginTrustedPolicySource,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[allow(unused_imports)]
pub use plugin_trust::{TrustedCatalogEntry, TrustedCatalogSnapshot};

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

#[allow(dead_code)]
pub fn bundled_snapshot() -> &'static TrustedCatalogSnapshot {
    plugin_trust::bundled_snapshot()
}

#[allow(dead_code)]
pub fn install_metadata_path(plugin_dir: &Path) -> std::path::PathBuf {
    plugin_trust::install_metadata_path(plugin_dir)
}

pub fn read_install_metadata(plugin_dir: &Path) -> Option<PluginInstallMetadata> {
    plugin_trust::read_install_metadata(plugin_dir).map(from_core_install_metadata)
}

pub fn write_install_metadata(
    plugin_dir: &Path,
    metadata: &PluginInstallMetadata,
) -> Result<(), String> {
    plugin_trust::write_install_metadata(plugin_dir, &to_core_install_metadata(metadata.clone()))
}

pub fn compute_plugin_tree_sha256(plugin_dir: &Path) -> Result<String, String> {
    plugin_trust::compute_plugin_tree_sha256(plugin_dir)
}

pub fn apply_runtime_integrity_state(
    mut plugin: PluginInfo,
    plugin_dir: &Path,
    metadata: &PluginInstallMetadata,
) -> Result<PluginInfo, String> {
    let state = plugin_trust::PluginRuntimeTrustState {
        trust_level_display: to_core_trust_level(plugin.trust_level_display.clone()),
        execution_class: to_core_execution_class(plugin.execution_class.clone()),
        review_status: to_core_review_status(plugin.review_status.clone()),
        integrity_status: to_core_integrity_status(plugin.integrity_status.clone()),
        permission_profile: to_core_permission_profile(plugin.permission_profile.clone()),
        hash_matched: plugin.hash_matched,
        verified_hash: plugin.verified_hash.clone(),
        revoked: plugin.revoked,
        requires_explicit_consent: plugin.requires_explicit_consent,
    };

    let state = plugin_trust::apply_runtime_integrity_state(
        state,
        plugin_dir,
        &to_core_install_metadata(metadata.clone()),
    )?;

    plugin.trust_level_display = from_core_trust_level(state.trust_level_display);
    plugin.execution_class = from_core_execution_class(state.execution_class);
    plugin.review_status = from_core_review_status(state.review_status);
    plugin.integrity_status = from_core_integrity_status(state.integrity_status);
    plugin.permission_profile = from_core_permission_profile(state.permission_profile);
    plugin.hash_matched = state.hash_matched;
    plugin.verified_hash = state.verified_hash;
    plugin.requires_explicit_consent = state.requires_explicit_consent;
    Ok(plugin)
}

#[allow(dead_code)]
pub fn enable_grants_path(data_dir: &Path) -> std::path::PathBuf {
    plugin_trust::enable_grants_path(data_dir)
}

pub fn load_enable_grants(data_dir: &Path) -> Result<Vec<PersistedPluginEnableGrant>, String> {
    plugin_trust::load_enable_grants(data_dir)
        .map(|grants| grants.into_iter().map(from_core_grant).collect())
}

#[allow(dead_code)]
pub fn save_enable_grants(
    data_dir: &Path,
    grants: &[PersistedPluginEnableGrant],
) -> Result<(), String> {
    let grants = grants
        .iter()
        .cloned()
        .map(to_core_grant)
        .collect::<Vec<_>>();
    plugin_trust::save_enable_grants(data_dir, &grants)
}

#[allow(dead_code)]
pub fn permissions_fingerprint(permissions: &[String]) -> String {
    plugin_trust::permissions_fingerprint(permissions)
}

#[allow(dead_code)]
pub fn required_grant_scope(plugin: &PluginInfo) -> Option<PluginEnableGrantScope> {
    plugin_trust::required_grant_scope(&to_core_enable_context(plugin)).map(from_core_grant_scope)
}

pub fn grant_scope_covers(
    provided: PluginEnableGrantScope,
    required: PluginEnableGrantScope,
) -> bool {
    plugin_trust::grant_scope_covers(to_core_grant_scope(provided), to_core_grant_scope(required))
}

pub fn evaluate_enable_requirement(
    plugin: &PluginInfo,
    grants: &[PersistedPluginEnableGrant],
) -> PluginEnableResult {
    let core_grants = grants
        .iter()
        .cloned()
        .map(to_core_grant)
        .collect::<Vec<_>>();
    let result =
        plugin_trust::evaluate_enable_requirement(&to_core_enable_context(plugin), &core_grants);

    PluginEnableResult {
        success: result.success,
        disabled_plugins: Vec::new(),
        confirmation_required: result.confirmation_required,
        block_reason: result.block_reason.map(from_core_block_reason),
        plugin: Some(plugin.clone()),
        grant_scope: result.grant_scope.map(from_core_grant_scope),
        message: result.message,
    }
}

pub fn upsert_enable_grant(
    data_dir: &Path,
    plugin: &PluginInfo,
    grant_scope: PluginEnableGrantScope,
) -> Result<(), String> {
    plugin_trust::upsert_enable_grant(
        data_dir,
        &to_core_enable_context(plugin),
        to_core_grant_scope(grant_scope),
    )
}

pub fn assess_plugin(
    manifest: &PluginManifest,
    distribution_class: PluginDistributionClass,
    archive_sha256: Option<&str>,
) -> PluginTrustAssessment {
    let assessment = plugin_trust::assess_plugin(
        &plugin_trust::PluginManifestInput {
            id: manifest.id.clone(),
            version: manifest.version.clone(),
            permissions: manifest.permissions.clone(),
        },
        to_core_distribution_class(distribution_class),
        archive_sha256,
    );

    PluginTrustAssessment {
        trust_level_display: from_core_trust_level(assessment.trust_level_display),
        execution_class: from_core_execution_class(assessment.execution_class),
        review_status: from_core_review_status(assessment.review_status),
        integrity_status: from_core_integrity_status(assessment.integrity_status),
        trusted_policy_source: from_core_policy_source(assessment.trusted_policy_source),
        permission_profile: from_core_permission_profile(assessment.permission_profile),
        publisher_id: assessment.publisher_id,
        trusted_catalog_matched: assessment.trusted_catalog_matched,
        hash_matched: assessment.hash_matched,
        verified_hash: assessment.verified_hash,
        verified_signature: assessment.verified_signature,
        reviewed_at: assessment.reviewed_at,
        revoked: assessment.revoked,
        exceeds_standard_sandbox: assessment.exceeds_standard_sandbox,
        requires_explicit_consent: assessment.requires_explicit_consent,
    }
}

fn to_core_enable_context(plugin: &PluginInfo) -> plugin_trust::PluginEnableContext {
    plugin_trust::PluginEnableContext {
        plugin_id: plugin.manifest.id.clone(),
        version: plugin.manifest.version.clone(),
        permissions: plugin.manifest.permissions.clone(),
        trust_level_display: to_core_trust_level(plugin.trust_level_display.clone()),
        review_status: to_core_review_status(plugin.review_status.clone()),
        revoked: plugin.revoked,
        verified_hash: plugin.verified_hash.clone(),
        requires_explicit_consent: plugin.requires_explicit_consent,
    }
}

fn to_core_install_metadata(value: PluginInstallMetadata) -> plugin_trust::PluginInstallMetadata {
    plugin_trust::PluginInstallMetadata {
        distribution_class: value.distribution_class.map(to_core_distribution_class),
        archive_sha256: value.archive_sha256,
        installed_tree_sha256: value.installed_tree_sha256,
    }
}

fn from_core_install_metadata(value: plugin_trust::PluginInstallMetadata) -> PluginInstallMetadata {
    PluginInstallMetadata {
        distribution_class: value.distribution_class.map(from_core_distribution_class),
        archive_sha256: value.archive_sha256,
        installed_tree_sha256: value.installed_tree_sha256,
    }
}

fn to_core_grant(value: PersistedPluginEnableGrant) -> plugin_trust::PersistedPluginEnableGrant {
    plugin_trust::PersistedPluginEnableGrant {
        plugin_id: value.plugin_id,
        version: value.version,
        hash: value.hash,
        permissions_fingerprint: value.permissions_fingerprint,
        grant_scope: to_core_grant_scope(value.grant_scope),
        granted_at: value.granted_at,
    }
}

fn from_core_grant(value: plugin_trust::PersistedPluginEnableGrant) -> PersistedPluginEnableGrant {
    PersistedPluginEnableGrant {
        plugin_id: value.plugin_id,
        version: value.version,
        hash: value.hash,
        permissions_fingerprint: value.permissions_fingerprint,
        grant_scope: from_core_grant_scope(value.grant_scope),
        granted_at: value.granted_at,
    }
}

fn to_core_distribution_class(
    value: PluginDistributionClass,
) -> plugin_trust::PluginDistributionClass {
    match value {
        PluginDistributionClass::Builtin => plugin_trust::PluginDistributionClass::Builtin,
        PluginDistributionClass::Market => plugin_trust::PluginDistributionClass::Market,
        PluginDistributionClass::StandardPackage => {
            plugin_trust::PluginDistributionClass::StandardPackage
        }
        PluginDistributionClass::ManualImport => {
            plugin_trust::PluginDistributionClass::ManualImport
        }
        PluginDistributionClass::LocalDirectory => {
            plugin_trust::PluginDistributionClass::LocalDirectory
        }
        PluginDistributionClass::TrustedCatalog => {
            plugin_trust::PluginDistributionClass::TrustedCatalog
        }
        PluginDistributionClass::Unknown => plugin_trust::PluginDistributionClass::Unknown,
    }
}

fn from_core_distribution_class(
    value: plugin_trust::PluginDistributionClass,
) -> PluginDistributionClass {
    match value {
        plugin_trust::PluginDistributionClass::Builtin => PluginDistributionClass::Builtin,
        plugin_trust::PluginDistributionClass::Market => PluginDistributionClass::Market,
        plugin_trust::PluginDistributionClass::StandardPackage => {
            PluginDistributionClass::StandardPackage
        }
        plugin_trust::PluginDistributionClass::ManualImport => {
            PluginDistributionClass::ManualImport
        }
        plugin_trust::PluginDistributionClass::LocalDirectory => {
            PluginDistributionClass::LocalDirectory
        }
        plugin_trust::PluginDistributionClass::TrustedCatalog => {
            PluginDistributionClass::TrustedCatalog
        }
        plugin_trust::PluginDistributionClass::Unknown => PluginDistributionClass::Unknown,
    }
}

fn from_core_trust_level(value: plugin_trust::PluginTrustLevelDisplay) -> PluginTrustLevelDisplay {
    match value {
        plugin_trust::PluginTrustLevelDisplay::Builtin => PluginTrustLevelDisplay::Builtin,
        plugin_trust::PluginTrustLevelDisplay::Trusted => PluginTrustLevelDisplay::Trusted,
        plugin_trust::PluginTrustLevelDisplay::StandardSandbox => {
            PluginTrustLevelDisplay::StandardSandbox
        }
        plugin_trust::PluginTrustLevelDisplay::Unreviewed => PluginTrustLevelDisplay::Unreviewed,
    }
}

fn to_core_trust_level(value: PluginTrustLevelDisplay) -> plugin_trust::PluginTrustLevelDisplay {
    match value {
        PluginTrustLevelDisplay::Builtin => plugin_trust::PluginTrustLevelDisplay::Builtin,
        PluginTrustLevelDisplay::Trusted => plugin_trust::PluginTrustLevelDisplay::Trusted,
        PluginTrustLevelDisplay::StandardSandbox => {
            plugin_trust::PluginTrustLevelDisplay::StandardSandbox
        }
        PluginTrustLevelDisplay::Unreviewed => plugin_trust::PluginTrustLevelDisplay::Unreviewed,
    }
}

fn from_core_execution_class(value: plugin_trust::PluginExecutionClass) -> PluginExecutionClass {
    match value {
        plugin_trust::PluginExecutionClass::BuiltinFull => PluginExecutionClass::BuiltinFull,
        plugin_trust::PluginExecutionClass::TrustedFull => PluginExecutionClass::TrustedFull,
        plugin_trust::PluginExecutionClass::Sandboxed => PluginExecutionClass::Sandboxed,
        plugin_trust::PluginExecutionClass::Restricted => PluginExecutionClass::Restricted,
    }
}

fn to_core_execution_class(value: PluginExecutionClass) -> plugin_trust::PluginExecutionClass {
    match value {
        PluginExecutionClass::BuiltinFull => plugin_trust::PluginExecutionClass::BuiltinFull,
        PluginExecutionClass::TrustedFull => plugin_trust::PluginExecutionClass::TrustedFull,
        PluginExecutionClass::Sandboxed => plugin_trust::PluginExecutionClass::Sandboxed,
        PluginExecutionClass::Restricted => plugin_trust::PluginExecutionClass::Restricted,
    }
}

fn from_core_review_status(value: plugin_trust::PluginReviewStatus) -> PluginReviewStatus {
    match value {
        plugin_trust::PluginReviewStatus::Builtin => PluginReviewStatus::Builtin,
        plugin_trust::PluginReviewStatus::SealanternReviewed => {
            PluginReviewStatus::SealanternReviewed
        }
        plugin_trust::PluginReviewStatus::Unreviewed => PluginReviewStatus::Unreviewed,
        plugin_trust::PluginReviewStatus::Revoked => PluginReviewStatus::Revoked,
    }
}

fn to_core_review_status(value: PluginReviewStatus) -> plugin_trust::PluginReviewStatus {
    match value {
        PluginReviewStatus::Builtin => plugin_trust::PluginReviewStatus::Builtin,
        PluginReviewStatus::SealanternReviewed => {
            plugin_trust::PluginReviewStatus::SealanternReviewed
        }
        PluginReviewStatus::Unreviewed => plugin_trust::PluginReviewStatus::Unreviewed,
        PluginReviewStatus::Revoked => plugin_trust::PluginReviewStatus::Revoked,
    }
}

fn from_core_integrity_status(value: plugin_trust::PluginIntegrityStatus) -> PluginIntegrityStatus {
    match value {
        plugin_trust::PluginIntegrityStatus::Bundled => PluginIntegrityStatus::Bundled,
        plugin_trust::PluginIntegrityStatus::VerifiedHash => PluginIntegrityStatus::VerifiedHash,
        plugin_trust::PluginIntegrityStatus::VerifiedSignature => {
            PluginIntegrityStatus::VerifiedSignature
        }
        plugin_trust::PluginIntegrityStatus::Unsigned => PluginIntegrityStatus::Unsigned,
        plugin_trust::PluginIntegrityStatus::Mismatch => PluginIntegrityStatus::Mismatch,
        plugin_trust::PluginIntegrityStatus::Unknown => PluginIntegrityStatus::Unknown,
    }
}

fn to_core_integrity_status(value: PluginIntegrityStatus) -> plugin_trust::PluginIntegrityStatus {
    match value {
        PluginIntegrityStatus::Bundled => plugin_trust::PluginIntegrityStatus::Bundled,
        PluginIntegrityStatus::VerifiedHash => plugin_trust::PluginIntegrityStatus::VerifiedHash,
        PluginIntegrityStatus::VerifiedSignature => {
            plugin_trust::PluginIntegrityStatus::VerifiedSignature
        }
        PluginIntegrityStatus::Unsigned => plugin_trust::PluginIntegrityStatus::Unsigned,
        PluginIntegrityStatus::Mismatch => plugin_trust::PluginIntegrityStatus::Mismatch,
        PluginIntegrityStatus::Unknown => plugin_trust::PluginIntegrityStatus::Unknown,
    }
}

fn from_core_policy_source(
    value: plugin_trust::PluginTrustedPolicySource,
) -> PluginTrustedPolicySource {
    match value {
        plugin_trust::PluginTrustedPolicySource::Builtin => PluginTrustedPolicySource::Builtin,
        plugin_trust::PluginTrustedPolicySource::BundledSnapshot => {
            PluginTrustedPolicySource::BundledSnapshot
        }
        plugin_trust::PluginTrustedPolicySource::RemoteSignedCatalog => {
            PluginTrustedPolicySource::RemoteSignedCatalog
        }
        plugin_trust::PluginTrustedPolicySource::LocalAttestation => {
            PluginTrustedPolicySource::LocalAttestation
        }
        plugin_trust::PluginTrustedPolicySource::None => PluginTrustedPolicySource::None,
    }
}

fn from_core_permission_profile(
    value: plugin_trust::PluginPermissionProfile,
) -> PluginPermissionProfile {
    match value {
        plugin_trust::PluginPermissionProfile::BuiltinFull => PluginPermissionProfile::BuiltinFull,
        plugin_trust::PluginPermissionProfile::TrustedFull => PluginPermissionProfile::TrustedFull,
        plugin_trust::PluginPermissionProfile::SandboxedNormal => {
            PluginPermissionProfile::SandboxedNormal
        }
        plugin_trust::PluginPermissionProfile::SandboxedExtended => {
            PluginPermissionProfile::SandboxedExtended
        }
        plugin_trust::PluginPermissionProfile::Unreviewed => PluginPermissionProfile::Unreviewed,
    }
}

fn to_core_permission_profile(
    value: PluginPermissionProfile,
) -> plugin_trust::PluginPermissionProfile {
    match value {
        PluginPermissionProfile::BuiltinFull => plugin_trust::PluginPermissionProfile::BuiltinFull,
        PluginPermissionProfile::TrustedFull => plugin_trust::PluginPermissionProfile::TrustedFull,
        PluginPermissionProfile::SandboxedNormal => {
            plugin_trust::PluginPermissionProfile::SandboxedNormal
        }
        PluginPermissionProfile::SandboxedExtended => {
            plugin_trust::PluginPermissionProfile::SandboxedExtended
        }
        PluginPermissionProfile::Unreviewed => plugin_trust::PluginPermissionProfile::Unreviewed,
    }
}

fn from_core_grant_scope(value: plugin_trust::PluginEnableGrantScope) -> PluginEnableGrantScope {
    match value {
        plugin_trust::PluginEnableGrantScope::Once => PluginEnableGrantScope::Once,
        plugin_trust::PluginEnableGrantScope::Version => PluginEnableGrantScope::Version,
        plugin_trust::PluginEnableGrantScope::Hash => PluginEnableGrantScope::Hash,
    }
}

fn to_core_grant_scope(value: PluginEnableGrantScope) -> plugin_trust::PluginEnableGrantScope {
    match value {
        PluginEnableGrantScope::Once => plugin_trust::PluginEnableGrantScope::Once,
        PluginEnableGrantScope::Version => plugin_trust::PluginEnableGrantScope::Version,
        PluginEnableGrantScope::Hash => plugin_trust::PluginEnableGrantScope::Hash,
    }
}

fn from_core_block_reason(value: plugin_trust::PluginEnableBlockReason) -> PluginEnableBlockReason {
    match value {
        plugin_trust::PluginEnableBlockReason::UserConfirmationRequired => {
            PluginEnableBlockReason::UserConfirmationRequired
        }
        plugin_trust::PluginEnableBlockReason::Revoked => PluginEnableBlockReason::Revoked,
    }
}
