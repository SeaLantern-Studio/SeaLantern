use crate::models::plugin::{
    PluginDistributionClass, PluginEnableBlockReason, PluginEnableGrantScope, PluginEnableResult,
    PluginExecutionClass, PluginInfo, PluginIntegrityStatus, PluginManifest,
    PluginPermissionProfile, PluginReviewStatus, PluginTrustLevelDisplay,
    PluginTrustedPolicySource,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[allow(unused_imports)]
pub use sea_lantern_plugin_trust_core::{TrustedCatalogEntry, TrustedCatalogSnapshot};

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
    sea_lantern_plugin_trust_core::bundled_snapshot()
}

#[allow(dead_code)]
pub fn install_metadata_path(plugin_dir: &Path) -> std::path::PathBuf {
    sea_lantern_plugin_trust_core::install_metadata_path(plugin_dir)
}

pub fn read_install_metadata(plugin_dir: &Path) -> Option<PluginInstallMetadata> {
    sea_lantern_plugin_trust_core::read_install_metadata(plugin_dir).map(from_core_install_metadata)
}

pub fn write_install_metadata(
    plugin_dir: &Path,
    metadata: &PluginInstallMetadata,
) -> Result<(), String> {
    sea_lantern_plugin_trust_core::write_install_metadata(
        plugin_dir,
        &to_core_install_metadata(metadata.clone()),
    )
}

pub fn compute_plugin_tree_sha256(plugin_dir: &Path) -> Result<String, String> {
    sea_lantern_plugin_trust_core::compute_plugin_tree_sha256(plugin_dir)
}

