mod archive;
mod error;
mod evidence;
#[path = "formats/lib.rs"]
mod formats;
mod model;
mod resolver;

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;

use sha2::{Digest, Sha256};

pub use error::ServerInspectionError;
pub use model::*;

use evidence::{EvidenceCollector, NewEvidence};
use formats::manifest::ParsedManifest;
use formats::mojang_version::MojangVersionDocument;
use resolver::{resolve, DetectionClaim};

const MANIFEST_ENTRY: &str = "META-INF/MANIFEST.MF";
const MOJANG_VERSION_ENTRY: &str = "version.json";

/// 控制静态检查的资源预算；检查过程不会执行 JAR、脚本或 shell 展开。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectionOptions {
    pub max_archive_entries: usize,
    pub max_metadata_entry_bytes: u64,
    pub max_total_metadata_bytes: u64,
    /// 后续嵌套归档检测的单归档上限；设为 0 可禁用嵌套归档读取。
    pub max_nested_archive_bytes: u64,
    /// 根归档深度为 0；设为 0 时仅检查根归档。
    pub max_archive_depth: usize,
    pub compute_sha256: bool,
}

impl Default for InspectionOptions {
    fn default() -> Self {
        Self {
            max_archive_entries: 50_000,
            max_metadata_entry_bytes: 4 * 1024 * 1024,
            max_total_metadata_bytes: 32 * 1024 * 1024,
            max_nested_archive_bytes: 128 * 1024 * 1024,
            max_archive_depth: 2,
            compute_sha256: false,
        }
    }
}

