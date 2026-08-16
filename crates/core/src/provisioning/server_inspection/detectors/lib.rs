mod fabric;
mod forge;
mod generic;
mod hybrid;
mod limbo;
mod manifest_attributes;
mod paperclip;
mod proxy;
mod sponge;
mod vanilla;

use std::path::{Path, PathBuf};

use super::archive::ArchiveMetadata;
use super::directory::DirectoryMetadata;
use super::evidence::{EvidenceCollector, NewEvidence};
use super::formats::manifest;
use super::model::{
    ArtifactInfo, ArtifactRole, Attributed, Detected, DetectionTarget, DiagnosticSeverity,
    EvidenceLocation, EvidenceSource, InspectionDiagnostic, LaunchProfile, MavenCoordinate,
    MinecraftVersionInfo, ReleaseChannel, ServerCategory, ServerComponent, ServerComponentKind,
    ServerEcosystem, ServerIdentityInfo, ServerProduct,
};
use super::resolver::{DetectionClaim, resolve, resolve_attributed, resolve_server_implementation};
use super::{DetectionOutcome, detection_outcome, server_implementation_outcome};

const PAPER_ECOSYSTEMS: &[ServerEcosystem] = &[ServerEcosystem::Paper, ServerEcosystem::Bukkit];
const VANILLA_ECOSYSTEMS: &[ServerEcosystem] = &[ServerEcosystem::Vanilla];
const BUKKIT_ECOSYSTEMS: &[ServerEcosystem] = &[ServerEcosystem::Bukkit];
const FABRIC_ECOSYSTEMS: &[ServerEcosystem] = &[ServerEcosystem::Fabric];
const LEGACY_FABRIC_ECOSYSTEMS: &[ServerEcosystem] = &[ServerEcosystem::LegacyFabric];
const FORGE_ECOSYSTEMS: &[ServerEcosystem] = &[ServerEcosystem::Forge];
const NEOFORGE_ECOSYSTEMS: &[ServerEcosystem] = &[ServerEcosystem::NeoForge];
const BUNGEE_ECOSYSTEMS: &[ServerEcosystem] = &[ServerEcosystem::Bungee];
const VELOCITY_ECOSYSTEMS: &[ServerEcosystem] = &[ServerEcosystem::Velocity];
const SPONGE_VANILLA_ECOSYSTEMS: &[ServerEcosystem] =
    &[ServerEcosystem::Sponge, ServerEcosystem::Vanilla];
const MOHIST_ECOSYSTEMS: &[ServerEcosystem] = &[ServerEcosystem::Bukkit, ServerEcosystem::Forge];
const NEOFORGE_HYBRID_ECOSYSTEMS: &[ServerEcosystem] =
    &[ServerEcosystem::Bukkit, ServerEcosystem::NeoForge];
const NO_ECOSYSTEMS: &[ServerEcosystem] = &[];

