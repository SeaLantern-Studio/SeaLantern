use serde::{Deserialize, Serialize};

use super::manifest::PluginManifest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginSource {
    Local,
    Builtin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginRuntimeKind {
    Lua,
    Rust,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PluginTrustLevelDisplay {
    Builtin,
    Trusted,
    #[default]
    StandardSandbox,
    Unreviewed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PluginExecutionClass {
    BuiltinFull,
    TrustedFull,
    #[default]
    Sandboxed,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PluginReviewStatus {
    Builtin,
    SealanternReviewed,
    #[default]
    Unreviewed,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
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
pub enum PluginTrustedPolicySource {
    Builtin,
    BundledSnapshot,
    RemoteSignedCatalog,
    LocalAttestation,
    #[default]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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
pub enum PluginPermissionProfile {
    BuiltinFull,
    TrustedFull,
    #[default]
    SandboxedNormal,
    SandboxedExtended,
    Unreviewed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PluginEnableGrantScope {
    Once,
    #[default]
    Version,
    Hash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginEnableConfirmation {
    pub grant_scope: PluginEnableGrantScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginEnableBlockReason {
    UserConfirmationRequired,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEnableResult {
    pub success: bool,
    #[serde(default)]
    pub disabled_plugins: Vec<String>,
    #[serde(default)]
    pub confirmation_required: bool,
    #[serde(default)]
    pub block_reason: Option<PluginEnableBlockReason>,
    #[serde(default)]
    pub plugin: Option<PluginInfo>,
    #[serde(default)]
    pub grant_scope: Option<PluginEnableGrantScope>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginActions {
    #[serde(default = "default_true")]
    pub can_toggle: bool,
    #[serde(default = "default_true")]
    pub can_delete: bool,
    #[serde(default = "default_true")]
    pub can_check_update: bool,
}

/// 插件当前状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PluginState {
    Loaded,
    Enabled,
    Disabled,
    Error(String),
}

/// 已载入插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub path: String,
    #[serde(default = "default_plugin_source")]
    pub source: PluginSource,
    #[serde(default = "default_plugin_runtime_kind")]
    pub runtime: PluginRuntimeKind,
    #[serde(default = "default_plugin_actions")]
    pub actions: PluginActions,
    #[serde(default)]
    pub missing_dependencies: Vec<MissingDependency>,
    #[serde(default = "default_plugin_trust_level_display")]
    pub trust_level_display: PluginTrustLevelDisplay,
    #[serde(default = "default_plugin_execution_class")]
    pub execution_class: PluginExecutionClass,
    #[serde(default = "default_plugin_review_status")]
    pub review_status: PluginReviewStatus,
    #[serde(default = "default_plugin_integrity_status")]
    pub integrity_status: PluginIntegrityStatus,
    #[serde(default = "default_plugin_trusted_policy_source")]
    pub trusted_policy_source: PluginTrustedPolicySource,
    #[serde(default = "default_plugin_permission_profile")]
    pub permission_profile: PluginPermissionProfile,
    #[serde(default)]
    pub publisher_id: Option<String>,
    #[serde(default = "default_plugin_distribution_class")]
    pub distribution_class: PluginDistributionClass,
    #[serde(default)]
    pub trusted_catalog_matched: bool,
    #[serde(default)]
    pub hash_matched: bool,
    #[serde(default)]
    pub verified_hash: Option<String>,
    #[serde(default)]
    pub verified_signature: bool,
    #[serde(default)]
    pub reviewed_at: Option<String>,
    #[serde(default)]
    pub revoked: bool,
    #[serde(default)]
    pub exceeds_standard_sandbox: bool,
    #[serde(default)]
    pub requires_explicit_consent: bool,
}

/// 缺失依赖信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingDependency {
    pub id: String,
    pub version_requirement: Option<String>,
    pub required: bool,
}

/// 单个插件安装结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallResult {
    pub plugin: PluginInfo,
    pub missing_dependencies: Vec<MissingDependency>,
    #[serde(default)]
    pub untrusted_url: bool,
    #[serde(default)]
    pub suggested_trust_level: Option<PluginTrustLevelDisplay>,
    #[serde(default)]
    pub integrity_status: Option<PluginIntegrityStatus>,
    #[serde(default)]
    pub review_status: Option<PluginReviewStatus>,
    #[serde(default)]
    pub distribution_class: Option<PluginDistributionClass>,
    #[serde(default)]
    pub permission_profile: Option<PluginPermissionProfile>,
    #[serde(default)]
    pub trusted_catalog_matched: bool,
    #[serde(default)]
    pub hash_matched: bool,
    #[serde(default)]
    pub exceeds_standard_sandbox: bool,
    #[serde(default)]
    pub install_notices: Vec<PluginInstallIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginInstallIssueCode {
    IncompatibleSealanternVersion,
    RequestsTrustedCapabilities,
    ExceedsStandardSandbox,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginInstallIssue {
    pub code: String,
    #[serde(default)]
    pub args: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallCommandError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub args: serde_json::Map<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_kind: Option<String>,
}

impl PluginInstallIssueCode {
    pub fn as_code(&self) -> &'static str {
        match self {
            Self::IncompatibleSealanternVersion => {
                "plugins.install.issue.incompatible_sealantern_version"
            }
            Self::RequestsTrustedCapabilities => {
                "plugins.install.issue.requests_trusted_capabilities"
            }
            Self::ExceedsStandardSandbox => "plugins.install.issue.exceeds_standard_sandbox",
        }
    }
}

impl PluginInstallIssue {
    pub fn incompatible_sealantern_version(
        plugin_id: impl Into<String>,
        required_version: impl Into<String>,
        current_version: impl Into<String>,
    ) -> Self {
        let plugin_id = plugin_id.into();
        let required_version = required_version.into();
        let current_version = current_version.into();

        Self {
            code: PluginInstallIssueCode::IncompatibleSealanternVersion
                .as_code()
                .to_string(),
            args: serde_json::Map::from_iter([
                ("plugin_id".to_string(), serde_json::Value::String(plugin_id)),
                ("required_version".to_string(), serde_json::Value::String(required_version)),
                ("current_version".to_string(), serde_json::Value::String(current_version)),
            ]),
        }
    }

    pub fn requests_trusted_capabilities(
        plugin_id: impl Into<String>,
        permissions: Vec<String>,
    ) -> Self {
        Self {
            code: PluginInstallIssueCode::RequestsTrustedCapabilities
                .as_code()
                .to_string(),
            args: serde_json::Map::from_iter([
                ("plugin_id".to_string(), serde_json::Value::String(plugin_id.into())),
                (
                    "permissions".to_string(),
                    serde_json::Value::Array(
                        permissions
                            .into_iter()
                            .map(serde_json::Value::String)
                            .collect(),
                    ),
                ),
            ]),
        }
    }

    pub fn exceeds_standard_sandbox(
        plugin_id: impl Into<String>,
        permissions: Vec<String>,
    ) -> Self {
        Self {
            code: PluginInstallIssueCode::ExceedsStandardSandbox
                .as_code()
                .to_string(),
            args: serde_json::Map::from_iter([
                ("plugin_id".to_string(), serde_json::Value::String(plugin_id.into())),
                (
                    "permissions".to_string(),
                    serde_json::Value::Array(
                        permissions
                            .into_iter()
                            .map(serde_json::Value::String)
                            .collect(),
                    ),
                ),
            ]),
        }
    }

    pub fn into_command_error(
        self,
        fallback_message: impl Into<String>,
    ) -> PluginInstallCommandError {
        PluginInstallCommandError {
            code: self.code,
            message: fallback_message.into(),
            args: self.args,
            error_kind: Some("plugin_install".to_string()),
        }
    }
}

impl PluginInstallCommandError {
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| self.message.clone())
    }
}

/// 批量安装里的单项失败信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInstallError {
    pub path: String,
    pub error: String,
    #[serde(default)]
    pub issue: Option<PluginInstallIssue>,
}

/// 批量插件安装结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInstallResult {
    pub success: Vec<PluginInstallResult>,
    pub failed: Vec<BatchInstallError>,
}

fn default_true() -> bool {
    true
}

fn default_plugin_source() -> PluginSource {
    PluginSource::Local
}

fn default_plugin_runtime_kind() -> PluginRuntimeKind {
    PluginRuntimeKind::Lua
}

fn default_plugin_actions() -> PluginActions {
    PluginActions {
        can_toggle: true,
        can_delete: true,
        can_check_update: true,
    }
}

fn default_plugin_trust_level_display() -> PluginTrustLevelDisplay {
    PluginTrustLevelDisplay::StandardSandbox
}

fn default_plugin_execution_class() -> PluginExecutionClass {
    PluginExecutionClass::Sandboxed
}

fn default_plugin_review_status() -> PluginReviewStatus {
    PluginReviewStatus::Unreviewed
}

fn default_plugin_integrity_status() -> PluginIntegrityStatus {
    PluginIntegrityStatus::Unknown
}

fn default_plugin_trusted_policy_source() -> PluginTrustedPolicySource {
    PluginTrustedPolicySource::None
}

fn default_plugin_distribution_class() -> PluginDistributionClass {
    PluginDistributionClass::Unknown
}

fn default_plugin_permission_profile() -> PluginPermissionProfile {
    PluginPermissionProfile::SandboxedNormal
}
