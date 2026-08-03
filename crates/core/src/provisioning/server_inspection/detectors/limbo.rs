use std::path::{Path, PathBuf};

use super::super::archive::ArchiveMetadata;
use super::super::formats::manifest;
use super::super::model::{EvidenceSource, ServerCategory, ServerComponent, ServerComponentKind};
use super::manifest_attributes::attribute;
use super::{
    ecosystems_for_key, manifest_location, product_from_key, release_channel, ComponentFinding,
    Findings, ProductFinding, ProductValueFinding, Signal,
};

pub(super) fn detect(path: &Path, archive: &ArchiveMetadata, findings: &mut Findings) {
    let Some(manifest_bytes) = archive.manifest.as_deref() else {
        return;
    };
    let parsed = manifest::parse(manifest_bytes);
    let attributes = &parsed.summary.main_attributes;
    let (product_key, version_field) = match attribute(attributes, "Main-Class") {
        Some("com.loohp.limbo.Limbo") => ("limbo", Some("Limbo-Version")),
        Some("ua.nanit.limbo.NanoLimbo") => ("nanolimbo", None),
        _ => return,
    };

    findings.products.push(ProductFinding {
        signal: Signal {
            value: product_from_key(product_key),
            detector: "limbo-main-class",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, "Main-Class"),
            weight: 90,
            correlation_group: "limbo-main-class",
        },
        ecosystems: ecosystems_for_key(product_key, false),
    });
    findings.categories.push(Signal {
        value: ServerCategory::Limbo,
        detector: "limbo-main-class",
        source: EvidenceSource::ManifestMain,
        location: manifest_location(path, "Main-Class"),
        weight: 90,
        correlation_group: "limbo-main-class",
    });

    let Some(version_field) = version_field else {
        return;
    };
    let Some(version) = attribute(attributes, version_field) else {
        return;
    };
    let location = manifest_location(path, version_field);
    findings.product_versions.push(ProductValueFinding {
        product_key: product_key.to_string(),
        signal: Signal {
            value: version.to_string(),
            detector: "limbo-manifest-version",
            source: EvidenceSource::ManifestMain,
            location: location.clone(),
            weight: 90,
            correlation_group: "limbo-manifest-version",
        },
    });
    if let Some(channel) = release_channel(version) {
        findings.release_channels.push(ProductValueFinding {
            product_key: product_key.to_string(),
            signal: Signal {
                value: channel,
                detector: "limbo-manifest-version",
                source: EvidenceSource::ManifestMain,
                location: location.clone(),
                weight: 90,
                correlation_group: "limbo-manifest-version",
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
            detector: "limbo-manifest-version",
            source: EvidenceSource::ManifestMain,
            location,
            weight: 90,
            correlation_group: "limbo-manifest-version",
        },
    });
}