const PRODUCTS: &[ProductDefinition] = &[
    ProductDefinition::new(
        "pufferfish",
        "Pufferfish",
        PAPER_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "gg.pufferfish.pufferfish",
        "pufferfish-api",
    ),
    ProductDefinition::new(
        "legacy-fabric",
        "Legacy Fabric",
        LEGACY_FABRIC_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "",
        "",
    ),
    ProductDefinition::new(
        "divinemc",
        "DivineMC",
        PAPER_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "org.bxteam.divinemc",
        "divinemc-api",
    ),
    ProductDefinition::new(
        "aspaper",
        "AsPaper",
        PAPER_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "com.infernalsuite.asp",
        "aspaper-api",
    ),
    ProductDefinition::new(
        "purpur",
        "Purpur",
        PAPER_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "org.purpurmc.purpur",
        "purpur-api",
    ),
    ProductDefinition::new(
        "canvas",
        "Canvas",
        PAPER_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "io.canvasmc.canvas",
        "canvas-api",
    ),
    ProductDefinition::new(
        "leaves",
        "Leaves",
        PAPER_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "org.leavesmc.leaves",
        "leaves-api",
    ),
    ProductDefinition::new(
        "vanilla",
        "Vanilla",
        VANILLA_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "",
        "",
    ),
    ProductDefinition::new(
        "neoforge",
        "NeoForge",
        NEOFORGE_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "",
        "",
    ),
    ProductDefinition::new(
        "fabric",
        "Fabric",
        FABRIC_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "",
        "",
    ),
    ProductDefinition::new(
        "spigot",
        "Spigot",
        BUKKIT_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "",
        "",
    ),
    ProductDefinition::new(
        "paper",
        "Paper",
        PAPER_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "io.papermc.paper",
        "paper-api",
    ),
    ProductDefinition::new(
        "folia",
        "Folia",
        PAPER_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "dev.folia",
        "folia-api",
    ),
    ProductDefinition::new(
        "pluto",
        "Pluto",
        PAPER_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "dev.yive.pluto",
        "pluto-api",
    ),
    ProductDefinition::new(
        "forge",
        "Forge",
        FORGE_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "",
        "",
    ),
    ProductDefinition::new(
        "leaf",
        "Leaf",
        PAPER_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "cn.dreeam.leaf",
        "leaf-api",
    ),
    ProductDefinition::new(
        "arclight",
        "Arclight",
        BUKKIT_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "",
        "",
    ),
    ProductDefinition::new(
        "mohist",
        "Mohist",
        MOHIST_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "",
        "",
    ),
    ProductDefinition::new(
        "youer",
        "Youer",
        NEOFORGE_HYBRID_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "",
        "",
    ),
    ProductDefinition::new(
        "magma",
        "Magma",
        NEOFORGE_HYBRID_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "",
        "",
    ),
    ProductDefinition::new(
        "velocity-ctd",
        "Velocity-CTD",
        VELOCITY_ECOSYSTEMS,
        ServerCategory::Proxy,
        "",
        "",
    ),
    ProductDefinition::new(
        "velocity",
        "Velocity",
        VELOCITY_ECOSYSTEMS,
        ServerCategory::Proxy,
        "",
        "",
    ),
    ProductDefinition::new(
        "bungeecord",
        "BungeeCord",
        BUNGEE_ECOSYSTEMS,
        ServerCategory::Proxy,
        "",
        "",
    ),
    ProductDefinition::new(
        "waterfall",
        "Waterfall",
        BUNGEE_ECOSYSTEMS,
        ServerCategory::Proxy,
        "",
        "",
    ),
    ProductDefinition::new(
        "spongevanilla",
        "SpongeVanilla",
        SPONGE_VANILLA_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "",
        "",
    ),
    ProductDefinition::new("limbo", "Limbo", NO_ECOSYSTEMS, ServerCategory::Limbo, "", ""),
    ProductDefinition::new("nanolimbo", "NanoLimbo", NO_ECOSYSTEMS, ServerCategory::Limbo, "", ""),
    ProductDefinition::new(
        "craftbukkit",
        "CraftBukkit",
        BUKKIT_ECOSYSTEMS,
        ServerCategory::JavaGameServer,
        "",
        "",
    ),
];

#[derive(Debug, Clone, Copy)]
struct ProductDefinition {
    key: &'static str,
    display_name: &'static str,
    ecosystems: &'static [ServerEcosystem],
    category: ServerCategory,
    api_group: Option<&'static str>,
    api_artifact: Option<&'static str>,
}

