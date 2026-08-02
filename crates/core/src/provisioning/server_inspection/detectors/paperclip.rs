use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::super::archive::{
    ArchiveMetadata, LIBRARIES_LIST_ENTRY, PATCHES_LIST_ENTRY, VERSIONS_LIST_ENTRY,
};
use super::super::formats::paperclip_list;
use super::super::model::{ArtifactInfo, ArtifactRole, EvidenceSource, ServerCategory};
use super::{
    api_component, ecosystems_for_key, list_location, manifest_location, product_from_key,
    product_key_from_coordinate, release_channel, strip_jar_suffix, target_product_key,
    ComponentFinding, Findings, ProductFinding, ProductValueFinding, Signal,
};

const SHARED_PAPERCLIP_MAIN_CLASSES: &[&str] =
    &["io.papermc.paperclip.Main", "com.destroystokyo.paperclip.Paperclip"];

pub(super) fn detect(
    path: &Path,
    artifact: &ArtifactInfo,
    archive: &ArchiveMetadata,
    findings: &mut Findings,
) {
    let main_class = artifact.main_class.value.as_deref();
    let paperclip_main = main_class.is_some_and(|main_class| {
        SHARED_PAPERCLIP_MAIN_CLASSES.contains(&main_class)
            || matches!(main_class, "cn.dreeam.leaper.Main" | "org.leavesmc.leavesclip.Main")
    });
    if paperclip_main {
        findings.roles.push(Signal {
            value: ArtifactRole::Bootstrapper,
            detector: "paperclip-main-class",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, "Main-Class"),
            weight: 95,
            correlation_group: "paperclip-main-class",
        });
        findings.categories.push(Signal {
            value: ServerCategory::JavaGameServer,
            detector: "paperclip-main-class",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, "Main-Class"),
            weight: 90,
            correlation_group: "paperclip-main-class",
        });
    }
    if let Some(key) = match main_class {
        Some("cn.dreeam.leaper.Main") => Some("leaf"),
        Some("org.leavesmc.leavesclip.Main") => Some("leaves"),
        _ => None,
    } {
        findings.products.push(ProductFinding {
            signal: Signal {
                value: product_from_key(key),
                detector: "paperclip-main-class",
                source: EvidenceSource::ManifestMain,
                location: manifest_location(path, "Main-Class"),
                weight: 90,
                correlation_group: "paperclip-main-class",
            },
            ecosystems: ecosystems_for_key(key, true),
        });
    }

    let mut target_keys = BTreeSet::new();
    if let Some(content) = archive.versions_list.as_deref() {
        for entry in paperclip_list::parse_versions(content) {
            if let Some(key) = target_product_key(&entry.target_path, &entry.minecraft_version) {
                let paper_context = key != "vanilla" && paperclip_main;
                add_target_finding(
                    path,
                    VERSIONS_LIST_ENTRY,
                    entry.line,
                    &key,
                    &entry.minecraft_version,
                    95,
                    "versions-list",
                    paper_context,
                    findings,
                );
                target_keys.insert(key);
            }
        }
    }

    if let Some(content) = archive.patches_list.as_deref() {
        let patches = paperclip_list::parse_version_patches(content);
        if !patches.is_empty() && !paperclip_main {
            findings.roles.push(Signal {
                value: ArtifactRole::Bootstrapper,
                detector: "paperclip-patches-list",
                source: EvidenceSource::JarEntry,
                location: list_location(path, PATCHES_LIST_ENTRY, patches[0].line),
                weight: 90,
                correlation_group: "patches-list",
            });
        }
        for entry in patches {
            if let Some(key) = target_product_key(&entry.target_path, &entry.minecraft_version) {
                add_target_finding(
                    path,
                    PATCHES_LIST_ENTRY,
                    entry.line,
                    &key,
                    &entry.minecraft_version,
                    95,
                    "patches-list",
                    key != "vanilla",
                    findings,
                );
                target_keys.insert(key);
            }
        }
    }

    if let Some(content) = archive.libraries_list.as_deref() {
        for entry in paperclip_list::parse_libraries(content) {
            if let Some(coordinate) = entry.coordinate.as_ref() {
                let key = product_key_from_coordinate(coordinate).or_else(|| {
                    coordinate
                        .artifact
                        .strip_suffix("-api")
                        .filter(|key| target_keys.contains(*key))
                        .map(str::to_string)
                });
                if let Some(key) = key {
                    let location = list_location(path, LIBRARIES_LIST_ENTRY, entry.line);
                    findings.products.push(ProductFinding {
                        signal: Signal {
                            value: product_from_key(&key),
                            detector: "paperclip-api-coordinate",
                            source: EvidenceSource::MavenCoordinate,
                            location: location.clone(),
                            weight: 98,
                            correlation_group: "libraries-list-coordinate",
                        },
                        ecosystems: ecosystems_for_key(&key, key != "spigot"),
                    });
                    findings.product_versions.push(ProductValueFinding {
                        product_key: key.clone(),
                        signal: Signal {
                            value: coordinate.version.clone(),
                            detector: "paperclip-api-coordinate",
                            source: EvidenceSource::MavenCoordinate,
                            location: location.clone(),
                            weight: 98,
                            correlation_group: "libraries-list-coordinate",
                        },
                    });
                    if let Some(channel) = release_channel(&coordinate.version) {
                        findings.release_channels.push(ProductValueFinding {
                            product_key: key.clone(),
                            signal: Signal {
                                value: channel,
                                detector: "paperclip-api-coordinate",
                                source: EvidenceSource::MavenCoordinate,
                                location: location.clone(),
                                weight: 95,
                                correlation_group: "libraries-list-coordinate",
                            },
                        });
                    }
                    findings.components.push(ComponentFinding {
                        product_key: key.clone(),
                        signal: Signal {
                            value: api_component(
                                &key,
                                coordinate.version.clone(),
                                Some(coordinate.clone()),
                                PathBuf::from(&entry.target_path),
                            ),
                            detector: "paperclip-api-coordinate",
                            source: EvidenceSource::MavenCoordinate,
                            location,
                            weight: 98,
                            correlation_group: "libraries-list-coordinate",
                        },
                    });
                    continue;
                }
            }

            for key in &target_keys {
                let Some(version) = version_from_api_path(key, &entry.target_path) else {
                    continue;
                };
                let location = list_location(path, LIBRARIES_LIST_ENTRY, entry.line);
                findings.product_versions.push(ProductValueFinding {
                    product_key: key.clone(),
                    signal: Signal {
                        value: version.clone(),
                        detector: "paperclip-api-path",
                        source: EvidenceSource::JarEntry,
                        location: location.clone(),
                        weight: 85,
                        correlation_group: "libraries-list-path",
                    },
                });
                if let Some(channel) = release_channel(&version) {
                    findings.release_channels.push(ProductValueFinding {
                        product_key: key.clone(),
                        signal: Signal {
                            value: channel,
                            detector: "paperclip-api-path",
                            source: EvidenceSource::JarEntry,
                            location: location.clone(),
                            weight: 85,
                            correlation_group: "libraries-list-path",
                        },
                    });
                }
                findings.components.push(ComponentFinding {
                    product_key: key.clone(),
                    signal: Signal {
                        value: api_component(key, version, None, PathBuf::from(&entry.target_path)),
                        detector: "paperclip-api-path",
                        source: EvidenceSource::JarEntry,
                        location,
                        weight: 85,
                        correlation_group: "libraries-list-path",
                    },
                });
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn add_target_finding(
    path: &Path,
    archive_entry: &str,
    line: usize,
    key: &str,
    minecraft_version: &str,
    weight: u8,
    correlation_group: &'static str,
    paperclip_context: bool,
    findings: &mut Findings,
) {
    let location = list_location(path, archive_entry, line);
    findings.products.push(ProductFinding {
        signal: Signal {
            value: product_from_key(key),
            detector: "paperclip-target-list",
            source: EvidenceSource::JarEntry,
            location: location.clone(),
            weight,
            correlation_group,
        },
        ecosystems: ecosystems_for_key(key, paperclip_context),
    });
    findings.minecraft_versions.push(Signal {
        value: minecraft_version.to_string(),
        detector: "paperclip-target-list",
        source: EvidenceSource::JarEntry,
        location: location.clone(),
        weight: 95,
        correlation_group,
    });
    if key == "vanilla" {
        findings.product_versions.push(ProductValueFinding {
            product_key: key.to_string(),
            signal: Signal {
                value: minecraft_version.to_string(),
                detector: "vanilla-version-list",
                source: EvidenceSource::JarEntry,
                location,
                weight: 95,
                correlation_group,
            },
        });
    }
}

fn version_from_api_path(product_key: &str, path: &str) -> Option<String> {
    let filename = path.rsplit(['/', '\\']).next()?;
    let stem = strip_jar_suffix(filename)?;
    let prefix = format!("{product_key}-api-");
    stem.to_ascii_lowercase()
        .starts_with(&prefix)
        .then(|| stem.get(prefix.len()..))
        .flatten()
        .filter(|version| !version.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::version_from_api_path;

    #[test]
    fn extracts_version_from_wildcard_api_paths() {
        assert_eq!(
            version_from_api_path("spigot", "spigot-api-26.2-R0.1-SNAPSHOT.jar").as_deref(),
            Some("26.2-R0.1-SNAPSHOT")
        );
    }
}
