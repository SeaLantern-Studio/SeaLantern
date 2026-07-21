//! 单块下载。
//!
//! 发送 `Range` 请求并将响应流式写入指定文件位置。
//! 通过 `tokio::select!` 支持取消信号。

use std::sync::Arc;

use reqwest::StatusCode;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter, SeekFrom};

use crate::download::status::{DownloadError, DownloadStatus};
use crate::net::client::NetClient;
use crate::observability;

/// 下载单个块。
///
/// 向服务器请求 `bytes=start-end` 范围内的数据，
/// 定位到文件对应位置并写入流式数据。
///
/// # Parameters
///
/// - `client`: 配置好的 HTTP 客户端
/// - `url`: 下载地址
/// - `path`: 本地保存路径
/// - `start`: 块起始位置
/// - `end`: 块结束位置
/// - `status`: 共享下载状态（用于进度报告和取消检测）
///
/// # Returns
///
/// 成功时返回 `Ok(())`，失败时返回 `DownloadError`。
///
/// # Cancellation Behavior
///
/// 使用 `tokio::select!` 同时等待下载和取消信号：
/// - 如果在下载过程中收到取消信号 → 立即返回 `Cancelled`
/// - 如果下载完成后检查取消标志 → 返回 `Cancelled`
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

/// 验证块响应。
///
/// # Parameters
///
/// - `response`: 服务器响应
/// - `start`: 块起始位置（如果大于 0，必须返回 206）
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
