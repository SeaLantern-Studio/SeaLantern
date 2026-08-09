//! 应用更新检查服务实现。

use async_trait::async_trait;
use sealantern_extra::update::{
    ReleaseUpdateChecker, UpdateChecker as ExtraUpdateChecker, UpdateInfo as ExtraUpdateInfo,
};
use sealantern_interface::update::{UpdateInfo, UpdateSource};
use sealantern_interface::{UpdateCheckService, UpdateCheckServiceError};

use crate::error::UpdateCheckError;

/// 基于官方多来源检查器的应用更新服务。
pub struct CoreUpdateCheckService {
    checker: tokio::sync::OnceCell<ReleaseUpdateChecker>,
}

impl CoreUpdateCheckService {
    /// 构造服务；网络客户端延迟到首次检查时初始化。
    pub const fn new() -> Self {
        Self {
            checker: tokio::sync::OnceCell::const_new(),
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
        check_with(self.checker().await?, env!("CARGO_PKG_VERSION")).await
    }
}

async fn check_with<C>(
    checker: &C,
    current_version: &str,
) -> Result<UpdateInfo, UpdateCheckServiceError>
where
    C: ExtraUpdateChecker + ?Sized,
{
    let info = checker
        .check(current_version)
        .await
        .map_err(contract_error)?;
    update_to_contract(info).map_err(contract_error)
}

fn update_to_contract(info: ExtraUpdateInfo) -> Result<UpdateInfo, UpdateCheckError> {
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
    use sealantern_extra::update::UpdateCheckError as ExtraUpdateCheckError;

    use super::*;

    struct FakeChecker {
        info: Option<ExtraUpdateInfo>,
    }

    #[async_trait]
    impl ExtraUpdateChecker for FakeChecker {
        async fn check(
            &self,
            _current_version: &str,
        ) -> Result<ExtraUpdateInfo, ExtraUpdateCheckError> {
            self.info
                .clone()
                .ok_or_else(|| ExtraUpdateCheckError::ProviderFailed {
                    provider: "github",
                    message: "offline".to_owned(),
                })
        }
    }

    fn update(source: Option<&str>) -> ExtraUpdateInfo {
        ExtraUpdateInfo {
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
        let checker = FakeChecker { info: Some(update(Some("arch-aur"))) };

        let info = check_with(&checker, "1.0.0").await.expect("check update");

        assert_eq!(info.source, Some(UpdateSource::ArchAur));
        assert_eq!(info.latest_version, "2.0.0");
    }

    #[tokio::test]
    async fn rejects_unknown_update_source() {
        let checker = FakeChecker { info: Some(update(Some("mirror"))) };

        let error = check_with(&checker, "1.0.0")
            .await
            .expect_err("unknown source must fail");

        assert_eq!(error, UpdateCheckServiceError::CheckFailed);
    }

    #[tokio::test]
    async fn maps_provider_failure_to_contract_error() {
        let checker = FakeChecker { info: None };

        let error = check_with(&checker, "1.0.0")
            .await
            .expect_err("provider failure must fail");

        assert_eq!(error, UpdateCheckServiceError::CheckFailed);
    }
}
