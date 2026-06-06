use super::{
    build_hash_mismatch_error, calculate_sha256, file_name_from_url, remove_corrupted_download,
};

#[test]
fn file_name_from_url_strips_query_and_fragment() {
    assert_eq!(
        file_name_from_url("https://example.com/releases/SeaLantern.msi?download=1#latest"),
        "SeaLantern.msi"
    );
}

#[test]
fn file_name_from_url_falls_back_when_empty() {
    assert_eq!(file_name_from_url("https://example.com/releases/"), "update");
}

#[test]
fn calculate_sha256_matches_known_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("payload.bin");
    std::fs::write(&path, b"sea-lantern-update").unwrap();

    assert_eq!(
        calculate_sha256(&path).unwrap(),
        "5bbc7b2690d662041ab18aa89803cdc1816c7aa3ba33b87d649bcab2df0d4506"
    );
}

#[test]
fn build_hash_mismatch_error_includes_expected_and_actual_hashes() {
    let error = build_hash_mismatch_error("expected", "actual");

    assert_eq!(
        error,
        "Hash verification failed. Expected: expected, Got: actual"
    );
}

#[test]
fn remove_corrupted_download_surfaces_cleanup_failures() {
    let dir = tempfile::tempdir().unwrap();
    let bad_path = dir.path().join("payload.bin");
    std::fs::create_dir(&bad_path).unwrap();

    let error = remove_corrupted_download(
        &bad_path,
        build_hash_mismatch_error("expected", "actual"),
    )
    .expect_err("directory-backed payload path should not be silently ignored");

    assert!(error.contains("Hash verification failed. Expected: expected, Got: actual"));
    assert!(error.contains("failed to remove corrupted download"));
    assert!(error.contains("payload.bin"));
}
