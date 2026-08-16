use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::super::archive::{
    ARCLIGHT_LAUNCH_PROPERTIES_ENTRY, ArchiveMetadata, WRAPPER_METADATA_ENTRY,
};
use super::super::formats::{java_properties, manifest};
use super::super::model::{
    ArtifactRole, EvidenceLocation, EvidenceSource, MavenCoordinate, ServerComponent,
    ServerComponentKind, ServerEcosystem,
};
use super::manifest_attributes::{attribute, section_by_title};
use super::{
    ComponentFinding, Findings, ProductFinding, ProductValueFinding, Signal, ecosystems_for_key,
    manifest_location, manifest_section_location, product_from_key, release_channel,
};

pub(super) fn detect(path: &Path, archive: &ArchiveMetadata, findings: &mut Findings) {
    if let Some(manifest_bytes) = archive.manifest.as_deref() {
        let parsed = manifest::parse(manifest_bytes);
        detect_arclight(path, archive, &parsed.summary.main_attributes, findings);
        detect_mohist(path, &parsed.summary.main_attributes, &parsed.summary.sections, findings);
        detect_youer(path, &parsed.summary.main_attributes, &parsed.summary.sections, findings);
    }
    detect_magma(path, archive.wrapper_metadata.as_deref(), findings);
}

fn detect_arclight(
    path: &Path,
    archive: &ArchiveMetadata,
    attributes: &BTreeMap<String, String>,
    findings: &mut Findings,
) {
    let title = attribute(attributes, "Implementation-Title");
    let main_class = attribute(attributes, "Main-Class");
    if !title.is_some_and(|title| title.eq_ignore_ascii_case("Arclight"))
        && main_class != Some("io.izzel.arclight.server.Launcher")
    {
        return;
    }

    let launch_properties = archive
        .arclight_launch_properties
        .as_deref()
        .map(java_properties::parse)
        .unwrap_or_default();
    let launch_main = launch_properties.get("launch.mainClass");
    let mut ecosystems = vec![ServerEcosystem::Bukkit];
    let loader = launch_main.and_then(|main| {
        let main = main.to_ascii_lowercase();
        if main.contains(".neoforge.") {
            Some((ServerEcosystem::NeoForge, "neoforge", "NeoForge"))
        } else if main.contains(".forge.") {
            Some((ServerEcosystem::Forge, "forge", "Forge"))
        } else if main.contains(".fabric.") {
            Some((ServerEcosystem::Fabric, "fabric", "Fabric Loader"))
        } else {
            None
        }
    });
    if let Some((ecosystem, _, _)) = loader.as_ref() {
        ecosystems.push(ecosystem.clone());
    }

    findings.products.push(ProductFinding {
        signal: Signal {
            value: product_from_key("arclight"),
            detector: "arclight-manifest",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(
                path,
                if title.is_some() {
                    "Implementation-Title"
                } else {
                    "Main-Class"
                },
            ),
            weight: 90,
            correlation_group: "arclight-manifest",
        },
        ecosystems,
    });
    findings.roles.push(Signal {
        value: ArtifactRole::Launcher,
        detector: "arclight-manifest",
        source: EvidenceSource::ManifestMain,
        location: manifest_location(path, "Main-Class"),
        weight: 95,
        correlation_group: "arclight-manifest",
    });

    if let Some(version) = attribute(attributes, "Implementation-Version") {
        add_product_version(
            "arclight",
            version,
            "arclight-manifest",
            EvidenceSource::ManifestMain,
            manifest_location(path, "Implementation-Version"),
            90,
            findings,
        );
        if let Some(minecraft_version) = arclight_minecraft_version(version) {
            findings.minecraft_versions.push(Signal {
                value: minecraft_version.to_string(),
                detector: "arclight-manifest",
                source: EvidenceSource::ManifestMain,
                location: manifest_location(path, "Implementation-Version"),
                weight: 85,
                correlation_group: "arclight-manifest",
            });
        }
        add_component(
            "arclight",
            ServerComponentKind::Implementation,
            "arclight",
            "Arclight",
            Some(version),
            PathBuf::from("META-INF/MANIFEST.MF"),
            "arclight-manifest",
            "arclight-manifest",
            EvidenceSource::ManifestMain,
            manifest_location(path, "Implementation-Version"),
            90,
            findings,
        );
    }
    if let Some((_, key, name)) = loader {
        let location = EvidenceLocation {
            path: path.to_path_buf(),
            archive_entry: Some(ARCLIGHT_LAUNCH_PROPERTIES_ENTRY.to_string()),
            manifest_section: None,
            field: Some("launch.mainClass".to_string()),
        };
        add_component(
            "arclight",
            ServerComponentKind::ModLoader,
            key,
            name,
            None,
            PathBuf::from(ARCLIGHT_LAUNCH_PROPERTIES_ENTRY),
            "arclight-launch-properties",
            "arclight-launch-properties",
            EvidenceSource::PropertiesField,
            location,
            95,
            findings,
        );
    }
}

