use super::{
    compare_versions, compare_versions_checked, normalize_release_tag_version, parse_version,
};

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
fn compare_versions_checked_rejects_invalid_core_segment() {
    let error = compare_versions_checked("1.0.0", "1.x.0")
        .expect_err("invalid core version segment should not be silently coerced to zero");

    assert!(error.contains("版本号无效"), "unexpected error: {}", error);
    assert!(error.contains("1.x.0"), "unexpected error: {}", error);
    assert!(!compare_versions("1.0.0", "1.x.0"));
}

#[test]
fn compare_versions_checked_rejects_blank_version() {
    let error = compare_versions_checked("1.0.0", "   ")
        .expect_err("blank version should not be treated as 0.0.0 in checked mode");

    assert!(error.contains("不能为空"), "unexpected error: {}", error);
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
    assert_eq!(
        normalize_release_tag_version("SeaLantern_release-v1.2.3-rc.1"),
        "1.2.3-rc.1"
    );
}
