use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::super::super::{parse_startup_script_content, CoreKind, StartupScriptKind};
use super::super::archive::{
    ArchiveMetadata, BOOTSTRAP_LIST_ENTRY, BOOTSTRAP_PROPERTIES_ENTRY, FORGE_VERSION_ENTRY,
    NEOFORGE_VERSION_PROPERTIES_ENTRY, WRAPPER_METADATA_ENTRY,
};
use super::super::directory::{
    MetadataFile, ModLoaderFamily, ModLoaderInstallation, StartupScript,
};
use super::super::formats::{java_properties, jvm_args, manifest, paperclip_list};
use super::super::model::{
    ArtifactRole, EvidenceLocation, EvidenceSource, LaunchPlatform, LaunchProfile, LaunchTarget,
    MavenCoordinate, ReleaseChannel, ServerComponent, ServerComponentKind,
};
use super::{
    ecosystems_for_key, product_from_key, release_channel, ComponentFinding, Findings,
    ProductFinding, ProductValueFinding, Signal,
};

pub(super) fn detect_archive(
    archive_path: &Path,
    directory_launch: Option<(&Path, &Path)>,
    archive: &ArchiveMetadata,
    findings: &mut Findings,
) {
    detect_forge_bootstrap(archive_path, directory_launch, archive, findings);
    detect_neoforge_wrapper(archive_path, directory_launch, archive, findings);
}

pub(super) fn detect_installation(
    root: &Path,
    installation: &ModLoaderInstallation,
    findings: &mut Findings,
) {
    let (product_key, group, artifact) = match installation.family {
        ModLoaderFamily::Forge => ("forge", "net.minecraftforge", "forge"),
        ModLoaderFamily::NeoForge => ("neoforge", "net.neoforged", "neoforge"),
    };
    let coordinate_location = EvidenceLocation {
        path: root.join(&installation.relative_directory),
        archive_entry: None,
        manifest_section: None,
        field: Some("Maven coordinate directory".to_string()),
    };
    add_product(
        product_key,
        "modloader-coordinate-directory",
        EvidenceSource::DirectoryLayout,
        coordinate_location.clone(),
        95,
        "modloader-coordinate-directory",
        findings,
    );

    let (loader_version, coordinate_minecraft) = match installation.family {
        ModLoaderFamily::Forge => split_forge_coordinate(&installation.coordinate_version)
            .map_or((installation.coordinate_version.as_str(), None), |(minecraft, forge)| {
                (forge, Some(minecraft))
            }),
        ModLoaderFamily::NeoForge => (installation.coordinate_version.as_str(), None),
    };
    add_product_version(
        product_key,
        loader_version,
        "modloader-coordinate-directory",
        EvidenceSource::MavenCoordinate,
        coordinate_location.clone(),
        95,
        "modloader-coordinate-directory",
        findings,
    );
    if let Some(minecraft) = coordinate_minecraft {
        add_minecraft_version(
            minecraft,
            "forge-coordinate-directory",
            EvidenceSource::MavenCoordinate,
            coordinate_location.clone(),
            90,
            "modloader-coordinate-directory",
            findings,
        );
    }
    if let Some(channel) = release_channel(loader_version) {
        add_release_channel(
            product_key,
            channel,
            "modloader-coordinate-directory",
            EvidenceSource::MavenCoordinate,
            coordinate_location.clone(),
            90,
            "modloader-coordinate-directory",
            findings,
        );
    }
    findings.components.push(ComponentFinding {
        product_key: product_key.to_string(),
        signal: Signal {
            value: ServerComponent {
                kind: ServerComponentKind::ModLoader,
                key: product_key.to_string(),
                name: if product_key == "forge" {
                    "Forge".to_string()
                } else {
                    "NeoForge".to_string()
                },
                version: Some(loader_version.to_string()),
                release_channel: release_channel(loader_version),
                coordinate: Some(MavenCoordinate {
                    group: group.to_string(),
                    artifact: artifact.to_string(),
                    version: installation.coordinate_version.clone(),
                    classifier: None,
                    extension: None,
                }),
                source_path: Some(installation.relative_directory.clone()),
            },
            detector: "modloader-coordinate-directory",
            source: EvidenceSource::MavenCoordinate,
            location: coordinate_location,
            weight: 95,
            correlation_group: "modloader-coordinate-directory",
        },
    });

    if let (Some(nested_path), Some(metadata)) =
        (installation.nested_archive_path.as_ref(), installation.nested_metadata.as_ref())
    {
        match installation.family {
            ModLoaderFamily::Forge => {
                detect_forge_version_json(root, nested_path, metadata, findings)
            }
            ModLoaderFamily::NeoForge => {
                detect_neoforge_properties(root, nested_path, metadata, findings)
            }
        }
    }
    if let Some(arguments) = installation.windows_args.as_ref() {
        detect_argument_file(
            root,
            product_key,
            &installation.coordinate_version,
            LaunchPlatform::Windows,
            arguments,
            findings,
        );
    }
    if let Some(arguments) = installation.unix_args.as_ref() {
        detect_argument_file(
            root,
            product_key,
            &installation.coordinate_version,
            LaunchPlatform::Unix,
            arguments,
            findings,
        );
    }
}

