use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::instance::{
    InstanceSpec, LocalLaunch, ServerMetadataComponent, ServerMetadataDiagnostic,
    ServerMetadataFingerprint, ServerMetadataIdentity, ServerMetadataJava, ServerMetadataLaunch,
    ServerMetadataMinecraft, ServerMetadataSnapshot, ServerMetadataSubject,
    ServerMetadataSubjectKind,
};

use super::launch_adapter::adapt_launch_profile;
use super::server_inspection::{
    detection_outcome, inspect_server_artifact, server_implementation_outcome, Detected,
    DetectionOutcome, DiagnosticSeverity, InspectionDiagnostic, InspectionOptions, LaunchPlatform,
    LaunchProfile, LaunchTarget, ReleaseChannel, ServerInspectionError, ServerInspectionReport,
};

/// 控制检查结果是否可以替换已有的启动配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchProfilePolicy {
    /// 只返回可兼容候选，不改变已有 `LocalLaunch`。
    PreserveExisting,
    /// 采纳唯一最高置信度候选；候选的模式和目标路径替换旧值，
    /// profile 未提供 JVM 参数时保留已有 JVM 参数，Java 可执行文件始终保留。
    AdoptBestCompatible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerInspectionProjectionOptions {
    pub launch_profile_policy: LaunchProfilePolicy,
    pub inspected_at_unix_secs: Option<u64>,
}

