#![allow(dead_code)]

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;

#[path = "../src/shared.rs"]
mod shared;

use sea_lantern_java_installer_core::download_and_install_java;

#[test]
fn bytes_to_mb_formats_with_two_decimals() {
    assert_eq!(shared::bytes_to_mb(0), "0.00MB");
    assert_eq!(shared::bytes_to_mb(1024 * 1024), "1.00MB");
}

#[test]
fn resolve_install_source_prefers_single_nested_directory() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let nested = dir.path().join("jdk-21");
    fs::create_dir_all(&nested).expect("nested runtime dir should exist");

    let resolved = shared::resolve_install_source(dir.path())
        .expect("single nested runtime dir should resolve");

    assert_eq!(resolved, nested);
}

#[test]
fn resolve_install_source_keeps_temp_dir_when_multiple_entries_exist() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    fs::create_dir_all(dir.path().join("jdk-21")).expect("runtime dir should exist");
    fs::write(dir.path().join("release.txt"), b"meta").expect("metadata file should write");

    let resolved = shared::resolve_install_source(dir.path())
        .expect("multiple entries should keep temp dir as install source");

    assert_eq!(resolved, dir.path().to_path_buf());
}

#[test]
fn resolve_install_source_surfaces_directory_read_failures() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let missing = dir.path().join("missing-install-source");

    let error = shared::resolve_install_source(&missing)
        .expect_err("install source resolution should surface read failures");

    assert!(error.contains("读取安装临时目录失败"), "unexpected error: {}", error);
}

#[test]
fn resolve_java_binary_path_points_to_platform_bin_location() {
    let path = shared::resolve_java_binary_path(std::path::Path::new("E:/java/jdk-21"));
    let normalized = path.to_string_lossy().replace('\\', "/");

    if cfg!(target_os = "windows") {
        assert_eq!(normalized, "E:/java/jdk-21/bin/java.exe");
    } else {
        assert_eq!(normalized, "E:/java/jdk-21/bin/java");
    }
}

fn serve_single_response(status_line: &str, body: &[u8], content_type: &str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
    let addr = listener.local_addr().expect("test server addr should resolve");
    let body = body.to_vec();
    let status_line = status_line.to_string();
    let content_type = content_type.to_string();

    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("test server should accept");
        let mut buffer = [0u8; 2048];
        let _ = stream.read(&mut buffer);
        let response = format!(
            "HTTP/1.1 {status_line}\r\nContent-Length: {}\r\nContent-Type: {content_type}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("response header should write");
        stream.write_all(&body).expect("response body should write");
    });

    format!("http://{addr}/java-runtime")
}

#[tokio::test]
async fn download_and_install_java_surfaces_http_status_before_archive_parse() {
    let app_dir = tempfile::tempdir().expect("temp dir should exist");
    let cancel_flag = AtomicBool::new(false);
    let url = serve_single_response("404 Not Found", b"missing", "text/plain");

    let error = download_and_install_java(
        &url,
        "jdk-test-http-404",
        app_dir.path(),
        &cancel_flag,
        |_| {},
    )
    .await
    .expect_err("404 response should fail");

    assert!(error.contains("下载请求失败（HTTP 404 Not Found）"), "unexpected error: {}", error);
    assert!(!error.contains("ZIP"), "unexpected error: {}", error);
    assert!(!error.contains("tar.gz"), "unexpected error: {}", error);
}

#[tokio::test]
async fn download_and_install_java_cleans_temp_dir_after_invalid_archive_failure() {
    let app_dir = tempfile::tempdir().expect("temp dir should exist");
    let cancel_flag = AtomicBool::new(false);
    let version_name = "jdk-test-invalid-archive";
    let url = serve_single_response("200 OK", b"not an archive", "application/octet-stream");

    let error = download_and_install_java(
        &url,
        version_name,
        app_dir.path(),
        &cancel_flag,
        |_| {},
    )
    .await
    .expect_err("invalid archive should fail");

    let runtimes_dir = app_dir.path().join("runtimes");
    let temp_dir = runtimes_dir.join(format!("temp_{version_name}"));

    assert!(!temp_dir.exists(), "temp dir should be cleaned after failure");
    assert!(
        error.contains("有效的 ZIP") || error.contains("有效的 tar.gz"),
        "unexpected error: {}",
        error
    );
}