pub fn apply_runtime_integrity_state(
    mut plugin: PluginInfo,
    plugin_dir: &Path,
    metadata: &PluginInstallMetadata,
) -> Result<PluginInfo, String> {
    let state = sea_lantern_plugin_trust_core::PluginRuntimeTrustState {
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

    let state = sea_lantern_plugin_trust_core::apply_runtime_integrity_state(
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
    sea_lantern_plugin_trust_core::enable_grants_path(data_dir)
}

pub fn load_enable_grants(data_dir: &Path) -> Result<Vec<PersistedPluginEnableGrant>, String> {
    sea_lantern_plugin_trust_core::load_enable_grants(data_dir)
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
    sea_lantern_plugin_trust_core::save_enable_grants(data_dir, &grants)
}

#[allow(dead_code)]
pub fn permissions_fingerprint(permissions: &[String]) -> String {
    sea_lantern_plugin_trust_core::permissions_fingerprint(permissions)
}

#[allow(dead_code)]
pub fn required_grant_scope(plugin: &PluginInfo) -> Option<PluginEnableGrantScope> {
    sea_lantern_plugin_trust_core::required_grant_scope(&to_core_enable_context(plugin))
        .map(from_core_grant_scope)
}

pub fn grant_scope_covers(
    provided: PluginEnableGrantScope,
    required: PluginEnableGrantScope,
) -> bool {
    sea_lantern_plugin_trust_core::grant_scope_covers(
        to_core_grant_scope(provided),
        to_core_grant_scope(required),
    )
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
    let result = sea_lantern_plugin_trust_core::evaluate_enable_requirement(
        &to_core_enable_context(plugin),
        &core_grants,
    );

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
    sea_lantern_plugin_trust_core::upsert_enable_grant(
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
    let assessment = sea_lantern_plugin_trust_core::assess_plugin(
        &sea_lantern_plugin_trust_core::PluginManifestInput {
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

fn to_core_enable_context(
    plugin: &PluginInfo,
) -> sea_lantern_plugin_trust_core::PluginEnableContext {
    sea_lantern_plugin_trust_core::PluginEnableContext {
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

fn to_core_install_metadata(
    value: PluginInstallMetadata,
) -> sea_lantern_plugin_trust_core::PluginInstallMetadata {
    sea_lantern_plugin_trust_core::PluginInstallMetadata {
        distribution_class: value.distribution_class.map(to_core_distribution_class),
        archive_sha256: value.archive_sha256,
        installed_tree_sha256: value.installed_tree_sha256,
    }
}

fn from_core_install_metadata(
    value: sea_lantern_plugin_trust_core::PluginInstallMetadata,
) -> PluginInstallMetadata {
    PluginInstallMetadata {
        distribution_class: value.distribution_class.map(from_core_distribution_class),
        archive_sha256: value.archive_sha256,
        installed_tree_sha256: value.installed_tree_sha256,
    }
}

fn to_core_grant(
    value: PersistedPluginEnableGrant,
) -> sea_lantern_plugin_trust_core::PersistedPluginEnableGrant {
    sea_lantern_plugin_trust_core::PersistedPluginEnableGrant {
        plugin_id: value.plugin_id,
        version: value.version,
        hash: value.hash,
        permissions_fingerprint: value.permissions_fingerprint,
        grant_scope: to_core_grant_scope(value.grant_scope),
        granted_at: value.granted_at,
    }
}

fn from_core_grant(
    value: sea_lantern_plugin_trust_core::PersistedPluginEnableGrant,
) -> PersistedPluginEnableGrant {
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
) -> sea_lantern_plugin_trust_core::PluginDistributionClass {
    match value {
        PluginDistributionClass::Builtin => {
            sea_lantern_plugin_trust_core::PluginDistributionClass::Builtin
        }
        PluginDistributionClass::Market => {
            sea_lantern_plugin_trust_core::PluginDistributionClass::Market
        }
        PluginDistributionClass::StandardPackage => {
            sea_lantern_plugin_trust_core::PluginDistributionClass::StandardPackage
        }
        PluginDistributionClass::ManualImport => {
            sea_lantern_plugin_trust_core::PluginDistributionClass::ManualImport
        }
        PluginDistributionClass::LocalDirectory => {
            sea_lantern_plugin_trust_core::PluginDistributionClass::LocalDirectory
        }
        PluginDistributionClass::TrustedCatalog => {
            sea_lantern_plugin_trust_core::PluginDistributionClass::TrustedCatalog
        }
        PluginDistributionClass::Unknown => {
            sea_lantern_plugin_trust_core::PluginDistributionClass::Unknown
        }
    }
}

fn from_core_distribution_class(
    value: sea_lantern_plugin_trust_core::PluginDistributionClass,
) -> PluginDistributionClass {
    match value {
        sea_lantern_plugin_trust_core::PluginDistributionClass::Builtin => {
            PluginDistributionClass::Builtin
        }
        sea_lantern_plugin_trust_core::PluginDistributionClass::Market => {
            PluginDistributionClass::Market
        }
        sea_lantern_plugin_trust_core::PluginDistributionClass::StandardPackage => {
            PluginDistributionClass::StandardPackage
        }
        sea_lantern_plugin_trust_core::PluginDistributionClass::ManualImport => {
            PluginDistributionClass::ManualImport
        }
        sea_lantern_plugin_trust_core::PluginDistributionClass::LocalDirectory => {
            PluginDistributionClass::LocalDirectory
        }
        sea_lantern_plugin_trust_core::PluginDistributionClass::TrustedCatalog => {
            PluginDistributionClass::TrustedCatalog
        }
        sea_lantern_plugin_trust_core::PluginDistributionClass::Unknown => {
            PluginDistributionClass::Unknown
        }
    }
}

fn from_core_trust_level(
    value: sea_lantern_plugin_trust_core::PluginTrustLevelDisplay,
) -> PluginTrustLevelDisplay {
    match value {
        sea_lantern_plugin_trust_core::PluginTrustLevelDisplay::Builtin => {
            PluginTrustLevelDisplay::Builtin
        }
        sea_lantern_plugin_trust_core::PluginTrustLevelDisplay::Trusted => {
            PluginTrustLevelDisplay::Trusted
        }
        sea_lantern_plugin_trust_core::PluginTrustLevelDisplay::StandardSandbox => {
            PluginTrustLevelDisplay::StandardSandbox
        }
        sea_lantern_plugin_trust_core::PluginTrustLevelDisplay::Unreviewed => {
            PluginTrustLevelDisplay::Unreviewed
        }
    }
}

fn to_core_trust_level(
    value: PluginTrustLevelDisplay,
) -> sea_lantern_plugin_trust_core::PluginTrustLevelDisplay {
    match value {
        PluginTrustLevelDisplay::Builtin => {
            sea_lantern_plugin_trust_core::PluginTrustLevelDisplay::Builtin
        }
        PluginTrustLevelDisplay::Trusted => {
            sea_lantern_plugin_trust_core::PluginTrustLevelDisplay::Trusted
        }
        PluginTrustLevelDisplay::StandardSandbox => {
            sea_lantern_plugin_trust_core::PluginTrustLevelDisplay::StandardSandbox
        }
        PluginTrustLevelDisplay::Unreviewed => {
            sea_lantern_plugin_trust_core::PluginTrustLevelDisplay::Unreviewed
        }
    }
}

fn from_core_execution_class(
    value: sea_lantern_plugin_trust_core::PluginExecutionClass,
) -> PluginExecutionClass {
    match value {
        sea_lantern_plugin_trust_core::PluginExecutionClass::BuiltinFull => {
            PluginExecutionClass::BuiltinFull
        }
        sea_lantern_plugin_trust_core::PluginExecutionClass::TrustedFull => {
            PluginExecutionClass::TrustedFull
        }
        sea_lantern_plugin_trust_core::PluginExecutionClass::Sandboxed => {
            PluginExecutionClass::Sandboxed
        }
        sea_lantern_plugin_trust_core::PluginExecutionClass::Restricted => {
            PluginExecutionClass::Restricted
        }
    }
}

fn to_core_execution_class(
    value: PluginExecutionClass,
) -> sea_lantern_plugin_trust_core::PluginExecutionClass {
    match value {
        PluginExecutionClass::BuiltinFull => {
            sea_lantern_plugin_trust_core::PluginExecutionClass::BuiltinFull
        }
        PluginExecutionClass::TrustedFull => {
            sea_lantern_plugin_trust_core::PluginExecutionClass::TrustedFull
        }
        PluginExecutionClass::Sandboxed => {
            sea_lantern_plugin_trust_core::PluginExecutionClass::Sandboxed
        }
        PluginExecutionClass::Restricted => {
            sea_lantern_plugin_trust_core::PluginExecutionClass::Restricted
        }
    }
}

fn from_core_review_status(
    value: sea_lantern_plugin_trust_core::PluginReviewStatus,
) -> PluginReviewStatus {
    match value {
        sea_lantern_plugin_trust_core::PluginReviewStatus::Builtin => PluginReviewStatus::Builtin,
        sea_lantern_plugin_trust_core::PluginReviewStatus::SealanternReviewed => {
            PluginReviewStatus::SealanternReviewed
        }
        sea_lantern_plugin_trust_core::PluginReviewStatus::Unreviewed => {
            PluginReviewStatus::Unreviewed
        }
        sea_lantern_plugin_trust_core::PluginReviewStatus::Revoked => PluginReviewStatus::Revoked,
    }
}

fn to_core_review_status(
    value: PluginReviewStatus,
) -> sea_lantern_plugin_trust_core::PluginReviewStatus {
    match value {
        PluginReviewStatus::Builtin => sea_lantern_plugin_trust_core::PluginReviewStatus::Builtin,
        PluginReviewStatus::SealanternReviewed => {
            sea_lantern_plugin_trust_core::PluginReviewStatus::SealanternReviewed
        }
        PluginReviewStatus::Unreviewed => {
            sea_lantern_plugin_trust_core::PluginReviewStatus::Unreviewed
        }
        PluginReviewStatus::Revoked => sea_lantern_plugin_trust_core::PluginReviewStatus::Revoked,
    }
}

fn from_core_integrity_status(
    value: sea_lantern_plugin_trust_core::PluginIntegrityStatus,
) -> PluginIntegrityStatus {
    match value {
        sea_lantern_plugin_trust_core::PluginIntegrityStatus::Bundled => {
            PluginIntegrityStatus::Bundled
        }
        sea_lantern_plugin_trust_core::PluginIntegrityStatus::VerifiedHash => {
            PluginIntegrityStatus::VerifiedHash
        }
        sea_lantern_plugin_trust_core::PluginIntegrityStatus::VerifiedSignature => {
            PluginIntegrityStatus::VerifiedSignature
        }
        sea_lantern_plugin_trust_core::PluginIntegrityStatus::Unsigned => {
            PluginIntegrityStatus::Unsigned
        }
        sea_lantern_plugin_trust_core::PluginIntegrityStatus::Mismatch => {
            PluginIntegrityStatus::Mismatch
        }
        sea_lantern_plugin_trust_core::PluginIntegrityStatus::Unknown => {
            PluginIntegrityStatus::Unknown
        }
    }
}

fn to_core_integrity_status(
    value: PluginIntegrityStatus,
) -> sea_lantern_plugin_trust_core::PluginIntegrityStatus {
    match value {
        PluginIntegrityStatus::Bundled => {
            sea_lantern_plugin_trust_core::PluginIntegrityStatus::Bundled
        }
        PluginIntegrityStatus::VerifiedHash => {
            sea_lantern_plugin_trust_core::PluginIntegrityStatus::VerifiedHash
        }
        PluginIntegrityStatus::VerifiedSignature => {
            sea_lantern_plugin_trust_core::PluginIntegrityStatus::VerifiedSignature
        }
        PluginIntegrityStatus::Unsigned => {
            sea_lantern_plugin_trust_core::PluginIntegrityStatus::Unsigned
        }
        PluginIntegrityStatus::Mismatch => {
            sea_lantern_plugin_trust_core::PluginIntegrityStatus::Mismatch
        }
        PluginIntegrityStatus::Unknown => {
            sea_lantern_plugin_trust_core::PluginIntegrityStatus::Unknown
        }
    }
}

fn from_core_policy_source(
    value: sea_lantern_plugin_trust_core::PluginTrustedPolicySource,
) -> PluginTrustedPolicySource {
    match value {
        sea_lantern_plugin_trust_core::PluginTrustedPolicySource::Builtin => {
            PluginTrustedPolicySource::Builtin
        }
        sea_lantern_plugin_trust_core::PluginTrustedPolicySource::BundledSnapshot => {
            PluginTrustedPolicySource::BundledSnapshot
        }
        sea_lantern_plugin_trust_core::PluginTrustedPolicySource::RemoteSignedCatalog => {
            PluginTrustedPolicySource::RemoteSignedCatalog
        }
        sea_lantern_plugin_trust_core::PluginTrustedPolicySource::LocalAttestation => {
            PluginTrustedPolicySource::LocalAttestation
        }
        sea_lantern_plugin_trust_core::PluginTrustedPolicySource::None => {
            PluginTrustedPolicySource::None
        }
    }
}

fn from_core_permission_profile(
    value: sea_lantern_plugin_trust_core::PluginPermissionProfile,
) -> PluginPermissionProfile {
    match value {
        sea_lantern_plugin_trust_core::PluginPermissionProfile::BuiltinFull => {
            PluginPermissionProfile::BuiltinFull
        }
        sea_lantern_plugin_trust_core::PluginPermissionProfile::TrustedFull => {
            PluginPermissionProfile::TrustedFull
        }
        sea_lantern_plugin_trust_core::PluginPermissionProfile::SandboxedNormal => {
            PluginPermissionProfile::SandboxedNormal
        }
        sea_lantern_plugin_trust_core::PluginPermissionProfile::SandboxedExtended => {
            PluginPermissionProfile::SandboxedExtended
        }
        sea_lantern_plugin_trust_core::PluginPermissionProfile::Unreviewed => {
            PluginPermissionProfile::Unreviewed
        }
    }
}

fn to_core_permission_profile(
    value: PluginPermissionProfile,
) -> sea_lantern_plugin_trust_core::PluginPermissionProfile {
    match value {
        PluginPermissionProfile::BuiltinFull => {
            sea_lantern_plugin_trust_core::PluginPermissionProfile::BuiltinFull
        }
        PluginPermissionProfile::TrustedFull => {
            sea_lantern_plugin_trust_core::PluginPermissionProfile::TrustedFull
        }
        PluginPermissionProfile::SandboxedNormal => {
            sea_lantern_plugin_trust_core::PluginPermissionProfile::SandboxedNormal
        }
        PluginPermissionProfile::SandboxedExtended => {
            sea_lantern_plugin_trust_core::PluginPermissionProfile::SandboxedExtended
        }
        PluginPermissionProfile::Unreviewed => {
            sea_lantern_plugin_trust_core::PluginPermissionProfile::Unreviewed
        }
    }
}

fn from_core_grant_scope(
    value: sea_lantern_plugin_trust_core::PluginEnableGrantScope,
) -> PluginEnableGrantScope {
    match value {
        sea_lantern_plugin_trust_core::PluginEnableGrantScope::Once => PluginEnableGrantScope::Once,
        sea_lantern_plugin_trust_core::PluginEnableGrantScope::Version => {
            PluginEnableGrantScope::Version
        }
        sea_lantern_plugin_trust_core::PluginEnableGrantScope::Hash => PluginEnableGrantScope::Hash,
    }
}

fn to_core_grant_scope(
    value: PluginEnableGrantScope,
) -> sea_lantern_plugin_trust_core::PluginEnableGrantScope {
    match value {
        PluginEnableGrantScope::Once => sea_lantern_plugin_trust_core::PluginEnableGrantScope::Once,
        PluginEnableGrantScope::Version => {
            sea_lantern_plugin_trust_core::PluginEnableGrantScope::Version
        }
        PluginEnableGrantScope::Hash => sea_lantern_plugin_trust_core::PluginEnableGrantScope::Hash,
    }
}

fn from_core_block_reason(
    value: sea_lantern_plugin_trust_core::PluginEnableBlockReason,
) -> PluginEnableBlockReason {
    match value {
        sea_lantern_plugin_trust_core::PluginEnableBlockReason::UserConfirmationRequired => {
            PluginEnableBlockReason::UserConfirmationRequired
        }
        sea_lantern_plugin_trust_core::PluginEnableBlockReason::Revoked => {
            PluginEnableBlockReason::Revoked
        }
    }
}