/// 检查单个服务端文件或安装目录，不执行其中的任何内容。
pub fn inspect_server_artifact(
    path: &Path,
    options: &InspectionOptions,
) -> Result<ServerInspectionReport, ServerInspectionError> {
    validate_options(options)?;
    let metadata = fs::metadata(path)
        .map_err(|source| ServerInspectionError::Metadata { path: path.to_path_buf(), source })?;
    if !metadata.is_file() && !metadata.is_dir() {
        return Err(ServerInspectionError::UnsupportedSubject { path: path.to_path_buf() });
    }

    let subject_kind = if metadata.is_dir() {
        InspectionSubjectKind::Directory
    } else {
        InspectionSubjectKind::File
    };
    let fingerprint = if metadata.is_file() && options.compute_sha256 {
        Some(calculate_sha256(path)?)
    } else {
        None
    };
    let subject = InspectionSubject {
        path: path.to_path_buf(),
        kind: subject_kind,
        size_bytes: metadata.is_file().then_some(metadata.len()),
        modified_at_unix_secs: metadata
            .modified()
            .ok()
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs()),
        fingerprint,
    };

    let mut evidence = EvidenceCollector::default();
    let mut diagnostics = Vec::new();
    let mut format_claims = Vec::new();
    let mut roles = Vec::new();
    let mut launches = Vec::new();
    let mut artifact = ArtifactInfo {
        format: Detected::default(),
        roles: Vec::new(),
        main_class: Detected::default(),
        premain_class: Detected::default(),
        agent_class: Detected::default(),
        automatic_module_name: Detected::default(),
        manifest: None,
    };
    let mut minecraft = None;
    let mut java = JavaRequirementInfo::default();

    if metadata.is_dir() {
        let format_evidence = evidence.push(NewEvidence {
            detector: "subject-kind",
            source: EvidenceSource::FileMetadata,
            location: EvidenceLocation::path(path.to_path_buf()),
            target: DetectionTarget::ArtifactFormat,
            candidate: "directory".to_string(),
            weight: 100,
            correlation_group: "file-metadata",
        });
        format_claims.push(claim(ArtifactFormat::Directory, format_evidence, 100, "file-metadata"));

        let role_evidence = evidence.push(NewEvidence {
            detector: "subject-kind",
            source: EvidenceSource::DirectoryLayout,
            location: EvidenceLocation::path(path.to_path_buf()),
            target: DetectionTarget::ArtifactRole,
            candidate: "installation_directory".to_string(),
            weight: 100,
            correlation_group: "directory-layout",
        });
        roles.push(Attributed {
            value: ArtifactRole::InstallationDirectory,
            confidence: 100,
            evidence: vec![role_evidence],
        });
    } else {
        let format = format_from_path(path);
        let format_weight = if format == ArtifactFormat::Unknown {
            50
        } else {
            85
        };
        let format_evidence = evidence.push(NewEvidence {
            detector: "file-extension",
            source: EvidenceSource::FileName,
            location: EvidenceLocation::path(path.to_path_buf()),
            target: DetectionTarget::ArtifactFormat,
            candidate: format_name(format).to_string(),
            weight: format_weight,
            correlation_group: "file-name",
        });
        format_claims.push(claim(format, format_evidence, format_weight, "file-name"));

        if format == ArtifactFormat::Jar {
            let archive_metadata = archive::read_metadata(path, options)?;
            diagnostics.extend(archive_metadata.diagnostics);
            let archive_evidence = evidence.push(NewEvidence {
                detector: "jar-container",
                source: EvidenceSource::JarEntry,
                location: EvidenceLocation::path(path.to_path_buf()),
                target: DetectionTarget::ArtifactFormat,
                candidate: "jar".to_string(),
                weight: 100,
                correlation_group: "archive-container",
            });
            format_claims.push(claim(
                ArtifactFormat::Jar,
                archive_evidence,
                100,
                "archive-container",
            ));

            if let Some(bytes) = archive_metadata.manifest {
                let parsed = formats::manifest::parse(&bytes);
                if parsed.used_lossy_utf8 {
                    diagnostics.push(InspectionDiagnostic {
                        severity: DiagnosticSeverity::Warning,
                        code: "manifest_invalid_utf8".to_string(),
                        message: format!(
                            "manifest in {} contains invalid UTF-8; invalid bytes were replaced",
                            path.display()
                        ),
                        evidence: Vec::new(),
                    });
                }
                apply_manifest(
                    path,
                    &parsed,
                    &mut artifact,
                    &mut roles,
                    &mut launches,
                    &mut evidence,
                );
                artifact.manifest = Some(parsed.summary);
            }

            if let Some(bytes) = archive_metadata.mojang_version {
                match formats::mojang_version::parse(&bytes) {
                    Ok(Some(document)) => {
                        let (minecraft_info, java_info, mut version_diagnostics) =
                            apply_mojang_version(path, document, &mut evidence);
                        minecraft = Some(minecraft_info);
                        java = java_info;
                        diagnostics.append(&mut version_diagnostics);
                    }
                    Ok(None) => diagnostics.push(InspectionDiagnostic {
                        severity: DiagnosticSeverity::Info,
                        code: "unrecognized_version_json".to_string(),
                        message: format!(
                            "version.json in {} does not match the Mojang version metadata shape",
                            path.display()
                        ),
                        evidence: Vec::new(),
                    }),
                    Err(source) => diagnostics.push(InspectionDiagnostic {
                        severity: DiagnosticSeverity::Warning,
                        code: "invalid_version_json".to_string(),
                        message: format!(
                            "could not parse version.json in {}: {source}",
                            path.display()
                        ),
                        evidence: Vec::new(),
                    }),
                }
            }
        }
    }

    artifact.format = resolve(format_claims);
    artifact.roles = roles;

    Ok(ServerInspectionReport {
        schema_version: SERVER_INSPECTION_SCHEMA_VERSION,
        subject,
        artifact,
        identity: ServerIdentityInfo::default(),
        minecraft,
        java,
        components: Vec::new(),
        launches,
        evidence: evidence.into_entries(),
        diagnostics,
    })
}

