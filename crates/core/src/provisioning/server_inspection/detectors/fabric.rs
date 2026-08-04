use std::path::{Path, PathBuf};

use super::super::archive::{ArchiveMetadata, INSTALL_PROPERTIES_ENTRY};
use super::super::formats::{java_properties, manifest};
use super::super::model::{
    ArtifactRole, EvidenceLocation, EvidenceSource, LaunchPlatform, LaunchProfile, LaunchTarget,
    ServerComponent, ServerComponentKind,
};
use super::{
    ecosystems_for_key, manifest_location, product_from_key, ComponentFinding, Findings,
    ProductFinding, ProductValueFinding, Signal,
};

const FABRIC_SERVER_LAUNCHER: &str = "net.fabricmc.installer.ServerLauncher";

pub(super) fn detect(
    subject_path: &Path,
    archive: &ArchiveMetadata,
    directory_launch: Option<(&Path, &Path)>,
    findings: &mut Findings,
) {
    let Some(manifest_bytes) = archive.manifest.as_deref() else {
        return;
    };
    let parsed_manifest = manifest::parse(manifest_bytes);
    let main_class = parsed_manifest.main_value("Main-Class");
    let title = parsed_manifest.main_value("Implementation-Title");
    let installer_version = parsed_manifest.main_value("Implementation-Version");
    let properties = archive
        .install_properties
        .as_deref()
        .map(java_properties::parse)
        .unwrap_or_default();
    let loader_version = properties.get("fabric-loader-version");
    let game_version = properties.get("game-version");

    let (product_key, product_weight, product_field) = match title {
        Some(title) if title.eq_ignore_ascii_case("LegacyFabricInstaller") => {
            ("legacy-fabric", 98, "Implementation-Title")
        }
        Some(title) if title.eq_ignore_ascii_case("FabricInstaller") => {
            ("fabric", 98, "Implementation-Title")
        }
        _ if main_class == Some(FABRIC_SERVER_LAUNCHER)
            && loader_version.is_some()
            && game_version.is_some() =>
        {
            ("fabric", 90, "Main-Class")
        }
        _ => return,
    };

    let product_location = EvidenceLocation {
        path: subject_path.to_path_buf(),
        archive_entry: Some("META-INF/MANIFEST.MF".to_string()),
        manifest_section: None,
        field: Some(product_field.to_string()),
    };
    findings.products.push(ProductFinding {
        signal: Signal {
            value: product_from_key(product_key),
            detector: "fabric-installer-manifest",
            source: EvidenceSource::ManifestMain,
            location: product_location,
            weight: product_weight,
            correlation_group: "fabric-installer-manifest",
        },
        ecosystems: ecosystems_for_key(product_key, false),
    });

    if main_class == Some(FABRIC_SERVER_LAUNCHER) {
        findings.roles.push(Signal {
            value: ArtifactRole::Launcher,
            detector: "fabric-server-launcher",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(subject_path, "Main-Class"),
            weight: 95,
            correlation_group: "fabric-installer-manifest",
        });
    }
    if title.is_some_and(|title| title.to_ascii_lowercase().contains("installer")) {
        findings.roles.push(Signal {
            value: ArtifactRole::Installer,
            detector: "fabric-installer-manifest",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(subject_path, "Implementation-Title"),
            weight: 95,
            correlation_group: "fabric-installer-manifest",
        });
    }

    let properties_location = EvidenceLocation {
        path: subject_path.to_path_buf(),
        archive_entry: Some(INSTALL_PROPERTIES_ENTRY.to_string()),
        manifest_section: None,
        field: None,
    };
    if loader_version.is_some() && game_version.is_some() {
        findings.products.push(ProductFinding {
            signal: Signal {
                value: product_from_key(product_key),
                detector: "fabric-install-properties",
                source: EvidenceSource::PropertiesField,
                location: EvidenceLocation {
                    field: Some("fabric-loader-version".to_string()),
                    ..properties_location.clone()
                },
                weight: 90,
                correlation_group: "fabric-install-properties",
            },
            ecosystems: ecosystems_for_key(product_key, false),
        });
    }
    if let Some(loader_version) = loader_version {
        let location = EvidenceLocation {
            field: Some("fabric-loader-version".to_string()),
            ..properties_location.clone()
        };
        findings.product_versions.push(ProductValueFinding {
            product_key: product_key.to_string(),
            signal: Signal {
                value: loader_version.clone(),
                detector: "fabric-install-properties",
                source: EvidenceSource::PropertiesField,
                location: location.clone(),
                weight: 98,
                correlation_group: "fabric-install-properties",
            },
        });
        findings.components.push(ComponentFinding {
            product_key: product_key.to_string(),
            signal: Signal {
                value: ServerComponent {
                    kind: ServerComponentKind::ModLoader,
                    key: product_key.to_string(),
                    name: if product_key == "legacy-fabric" {
                        "Legacy Fabric Loader".to_string()
                    } else {
                        "Fabric Loader".to_string()
                    },
                    version: Some(loader_version.clone()),
                    release_channel: None,
                    coordinate: None,
                    source_path: Some(PathBuf::from(INSTALL_PROPERTIES_ENTRY)),
                },
                detector: "fabric-install-properties",
                source: EvidenceSource::PropertiesField,
                location,
                weight: 98,
                correlation_group: "fabric-install-properties",
            },
        });
    }
    if let Some(game_version) = game_version {
        findings.minecraft_versions.push(Signal {
            value: game_version.clone(),
            detector: "fabric-install-properties",
            source: EvidenceSource::PropertiesField,
            location: EvidenceLocation {
                field: Some("game-version".to_string()),
                ..properties_location
            },
            weight: 98,
            correlation_group: "fabric-install-properties",
        });
    }
    if let Some(installer_version) = installer_version {
        findings.components.push(ComponentFinding {
            product_key: product_key.to_string(),
            signal: Signal {
                value: ServerComponent {
                    kind: ServerComponentKind::Installer,
                    key: "fabric-installer".to_string(),
                    name: if product_key == "legacy-fabric" {
                        "Legacy Fabric Installer".to_string()
                    } else {
                        "Fabric Installer".to_string()
                    },
                    version: Some(installer_version.to_string()),
                    release_channel: None,
                    coordinate: None,
                    source_path: Some(PathBuf::from("META-INF/MANIFEST.MF")),
                },
                detector: "fabric-installer-manifest",
                source: EvidenceSource::ManifestMain,
                location: manifest_location(subject_path, "Implementation-Version"),
                weight: 90,
                correlation_group: "fabric-installer-manifest",
            },
        });
    }

    if let Some((root, relative_jar)) = directory_launch {
        findings.launches.push(Signal {
            value: LaunchProfile {
                id: format!(
                    "{}-{}",
                    product_key,
                    relative_jar
                        .to_string_lossy()
                        .chars()
                        .map(|character| if character.is_ascii_alphanumeric() {
                            character.to_ascii_lowercase()
                        } else {
                            '-'
                        })
                        .collect::<String>()
                ),
                platform: LaunchPlatform::Any,
                working_directory: Some(root.to_path_buf()),
                target: LaunchTarget::Jar { path: root.join(relative_jar) },
                jvm_arguments: Vec::new(),
                program_arguments: Vec::new(),
                required_java_major: None,
            },
            detector: "fabric-server-launcher",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(subject_path, "Main-Class"),
            weight: 95,
            correlation_group: "fabric-installer-manifest",
        });
    }
}
