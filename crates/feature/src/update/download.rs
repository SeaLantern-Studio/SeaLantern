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

use std::io::{BufWriter, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::time::Duration;

use sealantern_infra::net::{NetClient, NetError, RetryPolicy};
use sha2::{Digest, Sha256};

use super::types::DownloadProgress;
use crate::observability;

/// 主下载地址等待响应头的上限。
///
/// 超过该时长仍未收到响应即认为来源不可达，交由镜像地址接手，避免在重试
/// 与退避上消耗用户等待时间。
///
/// 取值依据：实测 GitHub 的首字节时间约 2.2 秒，CNB 约 0.3–0.6 秒，而网络层
/// 连接超时为 15 秒。取 5 秒既给正常慢速留出余量，又避免 GitHub 挂起时让用户
/// 干等 15 秒。实机验证中该阈值在 5.006 秒精确触发。
///
/// 若后续 `update_download_mirror_used` 记录的失败原因长期以超时为主，说明该
/// 阈值偏紧，可考虑放宽或改为可配置项。
const PRIMARY_FIRST_BYTE_TIMEOUT: Duration = Duration::from_secs(5);

/// 单次下载尝试的结果。
enum AttemptOutcome {
    /// 响应体已完整写入目标文件，附带已接收字节数与落盘计算出的摘要。
    Downloaded {
        /// 实际写入的字节数。
        received: u64,
        /// 流式计算得到的 SHA-256 摘要；未要求校验时为 `None`。
        digest: Option<[u8; 32]>,
    },
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
    let file_path = cache_dir.join(&file_name);

    // 获取当前全局客户端，与全局代理设置保持一致。
    let client = sealantern_infra::net::global_client().map_err(|error| {
        let message = format!("HTTP client init failed: {error}");
        observability::update_download_failed(&url, &message);
        message
    })?;

    let expected_hash = expected_hash.as_deref();

    let sources = build_download_sources(url.as_str(), mirror_url.as_deref());

    let mut failures: Vec<String> = Vec::new();
    let mut last_failure: Option<String> = None;
    for (index, source) in sources.iter().enumerate() {
        // 走到后续来源说明前一个来源已失败，此时记录一次镜像回退。
        // 兜底文案仅用于防御，正常流程下 last_failure 必定已赋值。
        if index > 0 {
            let reason = last_failure.as_deref().unwrap_or("previous source failed");
            observability::update_download_mirror_used(&url, source.url, &reason);
        }

        // 每次尝试都写入独立的临时文件：既不会截断已有缓存，也不会与并发
        // 下载互相覆盖，只有校验通过后才替换到最终路径。
        let temp = TempDownload::new(&cache_dir, &file_name);

        let outcome = match try_download_once(&client, source, temp.path(), expected_hash).await {
            AttemptOutcome::Downloaded { received, digest } => {
                match verify_digest(temp.path(), digest, received, expected_hash) {
                    Ok(()) => temp.commit(&file_path),
                    Err(error) => Err(error),
                }
            }
            AttemptOutcome::Retryable(error) => Err(error),
            AttemptOutcome::Fatal(error) => {
                // 请求参数或客户端层面的问题，换源不会有帮助。
                failures.push(format!("{}: {error}", source.url));
                break;
            }
        };

        match outcome {
            Ok(()) => {
                let path = file_path.to_string_lossy().to_string();
                observability::update_download_completed(&path);
                return Ok(path);
            }
            Err(error) => {
                let reason = format!("{}: {error}", source.url);
                last_failure = Some(reason.clone());
                failures.push(reason);
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

/// 下载期间使用的临时文件。
///
/// 未提交即被丢弃时自动删除，因此中途失败、取消或提前 return 都不会在缓存
/// 目录留下残缺内容。
struct TempDownload {
    path: PathBuf,
    committed: bool,
}

impl TempDownload {
    fn new(cache_dir: &Path, file_name: &str) -> Self {
        Self {
            path: cache_dir.join(format!("{file_name}.{}.part", unique_suffix())),
            committed: false,
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    /// 把临时文件替换为最终缓存路径。
    fn commit(mut self, final_path: &Path) -> Result<(), String> {
        // Windows 下 rename 到已存在文件会失败，需要先移除旧文件。
        if final_path.exists() {
            std::fs::remove_file(final_path)
                .map_err(|error| format!("Failed to replace cached file: {error}"))?;
        }
        std::fs::rename(&self.path, final_path)
            .map_err(|error| format!("Failed to move download into place: {error}"))?;

        self.committed = true;
        Ok(())
    }
}

impl Drop for TempDownload {
    fn drop(&mut self) {
        if !self.committed {
            std::fs::remove_file(&self.path).ok();
        }
    }
}

/// 生成足以区分并发下载的临时文件后缀。
fn unique_suffix() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);

    format!("{}-{nanos}-{sequence}", std::process::id())
}

/// 单个下载来源及其尝试策略。
struct DownloadSource<'a> {
    /// 下载地址。
    url: &'a str,
    /// 等待响应头的上限；`None` 表示沿用客户端超时。
    first_byte_timeout: Option<Duration>,
    /// 该来源允许的重试次数（含响应体读取阶段）。
    max_retries: u32,
}

/// 按尝试顺序组装下载来源。
///
/// 只有在存在备用镜像时才让主地址「短超时且不重试」：此时快速失败并换源比在
/// 主地址上反复退避更划算。没有镜像时主地址是唯一希望，必须保留默认重试策略，
/// 否则短暂抖动就会直接导致更新失败。
///
/// 镜像地址与主地址相同时只保留一个，避免对同一地址重复发起完整请求。
fn build_download_sources<'a>(
    primary: &'a str,
    mirror: Option<&'a str>,
) -> Vec<DownloadSource<'a>> {
    let mirror = mirror
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != primary);
    let default_retries = RetryPolicy::default().max_retries;

    let Some(mirror) = mirror else {
        return vec![DownloadSource {
            url: primary,
            first_byte_timeout: None,
            max_retries: default_retries,
        }];
    };

    vec![
        DownloadSource {
            url: primary,
            first_byte_timeout: Some(PRIMARY_FIRST_BYTE_TIMEOUT),
            // 重试由换源代替，避免在主地址上白等。
            max_retries: 0,
        },
        DownloadSource {
            url: mirror,
            first_byte_timeout: None,
            max_retries: default_retries,
        },
    ]
}

/// 单次流式接收的结果。
enum StreamOutcome {
    /// 响应体已完整落盘。
    Completed {
        received: u64,
        digest: Option<[u8; 32]>,
    },
    /// 传输中断或内容不完整，同一来源重试有机会成功。
    Interrupted(String),
    /// 本地写入失败，换源或重试都无济于事。
    Fatal(String),
}

/// 按来源尝试下载，必要时在同一来源上重试响应体读取。
///
/// 请求层不做重试（`max_retries` 固定为 0），重试完全由本函数控制：只有在
/// 响应体传输中断、长度不符等可恢复情形下才重新发起请求，本地写盘失败则
/// 直接终止。
async fn try_download_once(
    client: &NetClient,
    source: &DownloadSource<'_>,
    file_path: &Path,
    expected_hash: Option<&str>,
) -> AttemptOutcome {
    let mut body_attempts: u32 = 0;

    loop {
        let builder = match client.get(source.url) {
            Ok(builder) => builder,
            Err(error) => {
                return AttemptOutcome::Fatal(format!("HTTP client init failed: {error}"));
            }
        };
        // 请求层不重试，避免与下面的响应体重试叠加。
        let builder = builder.retry(RetryPolicy { max_retries: 0, ..RetryPolicy::default() });

        let response = match send_request(builder, source.first_byte_timeout).await {
            Ok(response) => response,
            Err(outcome) => return outcome,
        };

        let status = response.status();
        if !status.is_success() {
            let message = format!("Download failed with status: {status}");
            return if is_worth_retrying_status(status.as_u16()) {
                AttemptOutcome::Retryable(message)
            } else {
                AttemptOutcome::Fatal(message)
            };
        }

        match stream_to_file(response, file_path, expected_hash).await {
            StreamOutcome::Completed { received, digest } => {
                return AttemptOutcome::Downloaded { received, digest };
            }
            StreamOutcome::Fatal(error) => return AttemptOutcome::Fatal(error),
            StreamOutcome::Interrupted(error) => {
                if body_attempts < source.max_retries {
                    body_attempts += 1;
                    tracing::debug!(
                        target: "sealantern.feature.update",
                        event_name = "update_download_body_retry",
                        url = source.url,
                        attempt = body_attempts,
                        reason = %error,
                        "retry download body on same source"
                    );
                    continue;
                }
                return AttemptOutcome::Retryable(error);
            }
        }
    }
}

/// 发送请求，可选地限制等待响应头的时长。
async fn send_request(
    builder: sealantern_infra::net::RequestBuilder<'_>,
    first_byte_timeout: Option<Duration>,
) -> Result<reqwest::Response, AttemptOutcome> {
    match first_byte_timeout {
        Some(timeout) => match tokio::time::timeout(timeout, builder.send()).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(error)) => Err(classify_net_error(error)),
            Err(_) => {
                Err(AttemptOutcome::Retryable(format!("no response within {}s", timeout.as_secs())))
            }
        },
        None => match builder.send().await {
            Ok(response) => Ok(response),
            Err(error) => Err(classify_net_error(error)),
        },
    }
}

