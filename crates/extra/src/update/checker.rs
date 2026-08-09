//! 多来源应用更新检查编排。

use async_trait::async_trait;
use sealantern_infra::net::{ClientConfig, NetClient, NetError, TimeoutPolicy};

use super::constants::{UPDATE_HTTP_CONNECT_TIMEOUT, UPDATE_HTTP_TIMEOUT, UPDATE_HTTP_USER_AGENT};
use super::error::UpdateCheckError;
use super::types::UpdateInfo;

/// 应用更新检查能力。
#[async_trait]
pub trait UpdateChecker: Send + Sync {
    /// 检查当前平台是否存在新版本。
    async fn check(&self, current_version: &str) -> Result<UpdateInfo, UpdateCheckError>;
}

/// 基于 SeaLantern 官方发布源的更新检查器。
pub struct ReleaseUpdateChecker {
    client: NetClient,
}

impl ReleaseUpdateChecker {
    /// 使用官方更新 User-Agent 构造检查器。
    pub fn new() -> Result<Self, UpdateCheckError> {
        let client = build_update_http_client(UPDATE_HTTP_USER_AGENT)
            .map_err(|source| UpdateCheckError::ClientInitialization { source })?;
        Ok(Self { client })
    }

    /// 使用既有 HTTP 客户端构造检查器，供上层注入代理或测试客户端。
    pub fn with_client(client: NetClient) -> Self {
        Self { client }
    }
}

/// 通过 infra 统一客户端构建更新检查网络入口。
pub(crate) fn build_update_http_client(user_agent: &str) -> Result<NetClient, NetError> {
    let config = ClientConfig {
        timeout: TimeoutPolicy {
            connect: UPDATE_HTTP_CONNECT_TIMEOUT,
            read: UPDATE_HTTP_TIMEOUT,
            total: UPDATE_HTTP_TIMEOUT,
        },
        user_agent: user_agent.to_owned(),
        ..ClientConfig::default()
    };
    NetClient::from_config(&config)
}

#[async_trait]
impl UpdateChecker for ReleaseUpdateChecker {
    async fn check(&self, current_version: &str) -> Result<UpdateInfo, UpdateCheckError> {
        #[cfg(debug_assertions)]
        {
            let _ = &self.client;
            crate::observability::update_check_started("disabled", current_version);
            crate::observability::update_check_completed("disabled", false, Some(current_version));
            return Ok(no_update(current_version));
        }

        #[cfg(all(not(debug_assertions), target_os = "linux"))]
        {
            let client = self.client.get_reqwest_client();
            if super::is_arch_linux() {
                return super::arch::check_aur_update_with_client(client, current_version)
                    .await
                    .map_err(|message| UpdateCheckError::ProviderFailed {
                        provider: "arch-aur",
                        message,
                    });
            }

            let (cnb, github) = tokio::join!(
                super::fetch_cnb_release(client, current_version),
                super::fetch_github_release(client, &super::get_github_config(), current_version)
            );
            return select_linux_result(cnb, github);
        }

        #[cfg(all(not(debug_assertions), not(target_os = "linux")))]
        {
            super::fetch_github_release(
                self.client.get_reqwest_client(),
                &super::get_github_config(),
                current_version,
            )
            .await
            .map_err(|message| UpdateCheckError::ProviderFailed { provider: "github", message })
        }
    }
}

#[cfg(debug_assertions)]
fn no_update(current_version: &str) -> UpdateInfo {
    UpdateInfo {
        has_update: false,
        latest_version: current_version.to_owned(),
        current_version: current_version.to_owned(),
        download_url: None,
        release_notes: None,
        published_at: None,
        source: None,
        sha256: None,
    }
}

#[cfg(any(test, all(not(debug_assertions), target_os = "linux")))]
fn select_linux_result(
    cnb: Result<UpdateInfo, String>,
    github: Result<UpdateInfo, String>,
) -> Result<UpdateInfo, UpdateCheckError> {
    match (cnb, github) {
        (_, Ok(github_info)) if github_info.has_update => Ok(github_info),
        (Ok(cnb_info), _) => Ok(cnb_info),
        (Err(_), Ok(github_info)) => Ok(github_info),
        (Err(cnb), Err(github)) => Err(UpdateCheckError::ProvidersFailed { cnb, github }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(source: &str, has_update: bool) -> UpdateInfo {
        UpdateInfo {
            has_update,
            latest_version: if has_update { "2.0.0" } else { "1.0.0" }.to_owned(),
            current_version: "1.0.0".to_owned(),
            download_url: has_update.then(|| "https://example.com/update".to_owned()),
            release_notes: None,
            published_at: None,
            source: Some(source.to_owned()),
            sha256: None,
        }
    }

    #[test]
    fn linux_selection_prefers_available_github_release() {
        let selected = select_linux_result(Ok(update("cnb", true)), Ok(update("github", true)))
            .expect("select update");

        assert_eq!(selected.source.as_deref(), Some("github"));
    }

    #[test]
    fn linux_selection_falls_back_to_cnb() {
        let selected = select_linux_result(Ok(update("cnb", true)), Err("offline".to_owned()))
            .expect("select update");

        assert_eq!(selected.source.as_deref(), Some("cnb"));
    }

    #[test]
    fn linux_selection_preserves_both_failures() {
        let error =
            select_linux_result(Err("cnb failed".to_owned()), Err("github failed".to_owned()))
                .expect_err("both providers should fail");

        assert!(matches!(
            error,
            UpdateCheckError::ProvidersFailed { ref cnb, ref github }
                if cnb == "cnb failed" && github == "github failed"
        ));
    }

    #[cfg(debug_assertions)]
    #[tokio::test]
    async fn debug_checker_skips_remote_requests() {
        let checker = ReleaseUpdateChecker::new().expect("create update checker");
        let info = checker.check("1.2.3").await.expect("check update");

        assert!(!info.has_update);
        assert_eq!(info.current_version, "1.2.3");
        assert_eq!(info.latest_version, "1.2.3");
        assert!(info.source.is_none());
    }
}