impl Default for ServerInspectionProjectionOptions {
    fn default() -> Self {
        Self {
            launch_profile_policy: LaunchProfilePolicy::PreserveExisting,
            inspected_at_unix_secs: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportLaunchCandidate {
    pub profile_id: String,
    pub confidence: u8,
    pub launch: LocalLaunch,
    pub required_java_major: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerInspectionProjection {
    pub diagnostics: Vec<InspectionDiagnostic>,
    pub launch_candidates: Vec<ImportLaunchCandidate>,
    pub adopted_launch_profile: Option<String>,
}

/// 将主机已经完成的服务端检查结果投影到实例导入元数据。
///
/// 该函数只处理报告，不读取路径、不执行脚本。检查失败、字段缺失或候选冲突时，
/// 保留调用方已有的值，并返回可供导入界面展示的诊断。
pub fn apply_server_inspection(
    instance: &mut InstanceSpec,
    inspection: Result<&ServerInspectionReport, &ServerInspectionError>,
) -> Vec<InspectionDiagnostic> {
    apply_server_inspection_with_options(instance, inspection, &Default::default()).diagnostics
}

/// 将检查结果投影到导入元数据，并按显式策略生成或采纳启动候选。
pub fn apply_server_inspection_with_options(
    instance: &mut InstanceSpec,
    inspection: Result<&ServerInspectionReport, &ServerInspectionError>,
    options: &ServerInspectionProjectionOptions,
) -> ServerInspectionProjection {
    tracing::debug!(
        target: "sealantern.core.provisioning.import",
        instance_id = %instance.id.as_str(),
        launch_policy = ?options.launch_profile_policy,
        "projecting server inspection into import metadata"
    );
    let report = match inspection {
        Ok(report) => report,
        Err(error) => {
            tracing::warn!(
                target: "sealantern.core.provisioning.import",
                instance_id = %instance.id.as_str(),
                error = %error,
                "server inspection projection skipped after inspection failure"
            );
            return ServerInspectionProjection {
                diagnostics: vec![InspectionDiagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: "server_inspection_failed".to_string(),
                    message: format!(
                        "server inspection failed ({error}); existing import metadata was preserved; retry inspection or choose values manually"
                    ),
                    evidence: Vec::new(),
                }],
                launch_candidates: Vec::new(),
                adopted_launch_profile: None,
            };
        }
    };

    let mut diagnostics = report.diagnostics.clone();
    if let Some(product) = report
        .identity
        .implementation
        .value
        .as_ref()
        .filter(|product| !product.key.trim().is_empty())
    {
        instance.core_type = product.key.trim().to_string();
    } else {
        diagnostics.push(unresolved_diagnostic(
            "server implementation",
            &report.identity.implementation,
            server_implementation_outcome(&report.identity.implementation),
            |product| Cow::Borrowed(product.key.as_str()),
        ));
    }

    if let Some(version) = report
        .identity
        .version
        .value
        .as_ref()
        .filter(|version| !version.trim().is_empty())
    {
        instance.core_version = version.trim().to_string();
    } else {
        diagnostics.push(unresolved_diagnostic(
            "server implementation version",
            &report.identity.version,
            detection_outcome(&report.identity.version),
            |version| Cow::Borrowed(version.as_str()),
        ));
    }

    match report.minecraft.as_ref() {
        Some(minecraft) => {
            if let Some(version) = minecraft
                .version
                .value
                .as_ref()
                .filter(|version| !version.trim().is_empty())
            {
                instance.game_version = version.trim().to_string();
            } else {
                diagnostics.push(unresolved_diagnostic(
                    "Minecraft version",
                    &minecraft.version,
                    detection_outcome(&minecraft.version),
                    |version| Cow::Borrowed(version.as_str()),
                ));
            }
        }
        None => diagnostics.push(missing_diagnostic("Minecraft version")),
    }

    let launch_candidates = compatible_launch_candidates(report, &mut diagnostics);
    let adopted_launch_profile =
        if options.launch_profile_policy == LaunchProfilePolicy::AdoptBestCompatible {
            adopt_best_launch(instance, &launch_candidates, &mut diagnostics)
        } else {
            None
        };

    instance.server_metadata = Some(snapshot_from_report(
        report,
        options
            .inspected_at_unix_secs
            .unwrap_or_else(current_unix_secs),
    ));

    if !diagnostics.is_empty() {
        tracing::warn!(
            target: "sealantern.core.provisioning.import",
            instance_id = %instance.id.as_str(),
            diagnostics = diagnostics.len(),
            launch_candidates = launch_candidates.len(),
            "server inspection projection completed with diagnostics"
        );
    } else {
        tracing::debug!(
            target: "sealantern.core.provisioning.import",
            instance_id = %instance.id.as_str(),
            launch_candidates = launch_candidates.len(),
            "server inspection projection completed"
        );
    }

    ServerInspectionProjection {
        diagnostics,
        launch_candidates,
        adopted_launch_profile,
    }
}

/// 检查文件或目录并立即生成导入投影。此函数是主机适配层的显式 I/O 边界。
pub fn inspect_and_apply_import_metadata(
    instance: &mut InstanceSpec,
    subject_path: &Path,
    inspection_options: &InspectionOptions,
    projection_options: &ServerInspectionProjectionOptions,
) -> Result<ServerInspectionProjection, ServerInspectionError> {
    tracing::debug!(
        target: "sealantern.core.provisioning.import",
        instance_id = %instance.id.as_str(),
        subject_path = %subject_path.display(),
        "inspecting artifact for import projection"
    );
    let mut options = *inspection_options;
    options.compute_sha256 = true;
    let report = inspect_server_artifact(subject_path, &options)?;
    Ok(apply_server_inspection_with_options(instance, Ok(&report), projection_options))
}

fn compatible_launch_candidates(
    report: &ServerInspectionReport,
    diagnostics: &mut Vec<InspectionDiagnostic>,
) -> Vec<ImportLaunchCandidate> {
    report
        .launches
        .iter()
        .filter_map(|launch| compatible_launch_candidate(launch, diagnostics))
        .collect()
}

fn compatible_launch_candidate(
    attributed: &super::server_inspection::Attributed<LaunchProfile>,
    diagnostics: &mut Vec<InspectionDiagnostic>,
) -> Option<ImportLaunchCandidate> {
    let profile = &attributed.value;
    if !platform_matches_host(profile.platform) {
        diagnostics.push(launch_diagnostic(
            "launch_profile_platform_mismatch",
            &profile.id,
            "launch profile targets a different host platform; it was rejected and not returned as a candidate",
        ));
        return None;
    }

    let launch = match adapt_launch_profile(profile) {
        Ok(launch) => launch,
        Err(error) => {
            let (code, detail) = error.diagnostic();
            diagnostics.push(launch_diagnostic(code, &profile.id, detail));
            return None;
        }
    };

    Some(ImportLaunchCandidate {
        profile_id: profile.id.clone(),
        confidence: attributed.confidence,
        launch,
        required_java_major: profile.required_java_major,
    })
}

fn adopt_best_launch(
    instance: &mut InstanceSpec,
    candidates: &[ImportLaunchCandidate],
    diagnostics: &mut Vec<InspectionDiagnostic>,
) -> Option<String> {
    if instance.launch.custom_command.is_some()
        || instance.launch.custom_executable.is_some()
        || !instance.launch.custom_arguments.is_empty()
    {
        diagnostics.push(InspectionDiagnostic {
            severity: DiagnosticSeverity::Info,
            code: "launch_profile_preserved_custom_configuration".to_string(),
            message: "existing custom launch configuration was preserved".to_string(),
            evidence: Vec::new(),
        });
        return None;
    }
    let best_confidence = candidates
        .iter()
        .map(|candidate| candidate.confidence)
        .max()?;
    let best = candidates
        .iter()
        .filter(|candidate| candidate.confidence == best_confidence)
        .collect::<Vec<_>>();
    if best.len() != 1 {
        diagnostics.push(InspectionDiagnostic {
            severity: DiagnosticSeverity::Warning,
            code: "launch_profile_ambiguous".to_string(),
            message: format!(
                "{} compatible launch profiles share confidence {best_confidence}; existing launch configuration was preserved",
                best.len()
            ),
            evidence: Vec::new(),
        });
        return None;
    }
    // 采用时由检测 profile 决定启动模式和目标路径；profile 未提供 JVM 参数时，
    // 保留调用方已有参数，避免导入确认无意中丢失内存或诊断配置。
    let candidate = best[0];
    let existing_jvm_arguments = instance.launch.jvm_arguments.clone();
    let java_executable = instance.launch.java_executable.clone();
    let mut adopted_launch = candidate.launch.clone();
    if adopted_launch.jvm_arguments.is_empty() {
        adopted_launch.jvm_arguments = existing_jvm_arguments;
    }
    adopted_launch.java_executable = java_executable;
    instance.launch = adopted_launch;
    Some(candidate.profile_id.clone())
}

fn platform_matches_host(platform: LaunchPlatform) -> bool {
    match platform {
        LaunchPlatform::Any => true,
        LaunchPlatform::Windows => cfg!(target_os = "windows"),
        LaunchPlatform::Unix => cfg!(unix),
    }
}

fn launch_diagnostic(code: &str, profile_id: &str, detail: &str) -> InspectionDiagnostic {
    InspectionDiagnostic {
        severity: DiagnosticSeverity::Warning,
        code: code.to_string(),
        message: format!("launch profile {profile_id}: {detail}"),
        evidence: Vec::new(),
    }
}

fn snapshot_from_report(
    report: &ServerInspectionReport,
    inspected_at_unix_secs: u64,
) -> ServerMetadataSnapshot {
    ServerMetadataSnapshot {
        schema_version: crate::instance::SERVER_METADATA_SNAPSHOT_SCHEMA_VERSION,
        inspected_at_unix_secs,
        subject: ServerMetadataSubject {
            kind: match report.subject.kind {
                super::server_inspection::InspectionSubjectKind::File => {
                    ServerMetadataSubjectKind::File
                }
                super::server_inspection::InspectionSubjectKind::Directory => {
                    ServerMetadataSubjectKind::Directory
                }
            },
            size_bytes: report.subject.size_bytes,
            modified_at_unix_secs: report.subject.modified_at_unix_secs,
            fingerprint: report.subject.fingerprint.as_ref().map(|fingerprint| {
                ServerMetadataFingerprint {
                    algorithm: fingerprint_algorithm_name(fingerprint.algorithm).to_string(),
                    value: fingerprint.value.clone(),
                }
            }),
        },
        identity: report
            .identity
            .implementation
            .value
            .as_ref()
            .map(|implementation| ServerMetadataIdentity {
                category: category_name(report.identity.category.value),
                implementation_key: implementation.key.clone(),
                implementation_name: implementation.display_name.clone(),
                implementation_confidence: report.identity.implementation.confidence,
                version: report.identity.version.value.clone(),
                version_confidence: report.identity.version.confidence,
                release_channel: report
                    .identity
                    .release_channel
                    .value
                    .map(release_channel_name),
                ecosystems: report
                    .identity
                    .ecosystems
                    .iter()
                    .map(|ecosystem| ecosystem_name(&ecosystem.value))
                    .collect(),
            }),
        minecraft: report
            .minecraft
            .as_ref()
            .map(|minecraft| ServerMetadataMinecraft {
                version: minecraft.version.value.clone(),
                version_confidence: minecraft.version.confidence,
                id: minecraft.id.clone(),
                name: minecraft.name.clone(),
                java_version: minecraft.java_version,
                stable: minecraft.stable,
            }),
        java: ServerMetadataJava {
            required_major: report.java.required_major.value,
            required_major_confidence: report.java.required_major.confidence,
            runtime_component: report.java.runtime_component.value.clone(),
        },
        components: report
            .components
            .iter()
            .map(|component| ServerMetadataComponent {
                kind: component_kind_name(component.value.kind).to_string(),
                key: component.value.key.clone(),
                name: component.value.name.clone(),
                version: component.value.version.clone(),
                confidence: component.confidence,
                source_path: component.value.source_path.clone(),
            })
            .collect(),
        launches: report
            .launches
            .iter()
            .map(|launch| {
                let (target_kind, target_path) = launch_target_summary(&launch.value.target);
                ServerMetadataLaunch {
                    id: launch.value.id.clone(),
                    platform: launch_platform_name(launch.value.platform).to_string(),
                    target_kind: target_kind.to_string(),
                    target_path,
                    confidence: launch.confidence,
                    required_java_major: launch.value.required_java_major,
                }
            })
            .collect(),
        diagnostics: report
            .diagnostics
            .iter()
            .map(|diagnostic| ServerMetadataDiagnostic {
                severity: diagnostic_severity_name(diagnostic.severity).to_string(),
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
            })
            .collect(),
    }
}

fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn fingerprint_algorithm_name(
    algorithm: super::server_inspection::FingerprintAlgorithm,
) -> &'static str {
    match algorithm {
        super::server_inspection::FingerprintAlgorithm::Sha256 => "sha256",
    }
}

fn category_name(category: Option<super::server_inspection::ServerCategory>) -> String {
    match category {
        Some(super::server_inspection::ServerCategory::JavaGameServer) => "java_game_server",
        Some(super::server_inspection::ServerCategory::BedrockGameServer) => "bedrock_game_server",
        Some(super::server_inspection::ServerCategory::Proxy) => "proxy",
        Some(super::server_inspection::ServerCategory::Limbo) => "limbo",
        Some(super::server_inspection::ServerCategory::Unknown) | None => "unknown",
    }
    .to_string()
}

fn release_channel_name(channel: ReleaseChannel) -> String {
    match channel {
        ReleaseChannel::Stable => "stable",
        ReleaseChannel::ReleaseCandidate => "release_candidate",
        ReleaseChannel::Beta => "beta",
        ReleaseChannel::Alpha => "alpha",
        ReleaseChannel::Snapshot => "snapshot",
        ReleaseChannel::Development => "development",
        ReleaseChannel::Unknown => "unknown",
    }
    .to_string()
}

fn ecosystem_name(ecosystem: &super::server_inspection::ServerEcosystem) -> String {
    match ecosystem {
        super::server_inspection::ServerEcosystem::Vanilla => "vanilla".to_string(),
        super::server_inspection::ServerEcosystem::Bukkit => "bukkit".to_string(),
        super::server_inspection::ServerEcosystem::Paper => "paper".to_string(),
        super::server_inspection::ServerEcosystem::Fabric => "fabric".to_string(),
        super::server_inspection::ServerEcosystem::LegacyFabric => "legacy_fabric".to_string(),
        super::server_inspection::ServerEcosystem::Quilt => "quilt".to_string(),
        super::server_inspection::ServerEcosystem::Forge => "forge".to_string(),
        super::server_inspection::ServerEcosystem::NeoForge => "neoforge".to_string(),
        super::server_inspection::ServerEcosystem::Sponge => "sponge".to_string(),
        super::server_inspection::ServerEcosystem::Bungee => "bungee".to_string(),
        super::server_inspection::ServerEcosystem::Velocity => "velocity".to_string(),
        super::server_inspection::ServerEcosystem::Other(value) => value.clone(),
    }
}

fn component_kind_name(kind: super::server_inspection::ServerComponentKind) -> &'static str {
    match kind {
        super::server_inspection::ServerComponentKind::Implementation => "implementation",
        super::server_inspection::ServerComponentKind::Api => "api",
        super::server_inspection::ServerComponentKind::ModLoader => "mod_loader",
        super::server_inspection::ServerComponentKind::Installer => "installer",
        super::server_inspection::ServerComponentKind::Launcher => "launcher",
        super::server_inspection::ServerComponentKind::Bootstrap => "bootstrap",
        super::server_inspection::ServerComponentKind::Mapping => "mapping",
        super::server_inspection::ServerComponentKind::Wrapper => "wrapper",
    }
}

fn launch_platform_name(platform: LaunchPlatform) -> &'static str {
    match platform {
        LaunchPlatform::Any => "any",
        LaunchPlatform::Windows => "windows",
        LaunchPlatform::Unix => "unix",
    }
}

