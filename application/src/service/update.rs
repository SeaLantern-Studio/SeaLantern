//! 应用更新检查服务实现。

use std::time::Duration;

use async_trait::async_trait;
use sealantern_contract::UpdateCheckServiceError;
use sealantern_contract::update::{UpdateInfo, UpdateSource};
use sealantern_feature::update::{
    ReleaseUpdateChecker, UpdateChecker as FeatureUpdateChecker, UpdateInfo as FeatureUpdateInfo,
};

use crate::error::UpdateCheckError;
use crate::port::UpdateCheckService;

/// 单次更新检查在应用层允许占用的总时长。
const UPDATE_CHECK_TIMEOUT: Duration = Duration::from_secs(30);
/// 成功检查结果的缓存时间。
const UPDATE_CHECK_SUCCESS_TTL: Duration = Duration::from_secs(5 * 60);
/// 失败检查结果的短暂退避时间。
const UPDATE_CHECK_FAILURE_TTL: Duration = Duration::from_secs(30);

#[derive(Clone)]
struct CachedUpdateCheck {
    checked_at: tokio::time::Instant,
    result: Result<UpdateInfo, UpdateCheckServiceError>,
}

impl CachedUpdateCheck {
    fn is_fresh(&self) -> bool {
        let ttl = if self.result.is_ok() {
            UPDATE_CHECK_SUCCESS_TTL
        } else {
            UPDATE_CHECK_FAILURE_TTL
        };
        self.checked_at.elapsed() < ttl
    }
}

/// 基于官方多来源检查器的应用更新服务。
pub struct CoreUpdateCheckService {
    checker: tokio::sync::OnceCell<ReleaseUpdateChecker>,
    cache: tokio::sync::Mutex<Option<CachedUpdateCheck>>,
}

impl CoreUpdateCheckService {
    /// 构造服务；网络客户端延迟到首次检查时初始化。
    pub const fn new() -> Self {
        Self {
            checker: tokio::sync::OnceCell::const_new(),
            cache: tokio::sync::Mutex::const_new(None),
        }
    }

    async fn checker(&self) -> Result<&ReleaseUpdateChecker, UpdateCheckServiceError> {
        self.checker
            .get_or_try_init(|| async { ReleaseUpdateChecker::new().map_err(contract_error) })
            .await
    }
}

impl Default for CoreUpdateCheckService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UpdateCheckService for CoreUpdateCheckService {
    async fn check(&self) -> Result<UpdateInfo, UpdateCheckServiceError> {
        cached_check_with(
            &self.cache,
            self.checker().await?,
            env!("CARGO_PKG_VERSION"),
            UPDATE_CHECK_TIMEOUT,
        )
        .await
    }
}

async fn cached_check_with<C>(
    cache: &tokio::sync::Mutex<Option<CachedUpdateCheck>>,
    checker: &C,
    current_version: &str,
    timeout: Duration,
) -> Result<UpdateInfo, UpdateCheckServiceError>
where
    C: FeatureUpdateChecker + ?Sized,
{
    // 持锁覆盖远程检查，使并发调用共享同一次 provider 请求。
    let mut cache = cache.lock().await;
    if let Some(cached) = cache.as_ref().filter(|cached| cached.is_fresh()) {
        return cached.result.clone();
    }

    let result = match tokio::time::timeout(timeout, check_with(checker, current_version)).await {
        Ok(result) => result,
        Err(_) => Err(contract_error(UpdateCheckError::TimedOut { timeout })),
    };
    *cache = Some(CachedUpdateCheck {
        checked_at: tokio::time::Instant::now(),
        result: result.clone(),
    });
    result
}

async fn check_with<C>(
    checker: &C,
    current_version: &str,
) -> Result<UpdateInfo, UpdateCheckServiceError>
where
    C: FeatureUpdateChecker + ?Sized,
{
    let info = checker
        .check(current_version)
        .await
        .map_err(contract_error)?;
    update_to_contract(info).map_err(contract_error)
}

fn update_to_contract(info: FeatureUpdateInfo) -> Result<UpdateInfo, UpdateCheckError> {
    Ok(UpdateInfo {
        has_update: info.has_update,
        latest_version: info.latest_version,
        current_version: info.current_version,
        download_url: info.download_url,
        release_notes: info.release_notes,
        published_at: info.published_at,
        source: info.source.map(parse_source).transpose()?,
        sha256: info.sha256,
    })
}

