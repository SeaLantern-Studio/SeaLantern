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
        .map_err(|e| format!("探测请求失败: {}", e))?;

    if !probe.status().is_success() && probe.status() != StatusCode::PARTIAL_CONTENT {
        return Err(format!("探测失败，状态码: {}", probe.status()));
    }

    let supports_range = probe.status() == StatusCode::PARTIAL_CONTENT;
    let total_size = parse_total_size(&probe, supports_range)?;

    Ok(RemoteFileInfo {
        total_size,
        supports_range,
    })
}

fn parse_total_size(response: &Response, supports_range: bool) -> Result<u64, String> {
    if supports_range {
        response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.rsplit('/').next())
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or("服务器返回 206，但缺少有效 Content-Range".to_string())
    } else {
        response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or("服务器未返回 Content-Length".to_string())
    }
}
