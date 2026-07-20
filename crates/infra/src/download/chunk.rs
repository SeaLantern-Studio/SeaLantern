//! Single chunk download.
//!
//! Sends a `Range` request and streams the response to the specified file position.
//! Supports cancellation signals via `tokio::select!`.

use std::sync::Arc;

use reqwest::StatusCode;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter, SeekFrom};

use crate::download::status::{DownloadError, DownloadStatus};
use crate::net::client::NetClient;
use crate::observability;

/// Downloads a single chunk.
///
/// Requests data in the `bytes=start-end` range from the server,
/// seeks to the corresponding file position and writes streaming data.
///
/// # Parameters
///
/// - `client`: Configured HTTP client
/// - `url`: Download URL
/// - `path`: Local save path
/// - `start`: Chunk start position
/// - `end`: Chunk end position
/// - `status`: Shared download status (for progress reporting and cancellation detection)
///
/// # Returns
///
/// Returns `Ok(())` on success, `DownloadError` on failure.
///
/// # Cancellation Behavior
///
/// Uses `tokio::select!` to wait for both download and cancellation signals:
/// - If cancellation is received during download → immediately returns `Cancelled`
/// - If cancellation flag is checked after download completes → returns `Cancelled`
pub(super) async fn download_chunk(
    client: &NetClient,
    url: &str,
    path: &str,
    start: u64,
    end: u64,
    status: Arc<DownloadStatus>,
) -> Result<(), DownloadError> {
    observability::chunk_started(url, start, end);

    tokio::select! {
        result = async {
            let range = format!("bytes={}-{}", start, end);
            let mut response = client.get_reqwest_client().get(url).header("Range", range).send().await?;

            validate_chunk_response(&response, start)?;

            let file = OpenOptions::new().write(true).open(path).await?;
            let mut writer = BufWriter::with_capacity(128 * 1024, file);
            writer.seek(SeekFrom::Start(start)).await?;

            let mut local_downloaded = 0u64;
            while let Some(chunk) = response.chunk().await? {
                if status.cancelled() {
                    return Err(DownloadError::Cancelled("任务已取消".to_string()));
                }

                let len = chunk.len() as u64;
                writer.write_all(&chunk).await?;

                local_downloaded += len;
                if local_downloaded > 512 * 1024 {
                    status
                        .downloaded
                        .fetch_add(local_downloaded, std::sync::atomic::Ordering::Relaxed);
                    local_downloaded = 0;
                }
            }

            writer.flush().await?;
            status
                .downloaded
                .fetch_add(local_downloaded, std::sync::atomic::Ordering::Relaxed);

            observability::chunk_completed(url, start, end);
            Ok(())
        } => {
            match result {
                Ok(ok) => Ok(ok),
                Err(err) => {
                    if status.cancelled() {
                        Err(DownloadError::Cancelled("任务已取消".to_string()))
                    } else {
                        observability::chunk_failed(url, start, end, &err);
                        Err(err)
                    }
                }
            }
        },
        _ = status.cancel_token.cancelled() => {
            Err(DownloadError::Cancelled("任务已取消".to_string()))
        }
    }
}

/// Validates chunk response.
///
/// # Parameters
///
/// - `response`: Server response
/// - `start`: Chunk start position (must return 206 if greater than 0)
fn validate_chunk_response(response: &reqwest::Response, start: u64) -> Result<(), DownloadError> {
    if start > 0 && response.status() != StatusCode::PARTIAL_CONTENT {
        return Err(DownloadError::Response(
            response.status().as_u16(),
            "服务器未按 Range 返回 206".to_string(),
        ));
    }

    if !response.status().is_success() && response.status() != StatusCode::PARTIAL_CONTENT {
        return Err(DownloadError::Response(
            response.status().as_u16(),
            format!("下载失败，状态码: {}", response.status()),
        ));
    }

    Ok(())
}
