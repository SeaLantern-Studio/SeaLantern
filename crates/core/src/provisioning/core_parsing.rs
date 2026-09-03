use std::fmt;
use std::path::{Path, PathBuf};

use super::server_inspection::{
    Attributed, Detected, DetectionOutcome, InspectionOptions, ReleaseChannel, ServerEcosystem,
    ServerInspectionError, ServerInspectionReport, inspect_server_artifact,
    server_implementation_outcome,
};

/// 一个可识别的服务端核心系列。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreKind {
    ArclightForge,
    ArclightNeoForge,
    Youer,
    Mohist,
    CatServer,
    SpongeForge,
    ArclightFabric,
    Banner,
    NeoForge,
    Forge,
    Quilt,
    Fabric,
    PufferfishPurpur,
    Pufferfish,
    SpongeVanilla,
    Purpur,
    Paper,
    Folia,
    Leaves,
    Leaf,
    Spigot,
    Bukkit,
    VanillaSnapshot,
    Vanilla,
    NukkitX,
    Bedrock,
    Velocity,
    BungeeCord,
    Lightfall,
    Travertine,
    Pumpkin,
    Unknown,
}

impl CoreKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ArclightForge => "arclight-forge",
            Self::ArclightNeoForge => "arclight-neoforge",
            Self::Youer => "youer",
            Self::Mohist => "mohist",
            Self::CatServer => "catserver",
            Self::SpongeForge => "spongeforge",
            Self::ArclightFabric => "arclight-fabric",
            Self::Banner => "banner",
            Self::NeoForge => "neoforge",
            Self::Forge => "forge",
            Self::Quilt => "quilt",
            Self::Fabric => "fabric",
            Self::PufferfishPurpur => "pufferfish_purpur",
            Self::Pufferfish => "pufferfish",
            Self::SpongeVanilla => "spongevanilla",
            Self::Purpur => "purpur",
            Self::Paper => "paper",
            Self::Folia => "folia",
            Self::Leaves => "leaves",
            Self::Leaf => "leaf",
            Self::Spigot => "spigot",
            Self::Bukkit => "bukkit",
            Self::VanillaSnapshot => "vanilla-snapshot",
            Self::Vanilla => "vanilla",
            Self::NukkitX => "nukkitx",
            Self::Bedrock => "bedrock",
            Self::Velocity => "velocity",
            Self::BungeeCord => "bungeecord",
            Self::Lightfall => "lightfall",
            Self::Travertine => "travertine",
            Self::Pumpkin => "pumpkin",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_filename(filename: &str) -> Self {
        let filename = filename.to_ascii_lowercase();
        for (kind, keywords) in CORE_KEYWORDS {
            if keywords.iter().any(|keyword| filename.contains(keyword)) {
                return *kind;
            }
        }
        Self::Unknown
    }
}

const CORE_KEYWORDS: &[(CoreKind, &[&str])] = &[
    (CoreKind::ArclightForge, &["arclight-forge"]),
    (CoreKind::ArclightNeoForge, &["arclight-neoforge"]),
    (CoreKind::ArclightFabric, &["arclight-fabric"]),
    (CoreKind::PufferfishPurpur, &["pufferfish_purpur", "pufferfish-purpur"]),
    (CoreKind::VanillaSnapshot, &["vanilla-snapshot"]),
    (CoreKind::Youer, &["youer"]),
    (CoreKind::Mohist, &["mohist"]),
    (CoreKind::CatServer, &["catserver"]),
    (CoreKind::SpongeForge, &["spongeforge"]),
    (CoreKind::Banner, &["banner"]),
    (CoreKind::NeoForge, &["neoforge"]),
    (CoreKind::Forge, &["forge"]),
    (CoreKind::Quilt, &["quilt"]),
    (CoreKind::Fabric, &["fabric"]),
    (CoreKind::Pufferfish, &["pufferfish"]),
    (CoreKind::SpongeVanilla, &["spongevanilla"]),
    (CoreKind::Purpur, &["purpur"]),
    (CoreKind::Paper, &["paper"]),
    (CoreKind::Folia, &["folia"]),
    (CoreKind::Leaves, &["leaves"]),
    (CoreKind::Leaf, &["leaf"]),
    (CoreKind::Spigot, &["spigot"]),
    (CoreKind::Bukkit, &["bukkit"]),
    (CoreKind::NukkitX, &["nukkitx", "nukkit"]),
    (CoreKind::Bedrock, &["bedrock"]),
    (CoreKind::Velocity, &["velocity"]),
    (CoreKind::BungeeCord, &["bungeecord"]),
    (CoreKind::Lightfall, &["lightfall"]),
    (CoreKind::Travertine, &["travertine"]),
    (CoreKind::Pumpkin, &["pumpkin"]),
    (CoreKind::Vanilla, &["vanilla"]),
];

