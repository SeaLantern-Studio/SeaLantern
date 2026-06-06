use super::super::common::downloader_t1;
use reqwest::{Client, Response, StatusCode};

pub(super) struct RemoteFileInfo {
    pub(super) total_size: u64,
    pub(super) supports_range: bool,
}

/// 探测远端文件大小和是否支持分片下载
pub(super) async fn probe_remote_file(
    client: &Client,
    url: &str,
) -> Result<RemoteFileInfo, String> {
    let probe = client
        .get(url)
        .header(reqwest::header::RANGE, "bytes=0-0")
        .send()
        .await
        .map_err(|e| downloader_t1("download.util.probe_request_failed", e.to_string()))?;

    if !probe.status().is_success() && probe.status() != StatusCode::PARTIAL_CONTENT {
        return Err(downloader_t1("download.util.probe_status_failed", probe.status().to_string()));
    }

    let supports_range = probe.status() == StatusCode::PARTIAL_CONTENT;
    let total_size = parse_total_size(&probe, supports_range)?;

    Ok(RemoteFileInfo { total_size, supports_range })
}

fn parse_total_size(response: &Response, supports_range: bool) -> Result<u64, String> {
    if supports_range {
        response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.rsplit('/').next())
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or_else(|| downloader_t1("download.util.content_range_missing", "206"))
    } else {
        response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or_else(|| downloader_t1("download.util.content_length_missing", "Content-Length"))
    }
}