pub(super) fn detect_script(root: &Path, script: &StartupScript, findings: &mut Findings) {
    let parsed = parse_startup_script_content(script.kind, &script.content);
    let product_key = match parsed.inferred_core {
        CoreKind::Forge => "forge",
        CoreKind::NeoForge => "neoforge",
        _ => return,
    };
    let location = EvidenceLocation {
        path: root.join(&script.relative_path),
        archive_entry: None,
        manifest_section: None,
        field: Some("Java launch".to_string()),
    };
    add_product(
        product_key,
        "modloader-startup-script",
        EvidenceSource::ArgumentFile,
        location.clone(),
        90,
        "startup-script",
        findings,
    );
    for (index, launch) in parsed.launches.into_iter().enumerate() {
        findings.launches.push(Signal {
            value: LaunchProfile {
                id: format!(
                    "{product_key}-{}-{}",
                    launch_id_part(&script.relative_path),
                    index + 1
                ),
                platform: platform_for_script(script.kind),
                working_directory: Some(root.to_path_buf()),
                target: LaunchTarget::Script { path: root.join(&script.relative_path) },
                jvm_arguments: launch.jvm_arguments,
                program_arguments: launch.application_arguments,
                required_java_major: None,
            },
            detector: "modloader-startup-script",
            source: EvidenceSource::ArgumentFile,
            location: location.clone(),
            weight: 90,
            correlation_group: "startup-script",
        });
    }
}