fn launch_target_summary(target: &LaunchTarget) -> (&'static str, Option<PathBuf>) {
    match target {
        LaunchTarget::Jar { path } => ("jar", Some(path.clone())),
        LaunchTarget::MainClass { .. } => ("main_class", None),
        LaunchTarget::ArgumentFiles { paths } => ("argument_files", paths.first().cloned()),
        LaunchTarget::Script { path } => ("script", Some(path.clone())),
    }
}

fn diagnostic_severity_name(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Info => "info",
        DiagnosticSeverity::Warning => "warning",
        DiagnosticSeverity::Error => "error",
    }
}

fn unresolved_diagnostic<'a, T, F>(
    field: &str,
    detected: &'a Detected<T>,
    outcome: DetectionOutcome,
    format_candidate: F,
) -> InspectionDiagnostic
where
    F: Fn(&'a T) -> Cow<'a, str>,
{
    if outcome == DetectionOutcome::Missing {
        return missing_diagnostic(field);
    }

    let candidates = detected
        .alternatives
        .iter()
        .map(|candidate| {
            format!("{} (confidence {})", format_candidate(&candidate.value), candidate.confidence)
        })
        .collect::<Vec<_>>()
        .join(", ");
    if matches!(outcome, DetectionOutcome::InsufficientEvidence { .. }) {
        return InspectionDiagnostic {
            severity: DiagnosticSeverity::Warning,
            code: format!(
                "server_inspection_{}_insufficient_evidence",
                field.replace(' ', "_").to_ascii_lowercase()
            ),
            message: format!(
                "{field} has insufficient evidence ({candidates}); existing import metadata was preserved; provide stronger metadata or choose a value manually"
            ),
            evidence: detected.evidence.clone(),
        };
    }

    InspectionDiagnostic {
        severity: DiagnosticSeverity::Warning,
        code: format!(
            "server_inspection_{}_ambiguous",
            field.replace(' ', "_").to_ascii_lowercase()
        ),
        message: format!(
            "{field} is ambiguous ({candidates}); existing import metadata was preserved; choose a value manually or provide stronger metadata"
        ),
        evidence: detected.evidence.clone(),
    }
}

fn missing_diagnostic(field: &str) -> InspectionDiagnostic {
    InspectionDiagnostic {
        severity: DiagnosticSeverity::Warning,
        code: format!(
            "server_inspection_{}_unresolved",
            field.replace(' ', "_").to_ascii_lowercase()
        ),
        message: format!(
            "{field} was not identified; existing import metadata was preserved; choose a value manually or provide stronger metadata"
        ),
        evidence: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        apply_server_inspection, apply_server_inspection_with_options, compatible_launch_candidate,
        unresolved_diagnostic, LaunchProfilePolicy, ServerInspectionProjectionOptions,
    };
    use crate::instance::{InstanceId, InstanceSpec, LocalLaunch, StartupMode};
    use crate::provisioning::server_inspection::{
        server_implementation_outcome, Attributed, Detected, DetectionCandidate, LaunchPlatform,
        LaunchProfile, LaunchTarget,
    };
    use crate::provisioning::{inspect_server_artifact, InspectionOptions};
    use zip::write::FileOptions;

    fn instance_spec() -> InstanceSpec {
        InstanceSpec {
            id: InstanceId::new("imported").expect("instance ID should be valid"),
            name: "Imported".to_string(),
            aliases: Vec::new(),
            core_type: "paper".to_string(),
            core_version: "old-core".to_string(),
            game_version: "old-game".to_string(),
            directory: PathBuf::from("managed/imported"),
            port: 25565,
            max_memory_mib: 4096,
            min_memory_mib: 1024,
            created_at_unix_secs: 100,
            last_started_at_unix_secs: None,
            server_metadata: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(PathBuf::from("imports/server.jar")),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: None,
                jvm_arguments: Vec::new(),
            },
        }
    }

    fn write_test_jar(filename: &str, entries: &[(&str, &str)]) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "sealantern-import-metadata-{}-{timestamp}-{filename}",
            std::process::id()
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
    fn projects_unknown_paperclip_product_and_preserves_open_product_key() {
        let mut instance = instance_spec();
        let path = write_test_jar(
            "custom-fork.jar",
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
        std::fs::remove_file(&path).expect("remove unknown Paperclip fixture");

        let diagnostics = apply_server_inspection(&mut instance, Ok(&report));

        assert!(diagnostics.is_empty());
        assert_eq!(instance.core_type, "custom-fork");
        assert_eq!(instance.core_version, "26.2.build.1-stable");
        assert_eq!(instance.game_version, "26.2");
    }

    #[test]
    fn preserves_existing_product_when_filename_is_the_only_evidence() {
        let mut instance = instance_spec();
        let path = write_test_jar("purpur.jar", &[("empty", "")]);
        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect metadata-free JAR");
        std::fs::remove_file(&path).expect("remove metadata-free JAR");

        let diagnostics = apply_server_inspection(&mut instance, Ok(&report));

        assert_eq!(instance.core_type, "paper");
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "server_inspection_server_implementation_insufficient_evidence"
        }));
        assert!(!diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "server_inspection_server_implementation_ambiguous"
        }));
    }

    #[test]
    fn multiple_low_confidence_candidates_are_reported_as_insufficient() {
        let detected = Detected {
            value: None,
            confidence: 25,
            evidence: Vec::new(),
            alternatives: vec![
                DetectionCandidate {
                    value: "paper".to_string(),
                    confidence: 25,
                    evidence: Vec::new(),
                },
                DetectionCandidate {
                    value: "purpur".to_string(),
                    confidence: 20,
                    evidence: Vec::new(),
                },
            ],
        };

        let diagnostic = unresolved_diagnostic(
            "server implementation",
            &detected,
            server_implementation_outcome(&detected),
            |value| Cow::Owned(format!("product:{value}")),
        );

        assert_eq!(
            diagnostic.code,
            "server_inspection_server_implementation_insufficient_evidence"
        );
        assert!(diagnostic.message.contains("product:paper"));
    }

    #[test]
    fn projects_paperclip_derived_product_and_build_version() {
        let mut instance = instance_spec();
        let path = write_test_jar(
            "purpur.jar",
            &[
                ("META-INF/MANIFEST.MF", "Main-Class: io.papermc.paperclip.Main\r\n\r\n"),
                ("META-INF/versions.list", "hash\t26.2\t26.2/purpur-26.2.jar\n"),
                (
                    "META-INF/libraries.list",
                    "hash\torg.purpurmc.purpur:purpur-api:26.2.build.2618-stable\tpurpur-api.jar\n",
                ),
            ],
        );
        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect Paperclip fixture");
        std::fs::remove_file(&path).expect("remove Paperclip fixture");

        let diagnostics = apply_server_inspection(&mut instance, Ok(&report));

        assert!(diagnostics.is_empty());
        assert_eq!(instance.core_type, "purpur");
        assert_eq!(instance.core_version, "26.2.build.2618-stable");
        assert_eq!(instance.game_version, "26.2");
    }

    #[test]
    fn projects_fabric_loader_version_instead_of_installer_version() {
        let mut instance = instance_spec();
        let path = write_test_jar(
            "fabric.jar",
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
        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect Fabric fixture");
        std::fs::remove_file(&path).expect("remove Fabric fixture");

        let diagnostics = apply_server_inspection(&mut instance, Ok(&report));

        assert!(diagnostics.is_empty());
        assert_eq!(instance.core_type, "fabric");
        assert_eq!(instance.core_version, "0.19.3");
        assert_ne!(instance.core_version, "1.1.1");
    }

    #[test]
    fn preserves_conflicting_fields_and_applies_selected_minecraft_version() {
        let mut instance = instance_spec();
        let path = write_test_jar(
            "conflicting.jar",
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
        std::fs::remove_file(&path).expect("remove conflicting Paperclip fixture");

        let diagnostics = apply_server_inspection(&mut instance, Ok(&report));

        assert_eq!(instance.core_type, "paper");
        assert_eq!(instance.core_version, "old-core");
        assert_eq!(instance.game_version, "26.2");
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "server_inspection_server_implementation_ambiguous"
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "server_inspection_server_implementation_version_unresolved"
        }));
    }

    #[test]
    fn preserves_existing_values_when_inspection_fails() {
        let mut instance = instance_spec();
        let path = std::env::temp_dir()
            .join(format!("sealantern-import-metadata-{}-invalid.jar", std::process::id()));
        std::fs::write(&path, b"not a ZIP archive").expect("write invalid JAR");
        let inspection = inspect_server_artifact(&path, &InspectionOptions::default());
        std::fs::remove_file(&path).expect("remove invalid JAR");

        let diagnostics = apply_server_inspection(&mut instance, inspection.as_ref());

        assert!(inspection.is_err());
        assert_eq!(instance.core_type, "paper");
        assert_eq!(instance.core_version, "old-core");
        assert_eq!(instance.game_version, "old-game");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "server_inspection_failed");
        assert!(diagnostics[0]
            .message
            .contains("existing import metadata was preserved"));
    }

    #[test]
    fn adopts_a_unique_jar_launch_and_persists_a_fingerprinted_snapshot() {
        let mut instance = instance_spec();
        instance.launch.jvm_arguments = vec![String::from("-Xmx4G")];
        let path = write_test_jar(
            "paper.jar",
            &[
                ("META-INF/MANIFEST.MF", "Main-Class: io.papermc.paperclip.Main\r\n\r\n"),
                ("META-INF/versions.list", "hash\t26.2\t26.2/paper-26.2.jar\n"),
                (
                    "META-INF/libraries.list",
                    "hash\tio.papermc.paper:paper-api:26.2.build.87-stable\tpaper-api.jar\n",
                ),
            ],
        );
        let inspection_options = InspectionOptions {
            compute_sha256: true,
            ..InspectionOptions::default()
        };
        let report = inspect_server_artifact(&path, &inspection_options)
            .expect("inspect launch candidate fixture");
        let projection = apply_server_inspection_with_options(
            &mut instance,
            Ok(&report),
            &ServerInspectionProjectionOptions {
                launch_profile_policy: LaunchProfilePolicy::AdoptBestCompatible,
                inspected_at_unix_secs: Some(42),
            },
        );
        std::fs::remove_file(&path).expect("remove launch candidate fixture");

        assert_eq!(projection.adopted_launch_profile.as_deref(), Some("manifest-main"));
        assert_eq!(instance.launch.startup_target, Some(path));
        assert_eq!(instance.launch.jvm_arguments, vec![String::from("-Xmx4G")]);
        assert_eq!(
            instance
                .server_metadata
                .as_ref()
                .map(|snapshot| snapshot.inspected_at_unix_secs),
            Some(42)
        );
        assert!(instance
            .server_metadata
            .as_ref()
            .and_then(|snapshot| snapshot.subject.fingerprint.as_ref())
            .is_some());
    }

    #[test]
    fn preserves_existing_launch_by_default_but_exposes_candidates() {
        let mut instance = instance_spec();
        let path = write_test_jar(
            "candidate.jar",
            &[("META-INF/MANIFEST.MF", "Main-Class: example.Server\r\n\r\n")],
        );
        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect candidate fixture");
        std::fs::remove_file(&path).expect("remove candidate fixture");

        let projection = apply_server_inspection_with_options(
            &mut instance,
            Ok(&report),
            &ServerInspectionProjectionOptions::default(),
        );

        assert_eq!(projection.launch_candidates.len(), 1);
        assert_eq!(instance.launch.startup_target, Some(PathBuf::from("imports/server.jar")));
        assert_eq!(
            instance
                .server_metadata
                .as_ref()
                .map(|snapshot| snapshot.schema_version),
            Some(1)
        );
    }

    #[test]
    fn rejects_platform_mismatch_instead_of_returning_a_manual_candidate() {
        let attributed = Attributed {
            value: LaunchProfile {
                id: "wrong-platform".to_string(),
                platform: if cfg!(target_os = "windows") {
                    LaunchPlatform::Unix
                } else {
                    LaunchPlatform::Windows
                },
                working_directory: None,
                target: LaunchTarget::MainClass { class_name: "example.Main".to_string() },
                jvm_arguments: Vec::new(),
                program_arguments: Vec::new(),
                required_java_major: None,
            },
            confidence: 90,
            evidence: Vec::new(),
        };
        let mut diagnostics = Vec::new();

        let candidate = compatible_launch_candidate(&attributed, &mut diagnostics);

        assert!(candidate.is_none());
        assert!(diagnostics[0]
            .message
            .contains("rejected and not returned as a candidate"));
    }
}