fn validate_options(options: &InspectionOptions) -> Result<(), ServerInspectionError> {
    if options.max_archive_entries == 0 {
        return Err(ServerInspectionError::InvalidOptions {
            detail: "max_archive_entries must be greater than zero",
        });
    }
    if options.max_metadata_entry_bytes == 0 {
        return Err(ServerInspectionError::InvalidOptions {
            detail: "max_metadata_entry_bytes must be greater than zero",
        });
    }
    if options.max_total_metadata_bytes == 0 {
        return Err(ServerInspectionError::InvalidOptions {
            detail: "max_total_metadata_bytes must be greater than zero",
        });
    }
    Ok(())
}

fn apply_manifest(
    path: &Path,
    manifest: &ParsedManifest,
    artifact: &mut ArtifactInfo,
    roles: &mut Vec<Attributed<ArtifactRole>>,
    launches: &mut Vec<Attributed<LaunchProfile>>,
    evidence: &mut EvidenceCollector,
) {
    artifact.main_class =
        detect_manifest_value(path, manifest, "Main-Class", DetectionTarget::MainClass, evidence);
    artifact.premain_class = detect_manifest_value(
        path,
        manifest,
        "Premain-Class",
        DetectionTarget::PremainClass,
        evidence,
    );
    artifact.agent_class =
        detect_manifest_value(path, manifest, "Agent-Class", DetectionTarget::AgentClass, evidence);
    artifact.automatic_module_name = detect_manifest_value(
        path,
        manifest,
        "Automatic-Module-Name",
        DetectionTarget::AutomaticModuleName,
        evidence,
    );

    if let Some(main_class) = artifact.main_class.value.as_ref() {
        let role_evidence = evidence.push(NewEvidence {
            detector: "jar-manifest",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, "Main-Class"),
            target: DetectionTarget::ArtifactRole,
            candidate: "runnable".to_string(),
            weight: 95,
            correlation_group: "manifest-main",
        });
        roles.push(Attributed {
            value: ArtifactRole::Runnable,
            confidence: 95,
            evidence: vec![role_evidence],
        });
        let launch_evidence = evidence.push(NewEvidence {
            detector: "jar-manifest",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, "Main-Class"),
            target: DetectionTarget::LaunchProfile,
            candidate: format!("java -jar {} ({main_class})", path.display()),
            weight: 95,
            correlation_group: "manifest-main",
        });
        launches.push(Attributed {
            value: LaunchProfile {
                id: "manifest-main".to_string(),
                platform: LaunchPlatform::Any,
                working_directory: path
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                    .map(Path::to_path_buf),
                target: LaunchTarget::Jar { path: path.to_path_buf() },
                jvm_arguments: Vec::new(),
                program_arguments: Vec::new(),
                required_java_major: None,
            },
            confidence: 95,
            evidence: vec![launch_evidence],
        });
    }
}

fn detect_manifest_value(
    path: &Path,
    manifest: &ParsedManifest,
    field: &'static str,
    target: DetectionTarget,
    evidence: &mut EvidenceCollector,
) -> Detected<String> {
    let Some(value) = manifest.main_value(field).map(str::to_string) else {
        return Detected::default();
    };
    let evidence_id = evidence.push(NewEvidence {
        detector: "jar-manifest",
        source: EvidenceSource::ManifestMain,
        location: manifest_location(path, field),
        target,
        candidate: value.clone(),
        weight: 100,
        correlation_group: "manifest-main",
    });
    resolve(vec![claim(value, evidence_id, 100, "manifest-main")])
}

