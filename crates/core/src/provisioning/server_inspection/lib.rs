mod archive;
#[path = "detectors/lib.rs"]
mod detectors;
mod directory;
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
pub(crate) use resolver::{detection_outcome, server_implementation_outcome, DetectionOutcome};
use resolver::{resolve, DetectionClaim};

const MANIFEST_ENTRY: &str = "META-INF/MANIFEST.MF";
const MOJANG_VERSION_ENTRY: &str = "version.json";
const SERVER_INSPECTION_TARGET: &str = "sealantern.core.provisioning.server_inspection";
const MINIMUM_SERVER_IMPLEMENTATION_CONFIDENCE: u8 = 50;

/// 控制静态检查的资源预算；检查过程不会执行 JAR、脚本或 shell 展开。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InspectionOptions {
    pub max_archive_entries: usize,
    /// 根目录中最多接受的服务端 JAR 数量，避免无界纳入普通库文件。
    pub max_root_archives: usize,
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
            max_root_archives: 64,
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
    tracing::debug!(
        target: SERVER_INSPECTION_TARGET,
        path = %path.display(),
        compute_sha256 = options.compute_sha256,
        max_archive_entries = options.max_archive_entries,
        max_archive_depth = options.max_archive_depth,
        "starting server artifact inspection"
    );
    let result = inspect_server_artifact_inner(path, options);
    match &result {
        Ok(report) => {
            let implementation = report
                .identity
                .implementation
                .value
                .as_ref()
                .map(|product| product.key.as_str())
                .unwrap_or("unknown");
            tracing::debug!(
                target: SERVER_INSPECTION_TARGET,
                path = %path.display(),
                subject_kind = ?report.subject.kind,
                implementation,
                diagnostics = report.diagnostics.len(),
                launches = report.launches.len(),
                "server artifact inspection completed"
            );
        }
        Err(error) => tracing::warn!(
            target: SERVER_INSPECTION_TARGET,
            path = %path.display(),
            error = %error,
            "server artifact inspection failed"
        ),
    }
    result
}