/// 把响应体流式写入文件并计算摘要。
async fn stream_to_file(
    response: reqwest::Response,
    file_path: &Path,
    expected_hash: Option<&str>,
) -> StreamOutcome {
    let declared_bytes = response.content_length();

    // 边读边写并同步喂给 hasher：落盘完成即校验完成，无需再读一次整个文件。
    let mut sink = match DownloadSink::create(file_path, expected_hash.is_some()) {
        Ok(sink) => sink,
        Err(error) => return StreamOutcome::Fatal(error),
    };

    let mut response = response;
    loop {
        match response.chunk().await {
            Ok(Some(chunk)) => {
                if let Err(error) = sink.write(&chunk) {
                    return StreamOutcome::Fatal(error);
                }
            }
            Ok(None) => break,
            Err(error) => {
                sink.discard();
                return StreamOutcome::Interrupted(format!("Failed to read response: {error}"));
            }
        }
    }

    let received = sink.received();
    tracing::info!(
        target: "sealantern.feature.update",
        event_name = "update_download_payload_received",
        declared_bytes = declared_bytes.unwrap_or(0),
        received_bytes = received,
        "update payload received"
    );

    // 服务端声明了长度却未给够，说明传输被截断，重来比落盘损坏文件更稳。
    // 分块传输不提供 `content_length` 时跳过该检查。
    if let Err(reason) = check_declared_length(declared_bytes, received) {
        sink.discard();
        return StreamOutcome::Interrupted(reason);
    }

    match sink.finish() {
        Ok(digest) => StreamOutcome::Completed { received, digest },
        Err(error) => StreamOutcome::Fatal(error),
    }
}