fn apply_mojang_version(
    path: &Path,
    document: MojangVersionDocument,
    evidence: &mut EvidenceCollector,
) -> (MinecraftVersionInfo, JavaRequirementInfo, Vec<InspectionDiagnostic>) {
    let mut version_claims = Vec::new();
    let mut version_evidence = Vec::new();
    if let Some(id) = document.id.as_ref() {
        let evidence_id = json_field_evidence(
            path,
            "id",
            id,
            DetectionTarget::MinecraftVersion,
            95,
            "mojang-version-json",
            evidence,
        );
        version_evidence.push(evidence_id);
        version_claims.push(claim(id.to_string(), evidence_id, 95, "mojang-version-json"));
    }
    if let Some(name) = document.name.as_ref() {
        let evidence_id = json_field_evidence(
            path,
            "name",
            name,
            DetectionTarget::MinecraftVersion,
            90,
            "mojang-version-json",
            evidence,
        );
        version_evidence.push(evidence_id);
        version_claims.push(claim(name.to_string(), evidence_id, 90, "mojang-version-json"));
    }
    let version = resolve(version_claims);

    record_minecraft_metadata(path, &document, &mut version_evidence, evidence);

    let required_major = document
        .java_version
        .map_or_else(Detected::default, |major| {
            let evidence_id = json_field_evidence(
                path,
                "java_version",
                &major.to_string(),
                DetectionTarget::JavaMajor,
                90,
                "mojang-java-version",
                evidence,
            );
            version_evidence.push(evidence_id);
            resolve(vec![claim(major, evidence_id, 90, "mojang-java-version")])
        });
    let runtime_component =
        document
            .java_component
            .as_ref()
            .map_or_else(Detected::default, |component| {
                let evidence_id = json_field_evidence(
                    path,
                    "java_component",
                    component,
                    DetectionTarget::JavaRuntimeComponent,
                    90,
                    "mojang-java-component",
                    evidence,
                );
                version_evidence.push(evidence_id);
                resolve(vec![claim(component.clone(), evidence_id, 90, "mojang-java-component")])
            });

    let diagnostics = if version.value.is_none() && version.alternatives.len() > 1 {
        vec![InspectionDiagnostic {
            severity: DiagnosticSeverity::Warning,
            code: "conflicting_minecraft_versions".to_string(),
            message: format!(
                "Mojang version metadata in {} contains conflicting id and name values",
                path.display()
            ),
            evidence: version
                .alternatives
                .iter()
                .flat_map(|candidate| candidate.evidence.iter().copied())
                .collect(),
        }]
    } else {
        Vec::new()
    };

    (
        MinecraftVersionInfo {
            version,
            id: document.id,
            name: document.name,
            world_version: document.world_version,
            series_id: document.series_id,
            protocol_version: document.protocol_version,
            pack_version: document.pack_version,
            build_time: document.build_time,
            java_component: document.java_component,
            java_version: document.java_version,
            stable: document.stable,
            use_editor: document.use_editor,
            extra: document.extra,
            evidence: version_evidence,
        },
        JavaRequirementInfo { required_major, runtime_component },
        diagnostics,
    )
}

fn record_minecraft_metadata(
    path: &Path,
    document: &MojangVersionDocument,
    version_evidence: &mut Vec<EvidenceId>,
    evidence: &mut EvidenceCollector,
) {
    let mut record = |field: &str, candidate: String| {
        version_evidence.push(json_field_evidence(
            path,
            field,
            &candidate,
            DetectionTarget::MinecraftVersion,
            95,
            "mojang-version-json",
            evidence,
        ));
    };
    if let Some(value) = document.world_version {
        record("world_version", value.to_string());
    }
    if let Some(value) = document.series_id.as_ref() {
        record("series_id", value.clone());
    }
    if let Some(value) = document.protocol_version {
        record("protocol_version", value.to_string());
    }
    if let Some(value) = document.pack_version.as_ref() {
        record("pack_version", format_pack_version(value));
    }
    if let Some(value) = document.build_time.as_ref() {
        record("build_time", value.clone());
    }
    if let Some(value) = document.stable {
        record("stable", value.to_string());
    }
    if let Some(value) = document.use_editor {
        record("use_editor", value.to_string());
    }
}

