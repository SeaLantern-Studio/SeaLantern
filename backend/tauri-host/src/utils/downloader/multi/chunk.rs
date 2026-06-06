use super::super::common::{downloader_t, downloader_t1};
use super::super::{DownloadError, DownloadStatus};
use reqwest::{Client, Response, StatusCode};
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter, SeekFrom};

/// 下载单个分片
pub(super) async fn download_chunk(
    client: Arc<Client>,
    url: String,
    path: String,
    start: u64,
    end: u64,
    status: Arc<DownloadStatus>,
) -> Result<(), DownloadError> {
    tokio::select! {
        result = async {
            let mut response = request_chunk(&client, &url, start, end).await?;
            validate_chunk_response(&response, start)?;

            let file = OpenOptions::new().write(true).open(&path).await?;
            let mut writer = BufWriter::with_capacity(128 * 1024, file);
            writer.seek(SeekFrom::Start(start)).await?;

            let mut local_downloaded = 0u64;
            while let Some(chunk) = response.chunk().await? {
                if status.cancelled() {
                    return Err(DownloadError::Cancelled(downloader_t("download.util.task_cancelled")));
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

            Ok(())
        } => {
            match result {
                Ok(_) => Ok(()),
                Err(err) => {
                    if status.cancelled() {
                        Err(DownloadError::Cancelled(downloader_t("download.util.task_cancelled")))
                    } else {
                        Err(err)
                    }
                }
            }
        },
        _ = status.cancel_token.cancelled() => {
            Err(DownloadError::Cancelled(downloader_t("download.util.task_cancelled")))
        }
    }
}

async fn request_chunk(
    client: &Client,
    url: &str,
    start: u64,
    end: u64,
) -> Result<Response, reqwest::Error> {
    let range = format!("bytes={}-{}", start, end);
    client.get(url).header("Range", range).send().await
}

fn validate_chunk_response(response: &Response, start: u64) -> Result<(), DownloadError> {
    if start > 0 && response.status() != StatusCode::PARTIAL_CONTENT {
        return Err(DownloadError::Cancelled(downloader_t1(
            "download.util.range_response_invalid",
            response.status().to_string(),
        )));
    }

    if !response.status().is_success() && response.status() != StatusCode::PARTIAL_CONTENT {
        return Err(DownloadError::Cancelled(downloader_t1(
            "download.util.download_status_failed",
            response.status().to_string(),
        )));
    }

    Ok(())
}
