use std::path::{Path, PathBuf};

use super::super::archive::ArchiveMetadata;
use super::super::formats::manifest;
use super::super::model::{ArtifactRole, EvidenceSource, ServerComponent, ServerComponentKind};
use super::manifest_attributes::attribute;
use super::{
    ComponentFinding, Findings, ProductFinding, ProductValueFinding, Signal, ecosystems_for_key,
    manifest_location, product_from_key, release_channel,
};

const SPONGE_VANILLA_INSTALLER_MAIN: &str = "org.spongepowered.vanilla.installer.InstallerMain";

pub(super) fn detect(path: &Path, archive: &ArchiveMetadata, findings: &mut Findings) {
    let Some(manifest_bytes) = archive.manifest.as_deref() else {
        return;
    };
    let parsed = manifest::parse(manifest_bytes);
    let attributes = &parsed.summary.main_attributes;
    let title = attribute(attributes, "Implementation-Title")
        .or_else(|| attribute(attributes, "Specification-Title"));
    let title_matches = title.is_some_and(|title| title.eq_ignore_ascii_case("SpongeVanilla"));
    let installer_main = attribute(attributes, "Main-Class") == Some(SPONGE_VANILLA_INSTALLER_MAIN);
    if !title_matches && !installer_main {
        return;
    }

    findings.products.push(ProductFinding {
        signal: Signal {
            value: product_from_key("spongevanilla"),
            detector: "spongevanilla-manifest",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(
                path,
                if title_matches {
                    "Implementation-Title"
                } else {
                    "Main-Class"
                },
            ),
            weight: if title_matches { 90 } else { 85 },
            correlation_group: "spongevanilla-manifest",
        },
        ecosystems: ecosystems_for_key("spongevanilla", false),
    });
    if installer_main {
        findings.roles.push(Signal {
            value: ArtifactRole::Installer,
            detector: "spongevanilla-installer-main",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, "Main-Class"),
            weight: 98,
            correlation_group: "spongevanilla-manifest",
        });
    }

    let Some(version) = attribute(attributes, "Implementation-Version") else {
        return;
    };
    let version_location = manifest_location(path, "Implementation-Version");
    findings.product_versions.push(ProductValueFinding {
        product_key: "spongevanilla".to_string(),
        signal: Signal {
            value: version.to_string(),
            detector: "spongevanilla-manifest",
            source: EvidenceSource::ManifestMain,
            location: version_location.clone(),
            weight: 90,
            correlation_group: "spongevanilla-manifest",
        },
    });
    if let Some(channel) = release_channel(version) {
        findings.release_channels.push(ProductValueFinding {
            product_key: "spongevanilla".to_string(),
            signal: Signal {
                value: channel,
                detector: "spongevanilla-manifest",
                source: EvidenceSource::ManifestMain,
                location: version_location.clone(),
                weight: 90,
                correlation_group: "spongevanilla-manifest",
            },
        });
    }
    if let Some((minecraft_version, _)) = version.split_once('-') {
        findings.minecraft_versions.push(Signal {
            value: minecraft_version.to_string(),
            detector: "spongevanilla-manifest",
            source: EvidenceSource::ManifestMain,
            location: version_location.clone(),
            weight: 90,
            correlation_group: "spongevanilla-manifest",
        });
    }
    findings.components.push(ComponentFinding {
        product_key: "spongevanilla".to_string(),
        signal: Signal {
            value: ServerComponent {
                kind: if installer_main {
                    ServerComponentKind::Installer
                } else {
                    ServerComponentKind::Implementation
                },
                key: if installer_main {
                    "spongevanilla-installer".to_string()
                } else {
                    "spongevanilla".to_string()
                },
                name: if installer_main {
                    "SpongeVanilla Installer".to_string()
                } else {
                    "SpongeVanilla".to_string()
                },
                version: Some(version.to_string()),
                release_channel: release_channel(version),
                coordinate: None,
                source_path: Some(PathBuf::from("META-INF/MANIFEST.MF")),
            },
            detector: "spongevanilla-manifest",
            source: EvidenceSource::ManifestMain,
            location: version_location,
            weight: 90,
            correlation_group: "spongevanilla-manifest",
        },
    });
}
