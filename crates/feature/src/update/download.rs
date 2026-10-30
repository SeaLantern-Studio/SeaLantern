//! 更新文件下载模块。
//!
//! 提供更新文件的 HTTP 下载以及下载后的 SHA256 校验功能。
//! 主要入口为 [`download_update_file_without_events`]。
//! 哈希计算委托给 `sealantern_infra::fs::hash`。
//!
//! # 下载源回退
//!
//! 镜像地址与主地址描述同一个发布产物，因此按顺序尝试：主地址只等待有限的
//! 首包时长且不重试，一旦超时或连接失败立即切换到镜像地址；镜像地址沿用默认
//! 重试策略。校验值不匹配时会删除已下载文件并尝试下一个来源，全部来源失败时
//! 返回合并后的失败信息。
//!
//! # 错误处理
//!
//! 所有失败路径均返回 `Err(String)`，并同步通过 `observability` 模块记录结构化日志。

use std::path::{Path, PathBuf};
use std::time::Duration;

use sealantern_infra::net::{NetClient, NetError, RetryPolicy};

use super::types::DownloadProgress;
use crate::observability;

/// 主下载地址等待响应头的上限。
///
/// 超过该时长仍未收到响应即认为来源不可达，交由镜像地址接手，避免在重试
/// 与退避上消耗用户等待时间。
const PRIMARY_FIRST_BYTE_TIMEOUT: Duration = Duration::from_secs(5);

/// 单次下载尝试的结果。
enum AttemptOutcome {
    /// 响应体读取完成，等待落盘与校验。
    Downloaded(Vec<u8>),
    /// 该来源当前不可用，可以尝试其他来源。
    Retryable(String),
    /// 换源也无法解决的失败。
    Fatal(String),
}

/// 从 URL 提取文件名
pub fn file_name_from_url(url: &str) -> String {
    let candidate = url.rsplit('/').next().unwrap_or("update");
    let candidate = candidate.split('?').next().unwrap_or("update");
    let candidate = candidate.split('#').next().unwrap_or("update");
    if candidate.trim().is_empty() {
        "update".to_string()
    } else {
        candidate.to_string()
    }
}

/// 下载更新文件 (不包含 Tauri 事件发送)
pub async fn download_update_file_without_events(
    url: String,
    mirror_url: Option<String>,
    expected_hash: Option<String>,
    cache_dir: PathBuf,
) -> Result<String, String> {
    observability::update_download_started(&url);

    std::fs::create_dir_all(&cache_dir).map_err(|error| {
        let message = format!("Failed to create cache directory: {error}");
        observability::update_download_failed(&url, &message);
        message
    })?;

    let file_name = file_name_from_url(&url);
    let file_path = cache_dir.join(file_name);

    // 获取当前全局客户端，与全局代理设置保持一致。
    let client = sealantern_infra::net::global_client().map_err(|error| {
        let message = format!("HTTP client init failed: {error}");
        observability::update_download_failed(&url, &message);
        message
    })?;

    let expected_hash = expected_hash.as_deref();

    // 主地址在前，镜像地址在后；只有主地址限制首包等待时长。
    let mut sources = vec![(url.as_str(), Some(PRIMARY_FIRST_BYTE_TIMEOUT))];
    if let Some(mirror) = mirror_url.as_deref() {
        sources.push((mirror, None));
    }

    let mut failures: Vec<String> = Vec::new();
    for (source, first_byte_timeout) in sources {
        match try_download_once(&client, source, first_byte_timeout).await {
            AttemptOutcome::Downloaded(bytes) => {
                match write_and_verify(&file_path, &bytes, expected_hash).await {
                    Ok(()) => {
                        let path = file_path.to_string_lossy().to_string();
                        observability::update_download_completed(&path);
                        return Ok(path);
                    }
                    // 校验失败说明该来源的文件不可信，换下一个来源再试。
                    Err(error) => failures.push(format!("{source}: {error}")),
                }
            }
            AttemptOutcome::Retryable(error) => failures.push(format!("{source}: {error}")),
            AttemptOutcome::Fatal(error) => {
                failures.push(format!("{source}: {error}"));
                // 请求参数或客户端层面的问题，换源不会有帮助。
                break;
            }
        }
    }

    let message = if failures.is_empty() {
        "no download source available".to_string()
    } else {
        failures.join("; ")
    };
    observability::update_download_failed(&url, &message);
    Err(message)
}