/// 从文件名或新版服务端检查报告投影的兼容核心元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreFileInfo {
    pub kind: CoreKind,
    pub core_version: Option<String>,
    pub minecraft_version: Option<String>,
    pub main_class: Option<String>,
}

impl CoreFileInfo {
    fn from_filename(filename: &str) -> Self {
        let kind = CoreKind::from_filename(filename);
        let minecraft_version = extract_minecraft_version(filename);
        let core_version = extract_version_tokens(filename)
            .into_iter()
            .rev()
            .find(|version| Some(version) != minecraft_version.as_ref());

        Self {
            kind,
            core_version,
            minecraft_version,
            main_class: None,
        }
    }

    fn from_report(filename: &str, report: &ServerInspectionReport) -> Self {
        let fallback = Self::from_filename(filename);
        let fallback_kind = fallback.kind;
        let fallback_core_version = fallback.core_version;
        let fallback_minecraft_version = fallback.minecraft_version;
        let implementation_outcome = server_implementation_outcome(&report.identity.implementation);
        let implementation_is_ambiguous = implementation_outcome == DetectionOutcome::Conflict;
        let kind = if matches!(
            implementation_outcome,
            DetectionOutcome::Selected | DetectionOutcome::Conflict
        ) {
            legacy_kind_from_report(report, fallback_kind)
        } else {
            fallback_kind
        };
        let core_version = selected_or_fallback(
            &report.identity.version,
            (!implementation_is_ambiguous)
                .then(|| {
                    manifest_main_attribute(report, "Implementation-Version")
                        .or(fallback_core_version)
                })
                .flatten(),
        );
        let minecraft_version = match report.minecraft.as_ref() {
            Some(minecraft) => selected_or_fallback(&minecraft.version, fallback_minecraft_version),
            None => fallback_minecraft_version,
        };

        Self {
            kind,
            core_version,
            minecraft_version,
            main_class: report.artifact.main_class.value.clone(),
        }
    }
}

fn has_detection_evidence<T>(detected: &Detected<T>) -> bool {
    detected.value.is_some() || !detected.alternatives.is_empty()
}

fn selected_or_fallback<T: Clone>(detected: &Detected<T>, fallback: Option<T>) -> Option<T> {
    if let Some(value) = detected.value.as_ref() {
        Some(value.clone())
    } else if detected.alternatives.is_empty() {
        fallback
    } else {
        None
    }
}

fn legacy_kind_from_report(report: &ServerInspectionReport, filename_kind: CoreKind) -> CoreKind {
    let Some(product) = report.identity.implementation.value.as_ref() else {
        // 多个高置信度实现候选时，不能用文件名擅自打破冲突。
        return CoreKind::Unknown;
    };
    match product.key.as_str() {
        "arclight" => {
            let kind = arclight_legacy_kind(&report.identity.ecosystems);
            if kind == CoreKind::Unknown {
                match filename_kind {
                    CoreKind::ArclightForge
                    | CoreKind::ArclightNeoForge
                    | CoreKind::ArclightFabric => filename_kind,
                    _ => CoreKind::Unknown,
                }
            } else {
                kind
            }
        }
        "youer" => CoreKind::Youer,
        "mohist" => CoreKind::Mohist,
        "neoforge" => CoreKind::NeoForge,
        "forge" => CoreKind::Forge,
        "fabric" | "legacy-fabric" => CoreKind::Fabric,
        "pufferfish" if filename_kind == CoreKind::PufferfishPurpur => CoreKind::PufferfishPurpur,
        "pufferfish" => CoreKind::Pufferfish,
        "spongevanilla" => CoreKind::SpongeVanilla,
        "purpur" => CoreKind::Purpur,
        "paper" => CoreKind::Paper,
        "folia" => CoreKind::Folia,
        "leaves" => CoreKind::Leaves,
        "leaf" => CoreKind::Leaf,
        "spigot" => CoreKind::Spigot,
        "craftbukkit" => CoreKind::Bukkit,
        "vanilla" if report.identity.release_channel.value == Some(ReleaseChannel::Snapshot) => {
            CoreKind::VanillaSnapshot
        }
        "vanilla"
            if !has_detection_evidence(&report.identity.release_channel)
                && filename_kind == CoreKind::VanillaSnapshot =>
        {
            CoreKind::VanillaSnapshot
        }
        "vanilla" => CoreKind::Vanilla,
        "velocity" => CoreKind::Velocity,
        "bungeecord" => CoreKind::BungeeCord,
        _ => CoreKind::Unknown,
    }
}