impl ProductDefinition {
    const fn new(
        key: &'static str,
        display_name: &'static str,
        ecosystems: &'static [ServerEcosystem],
        category: ServerCategory,
        api_group: &'static str,
        api_artifact: &'static str,
    ) -> Self {
        Self {
            key,
            display_name,
            ecosystems,
            category,
            api_group: if api_group.is_empty() {
                None
            } else {
                Some(api_group)
            },
            api_artifact: if api_artifact.is_empty() {
                None
            } else {
                Some(api_artifact)
            },
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct Signal<T> {
    pub(super) value: T,
    pub(super) detector: &'static str,
    pub(super) source: EvidenceSource,
    pub(super) location: EvidenceLocation,
    pub(super) weight: u8,
    pub(super) correlation_group: &'static str,
}

#[derive(Debug, Clone)]
pub(super) struct ProductFinding {
    pub(super) signal: Signal<ServerProduct>,
    pub(super) ecosystems: Vec<ServerEcosystem>,
}

#[derive(Debug, Clone)]
pub(super) struct ProductValueFinding<T> {
    pub(super) product_key: String,
    pub(super) signal: Signal<T>,
}

#[derive(Debug, Clone)]
pub(super) struct ComponentFinding {
    pub(super) product_key: String,
    pub(super) signal: Signal<ServerComponent>,
}

#[derive(Debug, Default)]
pub(super) struct Findings {
    pub(super) products: Vec<ProductFinding>,
    pub(super) categories: Vec<Signal<ServerCategory>>,
    pub(super) ecosystems: Vec<Signal<ServerEcosystem>>,
    pub(super) minecraft_versions: Vec<Signal<String>>,
    pub(super) product_versions: Vec<ProductValueFinding<String>>,
    pub(super) release_channels: Vec<ProductValueFinding<ReleaseChannel>>,
    pub(super) roles: Vec<Signal<ArtifactRole>>,
    pub(super) components: Vec<ComponentFinding>,
    pub(super) launches: Vec<Signal<LaunchProfile>>,
    pub(super) java_majors: Vec<Signal<u16>>,
}

impl Findings {
    pub(super) fn has_product(&self, key: &str) -> bool {
        self.products
            .iter()
            .any(|finding| finding.signal.value.key == key)
    }
}

pub(super) struct DetectorOutput {
    pub(super) identity: ServerIdentityInfo,
    pub(super) minecraft_version: Detected<String>,
    pub(super) roles: Vec<Attributed<ArtifactRole>>,
    pub(super) components: Vec<Attributed<ServerComponent>>,
    pub(super) launches: Vec<Attributed<LaunchProfile>>,
    pub(super) java_major: Detected<u16>,
    pub(super) diagnostics: Vec<InspectionDiagnostic>,
}

pub(super) fn detect_jar(
    path: &Path,
    artifact: &ArtifactInfo,
    archive: &ArchiveMetadata,
    minecraft: Option<&MinecraftVersionInfo>,
    java_major: Option<&Detected<u16>>,
    evidence: &mut EvidenceCollector,
) -> DetectorOutput {
    let mut findings = Findings::default();
    detect_filename(path, &mut findings);
    fabric::detect(path, archive, None, &mut findings);
    forge::detect_archive(path, None, archive, &mut findings);
    hybrid::detect(path, archive, &mut findings);
    proxy::detect(path, archive, &mut findings);
    sponge::detect(path, archive, &mut findings);
    limbo::detect(path, archive, &mut findings);
    generic::detect(path, archive, None, &mut findings);
    paperclip::detect(path, artifact, archive, &mut findings);
    vanilla::detect(path, artifact, minecraft, &mut findings);
    finalize(path, findings, minecraft, java_major, evidence)
}

pub(super) fn detect_directory(
    path: &Path,
    directory: &DirectoryMetadata,
    minecraft: Option<&MinecraftVersionInfo>,
    existing_java_major: Option<&Detected<u16>>,
    evidence: &mut EvidenceCollector,
) -> DetectorOutput {
    let mut findings = Findings::default();
    detect_filename(path, &mut findings);
    for root_archive in &directory.root_archives {
        let archive_path = path.join(&root_archive.relative_path);
        let artifact = root_artifact(&root_archive.metadata);
        let root_minecraft = root_minecraft_metadata(&root_archive.metadata);
        fabric::detect(
            &archive_path,
            &root_archive.metadata,
            Some((path, &root_archive.relative_path)),
            &mut findings,
        );
        forge::detect_archive(
            &archive_path,
            Some((path, &root_archive.relative_path)),
            &root_archive.metadata,
            &mut findings,
        );
        hybrid::detect(&archive_path, &root_archive.metadata, &mut findings);
        proxy::detect(&archive_path, &root_archive.metadata, &mut findings);
        sponge::detect(&archive_path, &root_archive.metadata, &mut findings);
        limbo::detect(&archive_path, &root_archive.metadata, &mut findings);
        generic::detect(
            &archive_path,
            &root_archive.metadata,
            Some((path, &root_archive.relative_path)),
            &mut findings,
        );
        paperclip::detect(&archive_path, &artifact, &root_archive.metadata, &mut findings);
        if artifact
            .main_class
            .value
            .as_deref()
            .is_some_and(|main_class| {
                matches!(
                    main_class,
                    "net.minecraft.bundler.Main" | "net.minecraft.server.MinecraftServer"
                )
            })
        {
            vanilla::detect(&archive_path, &artifact, root_minecraft.as_ref(), &mut findings);
        }
    }
    for installation in &directory.installations {
        forge::detect_installation(path, installation, &mut findings);
    }
    for script in &directory.scripts {
        forge::detect_script(path, script, &mut findings);
    }
    finalize(path, findings, minecraft, existing_java_major, evidence)
}

fn root_minecraft_metadata(archive: &ArchiveMetadata) -> Option<MinecraftVersionInfo> {
    let bytes = archive.mojang_version.as_deref()?;
    let document = super::formats::mojang_version::parse(bytes).ok()??;
    let id = document.id?;
    Some(MinecraftVersionInfo {
        version: Detected {
            value: Some(id.clone()),
            confidence: 95,
            evidence: Vec::new(),
            alternatives: Vec::new(),
        },
        id: Some(id),
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
        evidence: Vec::new(),
    })
}

fn root_artifact(archive: &ArchiveMetadata) -> ArtifactInfo {
    let mut artifact = ArtifactInfo {
        format: Detected::default(),
        roles: Vec::new(),
        main_class: Detected::default(),
        premain_class: Detected::default(),
        agent_class: Detected::default(),
        automatic_module_name: Detected::default(),
        manifest: None,
    };
    let Some(bytes) = archive.manifest.as_deref() else {
        return artifact;
    };
    let parsed = manifest::parse(bytes);
    if let Some(value) = parsed.main_value("Main-Class") {
        artifact.main_class = Detected {
            value: Some(value.to_string()),
            confidence: 100,
            evidence: Vec::new(),
            alternatives: Vec::new(),
        };
    }
    artifact.manifest = Some(parsed.summary);
    artifact
}

fn finalize(
    path: &Path,
    findings: Findings,
    minecraft: Option<&MinecraftVersionInfo>,
    existing_java_major: Option<&Detected<u16>>,
    evidence: &mut EvidenceCollector,
) -> DetectorOutput {
    let implementation = resolve_server_implementation(
        findings
            .products
            .iter()
            .map(|finding| {
                push_claim(
                    &finding.signal,
                    DetectionTarget::ServerImplementation,
                    finding.signal.value.key.clone(),
                    evidence,
                )
            })
            .collect(),
    );
    let selected_key = implementation
        .value
        .as_ref()
        .map(|product| product.key.as_str());

    let mut category_claims = findings
        .categories
        .iter()
        .map(|signal| {
            push_claim(
                signal,
                DetectionTarget::ServerCategory,
                category_name(signal.value).to_string(),
                evidence,
            )
        })
        .collect::<Vec<_>>();
    category_claims.extend(findings.products.iter().map(|finding| {
        let signal = Signal {
            value: product_definition(&finding.signal.value.key)
                .map_or(ServerCategory::JavaGameServer, |definition| definition.category),
            detector: finding.signal.detector,
            source: finding.signal.source,
            location: finding.signal.location.clone(),
            weight: finding.signal.weight,
            correlation_group: finding.signal.correlation_group,
        };
        push_claim(
            &signal,
            DetectionTarget::ServerCategory,
            category_name(signal.value).to_string(),
            evidence,
        )
    }));
    let category = resolve(category_claims);

    let version = resolve_product_values(
        &findings.product_versions,
        selected_key,
        DetectionTarget::ServerVersion,
        |value| value.clone(),
        evidence,
    );
    let release_channel = resolve_product_values(
        &findings.release_channels,
        selected_key,
        DetectionTarget::ReleaseChannel,
        |value| release_channel_name(*value).to_string(),
        evidence,
    );

    let ecosystems = {
        let mut claims = findings
            .ecosystems
            .iter()
            .map(|signal| {
                push_claim(
                    signal,
                    DetectionTarget::ServerEcosystem,
                    ecosystem_name(&signal.value),
                    evidence,
                )
            })
            .collect::<Vec<_>>();
        if let Some(selected_key) = selected_key {
            for finding in findings
                .products
                .iter()
                .filter(|finding| finding.signal.value.key == selected_key)
            {
                for ecosystem in &finding.ecosystems {
                    let signal = Signal {
                        value: ecosystem.clone(),
                        detector: finding.signal.detector,
                        source: finding.signal.source,
                        location: finding.signal.location.clone(),
                        weight: finding.signal.weight,
                        correlation_group: finding.signal.correlation_group,
                    };
                    let candidate = ecosystem_name(&signal.value);
                    claims.push(push_claim(
                        &signal,
                        DetectionTarget::ServerEcosystem,
                        candidate,
                        evidence,
                    ));
                }
            }
        }
        resolve_attributed(claims)
    };

    let components = resolve_attributed(
        findings
            .components
            .iter()
            .map(|finding| {
                push_claim(
                    &finding.signal,
                    DetectionTarget::Component,
                    format!("{}:{}", finding.product_key, finding.signal.value.key),
                    evidence,
                )
            })
            .collect(),
    );
    let roles = resolve_attributed(
        findings
            .roles
            .iter()
            .map(|signal| {
                push_claim(
                    signal,
                    DetectionTarget::ArtifactRole,
                    artifact_role_name(signal.value).to_string(),
                    evidence,
                )
            })
            .collect(),
    );
    let minecraft_version = resolve_minecraft_version(&findings, minecraft, evidence);
    let launches = resolve_attributed(
        findings
            .launches
            .iter()
            .map(|signal| {
                push_claim(
                    signal,
                    DetectionTarget::LaunchProfile,
                    signal.value.id.clone(),
                    evidence,
                )
            })
            .collect(),
    );
    let mut java_claims = findings
        .java_majors
        .iter()
        .map(|signal| {
            push_claim(signal, DetectionTarget::JavaMajor, signal.value.to_string(), evidence)
        })
        .collect::<Vec<_>>();
    if let Some(existing) = existing_java_major {
        java_claims.extend(existing_detected_claims(existing, "existing-java-requirement"));
    }
    let java_major = resolve(java_claims);

    let mut diagnostics = Vec::new();
    add_thresholded_resolution_diagnostic(
        path,
        "conflicting_server_implementations",
        "insufficient_server_implementation_evidence",
        "server implementation",
        &implementation,
        &mut diagnostics,
    );
    add_conflict_diagnostic(
        path,
        "conflicting_server_versions",
        "server implementation version",
        &version,
        &mut diagnostics,
    );
    add_conflict_diagnostic(
        path,
        "conflicting_minecraft_versions",
        "Minecraft version",
        &minecraft_version,
        &mut diagnostics,
    );

    DetectorOutput {
        identity: ServerIdentityInfo {
            category,
            implementation,
            version,
            release_channel,
            ecosystems,
        },
        minecraft_version,
        roles,
        components,
        launches,
        java_major,
        diagnostics,
    }
}

fn resolve_product_values<T, F>(
    findings: &[ProductValueFinding<T>],
    selected_key: Option<&str>,
    target: DetectionTarget,
    candidate: F,
    evidence: &mut EvidenceCollector,
) -> Detected<T>
where
    T: Clone + Eq,
    F: Fn(&T) -> String,
{
    let Some(selected_key) = selected_key else {
        return Detected::default();
    };
    resolve(
        findings
            .iter()
            .filter(|finding| finding.product_key == selected_key)
            .map(|finding| {
                push_claim(&finding.signal, target, candidate(&finding.signal.value), evidence)
            })
            .collect(),
    )
}

fn resolve_minecraft_version(
    findings: &Findings,
    minecraft: Option<&MinecraftVersionInfo>,
    evidence: &mut EvidenceCollector,
) -> Detected<String> {
    let mut claims = findings
        .minecraft_versions
        .iter()
        .map(|signal| {
            push_claim(signal, DetectionTarget::MinecraftVersion, signal.value.clone(), evidence)
        })
        .collect::<Vec<_>>();
    if let Some(minecraft) = minecraft {
        claims.extend(existing_detected_claims(&minecraft.version, "mojang-version-json"));
    }
    resolve(claims)
}

fn existing_detected_claims<T: Clone>(
    detected: &Detected<T>,
    correlation_group: &str,
) -> Vec<DetectionClaim<T>> {
    let mut claims = Vec::new();
    if let Some(value) = detected.value.as_ref() {
        claims.extend(detected.evidence.iter().map(|evidence| DetectionClaim {
            value: value.clone(),
            evidence: *evidence,
            weight: detected.confidence,
            correlation_group: correlation_group.to_string(),
        }));
    }
    claims.extend(detected.alternatives.iter().flat_map(|candidate| {
        candidate.evidence.iter().map(|evidence| DetectionClaim {
            value: candidate.value.clone(),
            evidence: *evidence,
            weight: candidate.confidence,
            correlation_group: correlation_group.to_string(),
        })
    }));
    claims
}

fn push_claim<T: Clone>(
    signal: &Signal<T>,
    target: DetectionTarget,
    candidate: String,
    evidence: &mut EvidenceCollector,
) -> DetectionClaim<T> {
    let evidence_id = evidence.push(NewEvidence {
        detector: signal.detector,
        source: signal.source,
        location: signal.location.clone(),
        target,
        candidate,
        weight: signal.weight,
        correlation_group: signal.correlation_group,
    });
    DetectionClaim {
        value: signal.value.clone(),
        evidence: evidence_id,
        weight: signal.weight,
        correlation_group: signal.correlation_group.to_string(),
    }
}

fn add_conflict_diagnostic<T>(
    path: &Path,
    code: &str,
    label: &str,
    detected: &Detected<T>,
    diagnostics: &mut Vec<InspectionDiagnostic>,
) {
    if detection_outcome(detected) != DetectionOutcome::Conflict {
        return;
    }
    diagnostics.push(InspectionDiagnostic {
        severity: DiagnosticSeverity::Warning,
        code: code.to_string(),
        message: format!("conflicting {label} evidence found in {}", path.display()),
        evidence: detected
            .alternatives
            .iter()
            .flat_map(|candidate| candidate.evidence.iter().copied())
            .collect(),
    });
}

fn add_thresholded_resolution_diagnostic<T>(
    path: &Path,
    conflict_code: &str,
    insufficient_code: &str,
    label: &str,
    detected: &Detected<T>,
    diagnostics: &mut Vec<InspectionDiagnostic>,
) {
    match server_implementation_outcome(detected) {
        DetectionOutcome::Conflict => {
            add_conflict_diagnostic(path, conflict_code, label, detected, diagnostics);
        }
        DetectionOutcome::InsufficientEvidence { minimum_confidence } => {
            diagnostics.push(InspectionDiagnostic {
                severity: DiagnosticSeverity::Info,
                code: insufficient_code.to_string(),
                message: format!(
                "{label} evidence in {} has confidence {}, below the required {minimum_confidence}",
                path.display(),
                detected.confidence
            ),
                evidence: detected.evidence.clone(),
            })
        }
        DetectionOutcome::Missing | DetectionOutcome::Selected => {}
    }
}

fn detect_filename(path: &Path, findings: &mut Findings) {
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return;
    };
    let stem = stem.to_ascii_lowercase();
    let Some(definition) = PRODUCTS
        .iter()
        .filter(|definition| contains_key_with_boundaries(&stem, definition.key))
        .max_by_key(|definition| definition.key.len())
    else {
        return;
    };
    findings.products.push(ProductFinding {
        signal: Signal {
            value: product_from_key(definition.key),
            detector: "server-file-name",
            source: EvidenceSource::FileName,
            location: EvidenceLocation::path(path.to_path_buf()),
            weight: 25,
            correlation_group: "file-name",
        },
        ecosystems: ecosystems_for_key(definition.key, false),
    });
}

fn contains_key_with_boundaries(value: &str, key: &str) -> bool {
    value.match_indices(key).any(|(index, matched)| {
        let before = value[..index].chars().next_back();
        let after = value[index + matched.len()..].chars().next();
        !before.is_some_and(|character| character.is_ascii_alphanumeric())
            && !after.is_some_and(|character| character.is_ascii_alphanumeric())
    })
}

pub(super) fn product_from_key(key: &str) -> ServerProduct {
    if let Some(definition) = product_definition(key) {
        return ServerProduct {
            key: definition.key.to_string(),
            display_name: definition.display_name.to_string(),
        };
    }
    ServerProduct {
        key: key.to_string(),
        display_name: display_name_from_key(key),
    }
}

pub(super) fn ecosystems_for_key(key: &str, paperclip_context: bool) -> Vec<ServerEcosystem> {
    let Some(definition) = product_definition(key) else {
        return if paperclip_context {
            vec![ServerEcosystem::Paper, ServerEcosystem::Bukkit]
        } else {
            Vec::new()
        };
    };
    definition.ecosystems.to_vec()
}

pub(super) fn product_key_from_coordinate(coordinate: &MavenCoordinate) -> Option<String> {
    PRODUCTS
        .iter()
        .find(|definition| {
            definition.api_group == Some(coordinate.group.as_str())
                && definition.api_artifact == Some(coordinate.artifact.as_str())
        })
        .map(|definition| definition.key.to_string())
}

pub(super) fn target_product_key(path: &str, minecraft_version: &str) -> Option<String> {
    let filename = path.rsplit(['/', '\\']).next()?;
    let stem = strip_jar_suffix(filename)?;
    if stem.eq_ignore_ascii_case(&format!("server-{minecraft_version}")) {
        return Some("vanilla".to_string());
    }
    let suffix = format!("-{minecraft_version}");
    let key = stem.strip_suffix(&suffix)?.to_ascii_lowercase();
    is_valid_product_key(&key).then_some(key)
}

pub(super) fn release_channel(version: &str) -> Option<ReleaseChannel> {
    let version = version.to_ascii_lowercase();
    if has_version_label(&version, "snapshot") {
        Some(ReleaseChannel::Snapshot)
    } else if version
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|token| {
            token == "rc"
                || token.strip_prefix("rc").is_some_and(|suffix| {
                    !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit())
                })
        })
    {
        Some(ReleaseChannel::ReleaseCandidate)
    } else if has_version_label(&version, "alpha") {
        Some(ReleaseChannel::Alpha)
    } else if has_version_label(&version, "beta") {
        Some(ReleaseChannel::Beta)
    } else if has_version_label(&version, "stable") {
        Some(ReleaseChannel::Stable)
    } else {
        None
    }
}