/// 按来源尝试一次下载。
///
/// `first_byte_timeout` 为 `Some` 时不重试并限制等待响应头的时长，用于主地址；
/// 为 `None` 时沿用客户端默认重试策略，用于镜像地址。
async fn try_download_once(
    client: &NetClient,
    url: &str,
    first_byte_timeout: Option<Duration>,
) -> AttemptOutcome {
    let builder = match client.get(url) {
        Ok(builder) => builder,
        Err(error) => return AttemptOutcome::Fatal(format!("HTTP client init failed: {error}")),
    };

    // 重试由编排层跨来源完成，主地址本身不再重试。
    let builder = if first_byte_timeout.is_some() {
        builder.retry(RetryPolicy { max_retries: 0, ..RetryPolicy::default() })
    } else {
        builder
    };

    let response = match first_byte_timeout {
        Some(timeout) => match tokio::time::timeout(timeout, builder.send()).await {
            Ok(Ok(response)) => response,
            Ok(Err(error)) => return classify_net_error(error),
            Err(_) => {
                return AttemptOutcome::Retryable(format!(
                    "no response within {}s",
                    timeout.as_secs()
                ));
            }
        },
        None => match builder.send().await {
            Ok(response) => response,
            Err(error) => return classify_net_error(error),
        },
    };

    let status = response.status();
    if !status.is_success() {
        let message = format!("Download failed with status: {status}");
        // 资源缺失（镜像尚未同步）、鉴权受限与服务端故障都值得换来源再试。
        return if status.is_server_error() || matches!(status.as_u16(), 401 | 403 | 404) {
            AttemptOutcome::Retryable(message)
        } else {
            AttemptOutcome::Fatal(message)
        };
    }

    match response.bytes().await {
        Ok(bytes) => AttemptOutcome::Downloaded(bytes.to_vec()),
        Err(error) => AttemptOutcome::Retryable(format!("Failed to read response: {error}")),
    }
}

/// 判断网络错误是否值得换源重试。
fn classify_net_error(error: NetError) -> AttemptOutcome {
    let retryable = match &error {
        // 连接建立失败与超时属于来源不可达。
        NetError::Request(_) | NetError::Timeout => true,
        // 响应类错误按状态码判断，与 `try_download_once` 中的判定保持一致。
        NetError::Response(status, _) => *status >= 500 || matches!(*status, 401 | 403 | 404),
        NetError::Config(_) | NetError::Parse(_) | NetError::Io(_) | NetError::Cancelled => false,
    };
    let message = format!("Download request failed: {error}");

    if retryable {
        AttemptOutcome::Retryable(message)
    } else {
        AttemptOutcome::Fatal(message)
    }
}

/// 落盘并按需校验 SHA256。
///
/// 校验不通过时删除已写入的文件，避免把损坏内容留在缓存目录里。
async fn write_and_verify(
    file_path: &Path,
    bytes: &[u8],
    expected_hash: Option<&str>,
) -> Result<(), String> {
    std::fs::write(file_path, bytes).map_err(|error| format!("Failed to write file: {error}"))?;

    let Some(expected) = expected_hash else {
        return Ok(());
    };

    let calculated = sealantern_infra::fs::sha256_file(file_path)
        .await
        .map_err(|error| format!("Failed to calculate hash: {error}"))?;
    let calculated_hex = sealantern_infra::fs::sha256_hex(calculated);
    let display_path = file_path.to_string_lossy().to_string();

    if calculated_hex.eq_ignore_ascii_case(expected) {
        observability::update_hash_verified(&display_path);
        return Ok(());
    }

    std::fs::remove_file(file_path).ok();
    let message = format!("Hash verification failed. Expected: {expected}, Got: {calculated_hex}");
    observability::update_hash_mismatch(&display_path, expected, &calculated_hex);
    Err(message)
}

/// 计算下载进度
pub fn calculate_progress(downloaded: u64, total: u64) -> DownloadProgress {
    let percent = if total > 0 {
        (downloaded as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    DownloadProgress { downloaded, total, percent }
}
