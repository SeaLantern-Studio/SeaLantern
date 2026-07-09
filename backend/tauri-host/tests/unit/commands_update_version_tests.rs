use super::*;

#[test]
fn compare_versions_handles_prerelease() {
    assert!(compare_versions("1.2.3-beta.1", "1.2.3"));
    assert!(!compare_versions("1.2.3", "1.2.3-beta.1"));
    assert!(compare_versions("1.2.3-beta.1", "1.2.3-beta.2"));
    assert!(!compare_versions("1.2.3-rc.2", "1.2.3-rc.1"));
}

#[test]
fn compare_versions_handles_basic_semver() {
    assert!(compare_versions("1.2.3", "1.2.4"));
    assert!(!compare_versions("1.2.4", "1.2.3"));
    assert!(compare_versions("v1.9.9", "2.0.0"));
    assert!(!compare_versions("2.0.0", "2.0.0"));
}

#[test]
fn parse_version_ignores_build_metadata() {
    assert_eq!(parse_version("1.2.3+abc"), parse_version("1.2.3+def"));
}

#[test]
fn normalize_release_tag_version_handles_prefixed_tag() {
    assert_eq!(normalize_release_tag_version("sea-lantern-v0.5.0"), "0.5.0");
}

#[test]
fn normalize_release_tag_version_handles_plain_version_tag() {
    assert_eq!(normalize_release_tag_version("v0.5.0"), "0.5.0");
}

#[test]
fn normalize_release_tag_version_handles_prerelease_tag() {
    assert_eq!(normalize_release_tag_version("SeaLantern_release-v1.2.3-rc.1"), "1.2.3-rc.1");
}