fn inspect_server_artifact_inner(
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
    let mut identity = ServerIdentityInfo::default();
    let mut components = Vec::new();

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

        let mut directory_metadata = directory::read_metadata(path, options)?;
        diagnostics.append(&mut directory_metadata.diagnostics);
        let mut root_version_seen = false;
        for root_archive in &directory_metadata.root_archives {
            let archive_path = path.join(&root_archive.relative_path);
            let Some(bytes) = root_archive.metadata.mojang_version.as_deref() else {
                continue;
            };
            match formats::mojang_version::parse(bytes) {
                Ok(Some(document)) => {
                    let (root_minecraft, root_java, mut version_diagnostics) =
                        apply_mojang_version(&archive_path, document, &mut evidence);
                    diagnostics.append(&mut version_diagnostics);
                    if !root_version_seen {
                        minecraft = Some(root_minecraft);
                        java = root_java;
                        root_version_seen = true;
                    } else {
                        diagnostics.push(InspectionDiagnostic {
                            severity: DiagnosticSeverity::Warning,
                            code: "multiple_root_version_json".to_string(),
                            message: format!(
                                "multiple root archives in {} provide version.json; the first recognized document was selected",
                                path.display()
                            ),
                            evidence: Vec::new(),
                        });
                    }
                }
                Ok(None) => diagnostics.push(InspectionDiagnostic {
                    severity: DiagnosticSeverity::Info,
                    code: "unrecognized_root_version_json".to_string(),
                    message: format!(
                        "version.json in {} does not match the Mojang version metadata shape",
                        archive_path.display()
                    ),
                    evidence: Vec::new(),
                }),
                Err(source) => diagnostics.push(InspectionDiagnostic {
                    severity: DiagnosticSeverity::Warning,
                    code: "invalid_root_version_json".to_string(),
                    message: format!(
                        "could not parse version.json in {}: {source}",
                        archive_path.display()
                    ),
                    evidence: Vec::new(),
                }),
            }
        }
        let detector_output = detectors::detect_directory(
            path,
            &directory_metadata,
            minecraft.as_ref(),
            Some(&java.required_major),
            &mut evidence,
        );
        apply_detector_output(
            detector_output,
            &mut identity,
            &mut minecraft,
            &mut java,
            &mut roles,
            &mut components,
            &mut launches,
            &mut diagnostics,
        );
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
            let mut archive_metadata = archive::read_metadata(path, options)?;
            diagnostics.append(&mut archive_metadata.diagnostics);
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

            if let Some(bytes) = archive_metadata.manifest.as_deref() {
                let parsed = formats::manifest::parse(bytes);
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

            if let Some(bytes) = archive_metadata.mojang_version.as_deref() {
                match formats::mojang_version::parse(bytes) {
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

            let detector_output = detectors::detect_jar(
                path,
                &artifact,
                &archive_metadata,
                minecraft.as_ref(),
                Some(&java.required_major),
                &mut evidence,
            );
            apply_detector_output(
                detector_output,
                &mut identity,
                &mut minecraft,
                &mut java,
                &mut roles,
                &mut components,
                &mut launches,
                &mut diagnostics,
            );
        }
    }

    artifact.format = resolve(format_claims);
    artifact.roles = roles;

    Ok(ServerInspectionReport {
        schema_version: SERVER_INSPECTION_SCHEMA_VERSION,
        subject,
        artifact,
        identity,
        minecraft,
        java,
        components,
        launches,
        evidence: evidence.into_entries(),
        diagnostics,
    })
}

#[allow(clippy::too_many_arguments)]
fn apply_detector_output(
    detector_output: detectors::DetectorOutput,
    identity: &mut ServerIdentityInfo,
    minecraft: &mut Option<MinecraftVersionInfo>,
    java: &mut JavaRequirementInfo,
    roles: &mut Vec<Attributed<ArtifactRole>>,
    components: &mut Vec<Attributed<ServerComponent>>,
    launches: &mut Vec<Attributed<LaunchProfile>>,
    diagnostics: &mut Vec<InspectionDiagnostic>,
) {
    *identity = detector_output.identity;
    if detector_output.minecraft_version.confidence > 0 {
        let minecraft_info = minecraft.get_or_insert_with(MinecraftVersionInfo::default);
        for evidence_id in detector_output.minecraft_version.evidence.iter().chain(
            detector_output
                .minecraft_version
                .alternatives
                .iter()
                .flat_map(|candidate| candidate.evidence.iter()),
        ) {
            if !minecraft_info.evidence.contains(evidence_id) {
                minecraft_info.evidence.push(*evidence_id);
            }
        }
        minecraft_info.version = detector_output.minecraft_version;
    }
    roles.extend(detector_output.roles);
    *components = detector_output.components;
    launches.extend(detector_output.launches);
    java.required_major = detector_output.java_major;
    for diagnostic in detector_output.diagnostics {
        diagnostics.retain(|existing| existing.code != diagnostic.code);
        diagnostics.push(diagnostic);
    }
}

fn validate_options(options: &InspectionOptions) -> Result<(), ServerInspectionError> {
    if options.max_archive_entries == 0 {
        return Err(ServerInspectionError::InvalidOptions {
            detail: "max_archive_entries must be greater than zero",
        });
    }
    if options.max_root_archives == 0 {
        return Err(ServerInspectionError::InvalidOptions {
            detail: "max_root_archives must be greater than zero",
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
        inspect_server_artifact, ArtifactFormat, ArtifactRole, DetectionTarget, InspectionOptions,
        InspectionSubjectKind, LaunchPlatform, LaunchTarget, ReleaseChannel, ServerCategory,
        ServerComponentKind, ServerEcosystem, ServerInspectionError,
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
        write_test_jar_entries(
            path,
            &[("META-INF/MANIFEST.MF", manifest), ("version.json", version_json)],
        );
    }

    fn write_test_jar_entries(path: &Path, entries: &[(&str, &str)]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create test JAR parent directory");
        }
        let file = File::create(path).expect("create test JAR");
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
    }

    fn product_key(report: &super::ServerInspectionReport) -> Option<&str> {
        report
            .identity
            .implementation
            .value
            .as_ref()
            .map(|product| product.key.as_str())
    }

    #[test]
    fn distinguishes_proxy_forks_that_share_main_classes() {
        let cases = [
            (
                "velocity.jar",
                "Implementation-Title: Velocity\r\n",
                "4.1.0-SNAPSHOT (git-e11584ba-b13)",
                "com.velocitypowered.proxy.Velocity",
                "velocity",
                ServerEcosystem::Velocity,
            ),
            (
                "velocity-ctd.jar",
                "Implementation-Title: Velocity-CTD\r\n",
                "4.1.0-SNAPSHOT-git-6fd8e660",
                "com.velocitypowered.proxy.Velocity",
                "velocity-ctd",
                ServerEcosystem::Velocity,
            ),
            (
                "bungeecord.jar",
                "",
                "git:BungeeCord-Bootstrap:26.1-R0.1-SNAPSHOT:2e72932:2085",
                "net.md_5.bungee.Bootstrap",
                "bungeecord",
                ServerEcosystem::Bungee,
            ),
            (
                "waterfall.jar",
                "",
                "git:Waterfall-Bootstrap:26.1-R0.1-SNAPSHOT:4bc2b02:615",
                "net.md_5.bungee.Bootstrap",
                "waterfall",
                ServerEcosystem::Bungee,
            ),
        ];

        for (filename, title, version, main_class, expected_key, ecosystem) in cases {
            let path = temporary_path(filename);
            let manifest = format!(
                "Manifest-Version: 1.0\r\n{title}Implementation-Version: {version}\r\nJava-Version: 11\r\nMain-Class: {main_class}\r\n\r\n"
            );
            write_test_jar_entries(&path, &[("META-INF/MANIFEST.MF", &manifest)]);

            let report = inspect_server_artifact(&path, &InspectionOptions::default())
                .expect("inspect proxy fixture");
            fs::remove_file(&path).expect("remove proxy fixture");

            assert_eq!(product_key(&report), Some(expected_key));
            assert_eq!(report.identity.category.value, Some(ServerCategory::Proxy));
            assert_eq!(report.identity.version.value.as_deref(), Some(version));
            assert_eq!(report.identity.release_channel.value, Some(ReleaseChannel::Snapshot));
            assert!(report
                .identity
                .ecosystems
                .iter()
                .any(|candidate| candidate.value == ecosystem));
            assert_eq!(report.java.required_major.value, Some(11));
            assert!(report.evidence.iter().any(|evidence| {
                evidence.detector == "proxy-manifest-product"
                    && evidence.target == DetectionTarget::ServerCategory
                    && evidence.candidate == "proxy"
            }));
        }
    }

    #[test]
    fn shared_proxy_main_class_does_not_choose_a_specific_product() {
        let path = temporary_path("server.jar");
        write_test_jar_entries(
            &path,
            &[("META-INF/MANIFEST.MF", "Main-Class: com.velocitypowered.proxy.Velocity\r\n\r\n")],
        );

        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect product-neutral proxy fixture");
        fs::remove_file(&path).expect("remove proxy fixture");

        assert!(report.identity.implementation.value.is_none());
        assert_eq!(report.identity.category.value, Some(ServerCategory::Proxy));
        assert_eq!(report.identity.ecosystems.len(), 1);
        assert_eq!(report.identity.ecosystems[0].value, ServerEcosystem::Velocity);
    }

    #[test]
    fn detects_arclight_and_preserves_its_loader_ecosystem() {
        let path = temporary_path("arclight.jar");
        write_test_jar_entries(
            &path,
            &[
                (
                    "META-INF/MANIFEST.MF",
                    "Main-Class: io.izzel.arclight.server.Launcher\r\nImplementation-Title: Arclight\r\nImplementation-Version: arclight-1.21.1-1.0.2-SNAPSHOT-8086b06\r\n\r\n",
                ),
                (
                    "arclight-server-launch.properties",
                    "launch.mainClass=io.izzel.arclight.boot.forge.application.Main_Forge\n",
                ),
            ],
        );

        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect Arclight fixture");
        fs::remove_file(&path).expect("remove Arclight fixture");

        assert_eq!(product_key(&report), Some("arclight"));
        assert_eq!(
            report.identity.version.value.as_deref(),
            Some("arclight-1.21.1-1.0.2-SNAPSHOT-8086b06")
        );
        assert_eq!(
            report
                .minecraft
                .as_ref()
                .and_then(|minecraft| minecraft.version.value.as_deref()),
            Some("1.21.1")
        );
        assert!(report
            .identity
            .ecosystems
            .iter()
            .any(|candidate| candidate.value == ServerEcosystem::Forge));
        let forge_component = report
            .components
            .iter()
            .find(|component| component.value.key == "forge")
            .expect("Forge loader component");
        assert!(forge_component.evidence.iter().all(|evidence_id| {
            report
                .evidence
                .iter()
                .find(|evidence| evidence.id == *evidence_id)
                .is_some_and(|evidence| evidence.detector == "arclight-launch-properties")
        }));
        assert!(report
            .artifact
            .roles
            .iter()
            .any(|role| role.value == ArtifactRole::Launcher));
    }

    #[test]
    fn extracts_mohist_named_section_components_without_merging_sections() {
        let path = temporary_path("mohist.jar");
        write_test_jar_entries(
            &path,
            &[(
                "META-INF/MANIFEST.MF",
                "Main-Class: com.mohistmc.MohistMCStart\r\n\r\nName: com/mohistmc/\r\nImplementation-Title: Mohist\r\nImplementation-Version: 1.20.2-00000000\r\n\r\nName: net/minecraftforge/versions/forge/\r\nImplementation-Title: net.minecraftforge\r\nImplementation-Version: 48.1.0\r\n\r\nName: org/bukkit/craftbukkit/v1_20_R2/\r\nImplementation-Title: Spigot\r\nImplementation-Version: build-48.1.0\r\n\r\nName: net/minecraftforge/versions/mcp/\r\nSpecification-Version: 1.20.2\r\nImplementation-Title: MCP\r\nImplementation-Version: 20230921.100330\r\n\r\n",
            )],
        );

        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect Mohist fixture");
        fs::remove_file(&path).expect("remove Mohist fixture");

        assert_eq!(product_key(&report), Some("mohist"));
        assert_eq!(report.identity.version.value.as_deref(), Some("1.20.2-00000000"));
        assert_eq!(
            report
                .minecraft
                .as_ref()
                .and_then(|minecraft| minecraft.version.value.as_deref()),
            Some("1.20.2")
        );
        assert!(report
            .components
            .iter()
            .any(|component| component.value.key == "forge"
                && component.value.version.as_deref() == Some("48.1.0")));
        assert!(report
            .components
            .iter()
            .any(|component| component.value.key == "mcp"
                && component.value.version.as_deref() == Some("20230921.100330")));
    }

    #[test]
    fn detects_youer_sections_and_magma_wrapper_metadata() {
        let youer_path = temporary_path("youer.jar");
        write_test_jar_entries(
            &youer_path,
            &[(
                "META-INF/MANIFEST.MF",
                "Main-Class: com.mohistmc.launcher.youer.Main\r\n\r\nName: com/mohistmc/launcher/youer/\r\nImplementation-Title: Youer\r\nImplementation-Version: 1.21.1-d380773e\r\n\r\n",
            )],
        );
        let youer = inspect_server_artifact(&youer_path, &InspectionOptions::default())
            .expect("inspect Youer fixture");
        fs::remove_file(&youer_path).expect("remove Youer fixture");
        assert_eq!(product_key(&youer), Some("youer"));
        assert!(youer
            .identity
            .ecosystems
            .iter()
            .any(|candidate| candidate.value == ServerEcosystem::NeoForge));

        let magma_path = temporary_path("server.jar");
        write_test_jar_entries(
            &magma_path,
            &[(
                "metadata.json",
                r#"{"version":"1.21.1","magma":{"groupId":"org.magmafoundation","artifactId":"magma","version":"21.1.70-beta"}}"#,
            )],
        );
        let magma = inspect_server_artifact(&magma_path, &InspectionOptions::default())
            .expect("inspect Magma fixture");
        fs::remove_file(&magma_path).expect("remove Magma fixture");
        assert_eq!(product_key(&magma), Some("magma"));
        assert_eq!(magma.identity.version.value.as_deref(), Some("21.1.70-beta"));
        assert_eq!(magma.identity.release_channel.value, Some(ReleaseChannel::Beta));
        assert!(magma
            .artifact
            .roles
            .iter()
            .any(|role| role.value == ArtifactRole::Wrapper));
    }

    #[test]
    fn detects_a_magma_wrapper_directory_and_its_root_jar_launch() {
        let path = temporary_path("magma-directory");
        fs::create_dir(&path).expect("create Magma directory");
        write_test_jar_entries(
            &path.join("server.jar"),
            &[
                (
                    "META-INF/MANIFEST.MF",
                    "Main-Class: app.mcjars.serverstarter.ServerStarter\r\n\r\n",
                ),
                (
                    "metadata.json",
                    r#"{"version":"1.21.1","magma":{"groupId":"org.magmafoundation","artifactId":"magma","version":"21.1.70-beta"}}"#,
                ),
            ],
        );

        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect Magma directory fixture");
        fs::remove_dir_all(&path).expect("remove Magma directory fixture");

        assert_eq!(product_key(&report), Some("magma"));
        assert!(report
            .launches
            .iter()
            .any(|launch| matches!(launch.value.target, LaunchTarget::Jar { ref path } if path.ends_with("server.jar"))));
        assert!(report
            .artifact
            .roles
            .iter()
            .any(|role| role.value == ArtifactRole::InstallationDirectory));
        assert!(report
            .artifact
            .roles
            .iter()
            .any(|role| role.value == ArtifactRole::Wrapper));
    }

    #[test]
    fn separates_spongevanilla_identity_from_its_installer_role() {
        let path = temporary_path("spongevanilla.jar");
        write_test_jar_entries(
            &path,
            &[(
                "META-INF/MANIFEST.MF",
                "Main-Class: org.spongepowered.vanilla.installer.InstallerMain\r\nImplementation-Title: SpongeVanilla\r\nImplementation-Version: 26.2-20.0.0-RC2673\r\n\r\n",
            )],
        );

        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect SpongeVanilla fixture");
        fs::remove_file(&path).expect("remove SpongeVanilla fixture");

        assert_eq!(product_key(&report), Some("spongevanilla"));
        assert_eq!(report.identity.category.value, Some(ServerCategory::JavaGameServer));
        assert_eq!(report.identity.release_channel.value, Some(ReleaseChannel::ReleaseCandidate));
        assert!(report
            .artifact
            .roles
            .iter()
            .any(|role| role.value == ArtifactRole::Installer));
        assert!(report
            .identity
            .ecosystems
            .iter()
            .any(|candidate| candidate.value == ServerEcosystem::Sponge));
    }

    #[test]
    fn detects_limbo_products_without_inventing_a_minecraft_version() {
        let cases = [
            (
                "limbo.jar",
                "com.loohp.limbo.Limbo",
                "Limbo-Version: 2026.0.2-ALPHA\r\n",
                "limbo",
            ),
            ("nanolimbo.jar", "ua.nanit.limbo.NanoLimbo", "", "nanolimbo"),
        ];

        for (filename, main_class, version_line, expected_key) in cases {
            let path = temporary_path(filename);
            let manifest = format!("Main-Class: {main_class}\r\n{version_line}\r\n");
            write_test_jar_entries(&path, &[("META-INF/MANIFEST.MF", &manifest)]);
            let report = inspect_server_artifact(&path, &InspectionOptions::default())
                .expect("inspect Limbo fixture");
            fs::remove_file(&path).expect("remove Limbo fixture");

            assert_eq!(product_key(&report), Some(expected_key));
            assert_eq!(report.identity.category.value, Some(ServerCategory::Limbo));
            assert!(report.minecraft.is_none());
        }
    }

    #[test]
    fn detects_craftbukkit_two_field_version_lists() {
        let path = temporary_path("craftbukkit.jar");
        write_test_jar_entries(
            &path,
            &[
                (
                    "META-INF/MANIFEST.MF",
                    "Main-Class: org.bukkit.craftbukkit.bootstrap.Main\r\n\r\n",
                ),
                ("META-INF/versions.list", "hash *craftbukkit-26.2-R0.1-SNAPSHOT.jar\n"),
            ],
        );

        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect CraftBukkit fixture");
        fs::remove_file(&path).expect("remove CraftBukkit fixture");

        assert_eq!(product_key(&report), Some("craftbukkit"));
        assert_eq!(report.identity.version.value.as_deref(), Some("26.2-R0.1-SNAPSHOT"));
        assert_eq!(
            report
                .minecraft
                .as_ref()
                .and_then(|minecraft| minecraft.version.value.as_deref()),
            Some("26.2")
        );
        assert!(report
            .identity
            .ecosystems
            .iter()
            .any(|candidate| candidate.value == ServerEcosystem::Bukkit));
        let implementation_detectors = report
            .identity
            .implementation
            .evidence
            .iter()
            .filter_map(|evidence_id| {
                report
                    .evidence
                    .iter()
                    .find(|evidence| evidence.id == *evidence_id)
                    .map(|evidence| evidence.detector.as_str())
            })
            .collect::<Vec<_>>();
        assert!(implementation_detectors.contains(&"craftbukkit-main-class"));
        assert!(implementation_detectors.contains(&"craftbukkit-versions-list"));
        assert!(!implementation_detectors.contains(&"craftbukkit-bundler"));
    }

    #[test]
    fn distinguishes_fabric_loader_from_installer_version() {
        let cases = [
            ("fabric", "FabricInstaller", "26.2"),
            ("legacy-fabric", "LegacyFabricInstaller", "1.13.2"),
        ];

        for (product_key, title, minecraft_version) in cases {
            let path = temporary_path(&format!("{product_key}.jar"));
            let manifest = format!(
                "Manifest-Version: 1.0\r\nImplementation-Title: {title}\r\nImplementation-Version: 1.1.1\r\nMain-Class: net.fabricmc.installer.ServerLauncher\r\n\r\n"
            );
            let properties =
                format!("fabric-loader-version=0.19.3\ngame-version={minecraft_version}\n");
            write_test_jar_entries(
                &path,
                &[("META-INF/MANIFEST.MF", &manifest), ("install.properties", &properties)],
            );

            let report = inspect_server_artifact(&path, &InspectionOptions::default())
                .expect("inspect Fabric launcher");
            fs::remove_file(&path).expect("remove Fabric fixture");

            assert_eq!(
                report
                    .identity
                    .implementation
                    .value
                    .as_ref()
                    .map(|product| product.key.as_str()),
                Some(product_key)
            );
            assert_eq!(report.identity.version.value.as_deref(), Some("0.19.3"));
            assert_ne!(report.identity.version.value.as_deref(), Some("1.1.1"));
            assert_eq!(
                report
                    .minecraft
                    .as_ref()
                    .and_then(|minecraft| minecraft.version.value.as_deref()),
                Some(minecraft_version)
            );
            assert!(report
                .artifact
                .roles
                .iter()
                .any(|role| role.value == ArtifactRole::Installer));
            assert!(report
                .artifact
                .roles
                .iter()
                .any(|role| role.value == ArtifactRole::Launcher));
            assert!(report.components.iter().any(|component| {
                component.value.kind == ServerComponentKind::ModLoader
                    && component.value.version.as_deref() == Some("0.19.3")
            }));
            assert!(report.components.iter().any(|component| {
                component.value.kind == ServerComponentKind::Installer
                    && component.value.version.as_deref() == Some("1.1.1")
            }));
        }
    }

    #[test]
    fn discovers_a_fabric_launcher_in_an_installation_directory() {
        let root = temporary_path("fabric-installation");
        fs::create_dir_all(&root).expect("create Fabric installation directory");
        write_test_jar_entries(
            &root.join("fabric-server-launch.jar"),
            &[
                (
                    "META-INF/MANIFEST.MF",
                    "Implementation-Title: FabricInstaller\r\nImplementation-Version: 1.1.1\r\nMain-Class: net.fabricmc.installer.ServerLauncher\r\n\r\n",
                ),
                (
                    "install.properties",
                    "fabric-loader-version=0.19.3\ngame-version=26.2\n",
                ),
            ],
        );

        let report = inspect_server_artifact(&root, &InspectionOptions::default())
            .expect("inspect Fabric installation directory");
        fs::remove_dir_all(&root).expect("remove Fabric installation fixture");

        assert_eq!(
            report
                .identity
                .implementation
                .value
                .as_ref()
                .map(|product| product.key.as_str()),
            Some("fabric")
        );
        assert_eq!(report.identity.version.value.as_deref(), Some("0.19.3"));
        assert!(report.launches.iter().any(|launch| {
            matches!(
                &launch.value.target,
                LaunchTarget::Jar { path }
                    if path.file_name().and_then(|name| name.to_str())
                        == Some("fabric-server-launch.jar")
            )
        }));
    }

    #[test]
    fn inspects_a_forge_installation_directory_and_launch_profiles() {
        let root = temporary_path("forge-installation");
        fs::create_dir_all(&root).expect("create Forge fixture directory");
        write_test_jar_entries(
            &root.join("server.jar"),
            &[
                (
                    "META-INF/MANIFEST.MF",
                    "Manifest-Version: 1.0\r\nMain-Class: net.minecraftforge.bootstrap.shim.Main\r\n\r\nName: net/minecraftforge/bootstrap/shim/\r\nImplementation-Title: bs-shim\r\nImplementation-Version: 2.1.8\r\n\r\n",
                ),
                (
                    "bootstrap-shim.properties",
                    "Arguments=--launchTarget forge_server\nJava-Version=25\nMain-Class=net.minecraftforge.bootstrap.ForgeBootstrap\n",
                ),
                (
                    "bootstrap-shim.list",
                    "hash\tnet.minecraftforge:forge:26.2-65.1.0:server\tnet/minecraftforge/forge/26.2-65.1.0/forge-server.jar\n",
                ),
            ],
        );
        let version_directory = root.join("libraries/net/minecraftforge/forge/26.2-65.1.0");
        fs::create_dir_all(&version_directory).expect("create Forge version directory");
        fs::write(
            version_directory.join("win_args.txt"),
            "-Dexample=true -jar forge-26.2-65.1.0-shim.jar\n",
        )
        .expect("write Forge Windows args");
        fs::write(
            version_directory.join("unix_args.txt"),
            "-Dexample=true -jar forge-26.2-65.1.0-shim.jar\n",
        )
        .expect("write Forge Unix args");
        fs::write(
            root.join("run.bat"),
            "java @libraries/net/minecraftforge/forge/26.2-65.1.0/win_args.txt %*\n",
        )
        .expect("write Forge startup script");
        write_test_jar_entries(
            &root.join(
                "libraries/net/minecraftforge/fmlloader/26.2-65.1.0/fmlloader-26.2-65.1.0.jar",
            ),
            &[(
                "forge_version.json",
                r#"{"forge":"65.1.0","mc":"26.2","mcp":"20260616.103818"}"#,
            )],
        );

        let report = inspect_server_artifact(&root, &InspectionOptions::default())
            .expect("inspect Forge directory");
        fs::remove_dir_all(&root).expect("remove Forge fixture");

        assert_eq!(
            report
                .identity
                .implementation
                .value
                .as_ref()
                .map(|product| product.key.as_str()),
            Some("forge")
        );
        assert_eq!(report.identity.version.value.as_deref(), Some("65.1.0"));
        assert_eq!(
            report
                .minecraft
                .as_ref()
                .and_then(|minecraft| minecraft.version.value.as_deref()),
            Some("26.2")
        );
        assert_eq!(report.java.required_major.value, Some(25));
        assert!(report
            .components
            .iter()
            .any(|component| component.value.key == "forge"));
        assert!(report
            .components
            .iter()
            .any(|component| component.value.key == "mcp"));
        assert!(report
            .components
            .iter()
            .any(|component| component.value.key == "forge-bootstrap-shim"));
        assert!(report
            .launches
            .iter()
            .any(|launch| launch.value.platform == LaunchPlatform::Windows));
        assert!(report
            .launches
            .iter()
            .any(|launch| launch.value.platform == LaunchPlatform::Unix));
        assert!(report
            .launches
            .iter()
            .any(|launch| matches!(launch.value.target, LaunchTarget::Script { .. })));
    }

    #[test]
    fn keeps_multiple_installed_forge_versions_ambiguous() {
        let root = temporary_path("forge-multiple-versions");
        for version in ["1.20.1-47.2.0", "1.20.1-47.3.0"] {
            let directory = root
                .join("libraries/net/minecraftforge/forge")
                .join(version);
            fs::create_dir_all(&directory).expect("create Forge version directory");
            fs::write(directory.join("win_args.txt"), format!("-jar forge-{version}-shim.jar\n"))
                .expect("write Forge args");
        }

        let report = inspect_server_artifact(&root, &InspectionOptions::default())
            .expect("inspect multi-version Forge directory");
        fs::remove_dir_all(&root).expect("remove multi-version Forge fixture");

        assert_eq!(
            report
                .identity
                .implementation
                .value
                .as_ref()
                .map(|product| product.key.as_str()),
            Some("forge")
        );
        assert!(report.identity.version.value.is_none());
        assert_eq!(report.identity.version.alternatives.len(), 2);
        assert_eq!(
            report
                .components
                .iter()
                .filter(|component| component.value.kind == ServerComponentKind::ModLoader)
                .count(),
            2
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "conflicting_server_versions"));
    }

    #[test]
    fn inspects_a_neoforge_installation_directory_and_wrapper() {
        let root = temporary_path("neoforge-server.jar");
        fs::create_dir_all(&root).expect("create NeoForge fixture directory");
        write_test_jar_entries(
            &root.join("server.jar"),
            &[
                (
                    "META-INF/MANIFEST.MF",
                    "Manifest-Version: 1.0\r\nMain-Class: app.mcjars.serverstarter.ServerStarter\r\n\r\n",
                ),
                (
                    "metadata.json",
                    r#"{"version":"26.2","neoforge":"26.2.0.41-beta"}"#,
                ),
            ],
        );
        let version_directory = root.join("libraries/net/neoforged/neoforge/26.2.0.41-beta");
        fs::create_dir_all(&version_directory).expect("create NeoForge version directory");
        let arguments = concat!(
            "-classpath\n",
            "libraries/net/neoforged/loader.jar\n",
            "net.neoforged.fml.startup.Server\n",
            "--fml.neoForgeVersion 26.2.0.41-beta\n",
            "--fml.mcVersion 26.2\n",
        );
        fs::write(version_directory.join("win_args.txt"), arguments)
            .expect("write NeoForge Windows args");
        fs::write(version_directory.join("unix_args.txt"), arguments)
            .expect("write NeoForge Unix args");
        write_test_jar_entries(
            &version_directory.join("neoforge-26.2.0.41-beta-universal.jar"),
            &[(
                "net/neoforged/neoforge/common/version.properties",
                "neoforge_version=26.2.0.41-beta\nneoform_version=26.2-2\nminecraft_version=26.2\nbuild_type=BETA\n",
            )],
        );

        let report = inspect_server_artifact(&root, &InspectionOptions::default())
            .expect("inspect NeoForge directory");
        fs::remove_dir_all(&root).expect("remove NeoForge fixture");

        assert_eq!(
            report
                .identity
                .implementation
                .value
                .as_ref()
                .map(|product| product.key.as_str()),
            Some("neoforge")
        );
        assert_eq!(report.identity.version.value.as_deref(), Some("26.2.0.41-beta"));
        assert_eq!(report.identity.release_channel.value, Some(ReleaseChannel::Beta));
        assert_eq!(
            report
                .minecraft
                .as_ref()
                .and_then(|minecraft| minecraft.version.value.as_deref()),
            Some("26.2")
        );
        assert!(report
            .components
            .iter()
            .any(|component| component.value.key == "neoforge"));
        assert!(report
            .components
            .iter()
            .any(|component| component.value.key == "neoform"));
        assert!(report
            .artifact
            .roles
            .iter()
            .any(|role| role.value == ArtifactRole::Wrapper));
        assert!(report.launches.iter().any(|launch| {
            matches!(launch.value.target, LaunchTarget::Jar { .. })
                && launch.value.id == "neoforge-wrapper-jar"
        }));
        assert_eq!(
            report
                .launches
                .iter()
                .filter(|launch| matches!(launch.value.target, LaunchTarget::ArgumentFiles { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn distinguishes_paperclip_products_from_content_evidence() {
        struct Case {
            key: &'static str,
            minecraft: &'static str,
            main_class: &'static str,
            coordinate: &'static str,
            version: &'static str,
            channel: ReleaseChannel,
        }

        let cases = [
            Case {
                key: "paper",
                minecraft: "26.2",
                main_class: "io.papermc.paperclip.Main",
                coordinate: "io.papermc.paper:paper-api:26.2.build.87-stable",
                version: "26.2.build.87-stable",
                channel: ReleaseChannel::Stable,
            },
            Case {
                key: "purpur",
                minecraft: "26.2",
                main_class: "io.papermc.paperclip.Main",
                coordinate: "org.purpurmc.purpur:purpur-api:26.2.build.2618-stable",
                version: "26.2.build.2618-stable",
                channel: ReleaseChannel::Stable,
            },
            Case {
                key: "folia",
                minecraft: "26.2",
                main_class: "io.papermc.paperclip.Main",
                coordinate: "dev.folia:folia-api:26.2.build.1-beta",
                version: "26.2.build.1-beta",
                channel: ReleaseChannel::Beta,
            },
            Case {
                key: "pufferfish",
                minecraft: "1.21.10",
                main_class: "io.papermc.paperclip.Main",
                coordinate: "gg.pufferfish.pufferfish:pufferfish-api:1.21.10-R0.1-SNAPSHOT",
                version: "1.21.10-R0.1-SNAPSHOT",
                channel: ReleaseChannel::Snapshot,
            },
            Case {
                key: "aspaper",
                minecraft: "26.2",
                main_class: "io.papermc.paperclip.Main",
                coordinate: "com.infernalsuite.asp:aspaper-api:26.2.build.62-beta",
                version: "26.2.build.62-beta",
                channel: ReleaseChannel::Beta,
            },
            Case {
                key: "canvas",
                minecraft: "26.2",
                main_class: "io.papermc.paperclip.Main",
                coordinate: "io.canvasmc.canvas:canvas-api:26.2.build.890-stable",
                version: "26.2.build.890-stable",
                channel: ReleaseChannel::Stable,
            },
            Case {
                key: "divinemc",
                minecraft: "26.2",
                main_class: "io.papermc.paperclip.Main",
                coordinate: "org.bxteam.divinemc:divinemc-api:26.2.build.4-stable",
                version: "26.2.build.4-stable",
                channel: ReleaseChannel::Stable,
            },
            Case {
                key: "pluto",
                minecraft: "26.2",
                main_class: "io.papermc.paperclip.Main",
                coordinate: "dev.yive.pluto:pluto-api:26.2-R0.1-SNAPSHOT",
                version: "26.2-R0.1-SNAPSHOT",
                channel: ReleaseChannel::Snapshot,
            },
            Case {
                key: "leaf",
                minecraft: "26.2",
                main_class: "cn.dreeam.leaper.Main",
                coordinate: "cn.dreeam.leaf:leaf-api:26.2.build.45-alpha",
                version: "26.2.build.45-alpha",
                channel: ReleaseChannel::Alpha,
            },
            Case {
                key: "leaves",
                minecraft: "1.21.11",
                main_class: "org.leavesmc.leavesclip.Main",
                coordinate: "org.leavesmc.leaves:leaves-api:1.21.11-R0.1-SNAPSHOT",
                version: "1.21.11-R0.1-SNAPSHOT",
                channel: ReleaseChannel::Snapshot,
            },
        ];

        for case in cases {
            let path = temporary_path(&format!("{}-server.jar", case.key));
            let manifest = format!("Main-Class: {}\r\n\r\n", case.main_class);
            let versions = format!(
                "version-hash\t{}\t{}/{}-{}.jar\n",
                case.minecraft, case.minecraft, case.key, case.minecraft
            );
            let patches = format!(
                "versions\tinput\tpatch\toutput\t{0}/server-{0}.jar\t{0}/server.patch\t{0}/{1}-{0}.jar\n",
                case.minecraft, case.key
            );
            let artifact = case
                .coordinate
                .split(':')
                .nth(1)
                .expect("API artifact in coordinate");
            let libraries =
                format!("library-hash\t{}\t{artifact}-{}.jar\n", case.coordinate, case.version);
            let version_json = format!(
                r#"{{"id":"{}","name":"{}","world_version":4903,"stable":true}}"#,
                case.minecraft, case.minecraft
            );
            write_test_jar_entries(
                &path,
                &[
                    ("META-INF/MANIFEST.MF", &manifest),
                    ("META-INF/versions.list", &versions),
                    ("META-INF/patches.list", &patches),
                    ("META-INF/libraries.list", &libraries),
                    ("version.json", &version_json),
                ],
            );

            let report = inspect_server_artifact(&path, &InspectionOptions::default())
                .expect("inspect Paperclip fixture");
            fs::remove_file(&path).expect("remove Paperclip fixture");

            assert_eq!(
                report
                    .identity
                    .implementation
                    .value
                    .as_ref()
                    .map(|product| product.key.as_str()),
                Some(case.key)
            );
            assert_eq!(report.identity.implementation.confidence, 100);
            assert_eq!(report.identity.version.value.as_deref(), Some(case.version));
            assert_eq!(report.identity.release_channel.value, Some(case.channel));
            assert_eq!(report.identity.category.value, Some(ServerCategory::JavaGameServer));
            assert!(report
                .identity
                .ecosystems
                .iter()
                .any(|ecosystem| ecosystem.value == ServerEcosystem::Paper));
            assert!(report
                .identity
                .ecosystems
                .iter()
                .any(|ecosystem| ecosystem.value == ServerEcosystem::Bukkit));
            assert!(report
                .artifact
                .roles
                .iter()
                .any(|role| role.value == ArtifactRole::Bootstrapper));
            assert_eq!(report.components.len(), 1);
            assert_eq!(
                report.components[0]
                    .value
                    .coordinate
                    .as_ref()
                    .map(|coordinate| coordinate.version.as_str()),
                Some(case.version)
            );
            assert_eq!(
                report
                    .minecraft
                    .as_ref()
                    .and_then(|minecraft| minecraft.version.value.as_deref()),
                Some(case.minecraft)
            );
            let minecraft = report.minecraft.as_ref().expect("Minecraft metadata");
            assert!(minecraft
                .version
                .evidence
                .iter()
                .all(|evidence_id| minecraft.evidence.contains(evidence_id)));
        }
    }

    #[test]
    fn detects_vanilla_content_even_when_the_file_is_named_quilt() {
        let path = temporary_path("quilt-server.jar");
        write_test_jar_entries(
            &path,
            &[
                ("META-INF/MANIFEST.MF", "Main-Class: net.minecraft.bundler.Main\r\n\r\n"),
                ("META-INF/versions.list", "hash\t26.2\t26.2/server-26.2.jar\n"),
                (
                    "version.json",
                    r#"{"id":"26.2","name":"26.2","world_version":4903,"stable":true}"#,
                ),
            ],
        );

        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect renamed vanilla JAR");
        fs::remove_file(&path).expect("remove vanilla fixture");

        assert_eq!(
            report
                .identity
                .implementation
                .value
                .as_ref()
                .map(|product| product.key.as_str()),
            Some("vanilla")
        );
        assert_eq!(report.identity.implementation.confidence, 100);
        assert_eq!(report.identity.version.value.as_deref(), Some("26.2"));
        assert_eq!(report.identity.release_channel.value, Some(ReleaseChannel::Stable));
        assert_eq!(report.identity.ecosystems[0].value, ServerEcosystem::Vanilla);
    }

    #[test]
    fn supports_spigot_paperclip_with_a_wildcard_api_coordinate() {
        let path = temporary_path("spigot.jar");
        write_test_jar_entries(
            &path,
            &[
                ("META-INF/MANIFEST.MF", "Main-Class: io.papermc.paperclip.Main\r\n\r\n"),
                ("META-INF/versions.list", "hash\t26.2\t26.2/spigot-26.2.jar\n"),
                (
                    "META-INF/patches.list",
                    "versions\tinput\tpatch\toutput\t26.2/server-26.2.jar\t26.2/server.patch\t26.2/spigot-26.2.jar\n",
                ),
                (
                    "META-INF/libraries.list",
                    "hash\t*\tspigot-api-26.2-R0.1-SNAPSHOT.jar\n",
                ),
            ],
        );

        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect Spigot Paperclip fixture");
        fs::remove_file(&path).expect("remove Spigot fixture");

        assert_eq!(
            report
                .identity
                .implementation
                .value
                .as_ref()
                .map(|product| product.key.as_str()),
            Some("spigot")
        );
        assert_eq!(report.identity.version.value.as_deref(), Some("26.2-R0.1-SNAPSHOT"));
        assert_eq!(report.identity.release_channel.value, Some(ReleaseChannel::Snapshot));
        assert_eq!(report.identity.ecosystems.len(), 1);
        assert_eq!(report.identity.ecosystems[0].value, ServerEcosystem::Bukkit);
        assert!(report.components[0].value.coordinate.is_none());
    }

    #[test]
    fn accepts_an_unknown_paperclip_product_without_expanding_an_enum() {
        let path = temporary_path("custom-fork.jar");
        write_test_jar_entries(
            &path,
            &[
                (
                    "META-INF/MANIFEST.MF",
                    "Main-Class: io.papermc.paperclip.Main\r\n\r\n",
                ),
                (
                    "META-INF/versions.list",
                    "hash\t26.2\t26.2/custom-fork-26.2.jar\n",
                ),
                (
                    "META-INF/libraries.list",
                    "hash\texample.server:custom-fork-api:26.2.build.1-stable\tcustom-fork-api.jar\n",
                ),
            ],
        );

        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect unknown Paperclip fixture");
        fs::remove_file(&path).expect("remove unknown Paperclip fixture");

        let product = report
            .identity
            .implementation
            .value
            .expect("open product identity");
        assert_eq!(product.key, "custom-fork");
        assert_eq!(product.display_name, "Custom Fork");
        assert_eq!(report.identity.version.value.as_deref(), Some("26.2.build.1-stable"));
        assert_eq!(report.identity.ecosystems.len(), 2);
    }

    #[test]
    fn leaves_strong_conflicting_product_evidence_unresolved() {
        let path = temporary_path("server.jar");
        write_test_jar_entries(
            &path,
            &[
                ("META-INF/MANIFEST.MF", "Main-Class: io.papermc.paperclip.Main\r\n\r\n"),
                ("META-INF/versions.list", "hash\t26.2\t26.2/purpur-26.2.jar\n"),
                (
                    "META-INF/libraries.list",
                    "hash\tio.papermc.paper:paper-api:26.2.build.87-stable\tpaper-api.jar\n",
                ),
            ],
        );

        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect conflicting Paperclip fixture");
        fs::remove_file(&path).expect("remove conflicting fixture");

        assert!(report.identity.implementation.value.is_none());
        assert_eq!(report.identity.implementation.alternatives.len(), 2);
        assert!(report.identity.ecosystems.is_empty());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "conflicting_server_implementations"));
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
    fn filename_only_product_remains_an_unselected_candidate() {
        let path = temporary_path("paper.jar");
        write_test_jar_entries(&path, &[("empty", "")]);

        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect metadata-free JAR");
        fs::remove_file(&path).expect("remove metadata-free JAR");

        assert!(report.identity.implementation.value.is_none());
        assert_eq!(report.identity.implementation.confidence, 25);
        assert_eq!(report.identity.implementation.alternatives.len(), 1);
        assert_eq!(report.identity.implementation.alternatives[0].value.key, "paper");
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "insufficient_server_implementation_evidence"
        }));
        assert!(!report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "conflicting_server_implementations" }));
    }

    #[test]
    fn empty_directories_are_reported_without_false_identity_claims() {
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

    #[test]
    fn discovers_a_paperclip_root_archive_without_a_server_filename() {
        let root = temporary_path("paperclip-root");
        fs::create_dir(&root).expect("create root directory");
        write_test_jar_entries(
            &root.join("paper-26.2.jar"),
            &[
                ("META-INF/MANIFEST.MF", "Main-Class: io.papermc.paperclip.Main\r\n\r\n"),
                ("META-INF/versions.list", "hash\t26.2\t26.2/paper-26.2.jar\n"),
                (
                    "META-INF/libraries.list",
                    "hash\tio.papermc.paper:paper-api:26.2.build.87-stable\tpaper-api.jar\n",
                ),
            ],
        );

        let report = inspect_server_artifact(&root, &InspectionOptions::default())
            .expect("inspect Paperclip root archive");
        fs::remove_dir_all(&root).expect("remove root directory");

        assert_eq!(product_key(&report), Some("paper"));
        assert!(report
            .launches
            .iter()
            .any(|launch| matches!(&launch.value.target, LaunchTarget::Jar { path } if path.ends_with("paper-26.2.jar"))));
    }

    #[test]
    fn skips_an_unrelated_root_library_jar() {
        let root = temporary_path("root-library-filter");
        fs::create_dir(&root).expect("create root directory");
        write_test_jar(
            &root.join("server.jar"),
            "Main-Class: net.minecraft.bundler.Main\r\n\r\n",
            r#"{"id":"26.2","world_version":4903}"#,
        );
        write_test_jar_entries(
            &root.join("a-library.jar"),
            &[("META-INF/MANIFEST.MF", "Main-Class: com.example.Library\r\n\r\n")],
        );

        let report = inspect_server_artifact(&root, &InspectionOptions::default())
            .expect("inspect root with unrelated library");
        fs::remove_dir_all(&root).expect("remove root directory");

        assert_eq!(product_key(&report), Some("vanilla"));
        assert_eq!(
            report
                .launches
                .iter()
                .filter(|launch| matches!(&launch.value.target, LaunchTarget::Jar { .. }))
                .count(),
            1
        );
        assert!(!report
            .launches
            .iter()
            .any(|launch| matches!(&launch.value.target, LaunchTarget::Jar { path } if path.ends_with("a-library.jar"))));
    }

    #[test]
    fn accepts_an_unknown_root_name_when_server_metadata_is_present() {
        let root = temporary_path("unknown-root-filter");
        fs::create_dir(&root).expect("create root directory");
        write_test_jar_entries(
            &root.join("custom-runtime.jar"),
            &[
                ("META-INF/MANIFEST.MF", "Main-Class: io.papermc.paperclip.Main\r\n\r\n"),
                ("META-INF/versions.list", "hash\t26.2\t26.2/custom-runtime-26.2.jar\n"),
            ],
        );

        let report = inspect_server_artifact(&root, &InspectionOptions::default())
            .expect("inspect unknown server root");
        fs::remove_dir_all(&root).expect("remove root directory");

        assert_eq!(product_key(&report), Some("custom-runtime"));
    }

    #[test]
    fn discovers_a_proxy_root_archive_without_a_server_filename() {
        let root = temporary_path("velocity-root");
        fs::create_dir(&root).expect("create root directory");
        write_test_jar_entries(
            &root.join("velocity.jar"),
            &[
                (
                    "META-INF/MANIFEST.MF",
                    "Main-Class: com.velocitypowered.proxy.Velocity\r\nImplementation-Title: Velocity\r\nImplementation-Version: 4.1.0\r\n\r\n",
                ),
            ],
        );

        let report = inspect_server_artifact(&root, &InspectionOptions::default())
            .expect("inspect Velocity root archive");
        fs::remove_dir_all(&root).expect("remove root directory");

        assert_eq!(product_key(&report), Some("velocity"));
        assert!(report
            .launches
            .iter()
            .any(|launch| matches!(&launch.value.target, LaunchTarget::Jar { path } if path.ends_with("velocity.jar"))));
    }

    #[test]
    fn root_archives_are_checked_when_nested_depth_is_zero() {
        let root = temporary_path("root-depth-zero");
        fs::create_dir(&root).expect("create root directory");
        write_test_jar(
            &root.join("server.jar"),
            "Main-Class: net.minecraft.bundler.Main\r\n\r\n",
            r#"{"id":"26.2","world_version":4903,"java_version":25}"#,
        );
        let options = InspectionOptions {
            max_archive_depth: 0,
            ..InspectionOptions::default()
        };

        let report = inspect_server_artifact(&root, &options).expect("inspect root archive");
        fs::remove_dir_all(&root).expect("remove root directory");

        assert_eq!(product_key(&report), Some("vanilla"));
        assert_eq!(
            report
                .minecraft
                .as_ref()
                .and_then(|minecraft| minecraft.version.value.as_deref()),
            Some("26.2")
        );
    }

    #[test]
    fn corrupt_optional_root_archives_do_not_abort_directory_inspection() {
        let root = temporary_path("corrupt-root");
        fs::create_dir(&root).expect("create root directory");
        write_test_jar(
            &root.join("server.jar"),
            "Main-Class: net.minecraft.bundler.Main\r\n\r\n",
            r#"{"id":"26.2","world_version":4903}"#,
        );
        fs::write(root.join("other.jar"), b"not a zip archive")
            .expect("write corrupt root archive");

        let report = inspect_server_artifact(&root, &InspectionOptions::default())
            .expect("corrupt optional root should be skipped");
        fs::remove_dir_all(&root).expect("remove root directory");

        assert_eq!(product_key(&report), Some("vanilla"));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "root_archive_unreadable"));
    }

    #[test]
    fn root_archive_limit_is_reported_without_losing_the_first_candidate() {
        let root = temporary_path("root-limit");
        fs::create_dir(&root).expect("create root directory");
        write_test_jar(
            &root.join("server.jar"),
            "Main-Class: net.minecraft.bundler.Main\r\n\r\n",
            r#"{"id":"26.2","world_version":4903}"#,
        );
        write_test_jar_entries(
            &root.join("paper.jar"),
            &[("META-INF/MANIFEST.MF", "Main-Class: io.papermc.paperclip.Main\r\n\r\n")],
        );
        let options = InspectionOptions {
            max_root_archives: 1,
            ..InspectionOptions::default()
        };

        let report =
            inspect_server_artifact(&root, &options).expect("inspect capped root directory");
        fs::remove_dir_all(&root).expect("remove root directory");

        assert_eq!(product_key(&report), Some("vanilla"));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "root_archive_limit_reached"));
    }

    #[test]
    fn rejects_an_empty_root_archive_budget() {
        let root = temporary_path("invalid-root-budget");
        fs::create_dir(&root).expect("create root directory");
        let options = InspectionOptions {
            max_root_archives: 0,
            ..InspectionOptions::default()
        };

        let error =
            inspect_server_artifact(&root, &options).expect_err("zero root budget must fail");
        fs::remove_dir_all(&root).expect("remove root directory");

        assert!(matches!(error, ServerInspectionError::InvalidOptions { .. }));
    }

    #[test]
    fn skips_a_corrupt_optional_modloader_archive() {
        let root = temporary_path("corrupt-nested-loader");
        let version = "1.20.1-47.2.0";
        let args_directory = root
            .join("libraries/net/minecraftforge/forge")
            .join(version);
        fs::create_dir_all(&args_directory).expect("create Forge args directory");
        fs::write(args_directory.join("win_args.txt"), "-jar forge-shim.jar\n")
            .expect("write Forge args");
        let nested_path = root
            .join("libraries/net/minecraftforge/fmlloader")
            .join(version)
            .join(format!("fmlloader-{version}.jar"));
        fs::create_dir_all(nested_path.parent().expect("nested archive parent"))
            .expect("create nested archive directory");
        fs::write(&nested_path, b"not a ZIP archive").expect("write corrupt nested archive");

        let report = inspect_server_artifact(&root, &InspectionOptions::default())
            .expect("corrupt optional nested archive should not abort inspection");
        fs::remove_dir_all(&root).expect("remove corrupt nested fixture");

        assert_eq!(
            report
                .identity
                .implementation
                .value
                .as_ref()
                .map(|product| product.key.as_str()),
            Some("forge")
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "optional_metadata_unreadable"));
    }
}