fn format_pack_version(pack: &MinecraftPackVersion) -> String {
    let resource = pack
        .resource
        .map(|version| format!("{}.{}", version.major, version.minor))
        .unwrap_or_else(|| "unknown".to_string());
    let data = pack
        .data
        .map(|version| format!("{}.{}", version.major, version.minor))
        .unwrap_or_else(|| "unknown".to_string());
    format!("resource={resource},data={data}")
}

fn manifest_location(path: &Path, field: &str) -> EvidenceLocation {
    EvidenceLocation {
        path: path.to_path_buf(),
        archive_entry: Some(MANIFEST_ENTRY.to_string()),
        manifest_section: None,
        field: Some(field.to_string()),
    }
}

#[allow(clippy::too_many_arguments)]
fn json_field_evidence(
    path: &Path,
    field: &str,
    candidate: &str,
    target: DetectionTarget,
    weight: u8,
    correlation_group: &'static str,
    evidence: &mut EvidenceCollector,
) -> EvidenceId {
    evidence.push(NewEvidence {
        detector: "mojang-version-json",
        source: EvidenceSource::JsonField,
        location: EvidenceLocation {
            path: path.to_path_buf(),
            archive_entry: Some(MOJANG_VERSION_ENTRY.to_string()),
            manifest_section: None,
            field: Some(field.to_string()),
        },
        target,
        candidate: candidate.to_string(),
        weight,
        correlation_group,
    })
}

fn claim<T>(
    value: T,
    evidence: EvidenceId,
    weight: u8,
    correlation_group: &str,
) -> DetectionClaim<T> {
    DetectionClaim {
        value,
        evidence,
        weight,
        correlation_group: correlation_group.to_string(),
    }
}

fn format_from_path(path: &Path) -> ArtifactFormat {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("jar") => ArtifactFormat::Jar,
        Some("zip") => ArtifactFormat::Zip,
        Some("bat" | "cmd" | "sh" | "ps1") => ArtifactFormat::Script,
        Some("exe") => ArtifactFormat::Executable,
        _ => ArtifactFormat::Unknown,
    }
}

fn format_name(format: ArtifactFormat) -> &'static str {
    match format {
        ArtifactFormat::Directory => "directory",
        ArtifactFormat::Jar => "jar",
        ArtifactFormat::Zip => "zip",
        ArtifactFormat::Script => "script",
        ArtifactFormat::Executable => "executable",
        ArtifactFormat::Unknown => "unknown",
    }
}

