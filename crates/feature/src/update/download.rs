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
    let file_path = cache_dir.join(file_name);

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
    for (index, (source, first_byte_timeout)) in sources.into_iter().enumerate() {
        // 走到后续来源说明前一个来源已失败，此时记录一次镜像回退。
        // 兜底文案仅用于防御，正常流程下 last_failure 必定已赋值。
        if index > 0 {
            let reason = last_failure.as_deref().unwrap_or("previous source failed");
            observability::update_download_mirror_used(&url, source, &reason);
        }

        match try_download_once(&client, source, first_byte_timeout, &file_path, expected_hash)
            .await
        {
            AttemptOutcome::Downloaded { received, digest } => {
                match verify_digest(&file_path, digest, received, expected_hash) {
                    Ok(()) => {
                        let path = file_path.to_string_lossy().to_string();
                        observability::update_download_completed(&path);
                        return Ok(path);
                    }
                    // 校验失败说明该来源的文件不可信，换下一个来源再试。
                    Err(error) => {
                        let reason = format!("{source}: {error}");
                        last_failure = Some(reason.clone());
                        failures.push(reason);
                    }
                }
            }
            AttemptOutcome::Retryable(error) => {
                let reason = format!("{source}: {error}");
                last_failure = Some(reason.clone());
                failures.push(reason);
            }
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

/// 按尝试顺序组装下载来源。
///
/// 主地址在前并限制首包等待时长，镜像地址在后且沿用默认重试策略。镜像地址
/// 与主地址相同时只保留一个，避免对同一地址重复发起完整请求。
fn build_download_sources<'a>(
    primary: &'a str,
    mirror: Option<&'a str>,
) -> Vec<(&'a str, Option<Duration>)> {
    let mut sources = vec![(primary, Some(PRIMARY_FIRST_BYTE_TIMEOUT))];
    if let Some(mirror) = mirror {
        // 空白镜像地址等同于未提供，交给单源流程处理。
        let mirror = mirror.trim();
        if !mirror.is_empty() && mirror != primary {
            sources.push((mirror, None));
        }
    }

    sources
}

/// 按来源尝试一次下载。
///
/// `first_byte_timeout` 为 `Some` 时不重试并限制等待响应头的时长，用于主地址；
/// 为 `None` 时沿用客户端默认重试策略，用于镜像地址。
async fn try_download_once(
    client: &NetClient,
    url: &str,
    first_byte_timeout: Option<Duration>,
    file_path: &Path,
    expected_hash: Option<&str>,
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
        return if is_worth_retrying_status(status.as_u16()) {
            AttemptOutcome::Retryable(message)
        } else {
            AttemptOutcome::Fatal(message)
        };
    }

    let declared_bytes = response.content_length();

    // 边读边写并同步喂给 hasher：落盘完成即校验完成，无需再读一次整个文件。
    let mut sink = match DownloadSink::create(file_path, expected_hash.is_some()) {
        Ok(sink) => sink,
        Err(error) => return AttemptOutcome::Fatal(error),
    };

    let mut response = response;
    loop {
        match response.chunk().await {
            Ok(Some(chunk)) => {
                if let Err(error) = sink.write(&chunk) {
                    // 本地磁盘写入失败，换源无法解决。
                    return AttemptOutcome::Fatal(error);
                }
            }
            Ok(None) => break,
            Err(error) => {
                // 已写入的部分是残缺文件，清掉再换源。
                sink.discard();
                return AttemptOutcome::Retryable(format!("Failed to read response: {error}"));
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

    // 服务端声明了长度却未给够，说明传输被截断，换源重来比落盘损坏文件更稳。
    // 分块传输不提供 `content_length` 时跳过该检查。
    if let Err(reason) = check_declared_length(declared_bytes, received) {
        sink.discard();
        return AttemptOutcome::Retryable(reason);
    }

    match sink.finish() {
        Ok(digest) => AttemptOutcome::Downloaded { received, digest },
        Err(error) => AttemptOutcome::Fatal(error),
    }
}

/// 判断某个失败状态码是否值得切换到下一个下载来源。
///
/// 资源缺失（镜像尚未同步）、鉴权受限与服务端故障都值得换来源再试；
/// 其余 4xx 通常是请求本身有问题，换源也不会有帮助。
///
/// 响应状态码与网络错误两条路径共用本函数，避免两处判定各自演化后出现分歧。
fn is_worth_retrying_status(status: u16) -> bool {
    (500..=599).contains(&status) || matches!(status, 401 | 403 | 404)
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

        // 401/403/404：鉴权受限与资源缺失，同样值得换源。
        for status in [401, 403, 404] {
            assert!(is_worth_retrying_status(status), "{status} should be retryable");
        }
    }

    #[test]
    fn retrying_status_rejects_client_errors_and_success() {
        // 非 401/403/404 的 4xx 属于请求本身有问题，换源无益。
        for status in [400, 402, 405, 406, 409, 410, 422, 429] {
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
        assert_eq!(sources[0].0, "https://primary/app.msi");
        assert_eq!(sources[0].1, Some(PRIMARY_FIRST_BYTE_TIMEOUT));
        // 镜像地址沿用客户端默认重试策略，不设首包超时。
        assert_eq!(sources[1].0, "https://mirror/app.msi");
        assert_eq!(sources[1].1, None);
    }

    #[test]
    fn sources_collapse_when_mirror_equals_primary() {
        // 同一地址只应请求一次，否则会白白多走一轮完整重试。
        let sources = build_download_sources("https://same/app.msi", Some("https://same/app.msi"));

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].0, "https://same/app.msi");
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
}