fn detect_mohist(
    path: &Path,
    attributes: &BTreeMap<String, String>,
    sections: &[super::super::model::ManifestSection],
    findings: &mut Findings,
) {
    let main_class = attribute(attributes, "Main-Class");
    let unique_main =
        matches!(main_class, Some("com.mohistmc.MohistMCStart" | "com.mohistmc.MohistMC"));
    let product_section = section_by_title(sections, "Mohist");
    if !unique_main && product_section.is_none() {
        return;
    }
    if unique_main {
        findings.products.push(ProductFinding {
            signal: Signal {
                value: product_from_key("mohist"),
                detector: "mohist-main-class",
                source: EvidenceSource::ManifestMain,
                location: manifest_location(path, "Main-Class"),
                weight: 90,
                correlation_group: "mohist-main-class",
            },
            ecosystems: ecosystems_for_key("mohist", false),
        });
        if main_class == Some("com.mohistmc.MohistMCStart") {
            findings.roles.push(Signal {
                value: ArtifactRole::Launcher,
                detector: "mohist-main-class",
                source: EvidenceSource::ManifestMain,
                location: manifest_location(path, "Main-Class"),
                weight: 95,
                correlation_group: "mohist-main-class",
            });
        }
    }
    let Some(product_section) = product_section else {
        return;
    };
    findings.products.push(ProductFinding {
        signal: Signal {
            value: product_from_key("mohist"),
            detector: "mohist-manifest-section",
            source: EvidenceSource::ManifestSection,
            location: manifest_section_location(
                path,
                product_section.name.as_deref(),
                "Implementation-Title",
            ),
            weight: 85,
            correlation_group: "mohist-product-section",
        },
        ecosystems: ecosystems_for_key("mohist", false),
    });
    if let Some(version) = attribute(&product_section.attributes, "Implementation-Version") {
        let version_location = manifest_section_location(
            path,
            product_section.name.as_deref(),
            "Implementation-Version",
        );
        add_product_version(
            "mohist",
            version,
            "mohist-manifest-section",
            EvidenceSource::ManifestSection,
            version_location.clone(),
            85,
            findings,
        );
        add_component(
            "mohist",
            ServerComponentKind::Implementation,
            "mohist",
            "Mohist",
            Some(version),
            PathBuf::from("META-INF/MANIFEST.MF"),
            "mohist-manifest-section",
            "mohist-product-section",
            EvidenceSource::ManifestSection,
            version_location,
            85,
            findings,
        );
    }

    add_section_component(
        path,
        sections,
        "mohist",
        "net.minecraftforge",
        ServerComponentKind::ModLoader,
        "forge",
        "Forge",
        "mohist-forge-section",
        findings,
    );
    add_section_component(
        path,
        sections,
        "mohist",
        "Spigot",
        ServerComponentKind::Api,
        "spigot",
        "Spigot",
        "mohist-spigot-section",
        findings,
    );
    if let Some(mcp) = section_by_title(sections, "MCP") {
        if let Some(minecraft_version) = attribute(&mcp.attributes, "Specification-Version") {
            findings.minecraft_versions.push(Signal {
                value: minecraft_version.to_string(),
                detector: "mohist-mcp-section",
                source: EvidenceSource::ManifestSection,
                location: manifest_section_location(
                    path,
                    mcp.name.as_deref(),
                    "Specification-Version",
                ),
                weight: 85,
                correlation_group: "mohist-mcp-section",
            });
        }
        if let Some(version) = attribute(&mcp.attributes, "Implementation-Version") {
            add_component(
                "mohist",
                ServerComponentKind::Mapping,
                "mcp",
                "MCP",
                Some(version),
                PathBuf::from("META-INF/MANIFEST.MF"),
                "mohist-mcp-section",
                "mohist-mcp-section",
                EvidenceSource::ManifestSection,
                manifest_section_location(path, mcp.name.as_deref(), "Implementation-Version"),
                85,
                findings,
            );
        }
    }
}

