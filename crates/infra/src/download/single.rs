//! 单线程下载和文本获取工具集。
//!
//! 提供流式文件下载和远程文本获取能力，不支持分块或预分配。
//! 适用于小文件下载、API 请求等场景。

use std::sync::Arc;

use futures::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::download::status::{DownloadError, DownloadStatus};
use crate::net::client::NetClient;
use crate::observability;

/// 使用单线程流式下载文件。
///
/// 发送 GET 请求并将响应主体边读边写入文件。
/// 不分段、不预分配；适用于不支持 Range 的服务器或小文件。
///
/// # Parameters
///
/// - `client`: 配置好的 HTTP 客户端
/// - `url`: 下载地址
/// - `output_path`: 本地保存路径
///
/// # Returns
///
/// 返回 `Arc<DownloadStatus>`，可通过 `snapshot()` 查询进度。
/// 如果服务器未返回 `Content-Length`，则 `total_size` 为 0。
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

/// 获取远程文本内容。
///
/// 将 GET 请求的响应主体以字符串形式返回。
/// 适用于调用 REST API、获取配置文件等场景。
///
/// # Parameters
///
/// - `client`: 配置好的 HTTP 客户端
/// - `url`: 请求地址
///
/// # Returns
///
/// 返回响应文本；失败或返回非 2xx 状态码时返回错误描述。
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

/// 获取远程二进制内容。
///
/// 将 GET 请求的响应主体以字节向量的形式返回。
///
/// # Parameters
///
/// - `client`: 配置好的 HTTP 客户端
/// - `url`: 请求地址
///
/// # Returns
///
/// 返回响应字节；失败或返回非 2xx 状态码时返回错误描述。
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