/// 判断某个失败状态码是否值得切换到下一个下载来源。
///
/// 资源缺失（镜像尚未同步）、鉴权受限、服务端故障与触发限流都值得换来源再试；
/// 其余 4xx 通常是请求本身有问题，换源也不会有帮助。
///
/// 429 尤其重要：GitHub 对发布产物下载有速率限制，此时换到镜像往往能直接成功。
///
/// 响应状态码与网络错误两条路径共用本函数，避免两处判定各自演化后出现分歧。
fn is_worth_retrying_status(status: u16) -> bool {
    (500..=599).contains(&status) || matches!(status, 401 | 403 | 404 | 429)
}

/// 判断网络错误是否值得换源重试。
fn classify_net_error(error: NetError) -> AttemptOutcome {
    let retryable = match &error {
        // 连接建立失败与超时属于来源不可达。
        NetError::Request(_) | NetError::Timeout => true,
        // 服务端返回的状态码交由统一判定处理。
        NetError::Response(status, _) => is_worth_retrying_status(*status),
        NetError::Config(_) | NetError::Parse(_) | NetError::Io(_) | NetError::Cancelled => false,
    };
    let message = format!("Download request failed: {error}");

    if retryable {
        AttemptOutcome::Retryable(message)
    } else {
        AttemptOutcome::Fatal(message)
    }
}

/// 比对服务端声明的长度与实际接收长度。
///
/// 仅在服务端给出 `Content-Length` 时生效：分块传输不提供该值，此时无法判断
/// 是否截断，直接放行。
fn check_declared_length(declared: Option<u64>, received: u64) -> Result<(), String> {
    match declared {
        Some(declared) if declared != received => {
            Err(format!("size mismatch: declared {declared}, received {received}"))
        }
        _ => Ok(()),
    }
}

/// 下载落盘器：边写边算摘要，避免把整个更新包读进内存，也不必写完后重读一遍。
struct DownloadSink {
    file: BufWriter<std::fs::File>,
    hasher: Sha256,
    received: u64,
    /// 是否需要在写入过程中累计摘要。
    ///
    /// 没有期望校验值时跳过哈希计算，省掉一次完整的数据搬运。
    hashing: bool,
    path: PathBuf,
}