pub(super) fn strip_jar_suffix(filename: &str) -> Option<&str> {
    let suffix_start = filename.len().checked_sub(4)?;
    filename
        .get(suffix_start..)?
        .eq_ignore_ascii_case(".jar")
        .then(|| filename.get(..suffix_start))
        .flatten()
}

fn has_version_label(version: &str, label: &str) -> bool {
    version
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|token| {
            token == label
                || token.strip_prefix(label).is_some_and(|suffix| {
                    !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit())
                })
        })
}

pub(super) fn list_location(path: &Path, entry: &str, line: usize) -> EvidenceLocation {
    EvidenceLocation {
        path: path.to_path_buf(),
        archive_entry: Some(entry.to_string()),
        manifest_section: None,
        field: Some(format!("line {line}")),
    }
}

pub(super) fn manifest_location(path: &Path, field: &str) -> EvidenceLocation {
    EvidenceLocation {
        path: path.to_path_buf(),
        archive_entry: Some("META-INF/MANIFEST.MF".to_string()),
        manifest_section: None,
        field: Some(field.to_string()),
    }
}

pub(super) fn manifest_section_location(
    path: &Path,
    section: Option<&str>,
    field: &str,
) -> EvidenceLocation {
    EvidenceLocation {
        path: path.to_path_buf(),
        archive_entry: Some("META-INF/MANIFEST.MF".to_string()),
        manifest_section: section.map(str::to_string),
        field: Some(field.to_string()),
    }
}