fn parse_source(source: String) -> Result<UpdateSource, UpdateCheckError> {
    match source.as_str() {
        "github" => Ok(UpdateSource::Github),
        "cnb" => Ok(UpdateSource::Cnb),
        "arch-aur" => Ok(UpdateSource::ArchAur),
        _ => Err(UpdateCheckError::InvalidResponse {
            detail: format!("unknown update source: {source}"),
        }),
    }
}

fn contract_error(error: impl Into<UpdateCheckError>) -> UpdateCheckServiceError {
    let error = error.into();
    tracing::error!(
        target: "sealantern.application.update",
        error = %error,
        "update check failed"
    );
    error.into()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use sealantern_feature::update::UpdateCheckError as FeatureUpdateCheckError;

    use super::*;

    struct FakeChecker {
        info: Option<FeatureUpdateInfo>,
        delay: Duration,
        calls: AtomicUsize,
    }

    #[async_trait]
    impl FeatureUpdateChecker for FakeChecker {
        async fn check(
            &self,
            _current_version: &str,
        ) -> Result<FeatureUpdateInfo, FeatureUpdateCheckError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            tokio::time::sleep(self.delay).await;
            self.info
                .clone()
                .ok_or_else(|| FeatureUpdateCheckError::ProviderFailed {
                    provider: "github",
                    message: "offline".to_owned(),
                })
        }
    }

    impl FakeChecker {
        fn with_info(info: FeatureUpdateInfo) -> Self {
            Self {
                info: Some(info),
                delay: Duration::ZERO,
                calls: AtomicUsize::new(0),
            }
        }

        fn failure() -> Self {
            Self {
                info: None,
                delay: Duration::ZERO,
                calls: AtomicUsize::new(0),
            }
        }
    }

    fn update(source: Option<&str>) -> FeatureUpdateInfo {
        FeatureUpdateInfo {
            has_update: true,
            latest_version: "2.0.0".to_owned(),
            current_version: "1.0.0".to_owned(),
            download_url: Some("https://example.com/update".to_owned()),
            release_notes: None,
            published_at: None,
            source: source.map(str::to_owned),
            sha256: None,
        }
    }

    #[tokio::test]
    async fn maps_known_update_source_to_contract() {
        let checker = FakeChecker::with_info(update(Some("arch-aur")));

        let info = check_with(&checker, "1.0.0").await.expect("check update");

        assert_eq!(info.source, Some(UpdateSource::ArchAur));
        assert_eq!(info.latest_version, "2.0.0");
    }

    #[tokio::test]
    async fn rejects_unknown_update_source() {
        let checker = FakeChecker::with_info(update(Some("mirror")));

        let error = check_with(&checker, "1.0.0")
            .await
            .expect_err("unknown source must fail");

        assert_eq!(error, UpdateCheckServiceError::CheckFailed);
    }

    #[tokio::test]
    async fn maps_provider_failure_to_contract_error() {
        let checker = FakeChecker::failure();

        let error = check_with(&checker, "1.0.0")
            .await
            .expect_err("provider failure must fail");

        assert_eq!(error, UpdateCheckServiceError::CheckFailed);
    }

    #[tokio::test]
    async fn concurrent_checks_share_one_provider_request() {
        let checker = FakeChecker {
            info: Some(update(Some("github"))),
            delay: Duration::from_millis(20),
            calls: AtomicUsize::new(0),
        };
        let cache = tokio::sync::Mutex::new(None);

        let (first, second) = tokio::join!(
            cached_check_with(&cache, &checker, "1.0.0", Duration::from_secs(1)),
            cached_check_with(&cache, &checker, "1.0.0", Duration::from_secs(1))
        );

        assert!(first.is_ok());
        assert!(second.is_ok());
        assert_eq!(checker.calls.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn total_timeout_is_reported_and_cached() {
        let checker = FakeChecker {
            info: Some(update(Some("github"))),
            delay: Duration::from_millis(50),
            calls: AtomicUsize::new(0),
        };
        let cache = tokio::sync::Mutex::new(None);

        let first = cached_check_with(&cache, &checker, "1.0.0", Duration::from_millis(1)).await;
        let second = cached_check_with(&cache, &checker, "1.0.0", Duration::from_millis(1)).await;

        assert_eq!(first, Err(UpdateCheckServiceError::CheckFailed));
        assert_eq!(second, Err(UpdateCheckServiceError::CheckFailed));
        assert_eq!(checker.calls.load(Ordering::Relaxed), 1);
    }
}
