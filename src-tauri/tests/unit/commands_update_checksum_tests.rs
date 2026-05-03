use super::*;

#[test]
fn parse_sha256_matches_target_file() {
    let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let content = format!("{hash}  SeaLantern-setup.exe");
    assert_eq!(
        parse_sha256_from_checksum_content(&content, "SeaLantern-setup.exe"),
        Some(hash.to_string())
    );
}

#[test]
fn parse_sha256_accepts_single_hash_file() {
    let hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    assert_eq!(
        parse_sha256_from_checksum_content(hash, "SeaLantern-setup.exe"),
        Some(hash.to_string())
    );
}

#[test]
fn parse_sha256_rejects_multi_hash_without_target_match() {
    let first = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let second = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    let content = format!("{first} other.exe\n{second} another.exe");
    assert_eq!(parse_sha256_from_checksum_content(&content, "SeaLantern-setup.exe"), None);
}