fn detect_forge_bootstrap(
    archive_path: &Path,
    directory_launch: Option<(&Path, &Path)>,
    archive: &ArchiveMetadata,
    findings: &mut Findings,
) {
    let Some(content) = archive.bootstrap_properties.as_deref() else {
        return;
    };
    let properties = java_properties::parse(content);
    if properties.get("Main-Class").map(String::as_str)
        != Some("net.minecraftforge.bootstrap.ForgeBootstrap")
    {
        return;
    }
    let location = archive_location(archive_path, BOOTSTRAP_PROPERTIES_ENTRY, Some("Main-Class"));
    add_product(
        "forge",
        "forge-bootstrap-properties",
        EvidenceSource::PropertiesField,
        location.clone(),
        95,
        "forge-bootstrap-properties",
        findings,
    );
    findings.roles.push(Signal {
        value: ArtifactRole::Bootstrapper,
        detector: "forge-bootstrap-properties",
        source: EvidenceSource::PropertiesField,
        location: location.clone(),
        weight: 95,
        correlation_group: "forge-bootstrap-properties",
    });
    findings.roles.push(Signal {
        value: ArtifactRole::Launcher,
        detector: "forge-bootstrap-properties",
        source: EvidenceSource::PropertiesField,
        location: location.clone(),
        weight: 90,
        correlation_group: "forge-bootstrap-properties",
    });
    if let Some(java_major) = properties
        .get("Java-Version")
        .and_then(|version| version.parse::<u16>().ok())
    {
        findings.java_majors.push(Signal {
            value: java_major,
            detector: "forge-bootstrap-properties",
            source: EvidenceSource::PropertiesField,
            location: archive_location(
                archive_path,
                BOOTSTRAP_PROPERTIES_ENTRY,
                Some("Java-Version"),
            ),
            weight: 95,
            correlation_group: "forge-bootstrap-properties",
        });
    }
    if let Some((root, relative_jar)) = directory_launch {
        findings.launches.push(Signal {
            value: LaunchProfile {
                id: "forge-bootstrap-jar".to_string(),
                platform: LaunchPlatform::Any,
                working_directory: Some(root.to_path_buf()),
                target: LaunchTarget::Jar { path: root.join(relative_jar) },
                jvm_arguments: Vec::new(),
                program_arguments: Vec::new(),
                required_java_major: properties
                    .get("Java-Version")
                    .and_then(|version| version.parse().ok()),
            },
            detector: "forge-bootstrap-properties",
            source: EvidenceSource::PropertiesField,
            location,
            weight: 95,
            correlation_group: "forge-bootstrap-properties",
        });
    }

    if let Some(content) = archive.bootstrap_list.as_deref() {
        for entry in paperclip_list::parse_libraries(content) {
            let Some(coordinate) = entry.coordinate else {
                continue;
            };
            if coordinate.group != "net.minecraftforge" || coordinate.artifact != "forge" {
                continue;
            }
            if let Some((minecraft, forge)) = split_forge_coordinate(&coordinate.version) {
                let coordinate_location = archive_location(
                    archive_path,
                    BOOTSTRAP_LIST_ENTRY,
                    Some(&format!("line {}", entry.line)),
                );
                add_product_version(
                    "forge",
                    forge,
                    "forge-bootstrap-list",
                    EvidenceSource::MavenCoordinate,
                    coordinate_location.clone(),
                    95,
                    "forge-bootstrap-list",
                    findings,
                );
                add_minecraft_version(
                    minecraft,
                    "forge-bootstrap-list",
                    EvidenceSource::MavenCoordinate,
                    coordinate_location,
                    95,
                    "forge-bootstrap-list",
                    findings,
                );
            }
        }
    }

    if let Some(manifest_bytes) = archive.manifest.as_deref() {
        let parsed = manifest::parse(manifest_bytes);
        for section in &parsed.summary.sections {
            let Some(version) = attribute(&section.attributes, "Implementation-Version") else {
                continue;
            };
            if attribute(&section.attributes, "Implementation-Title") != Some("bs-shim") {
                continue;
            }
            findings.components.push(ComponentFinding {
                product_key: "forge".to_string(),
                signal: Signal {
                    value: ServerComponent {
                        kind: ServerComponentKind::Bootstrap,
                        key: "forge-bootstrap-shim".to_string(),
                        name: "Forge Bootstrap Shim".to_string(),
                        version: Some(version.to_string()),
                        release_channel: None,
                        coordinate: None,
                        source_path: Some(PathBuf::from("server.jar")),
                    },
                    detector: "forge-bootstrap-manifest",
                    source: EvidenceSource::ManifestSection,
                    location: EvidenceLocation {
                        path: archive_path.to_path_buf(),
                        archive_entry: Some("META-INF/MANIFEST.MF".to_string()),
                        manifest_section: section.name.clone(),
                        field: Some("Implementation-Version".to_string()),
                    },
                    weight: 85,
                    correlation_group: "forge-bootstrap-manifest",
                },
            });
        }
    }
}

