use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SERVER_INSPECTION_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvidenceId(pub u32);

/// 一个需要从互相竞争的候选中选出的检测结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Detected<T> {
    pub value: Option<T>,
    pub confidence: u8,
    pub evidence: Vec<EvidenceId>,
    pub alternatives: Vec<DetectionCandidate<T>>,
}

impl<T> Default for Detected<T> {
    fn default() -> Self {
        Self {
            value: None,
            confidence: 0,
            evidence: Vec::new(),
            alternatives: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectionCandidate<T> {
    pub value: T,
    pub confidence: u8,
    pub evidence: Vec<EvidenceId>,
}

/// 一个允许与其他值并存的检测结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attributed<T> {
    pub value: T,
    pub confidence: u8,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInspectionReport {
    pub schema_version: u16,
    pub subject: InspectionSubject,
    pub artifact: ArtifactInfo,
    pub identity: ServerIdentityInfo,
    pub minecraft: Option<MinecraftVersionInfo>,
    pub java: JavaRequirementInfo,
    pub components: Vec<Attributed<ServerComponent>>,
    pub launches: Vec<Attributed<LaunchProfile>>,
    pub evidence: Vec<DetectionEvidence>,
    pub diagnostics: Vec<InspectionDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectionSubject {
    pub path: PathBuf,
    pub kind: InspectionSubjectKind,
    pub size_bytes: Option<u64>,
    pub modified_at_unix_secs: Option<u64>,
    pub fingerprint: Option<ArtifactFingerprint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionSubjectKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactFingerprint {
    pub algorithm: FingerprintAlgorithm,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FingerprintAlgorithm {
    Sha256,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactInfo {
    pub format: Detected<ArtifactFormat>,
    pub roles: Vec<Attributed<ArtifactRole>>,
    pub main_class: Detected<String>,
    pub premain_class: Detected<String>,
    pub agent_class: Detected<String>,
    pub automatic_module_name: Detected<String>,
    pub manifest: Option<ManifestSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactFormat {
    Directory,
    Jar,
    Zip,
    Script,
    Executable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactRole {
    Runnable,
    Bootstrapper,
    Installer,
    Launcher,
    Wrapper,
    Library,
    InstallationDirectory,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestSummary {
    pub main_attributes: BTreeMap<String, String>,
    pub sections: Vec<ManifestSection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestSection {
    pub name: Option<String>,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerIdentityInfo {
    pub category: Detected<ServerCategory>,
    pub implementation: Detected<ServerProduct>,
    pub version: Detected<String>,
    pub release_channel: Detected<ReleaseChannel>,
    pub ecosystems: Vec<Attributed<ServerEcosystem>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerCategory {
    JavaGameServer,
    BedrockGameServer,
    Proxy,
    Limbo,
    Unknown,
}

/// 产品标识保持开放，新增服务端不需要修改公共枚举。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServerProduct {
    pub key: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerEcosystem {
    Vanilla,
    Bukkit,
    Paper,
    Fabric,
    LegacyFabric,
    Quilt,
    Forge,
    NeoForge,
    Sponge,
    Bungee,
    Velocity,
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseChannel {
    Stable,
    Beta,
    Alpha,
    Snapshot,
    Development,
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MinecraftVersionInfo {
    pub version: Detected<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub world_version: Option<i64>,
    pub series_id: Option<String>,
    pub protocol_version: Option<i64>,
    pub pack_version: Option<MinecraftPackVersion>,
    pub build_time: Option<String>,
    pub java_component: Option<String>,
    pub java_version: Option<u16>,
    pub stable: Option<bool>,
    pub use_editor: Option<bool>,
    pub extra: BTreeMap<String, Value>,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinecraftPackVersion {
    pub resource: Option<PackFormatVersion>,
    pub data: Option<PackFormatVersion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackFormatVersion {
    pub major: u32,
    pub minor: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JavaRequirementInfo {
    pub required_major: Detected<u16>,
    pub runtime_component: Detected<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerComponent {
    pub kind: ServerComponentKind,
    pub key: String,
    pub name: String,
    pub version: Option<String>,
    pub release_channel: Option<ReleaseChannel>,
    pub coordinate: Option<MavenCoordinate>,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerComponentKind {
    Implementation,
    Api,
    ModLoader,
    Installer,
    Launcher,
    Bootstrap,
    Mapping,
    Wrapper,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MavenCoordinate {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub classifier: Option<String>,
    pub extension: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchProfile {
    pub id: String,
    pub platform: LaunchPlatform,
    pub working_directory: Option<PathBuf>,
    pub target: LaunchTarget,
    pub jvm_arguments: Vec<String>,
    pub program_arguments: Vec<String>,
    pub required_java_major: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchPlatform {
    Any,
    Windows,
    Unix,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LaunchTarget {
    Jar { path: PathBuf },
    MainClass { class_name: String },
    ArgumentFiles { paths: Vec<PathBuf> },
    Script { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectionEvidence {
    pub id: EvidenceId,
    pub detector: String,
    pub source: EvidenceSource,
    pub location: EvidenceLocation,
    pub target: DetectionTarget,
    pub candidate: String,
    pub weight: u8,
    pub correlation_group: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    FileMetadata,
    FileName,
    ManifestMain,
    ManifestSection,
    JarEntry,
    JsonField,
    PropertiesField,
    MavenCoordinate,
    ArgumentFile,
    DirectoryLayout,
    ClassEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceLocation {
    pub path: PathBuf,
    pub archive_entry: Option<String>,
    pub manifest_section: Option<String>,
    pub field: Option<String>,
}

impl EvidenceLocation {
    pub(crate) fn path(path: PathBuf) -> Self {
        Self {
            path,
            archive_entry: None,
            manifest_section: None,
            field: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionTarget {
    ArtifactFormat,
    ArtifactRole,
    MainClass,
    PremainClass,
    AgentClass,
    AutomaticModuleName,
    ServerCategory,
    ServerImplementation,
    ServerVersion,
    ServerEcosystem,
    ReleaseChannel,
    MinecraftVersion,
    JavaMajor,
    JavaRuntimeComponent,
    Component,
    LaunchProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectionDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}