pub(super) fn version_json_location(path: &Path, field: &str) -> EvidenceLocation {
    EvidenceLocation {
        path: path.to_path_buf(),
        archive_entry: Some("version.json".to_string()),
        manifest_section: None,
        field: Some(field.to_string()),
    }
}

fn product_definition(key: &str) -> Option<&'static ProductDefinition> {
    PRODUCTS.iter().find(|definition| definition.key == key)
}

fn is_valid_product_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 64
        && key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn display_name_from_key(key: &str) -> String {
    key.split(['-', '_', '.'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_ascii_uppercase().to_string() + characters.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn ecosystem_name(ecosystem: &ServerEcosystem) -> String {
    match ecosystem {
        ServerEcosystem::Vanilla => "vanilla".to_string(),
        ServerEcosystem::Bukkit => "bukkit".to_string(),
        ServerEcosystem::Paper => "paper".to_string(),
        ServerEcosystem::Fabric => "fabric".to_string(),
        ServerEcosystem::LegacyFabric => "legacy_fabric".to_string(),
        ServerEcosystem::Quilt => "quilt".to_string(),
        ServerEcosystem::Forge => "forge".to_string(),
        ServerEcosystem::NeoForge => "neoforge".to_string(),
        ServerEcosystem::Sponge => "sponge".to_string(),
        ServerEcosystem::Bungee => "bungee".to_string(),
        ServerEcosystem::Velocity => "velocity".to_string(),
        ServerEcosystem::Other(value) => value.clone(),
    }
}

fn category_name(category: ServerCategory) -> &'static str {
    match category {
        ServerCategory::JavaGameServer => "java_game_server",
        ServerCategory::BedrockGameServer => "bedrock_game_server",
        ServerCategory::Proxy => "proxy",
        ServerCategory::Limbo => "limbo",
        ServerCategory::Unknown => "unknown",
    }
}

fn release_channel_name(channel: ReleaseChannel) -> &'static str {
    match channel {
        ReleaseChannel::Stable => "stable",
        ReleaseChannel::ReleaseCandidate => "release_candidate",
        ReleaseChannel::Beta => "beta",
        ReleaseChannel::Alpha => "alpha",
        ReleaseChannel::Snapshot => "snapshot",
        ReleaseChannel::Development => "development",
        ReleaseChannel::Unknown => "unknown",
    }
}

fn artifact_role_name(role: ArtifactRole) -> &'static str {
    match role {
        ArtifactRole::Runnable => "runnable",
        ArtifactRole::Bootstrapper => "bootstrapper",
        ArtifactRole::Installer => "installer",
        ArtifactRole::Launcher => "launcher",
        ArtifactRole::Wrapper => "wrapper",
        ArtifactRole::Library => "library",
        ArtifactRole::InstallationDirectory => "installation_directory",
        ArtifactRole::Unknown => "unknown",
    }
}