impl DownloadSink {
    /// 创建落盘器，目标文件被截断为新建状态。
    fn create(path: &Path, hashing: bool) -> Result<Self, String> {
        let file = std::fs::File::create(path)
            .map_err(|error| format!("Failed to create file: {error}"))?;

        Ok(Self {
            file: BufWriter::new(file),
            hasher: Sha256::new(),
            received: 0,
            hashing,
            path: path.to_path_buf(),
        })
    }

    /// 追加一个分块。
    fn write(&mut self, chunk: &[u8]) -> Result<(), String> {
        self.file
            .write_all(chunk)
            .map_err(|error| format!("Failed to write file: {error}"))?;
        if self.hashing {
            self.hasher.update(chunk);
        }
        self.received += chunk.len() as u64;
        Ok(())
    }

    /// 已写入的字节数。
    fn received(&self) -> u64 {
        self.received
    }

    /// 冲刷并返回摘要；未启用校验时返回 `None`。
    fn finish(self) -> Result<Option<[u8; 32]>, String> {
        let Self { mut file, hasher, hashing, .. } = self;
        file.flush()
            .map_err(|error| format!("Failed to flush file: {error}"))?;
        // 数据必须真正落到磁盘后再交给安装流程，避免读到未落盘的缓存。
        file.get_ref()
            .sync_all()
            .map_err(|error| format!("Failed to sync file: {error}"))?;

        Ok(hashing.then(|| hasher.finalize().into()))
    }

    /// 放弃本次下载，删除残缺文件。
    fn discard(self) {
        drop(self.file);
        std::fs::remove_file(&self.path).ok();
    }
}

