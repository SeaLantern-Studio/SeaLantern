use super::{parse_sha256_from_checksum_content, resolve_asset_sha256};
use crate::types::ReleaseAsset;
use std::io::Write;
use std::net::TcpListener;

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

#[tokio::test(flavor = "current_thread")]
async fn resolve_asset_sha256_surfaces_checksum_asset_http_failures() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
    let addr = listener
        .local_addr()
        .expect("test server addr should resolve");

    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("test server should accept");
        stream
            .write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n")
            .expect("response should write");
    });

    let client = reqwest::Client::new();
    let target_asset = ReleaseAsset {
        name: "SeaLantern-setup.exe".to_string(),
        browser_download_url: format!("http://{addr}/SeaLantern-setup.exe"),
    };
    let checksum_asset = ReleaseAsset {
        name: "SeaLantern-setup.exe.sha256".to_string(),
        browser_download_url: format!("http://{addr}/SeaLantern-setup.exe.sha256"),
    };

    let error = resolve_asset_sha256(&client, &[checksum_asset], &target_asset)
        .await
        .expect_err("checksum asset fetch failures should be surfaced");

    server.join().expect("server thread should finish");

    assert!(error.contains("checksum asset SeaLantern-setup.exe.sha256 returned status 500"));
}