pub(super) fn api_component(
    product_key: &str,
    version: String,
    coordinate: Option<MavenCoordinate>,
    source_path: PathBuf,
) -> ServerComponent {
    let product = product_from_key(product_key);
    let release_channel = release_channel(&version);
    ServerComponent {
        kind: ServerComponentKind::Api,
        key: format!("{product_key}-api"),
        name: format!("{} API", product.display_name),
        version: Some(version),
        release_channel,
        coordinate,
        source_path: Some(source_path),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        contains_key_with_boundaries, product_definition, release_channel, target_product_key,
    };
    use crate::provisioning::server_inspection::{ReleaseChannel, ServerCategory};

    #[test]
    fn product_definitions_own_their_server_categories() {
        assert_eq!(
            product_definition("velocity").map(|definition| definition.category),
            Some(ServerCategory::Proxy)
        );
        assert_eq!(
            product_definition("nanolimbo").map(|definition| definition.category),
            Some(ServerCategory::Limbo)
        );
        assert_eq!(
            product_definition("paper").map(|definition| definition.category),
            Some(ServerCategory::JavaGameServer)
        );
    }

    #[test]
    fn filename_boundaries_do_not_treat_aspaper_as_paper() {
        assert!(contains_key_with_boundaries("aspaper-26.2", "aspaper"));
        assert!(!contains_key_with_boundaries("aspaper-26.2", "paper"));
        assert!(!contains_key_with_boundaries("paperclip", "paper"));
    }

    #[test]
    fn target_paths_produce_open_product_keys() {
        assert_eq!(target_product_key("26.2/canvas-26.2.jar", "26.2").as_deref(), Some("canvas"));
        assert_eq!(target_product_key("26.2/server-26.2.jar", "26.2").as_deref(), Some("vanilla"));
        assert_eq!(target_product_key("26.2/Paper-26.2.Jar", "26.2").as_deref(), Some("paper"));
    }

    #[test]
    fn release_channel_requires_a_version_label_boundary() {
        assert_eq!(release_channel("26.2.build.1-beta2"), Some(ReleaseChannel::Beta));
        assert_eq!(release_channel("20.0.0-RC2673"), Some(ReleaseChannel::ReleaseCandidate));
        assert_eq!(release_channel("20.0.0-RC"), Some(ReleaseChannel::ReleaseCandidate));
        assert_eq!(release_channel("26.2-unstable"), None);
    }
}
