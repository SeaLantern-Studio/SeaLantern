use std::path::{Path, PathBuf};

use super::super::archive::{ArchiveMetadata, VERSIONS_LIST_ENTRY};
use super::super::formats::manifest;
use super::super::model::{
    ArtifactRole, EvidenceSource, LaunchPlatform, LaunchProfile, LaunchTarget, ServerComponent,
    ServerComponentKind,
};
use super::{
    ComponentFinding, Findings, ProductFinding, ProductValueFinding, Signal, ecosystems_for_key,
    list_location, manifest_location, product_from_key, release_channel,
};

const CRAFTBUKKIT_MAIN: &str = "org.bukkit.craftbukkit.bootstrap.Main";
const FABRIC_SERVER_LAUNCHER: &str = "net.fabricmc.installer.ServerLauncher";

pub(super) fn detect(
    path: &Path,
    archive: &ArchiveMetadata,
    directory_launch: Option<(&Path, &Path)>,
    findings: &mut Findings,
) {
    let parsed_manifest = archive.manifest.as_deref().map(manifest::parse);
    let main_class = parsed_manifest
        .as_ref()
        .and_then(|manifest| manifest.main_value("Main-Class"));
    if main_class == Some(CRAFTBUKKIT_MAIN) {
        add_craftbukkit_product(
            EvidenceSource::ManifestMain,
            manifest_location(path, "Main-Class"),
            90,
            "craftbukkit-main-class",
            "craftbukkit-main-class",
            findings,
        );
        findings.roles.push(Signal {
            value: ArtifactRole::Bootstrapper,
            detector: "craftbukkit-main-class",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, "Main-Class"),
            weight: 90,
            correlation_group: "craftbukkit-main-class",
        });
    }

    if let Some(content) = archive.versions_list.as_deref() {
        for entry in craftbukkit_versions(content) {
            let location = list_location(path, VERSIONS_LIST_ENTRY, entry.line);
            add_craftbukkit_product(
                EvidenceSource::JarEntry,
                location.clone(),
                95,
                "craftbukkit-versions-list",
                "craftbukkit-versions-list",
                findings,
            );
            findings.product_versions.push(ProductValueFinding {
                product_key: "craftbukkit".to_string(),
                signal: Signal {
                    value: entry.product_version.clone(),
                    detector: "craftbukkit-versions-list",
                    source: EvidenceSource::JarEntry,
                    location: location.clone(),
                    weight: 95,
                    correlation_group: "craftbukkit-versions-list",
                },
            });
            findings.minecraft_versions.push(Signal {
                value: entry.minecraft_version,
                detector: "craftbukkit-versions-list",
                source: EvidenceSource::JarEntry,
                location: location.clone(),
                weight: 95,
                correlation_group: "craftbukkit-versions-list",
            });
            if let Some(channel) = release_channel(&entry.product_version) {
                findings.release_channels.push(ProductValueFinding {
                    product_key: "craftbukkit".to_string(),
                    signal: Signal {
                        value: channel,
                        detector: "craftbukkit-versions-list",
                        source: EvidenceSource::JarEntry,
                        location: location.clone(),
                        weight: 95,
                        correlation_group: "craftbukkit-versions-list",
                    },
                });
            }
            findings.components.push(ComponentFinding {
                product_key: "craftbukkit".to_string(),
                signal: Signal {
                    value: ServerComponent {
                        kind: ServerComponentKind::Implementation,
                        key: "craftbukkit".to_string(),
                        name: "CraftBukkit".to_string(),
                        version: Some(entry.product_version.clone()),
                        release_channel: release_channel(&entry.product_version),
                        coordinate: None,
                        source_path: Some(PathBuf::from(entry.target_path)),
                    },
                    detector: "craftbukkit-versions-list",
                    source: EvidenceSource::JarEntry,
                    location,
                    weight: 95,
                    correlation_group: "craftbukkit-versions-list",
                },
            });
        }
    }

    if let Some((root, relative_jar)) = directory_launch
        && main_class.is_some()
        && main_class != Some(FABRIC_SERVER_LAUNCHER)
    {
        findings.roles.push(Signal {
            value: ArtifactRole::Runnable,
            detector: "directory-root-jar-manifest",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, "Main-Class"),
            weight: 95,
            correlation_group: "directory-root-jar-manifest",
        });
        findings.launches.push(Signal {
            value: LaunchProfile {
                id: format!("root-jar-{}", launch_id(relative_jar)),
                platform: LaunchPlatform::Any,
                working_directory: Some(root.to_path_buf()),
                target: LaunchTarget::Jar { path: root.join(relative_jar) },
                jvm_arguments: Vec::new(),
                program_arguments: Vec::new(),
                required_java_major: None,
            },
            detector: "directory-root-jar-manifest",
            source: EvidenceSource::ManifestMain,
            location: manifest_location(path, "Main-Class"),
            weight: 95,
            correlation_group: "directory-root-jar-manifest",
        });
    }
}

fn add_craftbukkit_product(
    source: EvidenceSource,
    location: super::super::model::EvidenceLocation,
    weight: u8,
    detector: &'static str,
    correlation_group: &'static str,
    findings: &mut Findings,
) {
    findings.products.push(ProductFinding {
        signal: Signal {
            value: product_from_key("craftbukkit"),
            detector,
            source,
            location,
            weight,
            correlation_group,
        },
        ecosystems: ecosystems_for_key("craftbukkit", false),
    });
}

fn craftbukkit_versions(content: &[u8]) -> Vec<CraftBukkitVersion> {
    String::from_utf8_lossy(content)
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let target_path = line.split_whitespace().next_back()?.trim_start_matches('*');
            let filename = target_path.rsplit(['/', '\\']).next()?;
            let stem = super::strip_jar_suffix(filename)?;
            let prefix = "craftbukkit-";
            if stem.len() <= prefix.len() || !stem[..prefix.len()].eq_ignore_ascii_case(prefix) {
                return None;
            }
            let product_version = stem[prefix.len()..].to_string();
            let lower = product_version.to_ascii_lowercase();
            let minecraft_end = lower.find("-r")?;
            let minecraft_version = product_version[..minecraft_end].to_string();
            (!minecraft_version.is_empty()).then_some(CraftBukkitVersion {
                line: index + 1,
                target_path: target_path.to_string(),
                product_version,
                minecraft_version,
            })
        })
        .collect()
}

fn launch_id(path: &Path) -> String {
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

struct CraftBukkitVersion {
    line: usize,
    target_path: String,
    product_version: String,
    minecraft_version: String,
}

#[cfg(test)]
mod tests {
    use super::craftbukkit_versions;

    #[test]
    fn parses_two_field_craftbukkit_version_lists() {
        let entries = craftbukkit_versions(b"hash *craftbukkit-26.2-R0.1-SNAPSHOT.jar\n");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].minecraft_version, "26.2");
        assert_eq!(entries[0].product_version, "26.2-R0.1-SNAPSHOT");
    }
}