fn detect_neoforge_wrapper(
    archive_path: &Path,
    directory_launch: Option<(&Path, &Path)>,
    archive: &ArchiveMetadata,
    findings: &mut Findings,
) {
    let Some(content) = archive.wrapper_metadata.as_deref() else {
        return;
    };
    let Ok(Value::Object(metadata)) = serde_json::from_slice::<Value>(content) else {
        return;
    };
    let Some(neoforge) = metadata.get("neoforge").and_then(Value::as_str) else {
        return;
    };
    let location = archive_location(archive_path, WRAPPER_METADATA_ENTRY, Some("neoforge"));
    add_product(
        "neoforge",
        "neoforge-wrapper-metadata",
        EvidenceSource::JsonField,
        location.clone(),
        85,
        "neoforge-wrapper-metadata",
        findings,
    );
    add_product_version(
        "neoforge",
        neoforge,
        "neoforge-wrapper-metadata",
        EvidenceSource::JsonField,
        location.clone(),
        85,
        "neoforge-wrapper-metadata",
        findings,
    );
    if let Some(channel) = release_channel(neoforge) {
        add_release_channel(
            "neoforge",
            channel,
            "neoforge-wrapper-metadata",
            EvidenceSource::JsonField,
            location.clone(),
            85,
            "neoforge-wrapper-metadata",
            findings,
        );
    }
    if let Some(minecraft) = metadata.get("version").and_then(Value::as_str) {
        add_minecraft_version(
            minecraft,
            "neoforge-wrapper-metadata",
            EvidenceSource::JsonField,
            archive_location(archive_path, WRAPPER_METADATA_ENTRY, Some("version")),
            85,
            "neoforge-wrapper-metadata",
            findings,
        );
    }
    findings.roles.push(Signal {
        value: ArtifactRole::Wrapper,
        detector: "neoforge-wrapper-metadata",
        source: EvidenceSource::JsonField,
        location: location.clone(),
        weight: 90,
        correlation_group: "neoforge-wrapper-metadata",
    });
    findings.roles.push(Signal {
        value: ArtifactRole::Launcher,
        detector: "neoforge-wrapper-metadata",
        source: EvidenceSource::JsonField,
        location: location.clone(),
        weight: 85,
        correlation_group: "neoforge-wrapper-metadata",
    });
    if let Some((root, relative_jar)) = directory_launch {
        findings.launches.push(Signal {
            value: LaunchProfile {
                id: "neoforge-wrapper-jar".to_string(),
                platform: LaunchPlatform::Any,
                working_directory: Some(root.to_path_buf()),
                target: LaunchTarget::Jar { path: root.join(relative_jar) },
                jvm_arguments: Vec::new(),
                program_arguments: Vec::new(),
                required_java_major: None,
            },
            detector: "neoforge-wrapper-metadata",
            source: EvidenceSource::JsonField,
            location,
            weight: 85,
            correlation_group: "neoforge-wrapper-metadata",
        });
    }
}

fn detect_forge_version_json(
    root: &Path,
    nested_path: &Path,
    metadata: &ArchiveMetadata,
    findings: &mut Findings,
) {
    let Some(content) = metadata.forge_version.as_deref() else {
        return;
    };
    let Ok(Value::Object(version)) = serde_json::from_slice::<Value>(content) else {
        return;
    };
    let archive_path = root.join(nested_path);
    if let Some(forge) = version.get("forge").and_then(Value::as_str) {
        add_product(
            "forge",
            "forge-version-json",
            EvidenceSource::JsonField,
            archive_location(&archive_path, FORGE_VERSION_ENTRY, Some("forge")),
            98,
            "forge-version-json",
            findings,
        );
        add_product_version(
            "forge",
            forge,
            "forge-version-json",
            EvidenceSource::JsonField,
            archive_location(&archive_path, FORGE_VERSION_ENTRY, Some("forge")),
            98,
            "forge-version-json",
            findings,
        );
    }
    if let Some(minecraft) = version.get("mc").and_then(Value::as_str) {
        add_minecraft_version(
            minecraft,
            "forge-version-json",
            EvidenceSource::JsonField,
            archive_location(&archive_path, FORGE_VERSION_ENTRY, Some("mc")),
            98,
            "forge-version-json",
            findings,
        );
    }
    if let Some(mapping) = version.get("mcp").and_then(Value::as_str) {
        findings.components.push(ComponentFinding {
            product_key: "forge".to_string(),
            signal: Signal {
                value: ServerComponent {
                    kind: ServerComponentKind::Mapping,
                    key: "mcp".to_string(),
                    name: "MCP Mappings".to_string(),
                    version: Some(mapping.to_string()),
                    release_channel: None,
                    coordinate: None,
                    source_path: Some(nested_path.to_path_buf()),
                },
                detector: "forge-version-json",
                source: EvidenceSource::JsonField,
                location: archive_location(&archive_path, FORGE_VERSION_ENTRY, Some("mcp")),
                weight: 95,
                correlation_group: "forge-version-json",
            },
        });
    }
}

