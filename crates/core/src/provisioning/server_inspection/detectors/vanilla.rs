use std::path::Path;

use super::super::model::{ArtifactInfo, EvidenceSource, MinecraftVersionInfo, ReleaseChannel};
use super::{
    Findings, ProductFinding, ProductValueFinding, Signal, ecosystems_for_key, manifest_location,
    product_from_key, version_json_location,
};

pub(super) fn detect(
    path: &Path,
    artifact: &ArtifactInfo,
    minecraft: Option<&MinecraftVersionInfo>,
    findings: &mut Findings,
) {
    let vanilla_main = matches!(
        artifact.main_class.value.as_deref(),
        Some("net.minecraft.bundler.Main" | "net.minecraft.server.MinecraftServer")
    );
    if vanilla_main {
        findings.products.push(ProductFinding {
            signal: Signal {
                value: product_from_key("vanilla"),
                detector: "vanilla-main-class",
                source: EvidenceSource::ManifestMain,
                location: manifest_location(path, "Main-Class"),
                weight: 90,
                correlation_group: "vanilla-main-class",
            },
            ecosystems: ecosystems_for_key("vanilla", false),
        });
    }
    if !findings.has_product("vanilla") {
        return;
    }
    let Some(minecraft) = minecraft else {
        return;
    };
    if let Some(version) = minecraft.version.value.as_ref() {
        findings.product_versions.push(ProductValueFinding {
            product_key: "vanilla".to_string(),
            signal: Signal {
                value: version.clone(),
                detector: "vanilla-version-json",
                source: EvidenceSource::JsonField,
                location: version_json_location(path, "id"),
                weight: 95,
                correlation_group: "mojang-version-json",
            },
        });
    }
    if minecraft.stable == Some(true) {
        findings.release_channels.push(ProductValueFinding {
            product_key: "vanilla".to_string(),
            signal: Signal {
                value: ReleaseChannel::Stable,
                detector: "vanilla-version-json",
                source: EvidenceSource::JsonField,
                location: version_json_location(path, "stable"),
                weight: 90,
                correlation_group: "mojang-version-json",
            },
        });
    }
}
