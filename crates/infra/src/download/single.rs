//! Single-threaded download and text fetching utilities.
//!
//! Provides streaming file download and remote text fetching capabilities, without chunking or pre-allocation.
//! Suitable for small file downloads, API requests, etc.

use std::sync::Arc;

use futures::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::download::status::{DownloadError, DownloadStatus};
use crate::net::client::NetClient;
use crate::observability;

/// Stream-download a file with a single thread.
///
/// Sends a GET request and writes the response body to a file as it is read.
/// No segments, no pre-allocation; suitable for servers that don't support Range or for small files.
///
/// # Parameters
///
/// - `client`: Configured HTTP client
/// - `url`: Download URL
/// - `output_path`: Local save path
///
/// # Returns
///
/// Returns `Arc<DownloadStatus>`, progress can be queried via `snapshot()`.
/// If the server does not return `Content-Length`, `total_size` is 0.
pub async fn stream_download(
    client: &NetClient,
    url: &str,
    output_path: &str,
) -> Result<Arc<DownloadStatus>, DownloadError> {
    tracing::info!(
        target: observability::DOWNLOAD_TARGET,
        event_name = observability::EVENT_DOWNLOAD_STARTED,
        url,
        "stream download started"
    );

    let response = client.get_reqwest_client().get(url).send().await?;

    if !response.status().is_success() {
        let msg = format!("服务器返回 {}", response.status());
        observability::download_failed(url, &msg);
        return Err(DownloadError::Response(response.status().as_u16(), msg));
    }

    if let Some(parent) = std::path::Path::new(output_path).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let total_size = response.content_length().unwrap_or(0);
    let status = Arc::new(DownloadStatus::new(total_size));

    let mut file = tokio::fs::File::create(output_path).await?;

    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        if status.cancelled() {
            let _ = tokio::fs::remove_file(output_path).await;
            return Err(DownloadError::Cancelled("下载已取消".to_string()));
        }

        let chunk = item?;
        let len = chunk.len() as u64;

        file.write_all(&chunk).await?;

        status
            .downloaded
            .fetch_add(len, std::sync::atomic::Ordering::Relaxed);
    }

    tracing::info!(
        target: observability::DOWNLOAD_TARGET,
        event_name = observability::EVENT_DOWNLOAD_COMPLETED,
        url,
        total_size,
        "stream download completed"
    );

    Ok(status)
}

/// Fetches remote text content.
///
/// Returns the response body of a GET request as a string.
/// Suitable for calling REST APIs, fetching config files, etc.
///
/// # Parameters
///
/// - `client`: Configured HTTP client
/// - `url`: Request URL
///
/// # Returns
///
/// Returns the response text; returns an error description on failure or non-2xx status code.
pub async fn fetch_to_string(client: &NetClient, url: &str) -> Result<String, DownloadError> {
    let response = client.get_reqwest_client().get(url).send().await?;

    if !response.status().is_success() {
        let msg = format!("服务器返回 {}", response.status());
        observability::download_failed(url, &msg);
        return Err(DownloadError::Response(response.status().as_u16(), msg));
    }

    let text = response.text().await?;

    tracing::debug!(
        target: observability::DOWNLOAD_TARGET,
        url,
        len = text.len(),
        "text request completed"
    );

    Ok(text)
}

/// Fetches remote binary content.
///
/// Returns the response body of a GET request as a byte vector.
///
/// # Parameters
///
/// - `client`: Configured HTTP client
/// - `url`: Request URL
///
/// # Returns
///
/// Returns the response bytes; returns an error description on failure or non-2xx status code.
pub async fn fetch_to_bytes(client: &NetClient, url: &str) -> Result<Vec<u8>, DownloadError> {
    let response = client.get_reqwest_client().get(url).send().await?;

    if !response.status().is_success() {
        let msg = format!("服务器返回 {}", response.status());
        observability::download_failed(url, &msg);
        return Err(DownloadError::Response(response.status().as_u16(), msg));
    }

    let bytes = response.bytes().await.map(|b| b.to_vec())?;

    tracing::debug!(
        target: observability::DOWNLOAD_TARGET,
        url,
        len = bytes.len(),
        "binary request completed"
    );

    Ok(bytes)
}