fn detect_youer(
    path: &Path,
    attributes: &BTreeMap<String, String>,
    sections: &[super::super::model::ManifestSection],
    findings: &mut Findings,
) {
    let unique_main =
        attribute(attributes, "Main-Class") == Some("com.mohistmc.launcher.youer.Main");
    let section = section_by_title(sections, "Youer");
    if !unique_main && section.is_none() {
        return;
    }
    if unique_main {
        findings.products.push(ProductFinding {
            signal: Signal {
                value: product_from_key("youer"),
                detector: "youer-main-class",
                source: EvidenceSource::ManifestMain,
                location: manifest_location(path, "Main-Class"),
                weight: 90,
                correlation_group: "youer-main-class",
            },
            ecosystems: ecosystems_for_key("youer", false),
        });
        findings.roles.push(Signal {
            value: ArtifactRole::Launcher,
            detector: "youer-main-class",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, "Main-Class"),
            weight: 95,
            correlation_group: "youer-main-class",
        });
    }
    let Some(section) = section else {
        return;
    };
    findings.products.push(ProductFinding {
        signal: Signal {
            value: product_from_key("youer"),
            detector: "youer-manifest-section",
            source: EvidenceSource::ManifestSection,
            location: manifest_section_location(
                path,
                section.name.as_deref(),
                "Implementation-Title",
            ),
            weight: 85,
            correlation_group: "youer-product-section",
        },
        ecosystems: ecosystems_for_key("youer", false),
    });
    if let Some(version) = attribute(&section.attributes, "Implementation-Version") {
        let location =
            manifest_section_location(path, section.name.as_deref(), "Implementation-Version");
        add_product_version(
            "youer",
            version,
            "youer-manifest-section",
            EvidenceSource::ManifestSection,
            location.clone(),
            85,
            findings,
        );
        if let Some((minecraft_version, _)) = version.split_once('-') {
            findings.minecraft_versions.push(Signal {
                value: minecraft_version.to_string(),
                detector: "youer-manifest-section",
                source: EvidenceSource::ManifestSection,
                location: location.clone(),
                weight: 85,
                correlation_group: "youer-product-section",
            });
        }
        add_component(
            "youer",
            ServerComponentKind::Implementation,
            "youer",
            "Youer",
            Some(version),
            PathBuf::from("META-INF/MANIFEST.MF"),
            "youer-manifest-section",
            "youer-product-section",
            EvidenceSource::ManifestSection,
            location,
            85,
            findings,
        );
    }
}

fn detect_magma(path: &Path, metadata: Option<&[u8]>, findings: &mut Findings) {
    let Some(metadata) = metadata else {
        return;
    };
    let Ok(document) = serde_json::from_slice::<MagmaWrapperMetadata>(metadata) else {
        return;
    };
    let Some(magma) = document.magma else {
        return;
    };
    if !magma.group_id.eq_ignore_ascii_case("org.magmafoundation")
        || !magma.artifact_id.eq_ignore_ascii_case("magma")
    {
        return;
    }
    let base_location = |field: &str| EvidenceLocation {
        path: path.to_path_buf(),
        archive_entry: Some(WRAPPER_METADATA_ENTRY.to_string()),
        manifest_section: None,
        field: Some(field.to_string()),
    };
    findings.products.push(ProductFinding {
        signal: Signal {
            value: product_from_key("magma"),
            detector: "magma-wrapper-metadata",
            source: EvidenceSource::JsonField,
            location: base_location("magma.artifactId"),
            weight: 85,
            correlation_group: "magma-wrapper-metadata",
        },
        ecosystems: ecosystems_for_key("magma", false),
    });
    findings.roles.extend([
        Signal {
            value: ArtifactRole::Wrapper,
            detector: "magma-wrapper-metadata",
            source: EvidenceSource::JsonField,
            location: base_location("magma.launcherUrl"),
            weight: 85,
            correlation_group: "magma-wrapper-metadata",
        },
        Signal {
            value: ArtifactRole::Bootstrapper,
            detector: "magma-wrapper-metadata",
            source: EvidenceSource::JsonField,
            location: base_location("magma.hasLauncher"),
            weight: 85,
            correlation_group: "magma-wrapper-metadata",
        },
    ]);
    add_product_version(
        "magma",
        &magma.version,
        "magma-wrapper-metadata",
        EvidenceSource::JsonField,
        base_location("magma.version"),
        85,
        findings,
    );
    if let Some(minecraft_version) = document.version.filter(|value| !value.trim().is_empty()) {
        findings.minecraft_versions.push(Signal {
            value: minecraft_version,
            detector: "magma-wrapper-metadata",
            source: EvidenceSource::JsonField,
            location: base_location("version"),
            weight: 85,
            correlation_group: "magma-wrapper-metadata",
        });
    }
    let coordinate = MavenCoordinate {
        group: magma.group_id,
        artifact: magma.artifact_id,
        version: magma.version.clone(),
        classifier: None,
        extension: None,
    };
    let version = magma.version;
    findings.components.push(ComponentFinding {
        product_key: "magma".to_string(),
        signal: Signal {
            value: ServerComponent {
                kind: ServerComponentKind::Implementation,
                key: "magma".to_string(),
                name: "Magma".to_string(),
                version: Some(version.clone()),
                release_channel: release_channel(&version),
                coordinate: Some(coordinate),
                source_path: Some(PathBuf::from(WRAPPER_METADATA_ENTRY)),
            },
            detector: "magma-wrapper-metadata",
            source: EvidenceSource::JsonField,
            location: base_location("magma.version"),
            weight: 85,
            correlation_group: "magma-wrapper-metadata",
        },
    });
}