fn calculate_sha256(path: &Path) -> Result<ArtifactFingerprint, ServerInspectionError> {
    let mut file = File::open(path).map_err(|source| ServerInspectionError::FingerprintRead {
        path: path.to_path_buf(),
        source,
    })?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read =
            file.read(&mut buffer)
                .map_err(|source| ServerInspectionError::FingerprintRead {
                    path: path.to_path_buf(),
                    source,
                })?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }

    Ok(ArtifactFingerprint {
        algorithm: FingerprintAlgorithm::Sha256,
        value: format!("{:x}", digest.finalize()),
    })
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use zip::write::FileOptions;

    use super::{
        inspect_server_artifact, ArtifactFormat, ArtifactRole, InspectionOptions,
        InspectionSubjectKind, LaunchTarget,
    };

    fn temporary_path(suffix: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sealantern-server-inspection-{}-{timestamp}-{suffix}",
            std::process::id()
        ))
    }

    fn write_test_jar(path: &Path, manifest: &str, version_json: &str) {
        let file = File::create(path).expect("create test JAR");
        let mut archive = zip::ZipWriter::new(file);
        archive
            .start_file("META-INF/MANIFEST.MF", FileOptions::<()>::default())
            .expect("create manifest entry");
        archive
            .write_all(manifest.as_bytes())
            .expect("write manifest");
        archive
            .start_file("version.json", FileOptions::<()>::default())
            .expect("create version entry");
        archive
            .write_all(version_json.as_bytes())
            .expect("write version JSON");
        archive.finish().expect("finish test JAR");
    }

    #[test]
    fn inspects_manifest_and_current_mojang_version_metadata() {
        let path = temporary_path("server.jar");
        write_test_jar(
            &path,
            "Manifest-Version: 1.0\r\nMain-Class: net.minecraft.bundler.Main\r\nImplementation-Title: Minecraft\r\n\r\nName: product\r\nImplementation-Version: named-only\r\n\r\n",
            r#"{"id":"26.2","name":"26.2","world_version":4903,"series_id":"main","protocol_version":776,"pack_version":{"resource_major":88,"resource_minor":0,"data_major":107,"data_minor":1},"build_time":"2026-06-16T12:01:27+00:00","java_component":"java-runtime-epsilon","java_version":25,"stable":true,"use_editor":false,"vendor_field":"preserved"}"#,
        );

        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect test JAR");
        fs::remove_file(&path).expect("remove test JAR");

        assert_eq!(report.artifact.format.value, Some(ArtifactFormat::Jar));
        assert_eq!(report.artifact.format.confidence, 100);
        assert_eq!(report.artifact.main_class.value.as_deref(), Some("net.minecraft.bundler.Main"));
        assert!(report
            .artifact
            .roles
            .iter()
            .any(|role| role.value == ArtifactRole::Runnable));
        assert_eq!(
            report
                .artifact
                .manifest
                .as_ref()
                .expect("manifest")
                .sections
                .len(),
            1
        );
        assert!(matches!(report.launches[0].value.target, LaunchTarget::Jar { .. }));

        let minecraft = report.minecraft.expect("Minecraft metadata");
        assert_eq!(minecraft.version.value.as_deref(), Some("26.2"));
        assert_eq!(minecraft.world_version, Some(4903));
        assert_eq!(
            minecraft
                .pack_version
                .expect("pack version")
                .data
                .expect("data format")
                .minor,
            1
        );
        assert_eq!(
            minecraft
                .extra
                .get("vendor_field")
                .and_then(|value| value.as_str()),
            Some("preserved")
        );
        assert_eq!(report.java.required_major.value, Some(25));
        assert_eq!(report.java.runtime_component.value.as_deref(), Some("java-runtime-epsilon"));
    }

    #[test]
    fn skips_oversized_optional_metadata_without_losing_the_report() {
        let path = temporary_path("limited.jar");
        write_test_jar(
            &path,
            "Main-Class: example.Main\r\n\r\n",
            &format!(r#"{{"id":"26.2","world_version":4903,"padding":"{}"}}"#, "x".repeat(256)),
        );
        let options = InspectionOptions {
            max_metadata_entry_bytes: 128,
            ..InspectionOptions::default()
        };

        let report = inspect_server_artifact(&path, &options).expect("inspect bounded JAR");
        fs::remove_file(&path).expect("remove test JAR");

        assert_eq!(report.artifact.main_class.value.as_deref(), Some("example.Main"));
        assert!(report.minecraft.is_none());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "metadata_entry_too_large"));
    }

    #[test]
    fn directories_are_reported_without_scanning_their_contents() {
        let path = temporary_path("directory");
        fs::create_dir(&path).expect("create test directory");

        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect directory");
        fs::remove_dir(&path).expect("remove test directory");

        assert_eq!(report.subject.kind, InspectionSubjectKind::Directory);
        assert_eq!(report.artifact.format.value, Some(ArtifactFormat::Directory));
        assert_eq!(report.artifact.roles[0].value, ArtifactRole::InstallationDirectory);
    }

    #[test]
    fn optionally_calculates_a_sha256_fingerprint() {
        let path = temporary_path("payload.bin");
        fs::write(&path, b"abc").expect("write test file");
        let options = InspectionOptions {
            compute_sha256: true,
            ..InspectionOptions::default()
        };

        let report = inspect_server_artifact(&path, &options).expect("inspect test file");
        fs::remove_file(&path).expect("remove test file");

        assert_eq!(
            report.subject.fingerprint.expect("fingerprint").value,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
