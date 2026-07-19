//! 单线程下载与文本获取工具。
//!
//! 提供流式文件下载和远端文本读取能力，不涉及分片和预分配。
//! 适用于小文件下载、API 请求等场景。

use std::sync::Arc;

use futures::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::download::status::DownloadStatus;
use crate::net::client::NetClient;
use crate::observability;

/// 单线程流式下载文件。
///
/// 直接发送 GET 请求，边读取响应体边写入文件。
/// 不分片、不预分配，适用于不支持 Range 的服务器或小文件。
///
/// # Parameters
///
/// - `client`: 已配置的 HTTP 客户端
/// - `url`: 下载地址
/// - `output_path`: 本地保存路径
///
/// # Returns
///
/// 返回 `Arc<DownloadStatus>`，可通过 `snapshot()` 查询实时进度。
/// 若服务器未返回 `Content-Length`，`total_size` 为 0。
pub async fn stream_download(
    client: &NetClient,
    url: &str,
    output_path: &str,
) -> Result<Arc<DownloadStatus>, String> {
    tracing::info!(
        target: observability::DOWNLOAD_TARGET,
        event_name = observability::EVENT_DOWNLOAD_STARTED,
        url,
        "stream download started"
    );

    let response = client
        .get_reqwest_client()
        .get(url)
        .send()
        .await
        .map_err(|e| {
            observability::download_failed(url, &e);
            format!("请求失败: {}", e)
        })?;

    if !response.status().is_success() {
        let msg = format!("服务器返回 {}", response.status());
        observability::download_failed(url, &msg);
        return Err(msg);
    }

    if let Some(parent) = std::path::Path::new(output_path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let total_size = response.content_length().unwrap_or(0);
    let status = Arc::new(DownloadStatus::new(total_size));

    let mut file = tokio::fs::File::create(output_path)
        .await
        .map_err(|e| format!("创建文件失败: {}", e))?;

    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        if status.cancelled() {
            let _ = tokio::fs::remove_file(output_path).await;
            return Err("下载已取消".to_string());
        }

        let chunk = item.map_err(|e| format!("流读取失败: {}", e))?;
        let len = chunk.len() as u64;

        file.write_all(&chunk)
            .await
            .map_err(|e| format!("写入文件失败: {}", e))?;

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

/// 读取远端文本内容。
///
/// 以字符串形式返回 GET 请求的响应体。
/// 适用于调用 REST API、获取配置文件等场景。
///
/// # Parameters
///
/// - `client`: 已配置的 HTTP 客户端
/// - `url`: 请求地址
///
/// # Returns
///
/// 返回响应体文本；请求失败或状态码非 2xx 时返回错误描述。
pub async fn fetch_to_string(client: &NetClient, url: &str) -> Result<String, String> {
    let response = client
        .get_reqwest_client()
        .get(url)
        .send()
        .await
        .map_err(|e| {
            observability::download_failed(url, &e);
            format!("请求失败: {}", e)
        })?;

    if !response.status().is_success() {
        let msg = format!("服务器返回 {}", response.status());
        observability::download_failed(url, &msg);
        return Err(msg);
    }

    let text = response
        .text()
        .await
        .map_err(|e| format!("读取响应体失败: {}", e))?;

    tracing::debug!(
        target: observability::DOWNLOAD_TARGET,
        url,
        len = text.len(),
        "text request completed"
    );

    Ok(text)
}

/// 读取远端二进制内容。
///
/// 以字节数组形式返回 GET 请求的响应体。
///
/// # Parameters
///
/// - `client`: 已配置的 HTTP 客户端
/// - `url`: 请求地址
///
/// # Returns
///
/// 返回响应体字节数组；请求失败或状态码非 2xx 时返回错误描述。
pub async fn fetch_to_bytes(client: &NetClient, url: &str) -> Result<Vec<u8>, String> {
    let response = client
        .get_reqwest_client()
        .get(url)
        .send()
        .await
        .map_err(|e| {
            observability::download_failed(url, &e);
            format!("请求失败: {}", e)
        })?;

    if !response.status().is_success() {
        let msg = format!("服务器返回 {}", response.status());
        observability::download_failed(url, &msg);
        return Err(msg);
    }

    let bytes = response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("读取响应体失败: {}", e))?;

    tracing::debug!(
        target: observability::DOWNLOAD_TARGET,
        url,
        len = bytes.len(),
        "binary request completed"
    );

    Ok(bytes)
}
