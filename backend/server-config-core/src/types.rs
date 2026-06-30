use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerConfigFileKind {
    Properties,
    Toml,
    Yaml,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnownServerConfigRole {
    StartupPrimary,
    StartupLegacy,
    ServerProperties,
    Pumpkin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerConfigOwnership {
    ServiceManaged,
    ServerManaged,
    ThirdParty,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerConfigSourceKind {
    ServerRoot,
    ManualRoot,
    ManualFile,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerConfigSearchMode {
    Keyword,
    Regex,
    Similarity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerConfigSearchScope {
    Path,
    Content,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ServerConfigJsonMode {
    Disabled,
    #[default]
    Filtered,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfigDiscoveryOptions {
    #[serde(default)]
    pub manual_import_dirs: Vec<String>,
    #[serde(default)]
    pub manual_import_files: Vec<String>,
    #[serde(default)]
    pub json_mode: ServerConfigJsonMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredServerConfigFile {
    pub locator: String,
    pub relative_path: String,
    pub file_name: String,
    pub absolute_path: String,
    pub source_kind: ServerConfigSourceKind,
    pub source_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_relative_path: Option<String>,
    pub kind: ServerConfigFileKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub known_role: Option<KnownServerConfigRole>,
    pub ownership: ServerConfigOwnership,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerConfigSearchHit {
    pub locator: String,
    pub relative_path: String,
    pub file_name: String,
    pub absolute_path: String,
    pub source_kind: ServerConfigSourceKind,
    pub source_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_relative_path: Option<String>,
    pub kind: ServerConfigFileKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub known_role: Option<KnownServerConfigRole>,
    pub ownership: ServerConfigOwnership,
    pub priority: u32,
    pub score: u32,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_match: Option<ServerConfigContentMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerConfigContentMatch {
    pub line_number: usize,
    pub line_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerConfigDocument {
    pub relative_path: String,
    pub kind: ServerConfigFileKind,
    pub content: serde_json::Value,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub description: String,
    pub value_type: String,
    pub default_value: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProperties {
    pub entries: Vec<ConfigEntry>,
    pub raw: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CpuPolicyMode {
    #[default]
    Off,
    Count,
    Explicit,
}

impl CpuPolicyMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Count => "count",
            Self::Explicit => "explicit",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CpuPolicyConfig {
    #[serde(default)]
    pub mode: CpuPolicyMode,
    #[serde(default)]
    pub count: Option<u16>,
    #[serde(default)]
    pub explicit_set: Option<String>,
    #[serde(default = "default_true")]
    pub sync_active_processor_count: bool,
}

impl Default for CpuPolicyConfig {
    fn default() -> Self {
        Self {
            mode: CpuPolicyMode::Off,
            count: None,
            explicit_set: None,
            sync_active_processor_count: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum JvmPresetId {
    #[default]
    None,
    G1Basic,
    AikarG1,
    ThroughputBasic,
    PaperRecommendedLite,
}

impl JvmPresetId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::G1Basic => "g1_basic",
            Self::AikarG1 => "aikar_g1",
            Self::ThroughputBasic => "throughput_basic",
            Self::PaperRecommendedLite => "paper_recommended_lite",
        }
    }

    pub fn preset_args(&self) -> &'static [&'static str] {
        match self {
            Self::None => &[],
            Self::G1Basic => &[
                "-XX:+UseG1GC",
                "-XX:+ParallelRefProcEnabled",
                "-XX:MaxGCPauseMillis=200",
                "-XX:+UnlockExperimentalVMOptions",
            ],
            Self::AikarG1 => &[
                "-XX:+UseG1GC",
                "-XX:+ParallelRefProcEnabled",
                "-XX:MaxGCPauseMillis=200",
                "-XX:+UnlockExperimentalVMOptions",
                "-XX:+DisableExplicitGC",
                "-XX:+AlwaysPreTouch",
                "-XX:G1NewSizePercent=30",
                "-XX:G1MaxNewSizePercent=40",
                "-XX:G1HeapRegionSize=8M",
                "-XX:G1ReservePercent=20",
                "-XX:G1HeapWastePercent=5",
                "-XX:G1MixedGCCountTarget=4",
                "-XX:InitiatingHeapOccupancyPercent=15",
                "-XX:G1MixedGCLiveThresholdPercent=90",
                "-XX:G1RSetUpdatingPauseTimePercent=5",
                "-XX:SurvivorRatio=32",
                "-XX:+PerfDisableSharedMem",
                "-XX:MaxTenuringThreshold=1",
            ],
            Self::ThroughputBasic => {
                &["-XX:+UseParallelGC", "-XX:+UseAdaptiveSizePolicy", "-XX:MaxGCPauseMillis=500"]
            }
            Self::PaperRecommendedLite => &[
                "-XX:+UseG1GC",
                "-XX:+ParallelRefProcEnabled",
                "-XX:MaxGCPauseMillis=150",
                "-XX:+UnlockExperimentalVMOptions",
                "-XX:+DisableExplicitGC",
                "-Dusing.aikars.flags=https://mcflags.emc.gs",
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct JvmPresetConfig {
    #[serde(default)]
    pub preset: JvmPresetId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SLStartupConfig {
    #[serde(default)]
    pub max_memory: Option<u32>,
    #[serde(default)]
    pub min_memory: Option<u32>,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub cpu_policy: CpuPolicyConfig,
    #[serde(default)]
    pub jvm_preset: JvmPresetConfig,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StartupConfigPresence {
    pub max_memory: bool,
    pub min_memory: bool,
    pub jvm_args: bool,
    pub cpu_policy: bool,
    pub jvm_preset: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ServerStartupConfigDocument {
    pub config: SLStartupConfig,
    pub presence: StartupConfigPresence,
}

#[cfg(test)]
mod tests {
    use super::JvmPresetId;

    #[test]
    fn jvm_preset_id_as_str_is_stable() {
        assert_eq!(JvmPresetId::None.as_str(), "none");
        assert_eq!(JvmPresetId::AikarG1.as_str(), "aikar_g1");
        assert_eq!(JvmPresetId::PaperRecommendedLite.as_str(), "paper_recommended_lite");
    }

    #[test]
    fn jvm_preset_id_args_include_expected_flags() {
        assert!(JvmPresetId::G1Basic.preset_args().contains(&"-XX:+UseG1GC"));
        assert!(JvmPresetId::AikarG1
            .preset_args()
            .contains(&"-XX:+DisableExplicitGC"));
        assert!(JvmPresetId::PaperRecommendedLite
            .preset_args()
            .contains(&"-Dusing.aikars.flags=https://mcflags.emc.gs"));
    }
}
