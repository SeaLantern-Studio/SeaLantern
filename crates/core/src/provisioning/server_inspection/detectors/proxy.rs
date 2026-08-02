use std::path::{Path, PathBuf};

use super::super::archive::ArchiveMetadata;
use super::super::formats::manifest;
use super::super::model::{
    EvidenceSource, ServerCategory, ServerComponent, ServerComponentKind, ServerEcosystem,
};
use super::manifest_attributes::attribute;
use super::{
    ecosystems_for_key, manifest_location, product_from_key, release_channel, ComponentFinding,
    Findings, ProductFinding, ProductValueFinding, Signal,
};

const VELOCITY_MAIN: &str = "com.velocitypowered.proxy.Velocity";
const BUNGEE_MAIN: &str = "net.md_5.bungee.Bootstrap";

pub(super) fn detect(path: &Path, archive: &ArchiveMetadata, findings: &mut Findings) {
    let Some(manifest_bytes) = archive.manifest.as_deref() else {
        return;
    };
    let parsed = manifest::parse(manifest_bytes);
    let attributes = &parsed.summary.main_attributes;
    let main_class = attribute(attributes, "Main-Class");

    let shared_ecosystem = match main_class {
        Some(VELOCITY_MAIN) => Some(ServerEcosystem::Velocity),
        Some(BUNGEE_MAIN) => Some(ServerEcosystem::Bungee),
        _ => None,
    };
    if let Some(ecosystem) = shared_ecosystem {
        findings.categories.push(Signal {
            value: ServerCategory::Proxy,
            detector: "proxy-main-class",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, "Main-Class"),
            weight: 90,
            correlation_group: "proxy-main-class",
        });
        findings.ecosystems.push(Signal {
            value: ecosystem,
            detector: "proxy-main-class",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, "Main-Class"),
            weight: 85,
            correlation_group: "proxy-main-class",
        });
    }

    let title = attribute(attributes, "Implementation-Title");
    let implementation_version = attribute(attributes, "Implementation-Version");
    let product_key = match title {
        Some(title) if title.eq_ignore_ascii_case("Velocity-CTD") => Some("velocity-ctd"),
        Some(title) if title.eq_ignore_ascii_case("Velocity") => Some("velocity"),
        _ => implementation_version.and_then(|version| {
            let version = version.to_ascii_lowercase();
            if version.contains("waterfall-bootstrap") {
                Some("waterfall")
            } else if version.contains("bungeecord-bootstrap") {
                Some("bungeecord")
            } else {
                None
            }
        }),
    };
    let Some(product_key) = product_key else {
        return;
    };
    let identity_field = if title.is_some() {
        "Implementation-Title"
    } else {
        "Implementation-Version"
    };
    findings.products.push(ProductFinding {
        signal: Signal {
            value: product_from_key(product_key),
            detector: "proxy-manifest-product",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, identity_field),
            weight: 90,
            correlation_group: "proxy-product-manifest",
        },
        ecosystems: ecosystems_for_key(product_key, false),
    });

    if let Some(version) = implementation_version {
        let location = manifest_location(path, "Implementation-Version");
        findings.product_versions.push(ProductValueFinding {
            product_key: product_key.to_string(),
            signal: Signal {
                value: version.to_string(),
                detector: "proxy-manifest-version",
                source: EvidenceSource::ManifestMain,
                location: location.clone(),
                weight: 90,
                correlation_group: "proxy-product-manifest",
            },
        });
        if let Some(channel) = release_channel(version) {
            findings.release_channels.push(ProductValueFinding {
                product_key: product_key.to_string(),
                signal: Signal {
                    value: channel,
                    detector: "proxy-manifest-version",
                    source: EvidenceSource::ManifestMain,
                    location: location.clone(),
                    weight: 90,
                    correlation_group: "proxy-product-manifest",
                },
            });
        }
        findings.components.push(ComponentFinding {
            product_key: product_key.to_string(),
            signal: Signal {
                value: ServerComponent {
                    kind: ServerComponentKind::Implementation,
                    key: product_key.to_string(),
                    name: product_from_key(product_key).display_name,
                    version: Some(version.to_string()),
                    release_channel: release_channel(version),
                    coordinate: None,
                    source_path: Some(PathBuf::from("META-INF/MANIFEST.MF")),
                },
                detector: "proxy-manifest-version",
                source: EvidenceSource::ManifestMain,
                location,
                weight: 90,
                correlation_group: "proxy-product-manifest",
            },
        });
    }
    if let Some(java_major) = attribute(attributes, "Java-Version")
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|value| *value > 0)
    {
        findings.java_majors.push(Signal {
            value: java_major,
            detector: "proxy-java-version",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, "Java-Version"),
            weight: 90,
            correlation_group: "proxy-java-version",
        });
    }
}