/// 校验流式计算出的摘要是否与期望值一致。
///
/// 不一致时删除已写入的文件，避免把损坏内容留在缓存目录里。
fn verify_digest(
    file_path: &Path,
    digest: Option<[u8; 32]>,
    received: u64,
    expected_hash: Option<&str>,
) -> Result<(), String> {
    let Some(expected) = expected_hash else {
        return Ok(());
    };

    let display_path = file_path.to_string_lossy().to_string();
    // 要求校验却拿不到摘要，说明落盘器配置有误，按失败处理而不是静默放行。
    let Some(digest) = digest else {
        return Err("hash digest missing although verification was requested".to_string());
    };

    let calculated_hex = sealantern_infra::fs::digest_to_hex(&digest);
    if calculated_hex.eq_ignore_ascii_case(expected) {
        observability::update_hash_verified(&display_path);
        return Ok(());
    }

    std::fs::remove_file(file_path).ok();
    let message = format!(
        "Hash verification failed after {received} bytes. Expected: {expected}, Got: {calculated_hex}"
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrying_status_covers_server_errors_and_resource_missing() {
        // 5xx：服务端故障，换源有意义。
        for status in [500, 502, 503, 504, 599] {
            assert!(is_worth_retrying_status(status), "{status} should be retryable");
        }

        // 401/403/404：鉴权受限与资源缺失；429：触发限流，换到镜像通常可用。
        for status in [401, 403, 404, 429] {
            assert!(is_worth_retrying_status(status), "{status} should be retryable");
        }
    }

    #[test]
    fn retrying_status_rejects_client_errors_and_success() {
        // 非 401/403/404/429 的 4xx 属于请求本身有问题，换源无益。
        for status in [400, 402, 405, 406, 409, 410, 422, 451] {
            assert!(!is_worth_retrying_status(status), "{status} should be fatal");
        }

        // 1xx/2xx/3xx 与越界值都不应进入重试分支。
        for status in [0, 100, 200, 204, 301, 304, 399, 600, 999] {
            assert!(!is_worth_retrying_status(status), "{status} should be fatal");
        }
    }

    #[test]
    fn network_errors_are_classified_for_source_fallback() {
        let retryable_cases = [
            NetError::Request("connection refused".to_string()),
            NetError::Timeout,
            NetError::Response(500, String::new()),
            NetError::Response(404, String::new()),
            NetError::Response(403, String::new()),
        ];
        for error in retryable_cases {
            let label = format!("{error}");
            assert!(
                matches!(classify_net_error(error), AttemptOutcome::Retryable(_)),
                "{label} should be retryable"
            );
        }

        let fatal_cases = [
            NetError::Config("bad proxy".to_string()),
            NetError::Parse("invalid json".to_string()),
            NetError::Io(std::io::Error::other("disk full")),
            NetError::Cancelled,
            NetError::Response(400, String::new()),
        ];
        for error in fatal_cases {
            let label = format!("{error}");
            assert!(
                matches!(classify_net_error(error), AttemptOutcome::Fatal(_)),
                "{label} should be fatal"
            );
        }
    }

    #[test]
    fn retryable_message_keeps_original_cause() {
        let outcome = classify_net_error(NetError::Response(503, String::new()));
        match outcome {
            AttemptOutcome::Retryable(message) => {
                assert!(message.contains("503"), "message should carry status: {message}");
            }
            _ => panic!("expected retryable outcome"),
        }
    }

    #[test]
    fn sources_put_primary_first_with_first_byte_timeout() {
        let sources =
            build_download_sources("https://primary/app.msi", Some("https://mirror/app.msi"));

        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].url, "https://primary/app.msi");
        assert_eq!(sources[0].first_byte_timeout, Some(PRIMARY_FIRST_BYTE_TIMEOUT));
        // 有兜底时主地址不重试，快速让位给镜像。
        assert_eq!(sources[0].max_retries, 0);

        // 镜像地址沿用默认重试策略，不设首包超时。
        assert_eq!(sources[1].url, "https://mirror/app.msi");
        assert_eq!(sources[1].first_byte_timeout, None);
        assert_eq!(sources[1].max_retries, RetryPolicy::default().max_retries);
    }

    #[test]
    fn single_source_keeps_default_retry_policy() {
        // 没有镜像时唯一来源不能快速失败，否则一次抖动就让更新中断。
        let sources = build_download_sources("https://primary/app.msi", None);

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].first_byte_timeout, None);
        assert_eq!(sources[0].max_retries, RetryPolicy::default().max_retries);
        assert!(sources[0].max_retries > 0);
    }

    #[test]
    fn sources_collapse_when_mirror_equals_primary() {
        // 同一地址只应请求一次，否则会白白多走一轮完整重试。
        let sources = build_download_sources("https://same/app.msi", Some("https://same/app.msi"));

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].url, "https://same/app.msi");
        // 去重后退化为单源，同样要保留重试。
        assert_eq!(sources[0].max_retries, RetryPolicy::default().max_retries);
    }

    #[test]
    fn sources_ignore_missing_or_blank_mirror() {
        let url = "https://primary/app.msi";

        assert_eq!(build_download_sources(url, None).len(), 1);
        assert_eq!(build_download_sources(url, Some("")).len(), 1);
        assert_eq!(build_download_sources(url, Some("   ")).len(), 1);
    }

    #[test]
    fn sources_trim_mirror_before_comparing() {
        // 带空白的镜像地址应被 trim 后参与比较与使用。
        let sources =
            build_download_sources("https://same/app.msi", Some("  https://same/app.msi "));

        assert_eq!(sources.len(), 1);
    }

    #[test]
    fn declared_length_accepts_matching_size() {
        assert!(check_declared_length(Some(11_010_048), 11_010_048).is_ok());
    }

    #[test]
    fn declared_length_rejects_truncated_payload() {
        // 声明 100 字节却只收到 80 字节，说明传输被截断。
        let error = check_declared_length(Some(100), 80).expect_err("should reject short payload");
        assert!(error.contains("100"), "{error}");
        assert!(error.contains("80"), "{error}");
    }

    #[test]
    fn declared_length_rejects_overlong_payload() {
        assert!(check_declared_length(Some(100), 120).is_err());
    }

    #[test]
    fn declared_length_skips_check_without_content_length() {
        // 分块传输不提供长度，无法判断截断，应直接放行。
        assert!(check_declared_length(None, 12345).is_ok());
    }
    #[test]
    fn sink_digest_matches_whole_file_hash() {
        // 流式计算必须与整文件哈希完全等价，否则校验会误判。
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("payload.bin");
        let payload: Vec<u8> = (0..5000_u32).map(|value| (value % 251) as u8).collect();

        let mut sink = DownloadSink::create(&path, true).expect("create sink");
        // 按不等长分块写入，覆盖 chunk 边界落在数据中部的情况。
        for chunk in payload.chunks(997) {
            sink.write(chunk).expect("write chunk");
        }
        let received = sink.received();
        let digest = sink.finish().expect("finish sink").expect("digest");

        assert_eq!(received, payload.len() as u64);
        assert_eq!(std::fs::read(&path).expect("read back"), payload);
        assert_eq!(
            sealantern_infra::fs::digest_to_hex(&digest),
            sealantern_infra::fs::sha256_hex(&payload)
        );
    }

    #[test]
    fn sink_skips_hashing_when_disabled() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("payload.bin");

        let mut sink = DownloadSink::create(&path, false).expect("create sink");
        sink.write(b"sealantern").expect("write chunk");
        let digest = sink.finish().expect("finish sink");

        assert!(digest.is_none());
        assert_eq!(std::fs::read(&path).expect("read back"), b"sealantern");
    }

    #[test]
    fn sink_discard_removes_partial_file() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("payload.bin");

        let mut sink = DownloadSink::create(&path, true).expect("create sink");
        sink.write(b"partial").expect("write chunk");
        sink.discard();

        assert!(!path.exists());
    }

    #[test]
    fn verify_digest_accepts_matching_hash() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("payload.bin");
        std::fs::write(&path, b"sealantern").expect("write file");
        let payload = b"sealantern";
        let digest: [u8; 32] = Sha256::digest(payload).into();
        let expected = sealantern_infra::fs::sha256_hex(payload);

        assert!(verify_digest(&path, Some(digest), payload.len() as u64, Some(&expected)).is_ok());
    }

    #[test]
    fn verify_digest_removes_file_on_mismatch() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("payload.bin");
        std::fs::write(&path, b"tampered").expect("write file");
        // 摘要对应另一份内容，与期望值不一致。
        let digest: [u8; 32] = Sha256::digest(b"tampered").into();
        let expected = sealantern_infra::fs::sha256_hex(b"expected");

        let error = verify_digest(&path, Some(digest), 8, Some(&expected))
            .expect_err("mismatch should fail");

        assert!(error.contains("Hash verification failed"), "{error}");
        // 损坏内容不应留在缓存目录里。
        assert!(!path.exists());
    }

    #[test]
    fn verify_digest_accepts_uppercase_expected_hash() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("payload.bin");
        let payload = b"sealantern";
        std::fs::write(&path, payload).expect("write file");
        let digest: [u8; 32] = Sha256::digest(payload).into();
        let expected = sealantern_infra::fs::sha256_hex(payload).to_ascii_uppercase();

        assert!(verify_digest(&path, Some(digest), payload.len() as u64, Some(&expected)).is_ok());
    }

    #[test]
    fn verify_digest_skips_check_without_expected_hash() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("payload.bin");
        std::fs::write(&path, b"anything").expect("write file");

        assert!(verify_digest(&path, None, 8, None).is_ok());
        assert!(path.exists());
    }

    #[test]
    fn verify_digest_fails_when_digest_missing() {
        // 要求校验却没有摘要属于配置错误，不能静默放行。
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("payload.bin");
        std::fs::write(&path, b"anything").expect("write file");

        assert!(verify_digest(&path, None, 8, Some("deadbeef")).is_err());
    }

    #[test]
    fn temp_download_is_removed_when_dropped_uncommitted() {
        // 失败或取消时不能把残缺文件留在缓存目录。
        let dir = tempfile::tempdir().expect("create temp dir");
        let temp = TempDownload::new(dir.path(), "app.msi");
        std::fs::write(temp.path(), b"partial").expect("write partial");
        let path = temp.path().to_path_buf();

        drop(temp);

        assert!(!path.exists(), "uncommitted temp file should be removed");
    }

    #[test]
    fn temp_download_commit_replaces_existing_cache_file() {
        // 已存在的缓存文件必须被新内容整体替换，而不是被截断后留在原地。
        let dir = tempfile::tempdir().expect("create temp dir");
        let final_path = dir.path().join("app.msi");
        std::fs::write(&final_path, b"old installer").expect("write old");

        let temp = TempDownload::new(dir.path(), "app.msi");
        std::fs::write(temp.path(), b"new installer").expect("write new");
        temp.commit(&final_path).expect("commit download");

        assert_eq!(std::fs::read(&final_path).expect("read back"), b"new installer");
    }

    #[test]
    fn temp_download_paths_are_unique() {
        // 并发下载必须落在不同临时文件上，否则会互相覆盖。
        let dir = tempfile::tempdir().expect("create temp dir");
        let first = TempDownload::new(dir.path(), "app.msi");
        let second = TempDownload::new(dir.path(), "app.msi");

        assert_ne!(first.path(), second.path());
    }

    #[test]
    fn temp_download_keeps_file_after_commit() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let final_path = dir.path().join("app.msi");
        let temp = TempDownload::new(dir.path(), "app.msi");
        std::fs::write(temp.path(), b"payload").expect("write payload");
        temp.commit(&final_path).expect("commit download");

        assert!(final_path.exists());
        assert_eq!(std::fs::read(&final_path).expect("read back"), b"payload");
    }
}