fn arclight_legacy_kind(ecosystems: &[Attributed<ServerEcosystem>]) -> CoreKind {
    let mut result = CoreKind::Unknown;
    for ecosystem in ecosystems {
        let candidate = match ecosystem.value {
            ServerEcosystem::NeoForge => CoreKind::ArclightNeoForge,
            ServerEcosystem::Fabric => CoreKind::ArclightFabric,
            ServerEcosystem::Forge => CoreKind::ArclightForge,
            _ => continue,
        };
        if arclight_kind_priority(candidate) > arclight_kind_priority(result) {
            result = candidate;
        }
    }
    result
}

fn arclight_kind_priority(kind: CoreKind) -> u8 {
    match kind {
        CoreKind::ArclightNeoForge => 3,
        CoreKind::ArclightFabric => 2,
        CoreKind::ArclightForge => 1,
        _ => 0,
    }
}

fn manifest_main_attribute(report: &ServerInspectionReport, key: &str) -> Option<String> {
    report
        .artifact
        .manifest
        .as_ref()?
        .main_attributes
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// 检查服务端核心文件名，无需从磁盘读取。
pub fn inspect_core_filename(filename: &str) -> CoreFileInfo {
    let filename = Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(filename);
    CoreFileInfo::from_filename(filename)
}

/// 将服务端检查报告投影为旧的核心信息模型。
pub fn inspect_core_file(path: &Path) -> Result<CoreFileInfo, CoreParseError> {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CoreParseError::InvalidPath { path: path.to_path_buf() })?;
    if !path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("jar"))
    {
        return Ok(CoreFileInfo::from_filename(filename));
    }

    let report = inspect_server_artifact(path, &InspectionOptions::default())
        .map_err(|source| CoreParseError::Inspection { path: path.to_path_buf(), source })?;
    Ok(CoreFileInfo::from_report(filename, &report))
}

/// 从文本中提取第一个 Minecraft 风格的版本提示（例如 `1.20.1`）。
pub fn extract_minecraft_version(input: &str) -> Option<String> {
    extract_version_tokens(input)
        .into_iter()
        .find(|version| version.starts_with("1."))
}

fn extract_version_tokens(input: &str) -> Vec<String> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() || (index > 0 && bytes[index - 1].is_ascii_digit()) {
            index += 1;
            continue;
        }

        let started_at = index;
        index += 1;
        while index < bytes.len() && (bytes[index].is_ascii_digit() || bytes[index] == b'.') {
            index += 1;
        }
        let token = input[started_at..index].trim_end_matches('.');
        if !token.is_empty() {
            tokens.push(token.to_string());
        }
    }

    tokens
}

/// 描述检查服务端核心文件时的失败。
#[derive(Debug)]
pub enum CoreParseError {
    InvalidPath {
        path: PathBuf,
    },
    Inspection {
        path: PathBuf,
        source: ServerInspectionError,
    },
}