fn detect_neoforge_properties(
    root: &Path,
    nested_path: &Path,
    metadata: &ArchiveMetadata,
    findings: &mut Findings,
) {
    let Some(content) = metadata.neoforge_version_properties.as_deref() else {
        return;
    };
    let properties = java_properties::parse(content);
    let archive_path = root.join(nested_path);
    if let Some(neoforge) = properties.get("neoforge_version") {
        add_product(
            "neoforge",
            "neoforge-version-properties",
            EvidenceSource::PropertiesField,
            archive_location(
                &archive_path,
                NEOFORGE_VERSION_PROPERTIES_ENTRY,
                Some("neoforge_version"),
            ),
            98,
            "neoforge-version-properties",
            findings,
        );
        add_product_version(
            "neoforge",
            neoforge,
            "neoforge-version-properties",
            EvidenceSource::PropertiesField,
            archive_location(
                &archive_path,
                NEOFORGE_VERSION_PROPERTIES_ENTRY,
                Some("neoforge_version"),
            ),
            98,
            "neoforge-version-properties",
            findings,
        );
    }
    if let Some(minecraft) = properties.get("minecraft_version") {
        add_minecraft_version(
            minecraft,
            "neoforge-version-properties",
            EvidenceSource::PropertiesField,
            archive_location(
                &archive_path,
                NEOFORGE_VERSION_PROPERTIES_ENTRY,
                Some("minecraft_version"),
            ),
            98,
            "neoforge-version-properties",
            findings,
        );
    }
    if let Some(channel) = properties
        .get("build_type")
        .and_then(|build_type| release_channel(build_type))
    {
        add_release_channel(
            "neoforge",
            channel,
            "neoforge-version-properties",
            EvidenceSource::PropertiesField,
            archive_location(&archive_path, NEOFORGE_VERSION_PROPERTIES_ENTRY, Some("build_type")),
            95,
            "neoforge-version-properties",
            findings,
        );
    }
    if let Some(mapping) = properties.get("neoform_version") {
        findings.components.push(ComponentFinding {
            product_key: "neoforge".to_string(),
            signal: Signal {
                value: ServerComponent {
                    kind: ServerComponentKind::Mapping,
                    key: "neoform".to_string(),
                    name: "NeoForm".to_string(),
                    version: Some(mapping.clone()),
                    release_channel: None,
                    coordinate: None,
                    source_path: Some(nested_path.to_path_buf()),
                },
                detector: "neoforge-version-properties",
                source: EvidenceSource::PropertiesField,
                location: archive_location(
                    &archive_path,
                    NEOFORGE_VERSION_PROPERTIES_ENTRY,
                    Some("neoform_version"),
                ),
                weight: 95,
                correlation_group: "neoforge-version-properties",
            },
        });
    }
}

fn detect_argument_file(
    root: &Path,
    product_key: &'static str,
    coordinate_version: &str,
    platform: LaunchPlatform,
    arguments: &MetadataFile,
    findings: &mut Findings,
) {
    let parsed = jvm_args::parse(&arguments.content);
    if parsed.arguments.is_empty() {
        return;
    }
    let location = EvidenceLocation {
        path: root.join(&arguments.relative_path),
        archive_entry: None,
        manifest_section: None,
        field: None,
    };
    let version_flag = if product_key == "forge" {
        "--fml.forgeVersion"
    } else {
        "--fml.neoForgeVersion"
    };
    if let Some(version) = parsed.value_after(version_flag) {
        add_product_version(
            product_key,
            version,
            "modloader-argument-file",
            EvidenceSource::ArgumentFile,
            EvidenceLocation {
                field: Some(version_flag.to_string()),
                ..location.clone()
            },
            98,
            "modloader-argument-file",
            findings,
        );
        if let Some(channel) = release_channel(version) {
            add_release_channel(
                product_key,
                channel,
                "modloader-argument-file",
                EvidenceSource::ArgumentFile,
                EvidenceLocation {
                    field: Some(version_flag.to_string()),
                    ..location.clone()
                },
                95,
                "modloader-argument-file",
                findings,
            );
        }
    }
    if let Some(minecraft) = parsed.value_after("--fml.mcVersion") {
        add_minecraft_version(
            minecraft,
            "modloader-argument-file",
            EvidenceSource::ArgumentFile,
            EvidenceLocation {
                field: Some("--fml.mcVersion".to_string()),
                ..location.clone()
            },
            98,
            "modloader-argument-file",
            findings,
        );
    }
    if parsed.value_after(version_flag).is_some() || parsed.jar_target().is_some() {
        add_product(
            product_key,
            "modloader-argument-file",
            EvidenceSource::ArgumentFile,
            location.clone(),
            95,
            "modloader-argument-file",
            findings,
        );
    }
    findings.launches.push(Signal {
        value: LaunchProfile {
            id: format!(
                "{product_key}-{coordinate_version}-{}-args",
                match platform {
                    LaunchPlatform::Windows => "windows",
                    LaunchPlatform::Unix => "unix",
                    LaunchPlatform::Any => "any",
                }
            ),
            platform,
            working_directory: Some(root.to_path_buf()),
            target: LaunchTarget::ArgumentFiles {
                paths: vec![root.join(&arguments.relative_path)],
            },
            jvm_arguments: Vec::new(),
            program_arguments: Vec::new(),
            required_java_major: None,
        },
        detector: "modloader-argument-file",
        source: EvidenceSource::ArgumentFile,
        location,
        weight: 98,
        correlation_group: "modloader-argument-file",
    });
}

