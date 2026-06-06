//! 下载工具
//!
//! 这里提供多线程文件下载、单线程文本读取和下载进度快照

mod common;
mod multi;
mod single;
mod status;

pub use multi::MultiThreadDownloader;
pub use single::SingleThreadDownloader;
pub use status::{DownloadError, DownloadStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardcode_data::external_services::COMMON_HTTP_BROWSER_USER_AGENT;
    use axum::body::Body;
    use axum::extract::State;
    use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
    use axum::response::IntoResponse;
    use axum::routing::get;
    use axum::Router;
    use std::ops::RangeInclusive;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::net::TcpListener;

    #[derive(Clone)]
    struct TestDownloadPayload {
        bytes: Arc<Vec<u8>>,
    }

    async fn serve_test_payload(
        State(payload): State<TestDownloadPayload>,
        headers: HeaderMap,
    ) -> impl IntoResponse {
        let total_len = payload.bytes.len();

        if let Some(range_header) = headers.get(header::RANGE) {
            let range_value = range_header
                .to_str()
                .expect("range header should be valid utf-8");
            let range = parse_range_header(range_value, total_len)
                .expect("range header should be parseable for test payload");

            let start = *range.start();
            let end = *range.end();
            let bytes = payload.bytes[start..=end].to_vec();
            let content_range = format!("bytes {}-{}/{}", start, end, total_len);

            let mut response_headers = HeaderMap::new();
            response_headers.insert(
                header::CONTENT_RANGE,
                HeaderValue::from_str(&content_range).expect("content-range should be valid"),
            );
            response_headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&bytes.len().to_string())
                    .expect("content-length should be valid"),
            );

            return (StatusCode::PARTIAL_CONTENT, response_headers, Body::from(bytes))
                .into_response();
        }

        let mut response_headers = HeaderMap::new();
        response_headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&total_len.to_string()).expect("content-length should be valid"),
        );

        (StatusCode::OK, response_headers, Body::from((*payload.bytes).clone())).into_response()
    }

    fn parse_range_header(value: &str, total_len: usize) -> Option<RangeInclusive<usize>> {
        let bytes = value.strip_prefix("bytes=")?;
        let (start, end) = bytes.split_once('-')?;
        let start = start.parse::<usize>().ok()?;
        let end = if end.is_empty() {
            total_len.checked_sub(1)?
        } else {
            end.parse::<usize>().ok()?
        };

        if start > end || end >= total_len {
            return None;
        }

        Some(start..=end)
    }

    /// 测试多线程下载
    #[tokio::test]
    async fn test_multi_thread_download() -> Result<(), String> {
        let downloader = MultiThreadDownloader::new(COMMON_HTTP_BROWSER_USER_AGENT);

        let payload = TestDownloadPayload { bytes: Arc::new(vec![b'x'; 512 * 1024]) };
        let app = Router::new()
            .route("/download.bin", get(serve_test_payload))
            .with_state(payload.clone());
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .await
            .map_err(|e| e.to_string())?;
        let addr = listener.local_addr().map_err(|e| e.to_string())?;
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("test download server should run");
        });

        let url = format!("http://{}/download.bin", addr);
        let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let save_path = temp_dir.path().join("multi_thread_download_test.bin");
        let save_path_str = save_path.to_string_lossy().to_string();

        match downloader.download(&url, &save_path_str, 32).await {
            Ok(status_handle) => {
                println!("Downloaded to {:?}", save_path);
                loop {
                    let info = status_handle.snapshot().await;
                    println!(
                        "当前进度: {:.2}% ({} / {})",
                        info.progress_percentage, info.downloaded, info.total_size
                    );

                    if info.is_finished {
                        if let Some(error) = info.error {
                            return Err(error);
                        }

                        println!("下载完成！");
                        break;
                    }

                    tokio::time::sleep(Duration::from_millis(200)).await;
                }

                let bytes = std::fs::read(&save_path).map_err(|e| e.to_string())?;
                assert_eq!(bytes.len(), payload.bytes.len());
                assert!(bytes.iter().all(|byte| *byte == b'x'));
                Ok(())
            }
            Err(e) => {
                eprintln!("\n 下载中止: {}", e);

                if save_path.exists() {
                    let _ = std::fs::remove_file(&save_path);
                    println!("已清理不完整的文件。");
                }
                Err(e.to_string())
            }
        }
    }
}