impl fmt::Display for CoreParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath { path } => {
                write!(formatter, "core file path has no filename: {}", path.display())
            }
            Self::Inspection { path, source } => {
                write!(formatter, "could not inspect core file {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for CoreParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPath { .. } => None,
            Self::Inspection { source, .. } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use zip::write::FileOptions;

    use super::{
        CoreKind, CoreParseError, arclight_legacy_kind, extract_minecraft_version,
        inspect_core_file, inspect_core_filename,
    };
    use crate::provisioning::server_inspection::{Attributed, ServerEcosystem};

    fn write_test_jar(filename: &str, entries: &[(&str, &str)]) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "sealantern-core-provisioning-{}-{}-{}.jar",
            std::process::id(),
            timestamp,
            filename
        ));
        let file = File::create(&path).expect("create test JAR");
        let mut archive = zip::ZipWriter::new(file);
        for (name, content) in entries {
            archive
                .start_file(*name, FileOptions::<()>::default())
                .expect("create metadata entry");
            archive
                .write_all(content.as_bytes())
                .expect("write metadata entry");
        }
        archive.finish().expect("finish test JAR");
        path
    }

    #[test]
    fn filename_detection_prefers_neoforge_before_forge() {
        let parsed = inspect_core_filename("neoforge-1.20.6-20.6.119.jar");

        assert_eq!(parsed.kind, CoreKind::NeoForge);
        assert_eq!(parsed.minecraft_version.as_deref(), Some("1.20.6"));
        assert_eq!(parsed.core_version.as_deref(), Some("20.6.119"));
    }

    #[test]
    fn compatibility_projection_keeps_manifest_continuation_lines() {
        let path = write_test_jar(
            "unknown.jar",
            &[(
                "META-INF/MANIFEST.MF",
                "Manifest-Version: 1.0\r\nMain-Class: example.\r\n Main\r\nImplementation-Version: 20.6.119\r\n\r\n",
            )],
        );
        let parsed = inspect_core_file(&path).expect("inspect test JAR");
        std::fs::remove_file(&path).expect("remove test JAR");

        assert_eq!(parsed.main_class.as_deref(), Some("example.Main"));
        assert_eq!(parsed.core_version.as_deref(), Some("20.6.119"));
    }

    #[test]
    fn compatibility_projection_keeps_filename_fallback_without_metadata() {
        let path = write_test_jar("paper-1.21.4-123.jar", &[("empty", "")]);
        let parsed = inspect_core_file(&path).expect("inspect metadata-free JAR");
        std::fs::remove_file(&path).expect("remove test JAR");

        assert_eq!(parsed.kind, CoreKind::Paper);
        assert_eq!(parsed.minecraft_version.as_deref(), Some("1.21.4"));
        assert_eq!(parsed.core_version.as_deref(), Some("123"));
        assert_eq!(parsed.main_class, None);
    }

    #[test]
    fn minecraft_version_extraction_ignores_non_minecraft_versions() {
        assert_eq!(extract_minecraft_version("forge-47.2.0.jar"), None);
        assert_eq!(extract_minecraft_version("paper-1.21.4-123.jar"), Some("1.21.4".to_string()));
    }

    #[test]
    fn jar_manifest_keeps_neoforge_filename_over_legacy_forge_installer_class() {
        let path = write_test_jar(
            "neoforge-1.20.6-20.6.119-installer.jar",
            &[
                (
                    "META-INF/MANIFEST.MF",
                    "Manifest-Version: 1.0\r\nMain-Class: net.minecraftforge.installer.SimpleInstaller\r\nImplementation-Version: 20.6.119\r\n\r\n",
                ),
                ("metadata.json", r#"{"neoforge":"20.6.119"}"#),
            ],
        );

        let parsed = inspect_core_file(&path).expect("inspect test JAR");
        std::fs::remove_file(&path).expect("remove test JAR");

        assert_eq!(parsed.kind, CoreKind::NeoForge);
        assert_eq!(
            parsed.main_class.as_deref(),
            Some("net.minecraftforge.installer.SimpleInstaller")
        );
        assert_eq!(parsed.core_version.as_deref(), Some("20.6.119"));
    }

    #[test]
    fn compatibility_projection_prefers_paperclip_content_over_filename() {
        let path = write_test_jar(
            "paper-26.2.jar",
            &[
                ("META-INF/MANIFEST.MF", "Main-Class: io.papermc.paperclip.Main\r\n\r\n"),
                ("META-INF/versions.list", "hash\t26.2\t26.2/purpur-26.2.jar\n"),
                (
                    "META-INF/libraries.list",
                    "hash\torg.purpurmc.purpur:purpur-api:26.2.build.1-stable\tpurpur-api.jar\n",
                ),
            ],
        );
        let parsed = inspect_core_file(&path).expect("inspect Paperclip fixture");
        std::fs::remove_file(&path).expect("remove Paperclip fixture");

        assert_eq!(parsed.kind, CoreKind::Purpur);
        assert_eq!(parsed.minecraft_version.as_deref(), Some("26.2"));
        assert_eq!(parsed.core_version.as_deref(), Some("26.2.build.1-stable"));
    }

    #[test]
    fn compatibility_projection_maps_arclight_by_detected_ecosystem() {
        let path = write_test_jar(
            "arclight.jar",
            &[
                (
                    "META-INF/MANIFEST.MF",
                    "Main-Class: io.izzel.arclight.server.Launcher\r\nImplementation-Title: Arclight\r\nImplementation-Version: arclight-1.21.1-1.0.2\r\n\r\n",
                ),
                (
                    "arclight-server-launch.properties",
                    "launch.mainClass=io.izzel.arclight.boot.neoforge.application.Main_NeoForge\n",
                ),
            ],
        );
        let parsed = inspect_core_file(&path).expect("inspect Arclight fixture");
        std::fs::remove_file(&path).expect("remove Arclight fixture");

        assert_eq!(parsed.kind, CoreKind::ArclightNeoForge);
        assert_eq!(parsed.minecraft_version.as_deref(), Some("1.21.1"));
    }

    #[test]
    fn arclight_ecosystem_priority_is_independent_of_input_order() {
        let ecosystems = [
            Attributed {
                value: ServerEcosystem::Forge,
                confidence: 95,
                evidence: Vec::new(),
            },
            Attributed {
                value: ServerEcosystem::Fabric,
                confidence: 95,
                evidence: Vec::new(),
            },
            Attributed {
                value: ServerEcosystem::NeoForge,
                confidence: 95,
                evidence: Vec::new(),
            },
        ];

        assert_eq!(arclight_legacy_kind(&ecosystems), CoreKind::ArclightNeoForge);
    }

    #[test]
    fn compatibility_projection_does_not_masquerade_new_products() {
        let path = write_test_jar(
            "paper-26.2.jar",
            &[(
                "META-INF/MANIFEST.MF",
                "Main-Class: com.velocitypowered.proxy.Velocity\r\nImplementation-Title: Velocity-CTD\r\nImplementation-Version: 4.1.0-SNAPSHOT\r\n\r\n",
            )],
        );
        let parsed = inspect_core_file(&path).expect("inspect Velocity-CTD fixture");
        std::fs::remove_file(&path).expect("remove Velocity-CTD fixture");

        assert_eq!(parsed.kind, CoreKind::Unknown);
        assert_eq!(parsed.main_class.as_deref(), Some("com.velocitypowered.proxy.Velocity"));
        assert_eq!(parsed.core_version.as_deref(), Some("4.1.0-SNAPSHOT"));
    }

    #[test]
    fn compatibility_projection_prefers_detected_release_channel_over_filename() {
        let path = write_test_jar(
            "vanilla-snapshot.jar",
            &[
                ("META-INF/MANIFEST.MF", "Main-Class: net.minecraft.bundler.Main\r\n\r\n"),
                (
                    "version.json",
                    r#"{"id":"26.2","name":"26.2","world_version":4903,"stable":true}"#,
                ),
            ],
        );
        let parsed = inspect_core_file(&path).expect("inspect Vanilla fixture");
        std::fs::remove_file(&path).expect("remove Vanilla fixture");

        assert_eq!(parsed.kind, CoreKind::Vanilla);
        assert_eq!(parsed.minecraft_version.as_deref(), Some("26.2"));
        assert_eq!(parsed.core_version.as_deref(), Some("26.2"));
    }

    #[test]
    fn compatibility_projection_keeps_conflicting_products_unknown() {
        let path = write_test_jar(
            "paper-26.2.jar",
            &[
                ("META-INF/MANIFEST.MF", "Main-Class: io.papermc.paperclip.Main\r\n\r\n"),
                ("META-INF/versions.list", "hash\t26.2\t26.2/purpur-26.2.jar\n"),
                (
                    "META-INF/libraries.list",
                    "hash\tio.papermc.paper:paper-api:26.2.build.1-stable\tpaper-api.jar\n",
                ),
            ],
        );
        let parsed = inspect_core_file(&path).expect("inspect conflicting fixture");
        std::fs::remove_file(&path).expect("remove conflicting fixture");

        assert_eq!(parsed.kind, CoreKind::Unknown);
        assert_eq!(parsed.core_version, None);
        assert_eq!(parsed.minecraft_version.as_deref(), Some("26.2"));
    }

    #[test]
    fn compatibility_projection_wraps_new_inspection_errors() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "sealantern-core-provisioning-{}-{timestamp}-invalid.jar",
            std::process::id()
        ));
        std::fs::write(&path, b"not a ZIP archive").expect("write invalid JAR");

        let error = inspect_core_file(&path).expect_err("reject invalid JAR");
        std::fs::remove_file(&path).expect("remove invalid JAR");

        assert!(matches!(error, CoreParseError::Inspection { .. }));
        assert!(error.source().is_some());
    }
}