#[allow(clippy::too_many_arguments)]
fn add_product(
    key: &str,
    detector: &'static str,
    source: EvidenceSource,
    location: EvidenceLocation,
    weight: u8,
    correlation_group: &'static str,
    findings: &mut Findings,
) {
    findings.products.push(ProductFinding {
        signal: Signal {
            value: product_from_key(key),
            detector,
            source,
            location,
            weight,
            correlation_group,
        },
        ecosystems: ecosystems_for_key(key, false),
    });
}

#[allow(clippy::too_many_arguments)]
fn add_product_version(
    product_key: &str,
    version: &str,
    detector: &'static str,
    source: EvidenceSource,
    location: EvidenceLocation,
    weight: u8,
    correlation_group: &'static str,
    findings: &mut Findings,
) {
    findings.product_versions.push(ProductValueFinding {
        product_key: product_key.to_string(),
        signal: Signal {
            value: version.to_string(),
            detector,
            source,
            location,
            weight,
            correlation_group,
        },
    });
}

#[allow(clippy::too_many_arguments)]
fn add_minecraft_version(
    version: &str,
    detector: &'static str,
    source: EvidenceSource,
    location: EvidenceLocation,
    weight: u8,
    correlation_group: &'static str,
    findings: &mut Findings,
) {
    findings.minecraft_versions.push(Signal {
        value: version.to_string(),
        detector,
        source,
        location,
        weight,
        correlation_group,
    });
}

#[allow(clippy::too_many_arguments)]
fn add_release_channel(
    product_key: &str,
    channel: ReleaseChannel,
    detector: &'static str,
    source: EvidenceSource,
    location: EvidenceLocation,
    weight: u8,
    correlation_group: &'static str,
    findings: &mut Findings,
) {
    findings.release_channels.push(ProductValueFinding {
        product_key: product_key.to_string(),
        signal: Signal {
            value: channel,
            detector,
            source,
            location,
            weight,
            correlation_group,
        },
    });
}

fn split_forge_coordinate(version: &str) -> Option<(&str, &str)> {
    version.match_indices('-').find_map(|(separator, _)| {
        let minecraft = &version[..separator];
        let forge = &version[separator + 1..];
        let numeric_prefix = forge.split_once('-').map_or(forge, |(prefix, _)| prefix);
        (!minecraft.is_empty()
            && numeric_prefix
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_digit())
            && numeric_prefix.matches('.').count() >= 2)
            .then_some((minecraft, forge))
    })
}

fn launch_id_part(path: &Path) -> String {
    path.to_string_lossy()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn archive_location(path: &Path, entry: &str, field: Option<&str>) -> EvidenceLocation {
    EvidenceLocation {
        path: path.to_path_buf(),
        archive_entry: Some(entry.to_string()),
        manifest_section: None,
        field: field.map(str::to_string),
    }
}

fn attribute<'a>(attributes: &'a BTreeMap<String, String>, key: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.as_str())
}

fn platform_for_script(kind: StartupScriptKind) -> LaunchPlatform {
    match kind {
        StartupScriptKind::Batch | StartupScriptKind::PowerShell => LaunchPlatform::Windows,
        StartupScriptKind::Shell => LaunchPlatform::Unix,
    }
}

#[cfg(test)]
mod tests {
    use super::split_forge_coordinate;

    #[test]
    fn splits_forge_coordinate_at_the_final_separator() {
        assert_eq!(split_forge_coordinate("1.20.1-47.2.0"), Some(("1.20.1", "47.2.0")));
        assert_eq!(split_forge_coordinate("26.2-65.1.0"), Some(("26.2", "65.1.0")));
        assert_eq!(
            split_forge_coordinate("1.20.2-pre1-48.0.0-beta"),
            Some(("1.20.2-pre1", "48.0.0-beta"))
        );
    }
}