#[allow(clippy::too_many_arguments)]
fn add_section_component(
    path: &Path,
    sections: &[super::super::model::ManifestSection],
    product_key: &str,
    title: &str,
    kind: ServerComponentKind,
    key: &str,
    name: &str,
    detector: &'static str,
    findings: &mut Findings,
) {
    let Some(section) = section_by_title(sections, title) else {
        return;
    };
    let Some(version) = attribute(&section.attributes, "Implementation-Version") else {
        return;
    };
    add_component(
        product_key,
        kind,
        key,
        name,
        Some(version),
        PathBuf::from("META-INF/MANIFEST.MF"),
        detector,
        detector,
        EvidenceSource::ManifestSection,
        manifest_section_location(path, section.name.as_deref(), "Implementation-Version"),
        85,
        findings,
    );
}

#[allow(clippy::too_many_arguments)]
fn add_component(
    product_key: &str,
    kind: ServerComponentKind,
    key: &str,
    name: &str,
    version: Option<&str>,
    source_path: PathBuf,
    detector: &'static str,
    correlation_group: &'static str,
    source: EvidenceSource,
    location: EvidenceLocation,
    weight: u8,
    findings: &mut Findings,
) {
    findings.components.push(ComponentFinding {
        product_key: product_key.to_string(),
        signal: Signal {
            value: ServerComponent {
                kind,
                key: key.to_string(),
                name: name.to_string(),
                version: version.map(str::to_string),
                release_channel: version.and_then(release_channel),
                coordinate: None,
                source_path: Some(source_path),
            },
            detector,
            source,
            location,
            weight,
            correlation_group,
        },
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
    findings: &mut Findings,
) {
    findings.product_versions.push(ProductValueFinding {
        product_key: product_key.to_string(),
        signal: Signal {
            value: version.to_string(),
            detector,
            source,
            location: location.clone(),
            weight,
            correlation_group: detector,
        },
    });
    if let Some(channel) = release_channel(version) {
        findings.release_channels.push(ProductValueFinding {
            product_key: product_key.to_string(),
            signal: Signal {
                value: channel,
                detector,
                source,
                location,
                weight,
                correlation_group: detector,
            },
        });
    }
}

fn arclight_minecraft_version(version: &str) -> Option<&str> {
    let prefix = "arclight-";
    if version.len() < prefix.len() || !version[..prefix.len()].eq_ignore_ascii_case(prefix) {
        return None;
    }
    version[prefix.len()..]
        .split_once('-')
        .map(|(minecraft, _)| minecraft)
        .filter(|minecraft| !minecraft.is_empty())
}

#[derive(Deserialize)]
struct MagmaWrapperMetadata {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    magma: Option<MagmaMetadata>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MagmaMetadata {
    group_id: String,
    artifact_id: String,
    version: String,
}
